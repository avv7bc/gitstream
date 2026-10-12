use super::error::{classify_git_error, GitError};
use super::query::run_git;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
fn valid_revision(sha: &str) -> bool {
    !sha.is_empty() && !sha.starts_with('-') && !sha.contains('\0')
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
    allow_pushed: bool,
) -> Result<String, GitError> {
    if plan.is_empty() {
        return Err(fail("Interactive rebase plan cannot be empty"));
    }
    let mut resolved = Vec::with_capacity(plan.len());
    let mut seen = HashSet::new();
    for item in plan {
        if !valid_revision(&item.sha) {
            return Err(fail(format!(
                "Invalid commit id in rebase plan: {}",
                item.sha
            )));
        }
        let full = run_git(
            repo,
            &["rev-parse", "--verify", &format!("{}^{{commit}}", item.sha)],
        )?
        .trim()
        .to_owned();
        if !seen.insert(full.clone()) {
            return Err(fail(format!(
                "Duplicate commit in rebase plan: {}",
                item.sha
            )));
        }
        resolved.push((full, item));
    }

    let range = if let Some(base) = base {
        if !valid_revision(base) {
            return Err(fail(format!("Invalid rebase base: {base}")));
        }
        let base_full = run_git(
            repo,
            &["rev-parse", "--verify", &format!("{}^{{commit}}", base)],
        )?
        .trim()
        .to_owned();
        if run_git(repo, &["merge-base", "--is-ancestor", &base_full, "HEAD"]).is_err() {
            return Err(fail(format!(
                "Rebase base is not an ancestor of HEAD: {base}"
            )));
        }
        format!("{base_full}..HEAD")
    } else {
        "HEAD".to_owned()
    };
    let expected: HashSet<String> =
        run_git(repo, &["rev-list", "--reverse", "--first-parent", &range])?
            .lines()
            .map(str::trim)
            .filter(|sha| !sha.is_empty())
            .map(str::to_owned)
            .collect();
    if !run_git(repo, &["rev-list", "--merges", &range])?
        .trim()
        .is_empty()
    {
        return Err(fail(
            "Interactive rebase currently requires a linear history range",
        ));
    }
    let planned: HashSet<String> = resolved.iter().map(|(sha, _)| sha.clone()).collect();
    if expected != planned {
        return Err(fail(format!(
            "Rebase plan does not exactly cover the requested range (missing: {}, extra: {})",
            expected
                .difference(&planned)
                .next()
                .map(String::as_str)
                .unwrap_or("none"),
            planned
                .difference(&expected)
                .next()
                .map(String::as_str)
                .unwrap_or("none")
        )));
    }
    for (full, item) in &resolved {
        let remotes = run_git(repo, &["branch", "-r", "--contains", full])?;
        if !allow_pushed && remotes.lines().any(|line| !line.trim().is_empty()) {
            return Err(fail(format!(
                "Refusing to rewrite pushed commit {} ({})",
                &full[..full.len().min(12)],
                item.sha
            )));
        }
    }

    let mut lines = Vec::with_capacity(plan.len());
    for (full, item) in resolved {
        let subject = item.message.clone().unwrap_or_else(|| {
            run_git(repo, &["show", "-s", "--format=%s", &full]).unwrap_or_default()
        });
        let subject = subject.replace(['\n', '\r'], " ");
        lines.push(format!(
            "{} {} {}\n",
            action_name(&item.action),
            full,
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
    let editor = dir.join(format!("gitstream-rebase-{stamp}.sequence.sh"));
    let message_editor = dir.join(format!("gitstream-rebase-{stamp}.message.sh"));
    let queue = dir.join(format!("gitstream-rebase-{stamp}.messages"));
    let mut queue_data = String::new();
    let mut message_files = Vec::new();
    for (index, item) in plan.iter().enumerate() {
        if item.action == RebaseAction::Reword {
            if let Some(message) = &item.message {
                let path = dir.join(format!("gitstream-rebase-{stamp}-msg-{index}"));
                std::fs::write(&path, message)
                    .map_err(|e| fail(format!("write reword message: {e}")))?;
                queue_data.push_str(&format!("{}\n", path.display()));
                message_files.push(path);
            } else {
                queue_data.push_str("-\n");
            }
        }
    }
    std::fs::write(&todo, lines.concat()).map_err(|e| fail(format!("write rebase plan: {e}")))?;
    std::fs::write(&queue, queue_data).map_err(|e| fail(format!("write reword queue: {e}")))?;
    let script = format!("#!/bin/sh\ncat '{}' > \"$1\"\n", todo.display());
    std::fs::write(&editor, script).map_err(|e| fail(format!("write sequence editor: {e}")))?;
    let message_script = format!(
        "#!/bin/sh\nif [ -s '{}' ]; then entry=$(head -n 1 '{}'); tail -n +2 '{}' > '{}.tmp' && mv '{}.tmp' '{}'; if [ \"$entry\" != - ]; then cat \"$entry\" > \"$1\"; fi; fi\n",
        queue.display(), queue.display(), queue.display(), queue.display(), queue.display(), queue.display()
    );
    std::fs::write(&message_editor, message_script)
        .map_err(|e| fail(format!("write message editor: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&editor, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| fail(format!("chmod sequence editor: {e}")))?;
        std::fs::set_permissions(&message_editor, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| fail(format!("chmod message editor: {e}")))?;
    }
    let mut args = vec![
        "rebase",
        "-i",
        "--autostash",
        "--update-refs",
        "--keep-empty",
    ];
    if let Some(base) = base {
        args.push(base);
    } else {
        args.push("--root");
    }
    let out = Command::new("git")
        .current_dir(repo)
        .env("GIT_SEQUENCE_EDITOR", &editor)
        .env("GIT_EDITOR", &message_editor)
        .args(&args)
        .output()
        .map_err(|e| fail(format!("run interactive rebase: {e}")))?;
    let _ = std::fs::remove_file(&todo);
    let _ = std::fs::remove_file(&editor);
    let _ = std::fs::remove_file(&message_editor);
    let _ = std::fs::remove_file(&queue);
    for path in message_files {
        let _ = std::fs::remove_file(path);
    }
    if !out.status.success() {
        return Err(classify_git_error(&String::from_utf8_lossy(&out.stderr)));
    }
    Ok(run_git(repo, &["rev-parse", "HEAD"])
        .unwrap_or_default()
        .trim()
        .to_owned())
}

pub fn rebase_plan(
    repo: &Path,
    base: Option<&str>,
    plan: &[TodoItem],
    allow_pushed: bool,
) -> Result<String, GitError> {
    super::oplog::mutate(repo, "rebase.plan", || {
        rebase_plan_inner(repo, base, plan, allow_pushed)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn repo() -> std::path::PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "gitstream-history-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        run(&p, &["init", "-q"]);
        run(&p, &["config", "user.name", "T"]);
        run(&p, &["config", "user.email", "t@e"]);
        for (index, msg) in ["one", "two", "three"].iter().enumerate() {
            fs::write(p.join(format!("f{index}")), msg).unwrap();
            run(&p, &["add", "."]);
            run(&p, &["commit", "-qm", msg]);
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
            false,
        )
        .unwrap();
        let subjects = run_git(&p, &["log", "--format=%s", "--reverse"]).unwrap();
        assert!(subjects.lines().eq(["one", "three"]));
        let _ = fs::remove_dir_all(p);
    }

    #[test]
    fn rejects_duplicate_and_incomplete_plans() {
        let p = repo();
        let ids = commits(&p);
        let duplicate = vec![
            TodoItem {
                sha: ids[0].clone(),
                action: RebaseAction::Pick,
                message: None,
            },
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
        ];
        assert!(rebase_plan(&p, None, &duplicate, false).is_err());
        let incomplete = vec![TodoItem {
            sha: ids[0].clone(),
            action: RebaseAction::Pick,
            message: None,
        }];
        assert!(rebase_plan(&p, None, &incomplete, false).is_err());
        let _ = fs::remove_dir_all(p);
    }

    #[test]
    fn reword_writes_requested_message() {
        let p = repo();
        let ids = commits(&p);
        let mut plan = ids
            .iter()
            .map(|sha| TodoItem {
                sha: sha.clone(),
                action: RebaseAction::Pick,
                message: None,
            })
            .collect::<Vec<_>>();
        plan[1].action = RebaseAction::Reword;
        plan[1].message = Some("updated message".into());
        rebase_plan(&p, None, &plan, false).unwrap();
        let subjects = run_git(&p, &["log", "--format=%s", "--reverse"]).unwrap();
        assert!(subjects.lines().eq(["one", "updated message", "three"]));
        let _ = fs::remove_dir_all(p);
    }

    #[test]
    fn rejects_pushed_commit() {
        let p = repo();
        let ids = commits(&p);
        run(&p, &["update-ref", "refs/remotes/origin/main", &ids[1]]);
        let plan = ids
            .iter()
            .map(|sha| TodoItem {
                sha: sha.clone(),
                action: RebaseAction::Pick,
                message: None,
            })
            .collect::<Vec<_>>();
        let error = rebase_plan(&p, None, &plan, false).unwrap_err().to_string();
        assert!(error.contains("pushed"));
        let _ = fs::remove_dir_all(p);
    }

    #[test]
    fn allows_pushed_commit_only_when_explicitly_requested() {
        let p = repo();
        let ids = commits(&p);
        run(&p, &["update-ref", "refs/remotes/origin/main", &ids[1]]);
        let plan = ids
            .iter()
            .map(|sha| TodoItem {
                sha: sha.clone(),
                action: RebaseAction::Pick,
                message: None,
            })
            .collect::<Vec<_>>();
        assert!(rebase_plan(&p, None, &plan, true).is_ok());
        let _ = fs::remove_dir_all(p);
    }
}
