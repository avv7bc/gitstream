use super::error::{classify_git_error, GitError};
use super::oplog;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Serialize)]
pub struct ConflictSide {
    pub content: Option<String>,
    pub binary: bool,
    pub size: u64,
    pub oid: Option<String>,
    pub mode: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ConflictSides {
    pub path: String,
    pub base: ConflictSide,
    pub ours: ConflictSide,
    pub theirs: ConflictSide,
    pub merged_draft: Option<String>,
}
fn fail(message: impl Into<String>) -> GitError {
    GitError::CommandFailed {
        message: message.into(),
        hint: None,
    }
}
fn validate(path: &str) -> Result<(), GitError> {
    let p = Path::new(path);
    if path.is_empty()
        || p.is_absolute()
        || p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        || p.components().any(|c| c.as_os_str() == ".git")
        || path.contains('\0')
    {
        return Err(fail("Unsafe conflict path"));
    }
    Ok(())
}
fn git(repo: &Path, args: &[&str]) -> Result<std::process::Output, GitError> {
    Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .map_err(|e| fail(format!("run git resolver: {e}")))
}
fn stage(repo: &Path, stage: u8, path: &str) -> Result<ConflictSide, GitError> {
    let spec = format!(":{stage}:{path}");
    let out = git(repo, &["cat-file", "-s", &spec])?;
    if !out.status.success() {
        return Ok(ConflictSide {
            content: None,
            binary: false,
            size: 0,
            oid: None,
            mode: None,
        });
    }
    let size = String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .unwrap_or(0);
    let obj = git(repo, &["cat-file", "-p", &spec])?;
    if !obj.status.success() {
        return Ok(ConflictSide {
            content: None,
            binary: false,
            size,
            oid: None,
            mode: None,
        });
    }
    let binary = obj.stdout.contains(&0);
    let content = if binary {
        None
    } else {
        Some(String::from_utf8_lossy(&obj.stdout).into_owned())
    };
    let oid = String::from_utf8_lossy(&git(repo, &["rev-parse", &spec])?.stdout)
        .trim()
        .to_owned();
    let mode = String::from_utf8_lossy(&git(repo, &["ls-files", "--stage", "--", path])?.stdout)
        .split_whitespace()
        .next()
        .map(str::to_owned);
    Ok(ConflictSide {
        content,
        binary,
        size,
        oid: Some(oid),
        mode,
    })
}
pub fn get_conflict_sides(repo: &Path, path: &str) -> Result<ConflictSides, GitError> {
    validate(path)?;
    let merged_draft = std::fs::read(repo.join(path)).ok().and_then(|bytes| {
        if bytes.contains(&0) {
            None
        } else {
            Some(String::from_utf8_lossy(&bytes).into_owned())
        }
    });
    Ok(ConflictSides {
        path: path.to_owned(),
        base: stage(repo, 1, path)?,
        ours: stage(repo, 2, path)?,
        theirs: stage(repo, 3, path)?,
        merged_draft,
    })
}
pub fn save_resolved(
    repo: &Path,
    path: &str,
    content: &str,
    mark_resolved: bool,
) -> Result<(), GitError> {
    validate(path)?;
    oplog::mutate(repo, "conflict.save", || {
        let target = repo.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| fail(format!("write resolved file: {e}")))?;
        }
        std::fs::write(&target, content.as_bytes())
            .map_err(|e| fail(format!("write resolved file: {e}")))?;
        if mark_resolved {
            let out = git(repo, &["add", "--", path])?;
            if !out.status.success() {
                return Err(classify_git_error(&String::from_utf8_lossy(&out.stderr)));
            }
        }
        Ok(())
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn repo() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "gitstream-resolver-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        run(&p, &["init", "-q"]);
        run(&p, &["config", "user.name", "T"]);
        run(&p, &["config", "user.email", "t@e"]);
        fs::write(p.join("f"), "base\n").unwrap();
        run(&p, &["add", "."]);
        run(&p, &["commit", "-qm", "base"]);
        p
    }
    fn run(p: &std::path::Path, a: &[&str]) {
        assert!(Command::new("git")
            .current_dir(p)
            .args(a)
            .status()
            .unwrap()
            .success());
    }
    #[test]
    fn text_and_add_add_sides_are_safe() {
        let p = repo();
        run(&p, &["checkout", "-q", "-b", "side"]);
        fs::write(p.join("f"), "theirs\n").unwrap();
        run(&p, &["commit", "-qam", "side"]);
        run(&p, &["checkout", "-q", "-"]);
        fs::write(p.join("f"), "ours\n").unwrap();
        run(&p, &["commit", "-qam", "ours"]);
        let _ = Command::new("git")
            .current_dir(&p)
            .args(["merge", "side"])
            .status();
        let sides = get_conflict_sides(&p, "f").unwrap();
        assert_eq!(sides.base.content.as_deref(), Some("base\n"));
        assert_eq!(sides.ours.content.as_deref(), Some("ours\n"));
        assert_eq!(sides.theirs.content.as_deref(), Some("theirs\n"));
        assert!(get_conflict_sides(&p, "../f").is_err());
        let _ = fs::remove_dir_all(p);
    }
    #[test]
    fn binary_side_has_metadata_without_utf8_loss() {
        let p = repo();
        fs::write(p.join("f"), [0, 255, 1]).unwrap();
        run(&p, &["add", "f"]);
        let side = stage(&p, 0, "f").unwrap();
        assert!(side.binary);
        assert!(side.content.is_none());
        assert_eq!(side.size, 3);
        let _ = fs::remove_dir_all(p);
    }
}
