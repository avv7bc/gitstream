# GitStream Backend MVP — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Подключить git CLI backend через Tauri IPC, заменить моковые данные на реальные во всех компонентах.

**Architecture:** Rust backend (git CLI обёртка) → Tauri commands → Vue composables (ref + invoke) → компоненты.

**Tech Stack:** Rust, Tauri 2, git CLI, Vue 3 Composition API, TypeScript

---

## File Structure

### Rust backend (создать)
- `src-tauri/src/git/mod.rs` — модуль git
- `src-tauri/src/git/types.rs` — Serialize-структуры: FileStatus, CommitInfo, BranchInfo, TagInfo, StashEntry, DiffHunk, DiffLine, FileDiff, RepoInfo, RefLabel
- `src-tauri/src/git/error.rs` — GitError enum, classify_git_error(), Serialize impl
- `src-tauri/src/git/query.rs` — run_git(), status(), log(), branches(), tags(), stashes(), remotes(), diff_file(), show_commit(), repo_info()
- `src-tauri/src/git/mutation.rs` — stage(), unstage(), discard(), commit(), checkout(), fetch(), pull(), push(), clone_repo()
- `src-tauri/src/commands.rs` — #[tauri::command] функции

### Rust backend (модифицировать)
- `src-tauri/Cargo.toml` — добавить thiserror, tokio
- `src-tauri/src/main.rs` — подключить mod git, mod commands, зарегистрировать handler

### Frontend composables (создать)
- `src/composables/useRepo.ts`
- `src/composables/useFiles.ts`
- `src/composables/useBranches.ts`
- `src/composables/useLog.ts`
- `src/composables/useDiff.ts`
- `src/composables/useCommit.ts`
- `src/composables/useRemote.ts`

### Frontend (модифицировать)
- `src/types/index.ts` — добавить RepoInfo
- `src/components/BranchPanel.vue` — заменить моки на useBranches
- `src/components/FileList.vue` — заменить моки на useFiles
- `src/components/CommitGraph.vue` — заменить моки на useLog
- `src/components/CommitDetails.vue` — заменить моки на useLog
- `src/components/DiffView.vue` — заменить моки на useDiff
- `src/components/StatusBar.vue` — заменить моки на useRepo
- `src/components/dialogs/CommitDialog.vue` — подключить useCommit
- `src/components/dialogs/CloneDialog.vue` — подключить useRemote
- `src/components/dialogs/PushDialog.vue` — подключить useRemote
- `src/components/dialogs/PullDialog.vue` — подключить useRemote
- `src/components/dialogs/CheckoutDialog.vue` — подключить useBranches
- `src/App.vue` — инициализация repo, refreshAll

---

## Task 1: Rust types + error + run_git

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/git/mod.rs`
- Create: `src-tauri/src/git/types.rs`
- Create: `src-tauri/src/git/error.rs`
- Create: `src-tauri/src/git/query.rs` (только run_git)

- [ ] **Step 1: Обновить Cargo.toml**

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

- [ ] **Step 2: Создать git/types.rs**

```rust
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct FileStatus {
    pub path: String,
    pub state: String,
    pub staged: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RefLabel {
    pub name: String,
    pub kind: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CommitInfo {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub date: String,
    pub parents: Vec<String>,
    pub refs: Vec<RefLabel>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub is_current: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct TagInfo {
    pub name: String,
    pub oid: String,
    pub message: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiffLine {
    pub kind: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Serialize, Clone, Debug)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct RepoInfo {
    pub path: String,
    pub current_branch: String,
    pub head_oid: String,
}
```

- [ ] **Step 3: Создать git/error.rs**

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("{message}")]
    CommandFailed { message: String, hint: Option<String> },

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Merge conflict")]
    MergeConflict,

    #[error("Nothing to commit")]
    NothingToCommit,

    #[error("Repository not found at {0}")]
    RepoNotFound(String),
}

impl Serialize for GitError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub fn classify_git_error(stderr: &str) -> GitError {
    let s = stderr.to_lowercase();
    if s.contains("authentication failed") {
        GitError::AuthenticationFailed(
            "Check credentials: run `git config credential.helper`".into(),
        )
    } else if s.contains("permission denied (publickey)") {
        GitError::AuthenticationFailed(
            "SSH key not found or not added to agent. Run `ssh-add`".into(),
        )
    } else if s.contains("could not resolve host") {
        GitError::CommandFailed {
            message: stderr.trim().to_string(),
            hint: Some("Check network connection and remote URL".into()),
        }
    } else if s.contains("non-fast-forward") {
        GitError::CommandFailed {
            message: stderr.trim().to_string(),
            hint: Some("Remote has new commits. Pull first, or use force push".into()),
        }
    } else if s.contains("nothing to commit") {
        GitError::NothingToCommit
    } else if s.contains("conflict") && s.contains("merge") {
        GitError::MergeConflict
    } else {
        GitError::CommandFailed {
            message: stderr.trim().to_string(),
            hint: None,
        }
    }
}
```

- [ ] **Step 4: Создать git/query.rs с run_git**

```rust
use std::path::Path;
use std::process::Command;

use super::error::{classify_git_error, GitError};

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
```

- [ ] **Step 5: Создать git/mod.rs**

```rust
pub mod error;
pub mod query;
pub mod types;
```

- [ ] **Step 6: Обновить main.rs — добавить mod git**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod git;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Проверить компиляцию**

Run: `cd src-tauri && cargo check 2>&1`
Expected: компиляция без ошибок (warnings допустимы)

- [ ] **Step 8: Commit**

```bash
git add src-tauri/
git commit -m "backend: types, error handling, run_git helper"
```

---

## Task 2: Query functions (status, log, branches, tags, stashes, remotes, repo_info)

**Files:**
- Modify: `src-tauri/src/git/query.rs`

- [ ] **Step 1: Добавить status()**

В `query.rs` после `run_git`:

```rust
use super::types::*;

pub fn status(repo_path: &Path) -> Result<Vec<FileStatus>, GitError> {
    let output = run_git(repo_path, &["status", "--porcelain=v2"])?;
    let mut files = Vec::new();

    for line in output.lines() {
        if line.starts_with('1') || line.starts_with('2') {
            // Changed entries: 1 XY sub mH mI mW hH hI path
            // Renamed:         2 XY sub mH mI mW hH hI X\tscore\tpath\torigPath
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() < 9 {
                continue;
            }
            let xy = parts[1];
            let x = xy.as_bytes()[0] as char;
            let y = xy.as_bytes()[1] as char;

            let path = if line.starts_with('2') {
                // rename: path is after the tab-separated score
                parts[8].split('\t').nth(2).unwrap_or(parts[8]).to_string()
            } else {
                parts[8].to_string()
            };

            let (state, staged) = match (x, y) {
                ('M', '.') => ("modified", "staged"),
                ('.', 'M') => ("modified", "unstaged"),
                ('M', 'M') => ("modified", "partial"),
                ('A', '.') => ("added", "staged"),
                ('.', 'A') | ('?', '?') => ("added", "unstaged"),
                ('D', '.') => ("deleted", "staged"),
                ('.', 'D') => ("deleted", "unstaged"),
                ('R', '.') => ("renamed", "staged"),
                ('R', 'M') => ("renamed", "partial"),
                _ if xy.contains('U') || xy == "AA" || xy == "DD" => ("conflicted", "unstaged"),
                _ => ("modified", "unstaged"),
            };

            files.push(FileStatus {
                path,
                state: state.to_string(),
                staged: staged.to_string(),
            });
        } else if line.starts_with('?') {
            // Untracked: ? path
            let path = line[2..].to_string();
            files.push(FileStatus {
                path,
                state: "untracked".to_string(),
                staged: "unstaged".to_string(),
            });
        }
    }
    Ok(files)
}
```

- [ ] **Step 2: Добавить log()**

```rust
pub fn log(repo_path: &Path, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
    let format = "%H%x00%h%x00%s%x00%an%x00%ae%x00%aI%x00%P%x00%D";
    let limit_str = format!("-{}", limit);
    let output = run_git(repo_path, &["log", &format!("--format={}", format), &limit_str])?;

    let mut commits = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(8, '\0').collect();
        if parts.len() < 8 {
            continue;
        }

        let refs = parse_ref_labels(parts[7]);
        let parents: Vec<String> = parts[6]
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        commits.push(CommitInfo {
            oid: parts[0].to_string(),
            short_oid: parts[1].to_string(),
            message: parts[2].to_string(),
            author: parts[3].to_string(),
            author_email: parts[4].to_string(),
            date: parts[5].to_string(),
            parents,
            refs,
        });
    }
    Ok(commits)
}

fn parse_ref_labels(raw: &str) -> Vec<RefLabel> {
    if raw.trim().is_empty() {
        return Vec::new();
    }
    raw.split(", ")
        .filter_map(|r| {
            let r = r.trim();
            if r.is_empty() {
                return None;
            }
            if r == "HEAD" {
                return Some(RefLabel {
                    name: "HEAD".to_string(),
                    kind: "head".to_string(),
                });
            }
            let r = r.strip_prefix("HEAD -> ").unwrap_or(r);
            if r.starts_with("tag: ") {
                Some(RefLabel {
                    name: r[5..].to_string(),
                    kind: "tag".to_string(),
                })
            } else if r.contains('/') {
                Some(RefLabel {
                    name: r.to_string(),
                    kind: "remote-branch".to_string(),
                })
            } else {
                Some(RefLabel {
                    name: r.to_string(),
                    kind: "local-branch".to_string(),
                })
            }
        })
        .collect()
}
```

- [ ] **Step 3: Добавить branches()**

```rust
pub fn branches(repo_path: &Path) -> Result<Vec<BranchInfo>, GitError> {
    let format = "%(refname:short)%00%(upstream:short)%00%(upstream:track,nobracket)%00%(HEAD)";
    let output = run_git(repo_path, &["branch", "-a", &format!("--format={}", format)])?;

    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(4, '\0').collect();
        if parts.len() < 4 {
            continue;
        }
        let name = parts[0].to_string();
        if name.contains("HEAD") && name.contains("->") {
            continue; // skip "origin/HEAD -> origin/master"
        }
        let is_remote = name.starts_with("origin/") || name.contains('/');
        let upstream = if parts[1].is_empty() {
            None
        } else {
            Some(parts[1].to_string())
        };

        let (ahead, behind) = parse_track(parts[2]);
        let is_current = parts[3].trim() == "*";

        result.push(BranchInfo {
            name,
            is_remote,
            upstream,
            ahead,
            behind,
            is_current,
        });
    }
    Ok(result)
}

fn parse_track(track: &str) -> (u32, u32) {
    let mut ahead = 0u32;
    let mut behind = 0u32;
    for part in track.split(", ") {
        let part = part.trim();
        if part.starts_with("ahead ") {
            ahead = part[6..].parse().unwrap_or(0);
        } else if part.starts_with("behind ") {
            behind = part[7..].parse().unwrap_or(0);
        }
    }
    (ahead, behind)
}
```

- [ ] **Step 4: Добавить tags(), stashes(), remotes(), repo_info()**

```rust
pub fn tags(repo_path: &Path) -> Result<Vec<TagInfo>, GitError> {
    let format = "%(refname:short)%00%(*objectname:short)%00%(contents:subject)";
    let output = run_git(repo_path, &["tag", "-l", &format!("--format={}", format)])?;

    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '\0').collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let oid = parts.get(1).unwrap_or(&"").to_string();
        let message = parts.get(2).map(|s| s.to_string()).filter(|s| !s.is_empty());
        result.push(TagInfo { name, oid, message });
    }
    Ok(result)
}

pub fn stashes(repo_path: &Path) -> Result<Vec<StashEntry>, GitError> {
    let output = run_git(repo_path, &["stash", "list", "--format=%gd%x00%gs"]);
    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()), // no stashes is not an error
    };

    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '\0').collect();
        let index_str = parts.first().unwrap_or(&"");
        let index = index_str
            .strip_prefix("stash@{")
            .and_then(|s| s.strip_suffix('}'))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let message = parts.get(1).unwrap_or(&"").to_string();
        result.push(StashEntry { index, message });
    }
    Ok(result)
}

pub fn remotes(repo_path: &Path) -> Result<Vec<String>, GitError> {
    let output = run_git(repo_path, &["remote"])?;
    Ok(output.lines().filter(|l| !l.is_empty()).map(|l| l.to_string()).collect())
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
```

- [ ] **Step 5: Проверить компиляцию**

Run: `cd src-tauri && cargo check 2>&1`
Expected: без ошибок

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/git/query.rs
git commit -m "backend: query functions — status, log, branches, tags, stashes, remotes, repo_info"
```

---

## Task 3: Diff parsing

**Files:**
- Modify: `src-tauri/src/git/query.rs`

- [ ] **Step 1: Добавить diff_file() и parse_diff()**

В `query.rs`:

```rust
pub fn diff_file(repo_path: &Path, file: &str, staged: bool) -> Result<FileDiff, GitError> {
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
    let mut insertions = 0u32;
    let mut deletions = 0u32;
    let mut old_line = 0u32;
    let mut new_line = 0u32;
    let mut path = fallback_path.to_string();

    for line in diff_text.lines() {
        if line.starts_with("+++ b/") {
            path = line[6..].to_string();
        } else if line.starts_with("@@ ") {
            if !current_header.is_empty() {
                hunks.push(DiffHunk {
                    header: current_header.clone(),
                    lines: std::mem::take(&mut current_lines),
                });
            }
            current_header = line.to_string();
            // Parse @@ -old,count +new,count @@
            if let Some(nums) = line.strip_prefix("@@ ") {
                let parts: Vec<&str> = nums.split(' ').collect();
                if parts.len() >= 2 {
                    old_line = parts[0]
                        .trim_start_matches('-')
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                    new_line = parts[1]
                        .trim_start_matches('+')
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                }
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            insertions += 1;
            current_lines.push(DiffLine {
                kind: "added".to_string(),
                old_lineno: None,
                new_lineno: Some(new_line),
                content: line[1..].to_string(),
            });
            new_line += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
            current_lines.push(DiffLine {
                kind: "removed".to_string(),
                old_lineno: Some(old_line),
                new_lineno: None,
                content: line[1..].to_string(),
            });
            old_line += 1;
        } else if line.starts_with(' ') {
            current_lines.push(DiffLine {
                kind: "context".to_string(),
                old_lineno: Some(old_line),
                new_lineno: Some(new_line),
                content: line[1..].to_string(),
            });
            old_line += 1;
            new_line += 1;
        }
    }

    if !current_header.is_empty() {
        hunks.push(DiffHunk {
            header: current_header,
            lines: current_lines,
        });
    }

    FileDiff {
        path,
        hunks,
        insertions,
        deletions,
    }
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
            current_path = line
                .split(" b/")
                .last()
                .unwrap_or("")
                .to_string();
        }
        current_chunk.push_str(line);
        current_chunk.push('\n');
    }
    if !current_chunk.is_empty() {
        diffs.push(parse_diff_single(&current_chunk, &current_path));
    }
    diffs
}
```

- [ ] **Step 2: Проверить компиляцию**

Run: `cd src-tauri && cargo check 2>&1`
Expected: без ошибок

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/git/query.rs
git commit -m "backend: diff parsing — unified diff → FileDiff structs"
```

---

## Task 4: Mutation functions

**Files:**
- Create: `src-tauri/src/git/mutation.rs`
- Modify: `src-tauri/src/git/mod.rs`

- [ ] **Step 1: Создать mutation.rs**

```rust
use std::path::Path;
use std::process::Command;

use super::error::{classify_git_error, GitError};

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
    if amend {
        args.push("--amend");
    }
    run_git_mut(repo_path, &args)
}

pub fn checkout(repo_path: &Path, branch: &str) -> Result<(), GitError> {
    // Try git switch first; if it's a remote branch, create local tracking
    let result = run_git_mut(repo_path, &["switch", branch]);
    if result.is_ok() {
        return Ok(());
    }
    // Maybe it's a remote branch like "origin/feature" — create local
    if let Some(local) = branch.split('/').last() {
        run_git_mut(repo_path, &["switch", "-c", local, branch])?;
        return Ok(());
    }
    result.map(|_| ())
}

pub fn fetch(repo_path: &Path, remote: &str) -> Result<String, GitError> {
    run_git_mut(repo_path, &["fetch", remote])
}

pub fn pull(repo_path: &Path, remote: &str, rebase: bool) -> Result<String, GitError> {
    if rebase {
        run_git_mut(repo_path, &["pull", "--rebase", remote])
    } else {
        run_git_mut(repo_path, &["pull", remote])
    }
}

pub fn push(repo_path: &Path, remote: &str, force: bool) -> Result<String, GitError> {
    if force {
        run_git_mut(repo_path, &["push", "--force", remote])
    } else {
        run_git_mut(repo_path, &["push", remote])
    }
}

pub fn clone_repo(url: &str, dest: &str) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(["clone", url, dest])
        .output()
        .map_err(|e| GitError::CommandFailed {
            message: format!("Failed to run git clone: {}", e),
            hint: None,
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(classify_git_error(&stderr))
    }
}
```

- [ ] **Step 2: Обновить git/mod.rs**

```rust
pub mod error;
pub mod mutation;
pub mod query;
pub mod types;
```

- [ ] **Step 3: Сделать run_git публичной** в `query.rs`:

Изменить `pub fn run_git` → уже публичная (проверить). Если `pub(super)`, сделать `pub`.

- [ ] **Step 4: Проверить компиляцию**

Run: `cd src-tauri && cargo check 2>&1`
Expected: без ошибок

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/git/
git commit -m "backend: mutation functions — stage, unstage, discard, commit, checkout, fetch, pull, push, clone"
```

---

## Task 5: Tauri commands + main.rs registration

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Создать commands.rs**

```rust
use std::path::Path;

use crate::git::{query, mutation, types::*};

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
pub fn do_fetch(repo_path: String, remote: String) -> Result<String, String> {
    mutation::fetch(Path::new(&repo_path), &remote).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_pull(repo_path: String, remote: String, rebase: bool) -> Result<String, String> {
    mutation::pull(Path::new(&repo_path), &remote, rebase).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_push(repo_path: String, remote: String, force: bool) -> Result<String, String> {
    mutation::push(Path::new(&repo_path), &remote, force).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_clone(url: String, dest: String) -> Result<String, String> {
    mutation::clone_repo(&url, &dest).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Обновить main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod git;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_repo_info,
            commands::get_status,
            commands::get_log,
            commands::get_branches,
            commands::get_tags,
            commands::get_stashes,
            commands::get_remotes,
            commands::get_diff_file,
            commands::get_diff_commit,
            commands::stage_files,
            commands::unstage_files,
            commands::discard_files,
            commands::do_commit,
            commands::do_checkout,
            commands::do_fetch,
            commands::do_pull,
            commands::do_push,
            commands::do_clone,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Проверить компиляцию**

Run: `cd src-tauri && cargo check 2>&1`
Expected: без ошибок

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/
git commit -m "backend: Tauri commands — все IPC endpoints зарегистрированы"
```

---

## Task 6: Frontend composables

**Files:**
- Modify: `src/types/index.ts`
- Create: `src/composables/useRepo.ts`
- Create: `src/composables/useFiles.ts`
- Create: `src/composables/useBranches.ts`
- Create: `src/composables/useLog.ts`
- Create: `src/composables/useDiff.ts`
- Create: `src/composables/useCommit.ts`
- Create: `src/composables/useRemote.ts`

- [ ] **Step 1: Добавить RepoInfo в types/index.ts**

В конец файла:

```typescript
export interface RepoInfo {
  path: string;
  currentBranch: string;
  headOid: string;
}
```

- [ ] **Step 2: Создать useRepo.ts**

```typescript
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RepoInfo } from "@/types";

const repoPath = ref<string | null>(null);
const repoInfo = ref<RepoInfo | null>(null);

export function useRepo() {
  async function openRepo(path: string) {
    repoPath.value = path;
    repoInfo.value = await invoke<RepoInfo>("get_repo_info", { repoPath: path });
  }

  async function refreshInfo() {
    if (!repoPath.value) return;
    repoInfo.value = await invoke<RepoInfo>("get_repo_info", { repoPath: repoPath.value });
  }

  return { repoPath, repoInfo, openRepo, refreshInfo };
}
```

- [ ] **Step 3: Создать useFiles.ts**

```typescript
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileStatus } from "@/types";
import { useRepo } from "./useRepo";

const files = ref<FileStatus[]>([]);
const selectedFile = ref<string | null>(null);

export function useFiles() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) return;
    files.value = await invoke<FileStatus[]>("get_status", { repoPath: repoPath.value });
  }

  async function stageFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("stage_files", { repoPath: repoPath.value, files: paths });
    await refresh();
  }

  async function unstageFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("unstage_files", { repoPath: repoPath.value, files: paths });
    await refresh();
  }

  async function discardFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("discard_files", { repoPath: repoPath.value, files: paths });
    await refresh();
  }

  return { files, selectedFile, refresh, stageFiles, unstageFiles, discardFiles };
}
```

- [ ] **Step 4: Создать useBranches.ts**

```typescript
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { BranchInfo, TagInfo, StashEntry } from "@/types";
import { useRepo } from "./useRepo";

const branches = ref<BranchInfo[]>([]);
const tags = ref<TagInfo[]>([]);
const stashes = ref<StashEntry[]>([]);
const remotes = ref<string[]>([]);

export function useBranches() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) return;
    const [b, t, s, r] = await Promise.all([
      invoke<BranchInfo[]>("get_branches", { repoPath: repoPath.value }),
      invoke<TagInfo[]>("get_tags", { repoPath: repoPath.value }),
      invoke<StashEntry[]>("get_stashes", { repoPath: repoPath.value }),
      invoke<string[]>("get_remotes", { repoPath: repoPath.value }),
    ]);
    branches.value = b;
    tags.value = t;
    stashes.value = s;
    remotes.value = r;
  }

  async function checkout(branch: string) {
    if (!repoPath.value) return;
    await invoke("do_checkout", { repoPath: repoPath.value, branch });
  }

  return { branches, tags, stashes, remotes, refresh, checkout };
}
```

- [ ] **Step 5: Создать useLog.ts**

```typescript
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CommitInfo } from "@/types";
import { useRepo } from "./useRepo";

const commits = ref<CommitInfo[]>([]);
const selectedCommit = ref<string | null>(null);

export function useLog() {
  const { repoPath } = useRepo();

  async function refresh(limit?: number) {
    if (!repoPath.value) return;
    commits.value = await invoke<CommitInfo[]>("get_log", {
      repoPath: repoPath.value,
      limit: limit ?? 500,
    });
  }

  return { commits, selectedCommit, refresh };
}
```

- [ ] **Step 6: Создать useDiff.ts**

```typescript
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileDiff } from "@/types";
import { useRepo } from "./useRepo";

const currentDiff = ref<FileDiff | null>(null);

export function useDiff() {
  const { repoPath } = useRepo();

  async function diffFile(path: string, staged: boolean) {
    if (!repoPath.value) return;
    currentDiff.value = await invoke<FileDiff>("get_diff_file", {
      repoPath: repoPath.value,
      file: path,
      staged,
    });
  }

  async function diffCommit(oid: string) {
    if (!repoPath.value) return;
    const diffs = await invoke<FileDiff[]>("get_diff_commit", {
      repoPath: repoPath.value,
      oid,
    });
    currentDiff.value = diffs[0] ?? null;
  }

  function clearDiff() {
    currentDiff.value = null;
  }

  return { currentDiff, diffFile, diffCommit, clearDiff };
}
```

- [ ] **Step 7: Создать useCommit.ts**

```typescript
import { invoke } from "@tauri-apps/api/core";
import { useRepo } from "./useRepo";

export function useCommit() {
  const { repoPath } = useRepo();

  async function commit(message: string, amend: boolean) {
    if (!repoPath.value) return;
    await invoke("do_commit", { repoPath: repoPath.value, message, amend });
  }

  return { commit };
}
```

- [ ] **Step 8: Создать useRemote.ts**

```typescript
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRepo } from "./useRepo";

const isBusy = ref(false);
const lastError = ref<string | null>(null);

export function useRemote() {
  const { repoPath } = useRepo();

  async function wrapAsync(fn: () => Promise<unknown>) {
    isBusy.value = true;
    lastError.value = null;
    try {
      await fn();
    } catch (e) {
      lastError.value = String(e);
    } finally {
      isBusy.value = false;
    }
  }

  async function fetchRemote(remote: string) {
    await wrapAsync(() => invoke("do_fetch", { repoPath: repoPath.value!, remote }));
  }

  async function pull(remote: string, rebase: boolean) {
    await wrapAsync(() => invoke("do_pull", { repoPath: repoPath.value!, remote, rebase }));
  }

  async function push(remote: string, force: boolean) {
    await wrapAsync(() => invoke("do_push", { repoPath: repoPath.value!, remote, force }));
  }

  async function cloneRepo(url: string, dest: string) {
    await wrapAsync(() => invoke("do_clone", { url, dest }));
  }

  return { isBusy, lastError, fetchRemote, pull, push, cloneRepo };
}
```

- [ ] **Step 9: Проверить сборку фронтенда**

Run: `npx vue-tsc --noEmit 2>&1`
Expected: без ошибок

- [ ] **Step 10: Commit**

```bash
git add src/types/index.ts src/composables/
git commit -m "frontend: composables — useRepo, useFiles, useBranches, useLog, useDiff, useCommit, useRemote"
```

---

## Task 7: Интеграция composables в компоненты

**Files:**
- Modify: `src/App.vue`
- Modify: `src/components/BranchPanel.vue`
- Modify: `src/components/FileList.vue`
- Modify: `src/components/CommitGraph.vue`
- Modify: `src/components/CommitDetails.vue`
- Modify: `src/components/DiffView.vue`
- Modify: `src/components/StatusBar.vue`

Заменяем хардкод-моки на composable refs в каждом компоненте. Компоненты начинают использовать реальные данные.

Этот таск выполняется покомпонентно. Для каждого компонента: убираем локальные моковые данные, импортируем composable, подставляем refs.

- [ ] **Step 1: App.vue — добавить инициализацию**

В `<script setup>` добавить после импортов диалогов:

```typescript
import { onMounted } from "vue";
import { useRepo } from "./composables/useRepo";
import { useFiles } from "./composables/useFiles";
import { useBranches } from "./composables/useBranches";
import { useLog } from "./composables/useLog";

const { openRepo, repoPath } = useRepo();
const { refresh: refreshFiles } = useFiles();
const { refresh: refreshBranches } = useBranches();
const { refresh: refreshLog } = useLog();

async function refreshAll() {
  await Promise.all([refreshFiles(), refreshBranches(), refreshLog()]);
}

onMounted(async () => {
  // Открываем текущую директорию как репозиторий
  // В будущем: из аргументов CLI или последний открытый
  try {
    await openRepo(window.__TAURI_INTERNALS__ ? "." : "/home/avv/projects/gitstream");
    await refreshAll();
  } catch (e) {
    console.error("Failed to open repo:", e);
  }
});
```

Убедиться что `import { ref } from "vue"` заменён на `import { ref, onMounted, onUnmounted } from "vue"` (onMounted уже есть для Esc-хендлера).

- [ ] **Step 2: BranchPanel.vue — подключить useBranches**

Заменить `<script setup>` на:

```typescript
import { ref, computed } from "vue";
import { useBranches } from "@/composables/useBranches";
import type { BranchInfo, TagInfo, StashEntry } from "@/types";

const { branches, tags, stashes } = useBranches();

const filter = ref("");
const expandedSections = ref({
  local: true,
  remote: true,
  tags: false,
  stashes: false,
});

const localBranches = computed(() =>
  branches.value.filter((b: BranchInfo) => !b.isRemote)
);
const remoteBranches = computed(() =>
  branches.value.filter((b: BranchInfo) => b.isRemote)
);

function toggleSection(key: keyof typeof expandedSections.value) {
  expandedSections.value[key] = !expandedSections.value[key];
}
```

В template заменить `branch.isCurrent` → `branch.is_current` (Rust использует snake_case в JSON).

Обновить в template:
- `branch.isCurrent` → `branch.is_current`
- `branch.isRemote` — уже в computed, не нужно в template

- [ ] **Step 3: FileList.vue — подключить useFiles**

Заменить `<script setup>` на:

```typescript
import { ref } from "vue";
import { useFiles } from "@/composables/useFiles";
import { useDiff } from "@/composables/useDiff";

const { files, selectedFile } = useFiles();
const { diffFile } = useDiff();

const activeFilter = ref<string>("all");

const filters = [
  { key: "all", label: "All" },
  { key: "modified", label: "Modified" },
  { key: "staged", label: "Staged" },
  { key: "untracked", label: "Untracked" },
  { key: "conflicted", label: "Conflicted" },
];

const stateIcons: Record<string, { color: string; letter: string }> = {
  modified: { color: "var(--blue)", letter: "M" },
  added: { color: "var(--green)", letter: "A" },
  deleted: { color: "var(--red)", letter: "D" },
  renamed: { color: "var(--purple)", letter: "R" },
  conflicted: { color: "var(--red)", letter: "C" },
  untracked: { color: "var(--text-muted)", letter: "?" },
};

function fileName(path: string): string {
  return path.split("/").pop() || path;
}

function fileDir(path: string): string {
  const parts = path.split("/");
  parts.pop();
  return parts.join("/");
}

async function selectFile(path: string) {
  selectedFile.value = path;
  await diffFile(path, false);
}
```

В template заменить `@click="selectedFile = file.path"` на `@click="selectFile(file.path)"`.

- [ ] **Step 4: CommitGraph.vue — подключить useLog**

Заменить моковые данные в `<script setup>` на:

```typescript
import { ref } from "vue";
import { useLog } from "@/composables/useLog";
import type { CommitInfo, RefLabel } from "@/types";

const { commits, selectedCommit } = useLog();

const graphColors = [
  "var(--blue)", "var(--green)", "var(--purple)",
  "var(--orange)", "var(--teal)", "var(--yellow)", "var(--red)",
];

function refClass(r: RefLabel): string {
  return `ref-label ref-${r.kind}`;
}
```

В template заменить `rows` → `commits`, `row.commit.oid` → `commit.oid` и т.д. Граф SVG пока упростить — отображать коммиты как список без линий (column = 0 для всех).

Заменить `v-for="row in rows"` на `v-for="commit in commits"`. Убрать GraphRow обёртку — рисуем SVG с dot в column 0.

- [ ] **Step 5: CommitDetails.vue — подключить useLog**

```typescript
import { computed } from "vue";
import { useLog } from "@/composables/useLog";

const { commits, selectedCommit } = useLog();

const commit = computed(() =>
  commits.value.find((c) => c.oid === selectedCommit.value) ?? commits.value[0] ?? null
);
```

В template обернуть содержимое в `<template v-if="commit">`.

- [ ] **Step 6: DiffView.vue — подключить useDiff**

Заменить моковые данные:

```typescript
import { ref } from "vue";
import { useDiff } from "@/composables/useDiff";
import { useFiles } from "@/composables/useFiles";
import type { DiffMode } from "@/types";

const { currentDiff } = useDiff();
const { selectedFile } = useFiles();

const mode = ref<DiffMode>("unified");
const compareMode = "Working Tree vs Index";
```

В template заменить `hunks` → `currentDiff?.hunks ?? []`, `fileName` → `currentDiff?.path ?? ''`. Обернуть body в `v-if="currentDiff"`.

- [ ] **Step 7: StatusBar.vue — подключить useRepo**

```typescript
import { useRepo } from "@/composables/useRepo";
import { useRemote } from "@/composables/useRemote";

const { repoInfo } = useRepo();
const { isBusy, lastError } = useRemote();
```

В template: `repoInfo?.currentBranch ?? '—'` вместо хардкода "master". Status message: `lastError ?? (isBusy ? 'Working...' : 'Ready')`.

- [ ] **Step 8: Проверить сборку**

Run: `npx vue-tsc --noEmit 2>&1`
Expected: без ошибок (или минимальные правки camelCase ↔ snake_case)

- [ ] **Step 9: Commit**

```bash
git add src/
git commit -m "frontend: интеграция composables во все компоненты — реальные данные из backend"
```

---

## Task 8: Интеграция диалогов

**Files:**
- Modify: `src/components/dialogs/CommitDialog.vue`
- Modify: `src/components/dialogs/CloneDialog.vue`
- Modify: `src/components/dialogs/PushDialog.vue`
- Modify: `src/components/dialogs/PullDialog.vue`
- Modify: `src/components/dialogs/CheckoutDialog.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: CommitDialog.vue — подключить useCommit + useFiles**

```typescript
import { ref, computed } from "vue";
import { useCommit } from "@/composables/useCommit";
import { useFiles } from "@/composables/useFiles";

const emit = defineEmits<{ close: [] }>();

const { commit: doCommit } = useCommit();
const { files, refresh: refreshFiles } = useFiles();

const message = ref("");
const amend = ref(false);

const firstLineLength = computed(() => {
  const firstLine = message.value.split("\n")[0] || "";
  return firstLine.length;
});
const firstLineClass = computed(() => {
  if (firstLineLength.value > 72) return "error";
  if (firstLineLength.value > 50) return "warning";
  return "ok";
});

const stagedCount = computed(() => files.value.filter((f) => f.staged === "staged" || f.staged === "partial").length);

async function handleCommit() {
  await doCommit(message.value, amend.value);
  emit("close");
}

async function handleCommitAndPush() {
  await doCommit(message.value, amend.value);
  // push будет через useRemote, пока просто close
  emit("close");
}
```

Кнопки Commit и Commit & Push вызывают `handleCommit()` / `handleCommitAndPush()`.

- [ ] **Step 2: CloneDialog.vue — подключить useRemote**

```typescript
import { ref, computed } from "vue";
import { useRemote } from "@/composables/useRemote";
import { useRepo } from "@/composables/useRepo";

const emit = defineEmits<{ close: [] }>();

const { cloneRepo, isBusy } = useRemote();
const { openRepo } = useRepo();

const url = ref("");
const directory = ref("");
const error = ref("");

const autoName = computed(() => {
  if (!url.value) return "";
  const match = url.value.match(/\/([^/]+?)(\.git)?$/);
  return match ? match[1] : "";
});

const canClone = computed(() => !isBusy.value && url.value.trim() && (directory.value.trim() || autoName.value));

async function handleClone() {
  const dest = directory.value || autoName.value;
  try {
    await cloneRepo(url.value, dest);
    await openRepo(dest);
    emit("close");
  } catch (e) {
    error.value = String(e);
  }
}
```

- [ ] **Step 3: PushDialog.vue — подключить useRemote + useBranches**

```typescript
import { ref, computed } from "vue";
import { useRemote } from "@/composables/useRemote";
import { useBranches } from "@/composables/useBranches";
import { useRepo } from "@/composables/useRepo";

const emit = defineEmits<{ close: [] }>();

const { push, isBusy } = useRemote();
const { remotes } = useBranches();
const { repoInfo } = useRepo();

const selectedRemote = ref("origin");
const forcePush = ref(false);

const currentBranch = computed(() => repoInfo.value?.currentBranch ?? "master");

async function handlePush() {
  await push(selectedRemote.value, forcePush.value);
  emit("close");
}
```

- [ ] **Step 4: PullDialog.vue — подключить useRemote + useBranches**

```typescript
import { ref, computed } from "vue";
import { useRemote } from "@/composables/useRemote";
import { useBranches } from "@/composables/useBranches";
import { useRepo } from "@/composables/useRepo";

const emit = defineEmits<{ close: [] }>();

const { pull, isBusy } = useRemote();
const { remotes, branches } = useBranches();
const { repoInfo } = useRepo();

const selectedRemote = ref("origin");
const pullMode = ref<"merge" | "rebase">("merge");

const currentBranch = computed(() => repoInfo.value?.currentBranch ?? "master");
const currentBranchInfo = computed(() =>
  branches.value.find((b) => b.is_current)
);
const behindCount = computed(() => currentBranchInfo.value?.behind ?? 0);

async function handlePull() {
  await pull(selectedRemote.value, pullMode.value === "rebase");
  emit("close");
}
```

- [ ] **Step 5: CheckoutDialog.vue — подключить useBranches**

```typescript
import { ref, computed } from "vue";
import { useBranches } from "@/composables/useBranches";

const emit = defineEmits<{ close: [] }>();

const { branches, checkout } = useBranches();

const search = ref("");
const selectedBranch = ref<string | null>(null);

const filteredBranches = computed(() => {
  const q = search.value.toLowerCase();
  if (!q) return branches.value;
  return branches.value.filter((b) => b.name.toLowerCase().includes(q));
});

async function handleCheckout() {
  if (!selectedBranch.value) return;
  await checkout(selectedBranch.value);
  emit("close");
}
```

В template: `branch.isCurrent` → `branch.is_current`, `branch.isRemote` → `branch.is_remote`.

- [ ] **Step 6: App.vue — refresh после закрытия диалогов**

Заменить `@close="showCommitDialog = false"` на:

```
@close="showCommitDialog = false; refreshAll()"
```

Аналогично для Push, Pull, Checkout, Clone.

- [ ] **Step 7: Проверить сборку**

Run: `npx vue-tsc --noEmit 2>&1`
Expected: без ошибок

- [ ] **Step 8: Commit**

```bash
git add src/
git commit -m "frontend: диалоги подключены к backend — commit, clone, push, pull, checkout"
```

---

## Task 9: Финальная проверка

- [ ] **Step 1: Cargo check**

Run: `cd src-tauri && cargo check 2>&1`
Expected: без ошибок

- [ ] **Step 2: Frontend build**

Run: `npx vue-tsc --noEmit && npx vite build 2>&1`
Expected: без ошибок

- [ ] **Step 3: Финальный commit если есть незакоммиченные правки**

```bash
git add -A && git commit -m "backend MVP: финальные правки"
```
