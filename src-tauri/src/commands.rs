use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;

use crate::git::{query, mutation, types::*};

pub(crate) const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 10;

pub(crate) fn effective_timeout_secs(timeout_secs: Option<u64>) -> u64 {
    timeout_secs
        .filter(|&s| s > 0)
        .unwrap_or(DEFAULT_NETWORK_TIMEOUT_SECS)
}

/// Запускает `git` для сетевой операции с таймаутом. По истечении таймаута
/// процесс принудительно убивается. `repo_path = None` для `clone`.
async fn run_network_git(
    repo_path: Option<&Path>,
    args: &[String],
    timeout_secs: Option<u64>,
    label: &str,
) -> Result<String, String> {
    let secs = effective_timeout_secs(timeout_secs);

    let mut cmd = TokioCommand::new("git");
    if let Some(p) = repo_path {
        cmd.arg("-C").arg(p);
    }
    cmd.args(args.iter().map(|s| s.as_str()));
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to run git: {} (Is git installed and in PATH?)", e)
    })?;

    match tokio::time::timeout(Duration::from_secs(secs), child.wait()).await {
        Ok(Ok(status)) => {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut o) = child.stdout.take() {
                let _ = o.read_to_string(&mut stdout).await;
            }
            if let Some(mut e) = child.stderr.take() {
                let _ = e.read_to_string(&mut stderr).await;
            }
            if status.success() {
                Ok(stdout)
            } else {
                Err(crate::git::error::classify_git_error(&stderr).to_string())
            }
        }
        Ok(Err(e)) => Err(format!("git wait failed: {}", e)),
        Err(_) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            Err(format!(
                "Network timeout: {} превысил {} сек",
                label, secs
            ))
        }
    }
}

#[tauri::command]
pub fn get_repo_info(repo_path: String) -> Result<RepoInfo, String> {
    query::repo_info(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_status(repo_path: String) -> Result<Vec<FileStatus>, String> {
    query::status(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_log(repo_path: String, limit: Option<usize>) -> Result<Vec<CommitInfo>, String> {
    query::log(Path::new(&repo_path), limit.unwrap_or(500)).map_err(|e| e.to_string())
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
pub fn get_diff_file(repo_path: String, file: String, staged: bool) -> Result<FileDiff, String> {
    query::diff_file(Path::new(&repo_path), &file, staged).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_diff_commit(repo_path: String, oid: String) -> Result<Vec<FileDiff>, String> {
    query::diff_commit(Path::new(&repo_path), &oid).map_err(|e| e.to_string())
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
pub fn do_commit(repo_path: String, message: String, amend: bool) -> Result<String, String> {
    mutation::commit(Path::new(&repo_path), &message, amend).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_checkout(repo_path: String, branch: String) -> Result<(), String> {
    mutation::checkout(Path::new(&repo_path), &branch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_checkout_remote(repo_path: String, remote_branch: String, local_name: Option<String>) -> Result<(), String> {
    mutation::checkout_remote(Path::new(&repo_path), &remote_branch, local_name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn do_fetch(
    repo_path: String,
    remote: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::fetch_args(&remote);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "fetch").await
}

#[tauri::command]
pub async fn do_pull(
    repo_path: String,
    remote: String,
    rebase: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::pull_args(&remote, rebase);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "pull").await
}

#[tauri::command]
pub async fn do_push(
    repo_path: String,
    remote: String,
    force: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_args(&remote, force);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}

#[tauri::command]
pub async fn do_push_branch(
    repo_path: String,
    remote: String,
    branch: String,
    force: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_branch_args(&remote, &branch, force);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}

#[tauri::command]
pub async fn do_push_tag(
    repo_path: String,
    remote: String,
    name: String,
    delete: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_tag_args(&remote, &name, delete);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}

#[tauri::command]
pub async fn do_clone(
    url: String,
    dest: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::clone_args(&url, &dest);
    run_network_git(None, &args, timeout_secs, "clone").await
}

#[tauri::command]
pub fn do_merge(repo_path: String, branch: String) -> Result<String, String> {
    mutation::merge(Path::new(&repo_path), &branch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_rename_branch(repo_path: String, old_name: String, new_name: String) -> Result<(), String> {
    mutation::rename_branch(Path::new(&repo_path), &old_name, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_delete_branch(repo_path: String, branch: String, force: bool) -> Result<(), String> {
    mutation::delete_branch(Path::new(&repo_path), &branch, force).map_err(|e| e.to_string())
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
pub fn check_repo_path(path: String) -> Result<RepoPathCheck, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Ok(RepoPathCheck { exists: false, is_git_repo: false, display_name: String::new() });
    }
    let is_git = p.join(".git").exists() || p.join("HEAD").exists();
    let display_name = p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(RepoPathCheck { exists: true, is_git_repo: is_git, display_name })
}

#[cfg(test)]
mod network_timeout_tests {
    use super::*;
    use std::fs;
    use std::process::Command as StdCommand;

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
            StdCommand::new("git").current_dir(&dir).args(args).output().unwrap();
        };
        run(&["init", "-q"]);
        run(&["remote", "add", "origin", "https://192.0.2.1/dead.git"]);
        dir
    }

    #[tokio::test]
    async fn fetch_times_out_and_kills_process() {
        let dir = temp_repo_with_dead_remote();
        let args = crate::git::mutation::fetch_args("origin");
        let start = std::time::Instant::now();
        let res = run_network_git(Some(dir.as_path()), &args, Some(1), "fetch").await;
        let elapsed = start.elapsed();

        assert!(res.is_err(), "ожидали ошибку таймаута, получили {:?}", res);
        let msg = res.unwrap_err();
        assert!(
            msg.contains("timeout") || msg.contains("таймаут") || msg.contains("превысил"),
            "сообщение об ошибке должно сообщать о таймауте: {}",
            msg
        );
        assert!(
            elapsed < std::time::Duration::from_secs(10),
            "раннер не вернулся быстро после таймаута: {:?}",
            elapsed
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn zero_timeout_falls_back_to_default() {
        assert_eq!(effective_timeout_secs(Some(0)), DEFAULT_NETWORK_TIMEOUT_SECS);
        assert_eq!(effective_timeout_secs(None), DEFAULT_NETWORK_TIMEOUT_SECS);
        assert_eq!(effective_timeout_secs(Some(25)), 25);
    }
}
