use std::path::Path;

use super::error::GitError;

fn run_git_mut(repo_path: &Path, args: &[&str]) -> Result<String, GitError> {
    super::query::run_git(repo_path, args)
}

pub fn stage(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    let mut args = vec!["add", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn unstage(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    let mut args = vec!["restore", "--staged", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn discard(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    let mut args = vec!["restore", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn commit(repo_path: &Path, message: &str, amend: bool) -> Result<String, GitError> {
    let mut args = vec!["commit", "-m", message];
    if amend { args.push("--amend"); }
    run_git_mut(repo_path, &args)
}

pub fn checkout(repo_path: &Path, branch: &str) -> Result<(), GitError> {
    let result = run_git_mut(repo_path, &["switch", branch]);
    if result.is_ok() { return Ok(()); }
    if let Some(local) = branch.split('/').last() {
        run_git_mut(repo_path, &["switch", "-c", local, branch])?;
        return Ok(());
    }
    result.map(|_| ())
}

pub fn checkout_remote(repo_path: &Path, remote_branch: &str, local_name: Option<&str>) -> Result<(), GitError> {
    match local_name {
        Some(local) => {
            run_git_mut(repo_path, &["switch", "-c", local, "--track", remote_branch])?;
        }
        None => {
            run_git_mut(repo_path, &["checkout", "--detach", remote_branch])?;
        }
    }
    Ok(())
}

pub fn merge(repo_path: &Path, branch: &str) -> Result<String, GitError> {
    run_git_mut(repo_path, &["merge", branch])
}

pub fn rename_branch(repo_path: &Path, old_name: &str, new_name: &str) -> Result<(), GitError> {
    run_git_mut(repo_path, &["branch", "-m", old_name, new_name])?;
    Ok(())
}

pub fn delete_branch(repo_path: &Path, branch: &str, force: bool) -> Result<(), GitError> {
    let flag = if force { "-D" } else { "-d" };
    run_git_mut(repo_path, &["branch", flag, branch])?;
    Ok(())
}

pub fn create_tag(
    repo_path: &Path,
    name: &str,
    message: Option<&str>,
    target: Option<&str>,
    force: bool,
) -> Result<(), GitError> {
    let mut args: Vec<&str> = vec!["tag"];
    if force {
        args.push("-f");
    }
    if let Some(msg) = message {
        args.push("-a");
        args.push("-m");
        args.push(msg);
    }
    args.push(name);
    if let Some(t) = target {
        args.push(t);
    }
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn delete_tag(repo_path: &Path, name: &str) -> Result<(), GitError> {
    run_git_mut(repo_path, &["tag", "-d", name])?;
    Ok(())
}

pub fn fetch_args(remote: &str) -> Vec<String> {
    vec!["fetch".into(), remote.into()]
}

pub fn pull_args(remote: &str, rebase: bool) -> Vec<String> {
    if rebase {
        vec!["pull".into(), "--rebase".into(), remote.into()]
    } else {
        vec!["pull".into(), remote.into()]
    }
}

pub fn push_args(remote: &str, force: bool) -> Vec<String> {
    if force {
        vec!["push".into(), "--force".into(), remote.into()]
    } else {
        vec!["push".into(), remote.into()]
    }
}

pub fn push_branch_args(remote: &str, branch: &str, force: bool) -> Vec<String> {
    if force {
        vec!["push".into(), "--force".into(), remote.into(), branch.into()]
    } else {
        vec!["push".into(), remote.into(), branch.into()]
    }
}

pub fn push_tag_args(remote: &str, name: &str, delete: bool) -> Vec<String> {
    let refspec = if delete {
        format!(":refs/tags/{}", name)
    } else {
        format!("refs/tags/{}", name)
    };
    vec!["push".into(), remote.into(), refspec]
}

pub fn clone_args(url: &str, dest: &str) -> Vec<String> {
    vec!["clone".into(), url.into(), dest.into()]
}

#[cfg(test)]
mod tag_tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn temp_repo() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_tag_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            Command::new("git").current_dir(&dir).args(args).output().unwrap();
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t.t"]);
        run(&["config", "user.name", "t"]);
        fs::write(dir.join("a.txt"), "hello").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "init"]);
        dir
    }

    fn list_tags(dir: &std::path::Path) -> String {
        let out = Command::new("git")
            .current_dir(dir)
            .args(["tag", "-l"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    #[test]
    fn creates_lightweight_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        assert!(list_tags(&dir).contains("v1.0"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn creates_annotated_tag_with_message() {
        let dir = temp_repo();
        create_tag(&dir, "v2.0", Some("release two"), None, false).unwrap();
        let out = Command::new("git")
            .current_dir(&dir)
            .args(["cat-file", "-t", "v2.0"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "tag");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn force_overwrites_existing_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        assert!(create_tag(&dir, "v1.0", None, None, false).is_err());
        create_tag(&dir, "v1.0", None, None, true).unwrap();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn deletes_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        delete_tag(&dir, "v1.0").unwrap();
        assert!(!list_tags(&dir).contains("v1.0"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fetch_args_basic() {
        assert_eq!(fetch_args("origin"), vec!["fetch", "origin"]);
    }

    #[test]
    fn pull_args_rebase_toggle() {
        assert_eq!(pull_args("origin", false), vec!["pull", "origin"]);
        assert_eq!(pull_args("origin", true), vec!["pull", "--rebase", "origin"]);
    }

    #[test]
    fn push_args_force_toggle() {
        assert_eq!(push_args("origin", false), vec!["push", "origin"]);
        assert_eq!(push_args("origin", true), vec!["push", "--force", "origin"]);
    }

    #[test]
    fn push_branch_args_force_toggle() {
        assert_eq!(push_branch_args("origin", "main", false), vec!["push", "origin", "main"]);
        assert_eq!(
            push_branch_args("origin", "main", true),
            vec!["push", "--force", "origin", "main"]
        );
    }

    #[test]
    fn push_tag_args_delete_toggle() {
        assert_eq!(
            push_tag_args("origin", "v1.0", false),
            vec!["push", "origin", "refs/tags/v1.0"]
        );
        assert_eq!(
            push_tag_args("origin", "v1.0", true),
            vec!["push", "origin", ":refs/tags/v1.0"]
        );
    }

    #[test]
    fn clone_args_basic() {
        assert_eq!(
            clone_args("https://x/y.git", "/tmp/y"),
            vec!["clone", "https://x/y.git", "/tmp/y"]
        );
    }
}
