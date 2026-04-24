use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;
use crate::git::{query, mutation, types::*};

const NETWORK_TIMEOUT: Duration = Duration::from_secs(5);

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

async fn run_with_timeout<F>(op: F, label: &str) -> Result<String, String>
where
    F: FnOnce() -> Result<String, String> + Send + 'static,
{
    let msg = format!("Network timeout: {} took longer than 5 seconds", label);
    timeout(NETWORK_TIMEOUT, tokio::task::spawn_blocking(op))
        .await
        .map_err(|_| msg)?
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn do_fetch(repo_path: String, remote: String) -> Result<String, String> {
    run_with_timeout(move || {
        mutation::fetch(Path::new(&repo_path), &remote).map_err(|e| e.to_string())
    }, "fetch").await
}

#[tauri::command]
pub async fn do_pull(repo_path: String, remote: String, rebase: bool) -> Result<String, String> {
    run_with_timeout(move || {
        mutation::pull(Path::new(&repo_path), &remote, rebase).map_err(|e| e.to_string())
    }, "pull").await
}

#[tauri::command]
pub async fn do_push(repo_path: String, remote: String, force: bool) -> Result<String, String> {
    run_with_timeout(move || {
        mutation::push(Path::new(&repo_path), &remote, force).map_err(|e| e.to_string())
    }, "push").await
}

#[tauri::command]
pub async fn do_push_branch(repo_path: String, remote: String, branch: String, force: bool) -> Result<String, String> {
    run_with_timeout(move || {
        mutation::push_branch(Path::new(&repo_path), &remote, &branch, force).map_err(|e| e.to_string())
    }, "push").await
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
pub async fn do_clone(url: String, dest: String) -> Result<String, String> {
    run_with_timeout(move || {
        mutation::clone_repo(&url, &dest).map_err(|e| e.to_string())
    }, "clone").await
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
