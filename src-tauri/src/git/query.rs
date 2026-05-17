use std::path::Path;
use std::process::Command;

use super::error::{classify_git_error, GitError};
use super::types::*;

pub fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .output()
        .map_err(|e| GitError::CommandFailed {
            message: format!("Failed to run git: {}", e),
            hint: Some("Is git installed and in PATH?".into()),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(classify_git_error(&stderr))
    }
}

/// Запускает git, возвращая stdout независимо от кода возврата.
/// Нужно для `diff --no-index`, который при наличии различий выходит с кодом 1.
fn run_git_lenient(repo_path: &Path, args: &[&str]) -> String {
    Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Файл не отслеживается git (нет ни в индексе, ни в HEAD).
fn is_untracked(repo_path: &Path, file: &str) -> bool {
    run_git(repo_path, &["ls-files", "--error-unmatch", "--", file]).is_err()
}

pub fn status(repo_path: &Path) -> Result<Vec<FileStatus>, GitError> {
    let output = run_git(repo_path, &["status", "--porcelain=v2"])?;
    let mut files = Vec::new();
    for line in output.lines() {
        if line.starts_with('1') || line.starts_with('2') {
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() < 9 { continue; }
            let xy = parts[1];
            let x = xy.as_bytes()[0] as char;
            let y = xy.as_bytes()[1] as char;
            let path = if line.starts_with('2') {
                parts[8].split('\t').nth(1).unwrap_or(parts[8]).to_string()
            } else {
                parts[8].to_string()
            };
            let (state, staged) = match (x, y) {
                ('M', '.') => ("modified", "staged"),
                ('.', 'M') => ("modified", "unstaged"),
                ('M', 'M') => ("modified", "partial"),
                ('A', '.') => ("added", "staged"),
                ('.', 'A') => ("added", "unstaged"),
                ('D', '.') => ("deleted", "staged"),
                ('.', 'D') => ("deleted", "unstaged"),
                ('R', '.') => ("renamed", "staged"),
                ('R', 'M') => ("renamed", "partial"),
                _ if xy.contains('U') || xy == "AA" || xy == "DD" => ("conflicted", "unstaged"),
                _ => ("modified", "unstaged"),
            };
            files.push(FileStatus { path, state: state.to_string(), staged: staged.to_string() });
        } else if line.starts_with('?') {
            let path = line[2..].to_string();
            files.push(FileStatus { path, state: "untracked".to_string(), staged: "unstaged".to_string() });
        }
    }
    Ok(files)
}

pub fn log(repo_path: &Path, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
    let format = "%H%x00%h%x00%s%x00%an%x00%ae%x00%aI%x00%P%x00%D";
    let limit_str = format!("-{}", limit);
    let output = run_git(repo_path, &["log", &format!("--format={}", format), &limit_str])?;
    let mut commits = Vec::new();
    for line in output.lines() {
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(8, '\0').collect();
        if parts.len() < 8 { continue; }
        let refs = parse_ref_labels(parts[7]);
        let parents: Vec<String> = parts[6].split_whitespace().map(|s| s.to_string()).collect();
        commits.push(CommitInfo {
            oid: parts[0].to_string(), short_oid: parts[1].to_string(),
            message: parts[2].to_string(), author: parts[3].to_string(),
            author_email: parts[4].to_string(), date: parts[5].to_string(),
            parents, refs,
            column: 0,
            lines: Vec::new(),
        });
    }
    super::graph::assign_lanes(&mut commits);
    Ok(commits)
}

fn parse_ref_labels(raw: &str) -> Vec<RefLabel> {
    if raw.trim().is_empty() { return Vec::new(); }
    raw.split(", ").filter_map(|r| {
        let r = r.trim();
        if r.is_empty() { return None; }
        if r == "HEAD" { return Some(RefLabel { name: "HEAD".to_string(), kind: "head".to_string() }); }
        if let Some(rest) = r.strip_prefix("HEAD -> ") {
            return Some(RefLabel { name: rest.to_string(), kind: "current-branch".to_string() });
        }
        if let Some(t) = r.strip_prefix("tag: ") {
            Some(RefLabel { name: t.to_string(), kind: "tag".to_string() })
        } else if r.contains('/') {
            Some(RefLabel { name: r.to_string(), kind: "remote-branch".to_string() })
        } else {
            Some(RefLabel { name: r.to_string(), kind: "local-branch".to_string() })
        }
    }).collect()
}

pub fn branches(repo_path: &Path) -> Result<Vec<BranchInfo>, GitError> {
    let format = "%(refname:short)%00%(upstream:short)%00%(upstream:track,nobracket)%00%(HEAD)";
    let output = run_git(repo_path, &["branch", "-a", &format!("--format={}", format)])?;
    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(4, '\0').collect();
        if parts.len() < 4 { continue; }
        let name = parts[0].to_string();
        if name.contains("HEAD") && name.contains("->") { continue; }
        let is_remote = name.starts_with("origin/") || name.contains('/');
        let upstream = if parts[1].is_empty() { None } else { Some(parts[1].to_string()) };
        let (ahead, behind) = parse_track(parts[2]);
        let is_current = parts[3].trim() == "*";
        result.push(BranchInfo { name, is_remote, upstream, ahead, behind, is_current });
    }
    Ok(result)
}

fn parse_track(track: &str) -> (u32, u32) {
    let mut ahead = 0u32;
    let mut behind = 0u32;
    for part in track.split(", ") {
        let part = part.trim();
        if part.starts_with("ahead ") { ahead = part[6..].parse().unwrap_or(0); }
        else if part.starts_with("behind ") { behind = part[7..].parse().unwrap_or(0); }
    }
    (ahead, behind)
}

pub fn tags(repo_path: &Path) -> Result<Vec<TagInfo>, GitError> {
    let format = "%(refname:short)%00%(*objectname:short)%00%(contents:subject)";
    let output = run_git(repo_path, &["tag", "-l", &format!("--format={}", format)])?;
    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(3, '\0').collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let oid = parts.get(1).unwrap_or(&"").to_string();
        let message = parts.get(2).map(|s| s.to_string()).filter(|s| !s.is_empty());
        result.push(TagInfo { name, oid, message });
    }
    Ok(result)
}

pub fn stashes(repo_path: &Path) -> Result<Vec<StashEntry>, GitError> {
    let output = match run_git(repo_path, &["stash", "list", "--format=%gd%x00%gs%x00%ar"]) {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };
    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(3, '\0').collect();
        let index_str = parts.first().unwrap_or(&"");
        let index = index_str.strip_prefix("stash@{").and_then(|s| s.strip_suffix('}')).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        let message = parts.get(1).unwrap_or(&"").to_string();
        let date = parts.get(2).unwrap_or(&"").to_string();
        result.push(StashEntry { index, message, date });
    }
    Ok(result)
}

pub fn remotes(repo_path: &Path) -> Result<Vec<String>, GitError> {
    let output = run_git(repo_path, &["remote"])?;
    Ok(output.lines().filter(|l| !l.is_empty()).map(|l| l.to_string()).collect())
}

/// Состояние репозитория: незавершённая merge/rebase/cherry-pick/revert.
/// Возвращает "clean" | "merging" | "rebasing" | "cherry-picking" | "reverting".
pub fn repo_state(repo_path: &Path) -> Result<String, GitError> {
    let git_dir_raw = run_git(repo_path, &["rev-parse", "--git-dir"])?;
    let git_dir = git_dir_raw.trim();
    let base = Path::new(repo_path).join(git_dir);
    let exists = |p: &str| base.join(p).exists();

    let state = if exists("rebase-merge") || exists("rebase-apply") {
        "rebasing"
    } else if exists("CHERRY_PICK_HEAD") {
        "cherry-picking"
    } else if exists("REVERT_HEAD") {
        "reverting"
    } else if exists("MERGE_HEAD") {
        "merging"
    } else {
        "clean"
    };
    Ok(state.to_string())
}

pub fn repo_info(repo_path: &Path) -> Result<RepoInfo, GitError> {
    let path = run_git(repo_path, &["rev-parse", "--show-toplevel"])?;
    let branch = run_git(repo_path, &["branch", "--show-current"])?;
    let head = run_git(repo_path, &["rev-parse", "HEAD"]).unwrap_or_default();
    Ok(RepoInfo {
        path: path.trim().to_string(),
        current_branch: branch.trim().to_string(),
        head_oid: head.trim().to_string(),
    })
}

pub fn diff_file(repo_path: &Path, file: &str, staged: bool) -> Result<FileDiff, GitError> {
    // Untracked-файл отсутствует в индексе/HEAD — обычный `git diff` пуст.
    // Синтезируем дифф «всё добавлено» сравнением с /dev/null.
    if !staged && is_untracked(repo_path, file) {
        let output = run_git_lenient(repo_path, &["diff", "--no-index", "--", "/dev/null", file]);
        return Ok(parse_diff_single(&output, file));
    }
    let args = if staged {
        vec!["diff", "--cached", "--", file]
    } else {
        vec!["diff", "--", file]
    };
    let output = run_git(repo_path, &args).unwrap_or_default();
    Ok(parse_diff_single(&output, file))
}

pub fn diff_commit(repo_path: &Path, oid: &str) -> Result<Vec<FileDiff>, GitError> {
    let range = format!("{}^..{}", oid, oid);
    let output = run_git(repo_path, &["diff", &range])?;
    Ok(parse_diff_multi(&output))
}

fn parse_diff_single(diff_text: &str, fallback_path: &str) -> FileDiff {
    let mut hunks = Vec::new();
    let mut current_lines: Vec<DiffLine> = Vec::new();
    let mut current_header = String::new();
    let mut current_raw = String::new();
    let mut patch_header = String::new();
    let mut seen_hunk = false;
    let mut insertions = 0u32;
    let mut deletions = 0u32;
    let mut old_line = 0u32;
    let mut new_line = 0u32;
    let mut path = fallback_path.to_string();

    let push_hunk =
        |hunks: &mut Vec<DiffHunk>, header: &str, lines: &mut Vec<DiffLine>, raw: &mut String| {
            hunks.push(DiffHunk {
                header: header.to_string(),
                lines: std::mem::take(lines),
                raw: std::mem::take(raw),
            });
        };

    for line in diff_text.lines() {
        if line.starts_with("+++ b/") {
            path = line[6..].to_string();
        }
        if line.starts_with("@@ ") {
            if !current_header.is_empty() {
                push_hunk(&mut hunks, &current_header, &mut current_lines, &mut current_raw);
            }
            seen_hunk = true;
            current_header = line.to_string();
            current_raw.push_str(line);
            current_raw.push('\n');
            if let Some(nums) = line.strip_prefix("@@ ") {
                let parts: Vec<&str> = nums.split(' ').collect();
                if parts.len() >= 2 {
                    old_line = parts[0].trim_start_matches('-').split(',').next().and_then(|s| s.parse().ok()).unwrap_or(1);
                    new_line = parts[1].trim_start_matches('+').split(',').next().and_then(|s| s.parse().ok()).unwrap_or(1);
                }
            }
        } else if !seen_hunk {
            patch_header.push_str(line);
            patch_header.push('\n');
        } else if line.starts_with('+') && !line.starts_with("+++") {
            insertions += 1;
            current_lines.push(DiffLine { kind: "added".to_string(), old_lineno: None, new_lineno: Some(new_line), content: line[1..].to_string() });
            new_line += 1;
            current_raw.push_str(line);
            current_raw.push('\n');
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
            current_lines.push(DiffLine { kind: "removed".to_string(), old_lineno: Some(old_line), new_lineno: None, content: line[1..].to_string() });
            old_line += 1;
            current_raw.push_str(line);
            current_raw.push('\n');
        } else if line.starts_with(' ') {
            current_lines.push(DiffLine { kind: "context".to_string(), old_lineno: Some(old_line), new_lineno: Some(new_line), content: line[1..].to_string() });
            old_line += 1;
            new_line += 1;
            current_raw.push_str(line);
            current_raw.push('\n');
        } else if line.starts_with('\\') {
            // "\ No newline at end of file" — часть тела хунка
            current_raw.push_str(line);
            current_raw.push('\n');
        }
    }
    if !current_header.is_empty() {
        push_hunk(&mut hunks, &current_header, &mut current_lines, &mut current_raw);
    }
    FileDiff { path, hunks, insertions, deletions, header: patch_header }
}

fn parse_diff_multi(diff_text: &str) -> Vec<FileDiff> {
    let mut diffs = Vec::new();
    let mut current_chunk = String::new();
    let mut current_path = String::new();
    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            if !current_chunk.is_empty() {
                diffs.push(parse_diff_single(&current_chunk, &current_path));
            }
            current_chunk = String::new();
            current_path = line.split(" b/").last().unwrap_or("").to_string();
        }
        current_chunk.push_str(line);
        current_chunk.push('\n');
    }
    if !current_chunk.is_empty() {
        diffs.push(parse_diff_single(&current_chunk, &current_path));
    }
    diffs
}

#[cfg(test)]
mod ref_label_tests {
    use super::*;

    fn kinds(raw: &str) -> Vec<(String, String)> {
        parse_ref_labels(raw)
            .into_iter()
            .map(|r| (r.name, r.kind))
            .collect()
    }

    #[test]
    fn current_branch_from_head_arrow() {
        assert_eq!(
            kinds("HEAD -> main"),
            vec![("main".to_string(), "current-branch".to_string())]
        );
    }

    #[test]
    fn standalone_head_is_head() {
        assert_eq!(
            kinds("HEAD"),
            vec![("HEAD".to_string(), "head".to_string())]
        );
    }

    #[test]
    fn tag_remote_local_kinds() {
        assert_eq!(kinds("tag: v1.0"), vec![("v1.0".to_string(), "tag".to_string())]);
        assert_eq!(kinds("origin/main"), vec![("origin/main".to_string(), "remote-branch".to_string())]);
        assert_eq!(kinds("dev"), vec![("dev".to_string(), "local-branch".to_string())]);
    }

    #[test]
    fn combined_decoration() {
        assert_eq!(
            kinds("HEAD -> main, tag: v1, origin/main"),
            vec![
                ("main".to_string(), "current-branch".to_string()),
                ("v1".to_string(), "tag".to_string()),
                ("origin/main".to_string(), "remote-branch".to_string()),
            ]
        );
    }
}

#[cfg(test)]
mod diff_untracked_tests {
    use super::*;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        Command::new("git").arg("-C").arg(dir).args(args).output().unwrap();
    }

    #[test]
    fn untracked_file_diff_shows_content() {
        let dir = std::env::temp_dir().join(format!("gitstream_untracked_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        fs::write(dir.join("new.txt"), "alpha\nbeta\ngamma\n").unwrap();

        let diff = diff_file(&dir, "new.txt", false).unwrap();

        let added: Vec<&str> = diff
            .hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.kind == "added")
            .map(|l| l.content.as_str())
            .collect();
        assert_eq!(added, vec!["alpha", "beta", "gamma"]);
        assert_eq!(diff.path, "new.txt");

        let _ = fs::remove_dir_all(&dir);
    }
}
