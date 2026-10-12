use super::error::{classify_git_error, GitError};
use super::oplog;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WorktreeInfo {
    pub path: String,
    pub head: String,
    pub branch: Option<String>,
    pub detached: bool,
    pub locked: bool,
    pub prunable: bool,
}
fn fail(message: impl Into<String>) -> GitError {
    GitError::CommandFailed {
        message: message.into(),
        hint: None,
    }
}
fn run(repo: &Path, args: &[&str]) -> Result<String, GitError> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .map_err(|e| fail(format!("run git worktree: {e}")))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(classify_git_error(&String::from_utf8_lossy(&out.stderr)))
    }
}
fn safe_arg(value: &str, label: &str) -> Result<(), GitError> {
    if value.is_empty() || value.starts_with('-') || value.contains('\0') {
        Err(fail(format!("Invalid {label}")))
    } else {
        Ok(())
    }
}
pub fn list(repo: &Path) -> Result<Vec<WorktreeInfo>, GitError> {
    let raw = run(repo, &["worktree", "list", "--porcelain"])?;
    let mut result = Vec::new();
    let mut current: Option<WorktreeInfo> = None;
    for line in raw.lines() {
        if line.is_empty() {
            if let Some(item) = current.take() {
                result.push(item);
            }
            continue;
        }
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "worktree" => {
                if let Some(item) = current.take() {
                    result.push(item);
                }
                current = Some(WorktreeInfo {
                    path: value.to_owned(),
                    head: String::new(),
                    branch: None,
                    detached: false,
                    locked: false,
                    prunable: false,
                });
            }
            "HEAD" => {
                if let Some(item) = current.as_mut() {
                    item.head = value.to_owned();
                }
            }
            "branch" => {
                if let Some(item) = current.as_mut() {
                    item.branch = Some(
                        value
                            .strip_prefix("refs/heads/")
                            .unwrap_or(value)
                            .to_owned(),
                    );
                }
            }
            "detached" => {
                if let Some(item) = current.as_mut() {
                    item.detached = true;
                }
            }
            "locked" => {
                if let Some(item) = current.as_mut() {
                    item.locked = true;
                }
            }
            "prunable" => {
                if let Some(item) = current.as_mut() {
                    item.prunable = true;
                }
            }
            _ => {}
        }
    }
    if let Some(item) = current {
        result.push(item);
    }
    Ok(result)
}
pub fn add(
    repo: &Path,
    path: &str,
    branch: &str,
    new_branch: Option<&str>,
) -> Result<(), GitError> {
    safe_arg(path, "worktree path")?;
    safe_arg(branch, "worktree branch")?;
    let mut args = vec!["worktree", "add"];
    if let Some(name) = new_branch {
        safe_arg(name, "new branch")?;
        args.extend(["-b", name]);
    }
    args.extend(["--", path, branch]);
    oplog::mutate(repo, "worktree.add", || run(repo, &args).map(|_| ()))
}
pub fn remove(repo: &Path, path: &str, force: bool) -> Result<(), GitError> {
    safe_arg(path, "worktree path")?;
    let args = if force {
        vec!["worktree", "remove", "--force", "--", path]
    } else {
        vec!["worktree", "remove", "--", path]
    };
    oplog::mutate(repo, "worktree.remove", || run(repo, &args).map(|_| ()))
}
pub fn prune(repo: &Path) -> Result<(), GitError> {
    oplog::mutate(repo, "worktree.prune", || {
        run(repo, &["worktree", "prune"]).map(|_| ())
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn repo() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("gitstream-wt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        run(&p, &["init", "-q"]).unwrap();
        run(&p, &["config", "user.name", "T"]).unwrap();
        run(&p, &["config", "user.email", "t@e"]).unwrap();
        fs::write(p.join("f"), "x").unwrap();
        run(&p, &["add", "."]).unwrap();
        run(&p, &["commit", "-qm", "init"]).unwrap();
        p
    }
    #[test]
    fn porcelain_lists_branch_worktree() {
        let p = repo();
        let child = p.with_file_name(format!(
            "{}-child",
            p.file_name().unwrap().to_string_lossy()
        ));
        add(&p, child.to_str().unwrap(), "HEAD", Some("side")).unwrap();
        let rows = list(&p).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|w| w.branch.as_deref() == Some("side")));
        remove(&p, child.to_str().unwrap(), true).unwrap();
        let _ = fs::remove_dir_all(p);
    }
}
