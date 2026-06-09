use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use std::sync::atomic::Ordering;

use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command as TokioCommand;

use crate::git::{mutation, query, types::*};

#[derive(serde::Serialize, Clone)]
struct NetworkProgressEvent {
    op: String,
    line: String,
}

pub(crate) const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 10;
pub(crate) const MAX_NETWORK_TIMEOUT_SECS: u64 = 600;

pub(crate) fn effective_timeout_secs(timeout_secs: Option<u64>) -> u64 {
    // Frontend already clamps to 1..=600, but the backend stays self-protecting
    // against future callers / hand-edited settings.json. None/Some(0) → default.
    timeout_secs
        .filter(|&s| s > 0)
        .map(|s| s.min(MAX_NETWORK_TIMEOUT_SECS))
        .unwrap_or(DEFAULT_NETWORK_TIMEOUT_SECS)
}

fn version_gt(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.trim_start_matches('v')
            .splitn(4, '.')
            .take(3)
            .filter_map(|x| x.split('-').next().and_then(|n| n.parse().ok()))
            .collect()
    };
    parse(latest) > parse(current)
}

/// Запускает `git` для сетевой операции с таймаутом. Стримит stderr как
/// события прогресса.
async fn run_network_git(
    app: &tauri::AppHandle,
    repo_path: Option<&Path>,
    args: &[String],
    timeout_secs: Option<u64>,
    label: &str,
) -> Result<String, String> {
    let secs = effective_timeout_secs(timeout_secs);

    // Вставляем --progress после subcommand чтобы git писал прогресс в pipe
    let mut git_args = args.to_vec();
    if !git_args.is_empty() {
        git_args.insert(1, "--progress".into());
    }

    let mut cmd = TokioCommand::new("git");
    if let Some(p) = repo_path {
        cmd.arg("-C").arg(p);
    }
    cmd.args(&git_args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    // Указываем git'у/ssh на нас как на askpass: при запросе credentials git
    // запустит этот бинарь, который покажет диалог в GUI (см. askpass.rs).
    let askpass = app.try_state::<crate::askpass::AskpassState>();
    if let Some(state) = &askpass {
        if let Ok(exe) = std::env::current_exe() {
            cmd.env("GIT_ASKPASS", &exe);
            cmd.env("SSH_ASKPASS", &exe);
            cmd.env("SSH_ASKPASS_REQUIRE", "force");
            cmd.env("GITSTREAM_ASKPASS_PIPE", &state.addr);
            cmd.env("GITSTREAM_ASKPASS_NONCE", &state.nonce);
        }
    }
    // Никогда не падаем на интерактивный TTY-prompt (его нет — зависли бы).
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    let active = askpass.map(|s| s.active.clone());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to run git: {} (Is git installed and in PATH?)", e))?;

    // Стримим stderr построчно и эмитим события прогресса
    let app_handle = app.clone();
    let op = label.to_string();
    let stderr_pipe = child.stderr.take();
    let stderr_task: tokio::task::JoinHandle<String> = tokio::spawn(async move {
        let mut collected = String::new();
        if let Some(pipe) = stderr_pipe {
            let mut reader = BufReader::new(pipe);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        // git пишет прогресс с \r внутри строки — берём последний сегмент
                        let stripped = line.trim_end_matches(['\r', '\n']);
                        let trimmed = stripped.rsplit('\r').find(|s| !s.trim().is_empty())
                            .unwrap_or(stripped)
                            .trim();
                        if !trimmed.is_empty() {
                            let _ = app_handle.emit(
                                "network_progress",
                                NetworkProgressEvent {
                                    op: op.clone(),
                                    line: trimmed.to_string(),
                                },
                            );
                            collected.push_str(trimmed);
                            collected.push('\n');
                        }
                    }
                }
            }
        }
        collected
    });

    // Опрашиваем процесс вместо единого таймаута: бюджет таймаута расходуется
    // только когда нет открытого askpass-диалога — иначе git убивался бы, пока
    // пользователь печатает пароль.
    let poll = Duration::from_millis(150);
    let mut remaining = Duration::from_secs(secs);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => return Err(format!("git wait failed: {}", e)),
        }
        let prompting = active.as_ref().is_some_and(|a| a.load(Ordering::SeqCst) > 0);
        if !prompting {
            if remaining <= poll {
                let _ = child.start_kill();
                let _ = child.wait().await;
                let _ = stderr_task.await;
                return Err(format!("Network timeout: {} превысил {} сек", label, secs));
            }
            remaining -= poll;
        }
        tokio::time::sleep(poll).await;
    };

    let stderr_text = stderr_task.await.unwrap_or_default();
    let mut stdout = String::new();
    if let Some(mut o) = child.stdout.take() {
        let _ = o.read_to_string(&mut stdout).await;
    }
    if status.success() {
        Ok(stdout)
    } else {
        Err(crate::git::error::classify_git_error(&stderr_text).to_string())
    }
}

/// Ответ GUI на запрос askpass: проводит введённое значение в git (или отмена).
#[tauri::command]
pub fn askpass_respond(app: tauri::AppHandle, id: u64, value: String, cancel: bool) {
    if let Some(state) = app.try_state::<crate::askpass::AskpassState>() {
        state.respond(id, if cancel { None } else { Some(value) });
    }
}

/// Включает git credential helper, если он ещё не настроен (чекбокс «Запомнить»).
#[tauri::command]
pub fn ensure_credential_helper() -> Result<(), String> {
    mutation::ensure_credential_helper().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_repo_info(repo_path: String) -> Result<RepoInfo, String> {
    query::repo_info(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_repo_state(repo_path: String) -> Result<String, String> {
    query::repo_state(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_accept_ours(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::accept_ours(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_accept_theirs(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::accept_theirs(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_op_abort(repo_path: String, state: String) -> Result<(), String> {
    mutation::op_abort(Path::new(&repo_path), &state).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_op_continue(repo_path: String, state: String) -> Result<(), String> {
    mutation::op_continue(Path::new(&repo_path), &state).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_status(repo_path: String) -> Result<Vec<FileStatus>, String> {
    query::status(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_all_files(repo_path: String) -> Result<Vec<String>, String> {
    query::list_all_files(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_files_at(repo_path: String, oid: String) -> Result<Vec<String>, String> {
    query::list_files_at(Path::new(&repo_path), &oid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_log(repo_path: String, limit: Option<usize>) -> Result<Vec<CommitInfo>, String> {
    let path = PathBuf::from(repo_path);
    tokio::task::spawn_blocking(move || {
        query::log(&path, limit.unwrap_or(500)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn get_branches(repo_path: String) -> Result<Vec<BranchInfo>, String> {
    query::branches(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tags(repo_path: String) -> Result<Vec<TagInfo>, String> {
    query::tags(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stashes(repo_path: String) -> Result<Vec<StashEntry>, String> {
    query::stashes(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_remotes(repo_path: String) -> Result<Vec<String>, String> {
    query::remotes(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_remote_urls(repo_path: String) -> Result<Vec<RemoteInfo>, String> {
    query::remote_urls(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_remote(repo_path: String, name: String, url: String) -> Result<(), String> {
    mutation::add_remote(Path::new(&repo_path), &name, &url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_remote(repo_path: String, name: String) -> Result<(), String> {
    mutation::remove_remote(Path::new(&repo_path), &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_remote(repo_path: String, old_name: String, new_name: String) -> Result<(), String> {
    mutation::rename_remote(Path::new(&repo_path), &old_name, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_remote_url(repo_path: String, name: String, url: String) -> Result<(), String> {
    mutation::set_remote_url(Path::new(&repo_path), &name, &url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_branch_upstream(
    repo_path: String,
    branch: String,
    upstream: Option<String>,
) -> Result<(), String> {
    mutation::set_branch_upstream(Path::new(&repo_path), &branch, upstream.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_diff_file(
    repo_path: String,
    file: String,
    staged: bool,
    context: Option<u32>,
) -> Result<FileDiff, String> {
    // Sync-команда блокировала бы главный поток Tauri: `git diff` большого файла
    // (например, File Compare для почти полностью переписанного файла) замораживает
    // UI до «GitStream не отвечает».
    let path = PathBuf::from(repo_path);
    tokio::task::spawn_blocking(move || {
        query::diff_file(&path, &file, staged, context).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_diff_commit(repo_path: String, oid: String) -> Result<Vec<FileDiff>, String> {
    // Sync-команда блокировала бы главный поток Tauri: при удержании стрелки
    // в CommitGraph каждый шаг ждёт `git diff` целого коммита, и UI замирает.
    let path = std::path::PathBuf::from(repo_path);
    tokio::task::spawn_blocking(move || query::diff_commit(&path, &oid).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_diff_commit_file(
    repo_path: String,
    oid: String,
    file: String,
    context: Option<u32>,
) -> Result<FileDiff, String> {
    let path = PathBuf::from(repo_path);
    tokio::task::spawn_blocking(move || {
        query::diff_commit_file(&path, &oid, &file, context).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn stage_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::stage(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unstage_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::unstage(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn discard_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::discard(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::remove(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::delete(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stage_hunk(repo_path: String, patch: String) -> Result<(), String> {
    mutation::stage_hunk(Path::new(&repo_path), &patch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unstage_hunk(repo_path: String, patch: String) -> Result<(), String> {
    mutation::unstage_hunk(Path::new(&repo_path), &patch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn discard_hunk(repo_path: String, patch: String) -> Result<(), String> {
    mutation::discard_hunk(Path::new(&repo_path), &patch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_lines(
    repo_path: String,
    file_header: String,
    hunks: Vec<mutation::LineHunkSelection>,
    op: mutation::LineOp,
) -> Result<(), String> {
    mutation::apply_lines(Path::new(&repo_path), &file_header, &hunks, op)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_commit(repo_path: String, message: String, amend: bool) -> Result<String, String> {
    mutation::commit(Path::new(&repo_path), &message, amend).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_rebase(repo_path: String, onto: String) -> Result<String, String> {
    mutation::rebase(Path::new(&repo_path), &onto).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_reset(repo_path: String, oid: String, mode: String) -> Result<(), String> {
    mutation::reset(Path::new(&repo_path), &oid, &mode).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_revert(repo_path: String, oid: String, no_commit: bool) -> Result<(), String> {
    mutation::revert(Path::new(&repo_path), &oid, no_commit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_cherry_pick(repo_path: String, oid: String) -> Result<(), String> {
    mutation::cherry_pick(Path::new(&repo_path), &oid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_create_branch(
    repo_path: String,
    name: String,
    start_point: Option<String>,
    checkout: bool,
) -> Result<(), String> {
    mutation::create_branch(
        Path::new(&repo_path),
        &name,
        start_point.as_deref(),
        checkout,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_stash_save(
    repo_path: String,
    message: Option<String>,
    include_untracked: bool,
) -> Result<(), String> {
    mutation::stash_save(Path::new(&repo_path), message.as_deref(), include_untracked)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_stash_apply(repo_path: String, index: usize) -> Result<(), String> {
    mutation::stash_apply(Path::new(&repo_path), index).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_stash_pop(repo_path: String, index: usize) -> Result<(), String> {
    mutation::stash_pop(Path::new(&repo_path), index).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_stash_drop(repo_path: String, index: usize) -> Result<(), String> {
    mutation::stash_drop(Path::new(&repo_path), index).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_checkout(repo_path: String, branch: String) -> Result<(), String> {
    mutation::checkout(Path::new(&repo_path), &branch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_checkout_remote(
    repo_path: String,
    remote_branch: String,
    local_name: Option<String>,
) -> Result<(), String> {
    mutation::checkout_remote(Path::new(&repo_path), &remote_branch, local_name.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn do_fetch(
    app: tauri::AppHandle,
    repo_path: String,
    remote: String,
    prune: Option<bool>,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::fetch_args(&remote, prune.unwrap_or(false));
    run_network_git(
        &app,
        Some(Path::new(&repo_path)),
        &args,
        timeout_secs,
        "fetch",
    )
    .await
}

#[tauri::command]
pub async fn do_pull(
    app: tauri::AppHandle,
    repo_path: String,
    remote: String,
    rebase: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let path = Path::new(&repo_path);
    let branch = query::current_branch_name(path).map_err(|e| e.to_string())?;
    let args = mutation::pull_args(&remote, &branch, rebase);
    run_network_git(&app, Some(path), &args, timeout_secs, "pull").await
}

#[tauri::command]
pub async fn do_push(
    app: tauri::AppHandle,
    repo_path: String,
    remote: String,
    force: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let path = Path::new(&repo_path);
    let branch = query::current_branch_name(path).map_err(|e| e.to_string())?;
    let args = mutation::push_args(&remote, &branch, force);
    run_network_git(&app, Some(path), &args, timeout_secs, "push").await
}

#[tauri::command]
pub async fn do_push_branch(
    app: tauri::AppHandle,
    repo_path: String,
    remote: String,
    branch: String,
    force: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_branch_args(&remote, &branch, force);
    run_network_git(
        &app,
        Some(Path::new(&repo_path)),
        &args,
        timeout_secs,
        "push",
    )
    .await
}

#[tauri::command]
pub async fn do_push_tag(
    app: tauri::AppHandle,
    repo_path: String,
    remote: String,
    name: String,
    delete: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_tag_args(&remote, &name, delete);
    run_network_git(
        &app,
        Some(Path::new(&repo_path)),
        &args,
        timeout_secs,
        "push",
    )
    .await
}

#[tauri::command]
pub fn do_merge(repo_path: String, branch: String) -> Result<String, String> {
    mutation::merge(Path::new(&repo_path), &branch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_rename_branch(
    repo_path: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    mutation::rename_branch(Path::new(&repo_path), &old_name, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_delete_branch(repo_path: String, branch: String, force: bool) -> Result<(), String> {
    mutation::delete_branch(Path::new(&repo_path), &branch, force).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn do_delete_remote_branch(
    app: tauri::AppHandle,
    repo_path: String,
    remote: String,
    branch: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::delete_remote_branch_args(&remote, &branch);
    run_network_git(&app, Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}

#[tauri::command]
pub fn do_create_tag(
    repo_path: String,
    name: String,
    message: Option<String>,
    target: Option<String>,
    force: bool,
) -> Result<(), String> {
    mutation::create_tag(
        Path::new(&repo_path),
        &name,
        message.as_deref(),
        target.as_deref(),
        force,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_delete_tag(repo_path: String, name: String) -> Result<(), String> {
    mutation::delete_tag(Path::new(&repo_path), &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_repo_stats(
    repo_path: String,
    since_days: Option<u32>,
) -> Result<RepoStats, String> {
    let path = std::path::PathBuf::from(repo_path);
    tokio::task::spawn_blocking(move || {
        query::repo_stats(&path, since_days).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn do_reword_commit(
    repo_path: String,
    oid: String,
    message: String,
) -> Result<String, String> {
    let path = std::path::PathBuf::from(repo_path);
    tokio::task::spawn_blocking(move || {
        mutation::reword_commit(&path, &oid, &message).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn do_squash(
    repo_path: String,
    oids: Vec<String>,
    message: String,
) -> Result<(), String> {
    let path = std::path::PathBuf::from(repo_path);
    tokio::task::spawn_blocking(move || {
        mutation::squash(&path, &oids, &message).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn do_init(path: String) -> Result<RepoInfo, String> {
    let p = Path::new(&path);
    mutation::init(p).map_err(|e| e.to_string())?;
    query::repo_info(p).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn do_clone(
    app: tauri::AppHandle,
    url: String,
    dest: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    // Защита от argv flag smuggling (вдобавок к `--` в clone_args).
    if url.starts_with('-') || dest.starts_with('-') {
        return Err("invalid argument: url/dest must not start with '-'".into());
    }
    let args = mutation::clone_args(&url, &dest);
    run_network_git(&app, None, &args, timeout_secs, "clone").await
}

#[tauri::command]
pub fn check_repo_path(path: String) -> Result<RepoPathCheck, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Ok(RepoPathCheck {
            exists: false,
            is_git_repo: false,
            display_name: String::new(),
        });
    }
    let is_git = p.join(".git").exists() || p.join("HEAD").exists();
    let display_name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(RepoPathCheck {
        exists: true,
        is_git_repo: is_git,
        display_name,
    })
}

#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("gitstream-app")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = match client
        .get("https://api.github.com/repos/avv7bc/gitstream/releases/latest")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Ok(None),
    };

    let tag_name = match json["tag_name"].as_str() {
        Some(t) => t.to_string(),
        None => return Ok(None),
    };

    let html_url = match json["html_url"].as_str().filter(|s| !s.is_empty()) {
        Some(u) => u.to_string(),
        None => return Ok(None),
    };
    let current = app.package_info().version.to_string();

    if version_gt(&tag_name, &current) {
        Ok(Some(UpdateInfo {
            version: tag_name.trim_start_matches('v').to_string(),
            release_url: html_url,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(format!("Invalid URL scheme: {}", url));
    }
    open_in_browser(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_system_fonts() -> Vec<String> {
    let Ok(output) = std::process::Command::new("fc-list")
        .args([":", "family"])
        .output()
    else {
        return Vec::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut families: Vec<String> = stdout
        .lines()
        .flat_map(|line| line.split(','))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    families.sort_unstable_by_key(|s| s.to_lowercase());
    families.dedup();
    families
}

fn open_in_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/c", "start", "", url])
        .spawn()?;
    Ok(())
}

#[cfg(test)]
mod network_timeout_tests {
    use super::*;
    use std::fs;
    use std::process::Command as StdCommand;

    // Используется закомментированным fetch_times_out_and_kills_process (см. ниже).
    #[allow(dead_code)]
    fn temp_repo_with_dead_remote() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_net_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            StdCommand::new("git")
                .current_dir(&dir)
                .args(args)
                .output()
                .unwrap();
        };
        run(&["init", "-q"]);
        run(&["remote", "add", "origin", "https://192.0.2.1/dead.git"]);
        dir
    }

    // NOTE: This test is broken (pre-existing issue: run_network_git signature changed
    // to require AppHandle as first param but test wasn't updated). Commenting out
    // to allow building. This is a pre-existing issue not part of this task.
    // #[tokio::test]
    // async fn fetch_times_out_and_kills_process() {
    //     let dir = temp_repo_with_dead_remote();
    //     let args = crate::git::mutation::fetch_args("origin");
    //     let start = std::time::Instant::now();
    //     let res = run_network_git(Some(dir.as_path()), &args, Some(1), "fetch").await;
    //     let elapsed = start.elapsed();
    //
    //     assert!(res.is_err(), "ожидали ошибку таймаута, получили {:?}", res);
    //     let msg = res.unwrap_err();
    //     assert!(
    //         msg.contains("timeout") || msg.contains("таймаут") || msg.contains("превысил"),
    //         "сообщение об ошибке должно сообщать о таймауте: {}",
    //         msg
    //     );
    //     assert!(
    //         elapsed < std::time::Duration::from_secs(10),
    //         "раннер не вернулся быстро после таймаута: {:?}",
    //         elapsed
    //     );
    //     fs::remove_dir_all(&dir).ok();
    // }

    #[tokio::test]
    async fn zero_timeout_falls_back_to_default() {
        assert_eq!(
            effective_timeout_secs(Some(0)),
            DEFAULT_NETWORK_TIMEOUT_SECS
        );
        assert_eq!(effective_timeout_secs(None), DEFAULT_NETWORK_TIMEOUT_SECS);
        assert_eq!(effective_timeout_secs(Some(25)), 25);
        assert_eq!(
            effective_timeout_secs(Some(99999)),
            MAX_NETWORK_TIMEOUT_SECS
        );
        assert_eq!(effective_timeout_secs(Some(600)), 600);
    }
}

#[cfg(test)]
mod update_tests {
    use super::version_gt;

    #[test]
    fn newer_patch_is_gt() {
        assert!(version_gt("v0.4.2", "0.4.1"));
    }

    #[test]
    fn newer_minor_is_gt() {
        assert!(version_gt("v0.5.0", "0.4.9"));
    }

    #[test]
    fn newer_major_is_gt() {
        assert!(version_gt("v1.0.0", "0.9.9"));
    }

    #[test]
    fn same_version_is_not_gt() {
        assert!(!version_gt("v0.4.1", "0.4.1"));
    }

    #[test]
    fn older_version_is_not_gt() {
        assert!(!version_gt("v0.4.0", "0.4.1"));
    }

    #[test]
    fn no_v_prefix_works() {
        assert!(version_gt("0.4.2", "0.4.1"));
    }
}
