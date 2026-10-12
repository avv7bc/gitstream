use super::error::{classify_git_error, GitError};
use super::oplog;
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
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
    pub chunks: Vec<ConflictChunk>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ConflictChunk {
    pub ours: String,
    pub base: Option<String>,
    pub theirs: String,
}

/// Extract standard and diff3 conflict blocks from the checked-out draft.
/// Keeping this in the backend gives the resolver stable chunk boundaries even
/// when the UI renders only a subset of a large file.
pub fn extract_conflict_chunks(text: &str) -> Vec<ConflictChunk> {
    let mut chunks = Vec::new();
    let mut ours = String::new();
    let mut base = None;
    let mut theirs = String::new();
    let mut section = None::<&str>;
    for line in text.lines() {
        if line.starts_with("<<<<<<<") {
            ours.clear();
            base = None;
            theirs.clear();
            section = Some("ours");
        } else if line.starts_with("|||||||") && section == Some("ours") {
            base = Some(String::new());
            section = Some("base");
        } else if line == "=======" && section.is_some() {
            section = Some("theirs");
        } else if line.starts_with(">>>>>>>") && section == Some("theirs") {
            chunks.push(ConflictChunk {
                ours: ours.trim_end_matches('\n').to_owned(),
                base: base.take().map(|value| value.trim_end_matches('\n').to_owned()),
                theirs: theirs.trim_end_matches('\n').to_owned(),
            });
            section = None;
        } else if let Some(current) = section {
            match current {
                "ours" => { ours.push_str(line); ours.push('\n'); }
                "base" => {
                    let content = base.get_or_insert_with(String::new);
                    content.push_str(line);
                    content.push('\n');
                }
                "theirs" => { theirs.push_str(line); theirs.push('\n'); }
                _ => {}
            }
        }
    }
    chunks
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

/// Resolve a conflict path without following symlinks out of the repository.
/// `Path::join` alone is insufficient here: a tracked path such as
/// `links/file` can escape through a symlinked parent directory.
fn safe_target(repo: &Path, path: &str) -> Result<(PathBuf, PathBuf), GitError> {
    validate(path)?;
    let root = repo
        .canonicalize()
        .map_err(|e| fail(format!("resolve repository: {e}")))?;
    let target = root.join(path);
    let mut current = root.clone();
    for component in Path::new(path).components() {
        let Component::Normal(name) = component else {
            continue;
        };
        current.push(name);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(fail("Conflict path contains a symlink"));
                }
                if !meta.is_dir() && current != target {
                    return Err(fail("Conflict path parent is not a directory"));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(fail(format!("inspect conflict path: {e}"))),
        }
    }
    if let Ok(meta) = std::fs::symlink_metadata(&target) {
        if meta.file_type().is_symlink() {
            return Err(fail("Cannot overwrite a symlink conflict file"));
        }
        if meta.is_dir() {
            return Err(fail("Conflict path points to a directory"));
        }
    }
    let parent = target.parent().unwrap_or(&root).to_path_buf();
    Ok((target, parent))
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
    let (target, _) = safe_target(repo, path)?;
    let base = stage(repo, 1, path)?;
    let ours = stage(repo, 2, path)?;
    let theirs = stage(repo, 3, path)?;
    let merged_draft = merge_draft(repo, &target, &base, &ours, &theirs);
    let chunks = merged_draft
        .as_deref()
        .map(extract_conflict_chunks)
        .unwrap_or_default();
    Ok(ConflictSides {
        path: path.to_owned(),
        base,
        ours,
        theirs,
        merged_draft,
        chunks,
    })
}

fn merge_draft(
    repo: &Path,
    target: &Path,
    base: &ConflictSide,
    ours: &ConflictSide,
    theirs: &ConflictSide,
) -> Option<String> {
    if base.binary || ours.binary || theirs.binary {
        return None;
    }
    let Some(base_content) = base.content.as_deref() else {
        return std::fs::read(target).ok().and_then(|bytes| {
            (!bytes.contains(&0)).then(|| String::from_utf8_lossy(&bytes).into_owned())
        });
    };
    let root = std::env::temp_dir().join(format!(
        "gitstream-merge-file-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).ok()?;
    let ours_path = root.join("ours");
    let base_path = root.join("base");
    let theirs_path = root.join("theirs");
    let result = (|| {
        std::fs::write(&ours_path, ours.content.as_deref().unwrap_or_default()).ok()?;
        std::fs::write(&base_path, base_content).ok()?;
        std::fs::write(&theirs_path, theirs.content.as_deref().unwrap_or_default()).ok()?;
        let output = Command::new("git")
            .current_dir(repo)
            .args(["merge-file", "-p", "--diff3"])
            .arg(&ours_path)
            .arg(&base_path)
            .arg(&theirs_path)
            .output()
            .ok()?;
        // merge-file exits 1 when conflicts remain, but stdout is still the
        // useful diff3 draft. Codes above 1 indicate invocation failure.
        (output.status.success() || output.status.code() == Some(1))
            .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
    })();
    let _ = std::fs::remove_dir_all(root);
    result
}

pub fn open_mergetool(repo: &Path, path: &str) -> Result<(), GitError> {
    validate(path)?;
    let _ = safe_target(repo, path)?;
    let output = Command::new("git")
        .current_dir(repo)
        .args(["mergetool", "--no-prompt", "--", path])
        .output()
        .map_err(|e| fail(format!("run git mergetool: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(classify_git_error(&String::from_utf8_lossy(&output.stderr)))
    }
}
pub fn save_resolved(
    repo: &Path,
    path: &str,
    content: &str,
    mark_resolved: bool,
) -> Result<(), GitError> {
    let (_target, parent) = safe_target(repo, path)?;
    if content.as_bytes().contains(&0) {
        return Err(fail("Resolved content must be text"));
    }
    if content.is_empty() {
        let sides = get_conflict_sides(repo, path)?;
        if sides.base.binary || sides.ours.binary || sides.theirs.binary {
            return Err(fail("Empty content cannot replace a binary conflict"));
        }
    }
    oplog::mutate(repo, "conflict.save", || {
        std::fs::create_dir_all(&parent).map_err(|e| fail(format!("write resolved file: {e}")))?;
        // Re-check after creating missing parents in case a concurrent change
        // introduced a symlink between validation and the write.
        let (checked_target, _) = safe_target(repo, path)?;
        std::fs::write(&checked_target, content.as_bytes())
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
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_REPO: AtomicU64 = AtomicU64::new(0);
    fn repo() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "gitstream-resolver-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let p = p.with_file_name(format!(
            "{}-{}",
            p.file_name().unwrap().to_string_lossy(),
            NEXT_REPO.fetch_add(1, Ordering::Relaxed)
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
    fn extracts_standard_and_diff3_chunks() {
        let text = "before\n<<<<<<< ours\na\n||||||| base\nb\n=======\nc\n>>>>>>> theirs\nafter\n";
        let chunks = extract_conflict_chunks(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].ours, "a");
        assert_eq!(chunks[0].base.as_deref(), Some("b"));
        assert_eq!(chunks[0].theirs, "c");
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
        assert!(sides.merged_draft.as_deref().unwrap_or_default().contains("|||||||"));
        assert_eq!(sides.chunks.len(), 1);
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

    #[test]
    fn resolved_write_rejects_empty_binary_and_symlink_escape() {
        let p = repo();
        assert!(save_resolved(&p, "f", "", false).is_ok());
        assert!(save_resolved(&p, "f", "text\0data", false).is_err());
        let outside = p
            .parent()
            .unwrap()
            .join(format!("outside-{}", std::process::id()));
        fs::create_dir_all(&outside).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, p.join("escape")).unwrap();
        #[cfg(unix)]
        assert!(save_resolved(&p, "escape/file", "safe", false).is_err());
        let _ = fs::remove_dir_all(&outside);
        let _ = fs::remove_dir_all(p);
    }
}
