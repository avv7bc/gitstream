use super::error::GitError;
use super::query::run_git;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ReflogEntry {
    pub sha: String,
    pub selector: String,
    pub action: String,
    pub timestamp: i64,
    pub subject: String,
}

pub fn get(repo: &Path, refname: Option<&str>, limit: usize) -> Result<Vec<ReflogEntry>, GitError> {
    let name = refname.unwrap_or("HEAD");
    if name.starts_with('-') || name.contains('\0') || name.contains("..") {
        return Err(GitError::CommandFailed {
            message: "Invalid reflog ref name".into(),
            hint: None,
        });
    }
    let limit = limit.clamp(1, 1000).to_string();
    let out = run_git(
        repo,
        &[
            "reflog",
            "show",
            &format!("--max-count={limit}"),
            "--format=%H%x00%gD%x00%gs%x00%ct%x00%s",
            name,
        ],
    )?;
    Ok(out
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\0');
            Some(ReflogEntry {
                sha: fields.next()?.to_string(),
                selector: fields.next()?.to_string(),
                action: fields.next()?.to_string(),
                timestamp: fields.next()?.parse().ok()?,
                subject: fields.next().unwrap_or_default().to_string(),
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn parses_reflog() {
        let dir = std::env::temp_dir().join(format!("gitstream-reflog-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(args)
                .output()
                .unwrap()
        };
        assert!(git(&["init", "-q"]).status.success());
        assert!(git(&["config", "user.name", "T"]).status.success());
        assert!(git(&["config", "user.email", "t@e"]).status.success());
        fs::write(dir.join("f"), "x").unwrap();
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-qm", "first"]).status.success());
        let entries = get(&dir, None, 5).unwrap();
        assert!(!entries.is_empty());
        assert!(entries[0].sha.len() >= 7);
        assert!(entries[0].subject.contains("first"));
        let _ = fs::remove_dir_all(dir);
    }
}
