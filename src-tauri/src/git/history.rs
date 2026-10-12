use super::error::{classify_git_error, GitError};
use super::query::run_git;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RebaseAction {
    Pick,
    Reword,
    Edit,
    Squash,
    Fixup,
    Drop,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TodoItem {
    pub sha: String,
    pub action: RebaseAction,
    #[serde(default)]
    pub message: Option<String>,
}

fn fail(message: impl Into<String>) -> GitError {
    GitError::CommandFailed {
        message: message.into(),
        hint: None,
    }
}
fn valid_sha(sha: &str) -> bool {
    !sha.is_empty() && sha.chars().all(|c| c.is_ascii_hexdigit())
}
fn action_name(action: &RebaseAction) -> &'static str {
    match action {
        RebaseAction::Pick => "pick",
        RebaseAction::Reword => "reword",
        RebaseAction::Edit => "edit",
        RebaseAction::Squash => "squash",
        RebaseAction::Fixup => "fixup",
        RebaseAction::Drop => "drop",
    }
}

/// Execute one complete interactive-rebase plan. The sequence editor only writes
/// the validated todo supplied by the caller; Git still verifies commit ancestry.
fn rebase_plan_inner(
    repo: &Path,
    base: Option<&str>,
    plan: &[TodoItem],
) -> Result<String, GitError> {
    if plan.is_empty() {
        return Err(fail("Interactive rebase plan cannot be empty"));
    }
    let mut lines = Vec::with_capacity(plan.len());
    for item in plan {
        if !valid_sha(&item.sha) {
            return Err(fail(format!(
                "Invalid commit id in rebase plan: {}",
                item.sha
            )));
        }
        let full = run_git(
            repo,
            &["rev-parse", "--verify", &format!("{}^{{commit}}", item.sha)],
        )?;
        let subject = item.message.clone().unwrap_or_else(|| {
            run_git(repo, &["show", "-s", "--format=%s", &full]).unwrap_or_default()
        });
        let subject = subject.replace(['\n', '\r'], " ");
        lines.push(format!(
            "{} {} {}\n",
            action_name(&item.action),
            full.trim(),
            subject
        ));
    }
    let stamp = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| fail(e.to_string()))?
            .as_nanos()
    );
    let dir = std::env::temp_dir();
    let todo = dir.join(format!("gitstream-rebase-{stamp}.todo"));
    let editor = dir.join(format!("gitstream-rebase-{stamp}.sh"));
    std::fs::write(&todo, lines.concat()).map_err(|e| fail(format!("write rebase plan: {e}")))?;
    let script = format!("#!/bin/sh\ncat '{}' > \"$1\"\n", todo.display());
    std::fs::write(&editor, script).map_err(|e| fail(format!("write sequence editor: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&editor, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| fail(format!("chmod sequence editor: {e}")))?;
    }
    let mut args = vec!["rebase", "-i", "--autostash", "--update-refs"];
    if let Some(base) = base {
        if !valid_sha(base) {
            return Err(fail(format!("Invalid rebase base: {base}")));
        }
        args.push(base);
    } else {
        args.push("--root");
    }
    let out = Command::new("git")
        .current_dir(repo)
        .env("GIT_SEQUENCE_EDITOR", &editor)
        .env("GIT_EDITOR", "true")
        .args(&args)
        .output()
        .map_err(|e| fail(format!("run interactive rebase: {e}")))?;
    let _ = std::fs::remove_file(&todo);
    let _ = std::fs::remove_file(&editor);
    if !out.status.success() {
        return Err(classify_git_error(&String::from_utf8_lossy(&out.stderr)));
    }
    Ok(run_git(repo, &["rev-parse", "HEAD"])
        .unwrap_or_default()
        .trim()
        .to_owned())
}

pub fn rebase_plan(repo: &Path, base: Option<&str>, plan: &[TodoItem]) -> Result<String, GitError> {
    super::oplog::mutate(repo, "rebase.plan", || rebase_plan_inner(repo, base, plan))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn repo() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("gitstream-history-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        run(&p, &["init", "-q"]);
        run(&p, &["config", "user.name", "T"]);
        run(&p, &["config", "user.email", "t@e"]);
        for msg in ["one", "two", "three"] {
            run(&p, &["commit", "--allow-empty", "-qm", msg]);
        }
        p
    }
    fn run(p: &std::path::Path, args: &[&str]) {
        assert!(std::process::Command::new("git")
            .current_dir(p)
            .args(args)
            .status()
            .unwrap()
            .success());
    }
    fn commits(p: &std::path::Path) -> Vec<String> {
        run_git(p, &["log", "--format=%H", "--reverse"])
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }
    #[test]
    fn root_plan_reorders_and_drops() {
        let p = repo();
        let ids = commits(&p);
        rebase_plan(
            &p,
            None,
            &[
                TodoItem {
                    sha: ids[0].clone(),
                    action: RebaseAction::Pick,
                    message: None,
                },
                TodoItem {
                    sha: ids[2].clone(),
                    action: RebaseAction::Pick,
                    message: None,
                },
                TodoItem {
                    sha: ids[1].clone(),
                    action: RebaseAction::Drop,
                    message: None,
                },
            ],
        )
        .unwrap();
        let subjects = run_git(&p, &["log", "--format=%s", "--reverse"]).unwrap();
        assert!(subjects.lines().eq(["one", "three"]));
        let _ = fs::remove_dir_all(p);
    }
}
