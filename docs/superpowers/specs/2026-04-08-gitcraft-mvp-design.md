# GitStream MVP — Design Specification

## Overview

Git GUI client built with Rust + Slint. MVP scope: core daily Git operations with non-blocking UI.

**Tech stack:** Rust, Slint 1.12, tokio, gix, git CLI  
**Architecture:** CommandExecutor pattern with tokio mpsc channels

---

## 1. Architecture: CommandExecutor

### Data Flow

```
UI (Slint callbacks)
  │
  ▼
Command (enum) ──→ tokio::mpsc::channel ──→ CommandExecutor
                                                  │
                                              git CLI / gix
                                                  │
                                                  ▼
                                           CommandResult (enum)
                                                  │
                                  tokio::mpsc::channel ◄──┘
                                          │
                                          ▼
                                    UI update loop
                                (Slint Timer ~100ms, try_recv)
```

### Command Enum

```rust
pub enum Command {
    Refresh,
    Stage(Vec<PathBuf>),
    Unstage(Vec<PathBuf>),
    Commit { message: String, amend: bool },
    Checkout(String),
    Fetch { remote: Option<String> },
    Pull { remote: Option<String>, rebase: bool },
    Push { remote: Option<String>, force: bool },
    Clone { url: String, path: PathBuf },
    Discard(Vec<PathBuf>),
}
```

### CommandKind Enum (lightweight identifier for results)

```rust
pub enum CommandKind {
    Refresh, Stage, Unstage, Commit, Checkout,
    Fetch, Pull, Push, Clone, Discard,
}
```

### CommandResult Enum

```rust
pub enum CommandResult {
    Success { kind: CommandKind, refresh: bool },
    Error { kind: CommandKind, message: String, hint: Option<String> },
    Progress { kind: CommandKind, message: String, percent: Option<u8> },
}
```

### CommandExecutor Struct

```rust
pub struct CommandExecutor {
    repo_path: PathBuf,
    cmd_rx: mpsc::Receiver<Command>,
    result_tx: mpsc::Sender<CommandResult>,
}
```

- `run()` — infinite loop in tokio task, matches Command to git operation
- Network ops (fetch/pull/push/clone) run in `tokio::task::spawn_blocking`
- On repo switch — recreate executor with new `repo_path`

### UI Integration

- Slint Timer (~100ms) polls `result_rx.try_recv()`
- `Success { refresh: true }` → reload repo data
- `Error { message, hint }` → show in status bar (red, auto-dismiss 5s)
- `Progress` → update busy indicator in status bar

---

## 2. Diff Engine

### Query Functions (gitstream-git/src/query.rs)

```rust
diff_working_tree(repo_path, file_path) -> Vec<DiffHunk>    // unstaged
diff_index(repo_path, file_path) -> Vec<DiffHunk>           // staged
diff_commit(repo_path, oid) -> Vec<FileDiff>                 // commit vs parent
diff_commits(repo_path, oid_a, oid_b) -> Vec<FileDiff>      // arbitrary
```

### Data Structures (gitstream-git/src/types.rs)

```rust
pub enum LineKind { Context, Added, Removed }

pub struct DiffLine {
    pub kind: LineKind,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

pub struct DiffStats {
    pub insertions: u32,
    pub deletions: u32,
}

pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}
```

### UI Modes

- **Unified** — existing `diff-view.slint`, populated with real data
- **Side-by-side** — new `diff-side-by-side.slint`: two columns, synced scroll, added lines right, removed lines left, context both
- Toggle via buttons in diff panel header (already stubbed)

### Triggers

- Click file in file-list → show diff for that file
- Toggle staged/unstaged → refresh diff
- Select commit in graph → show commit diff

---

## 3. Modal Dialogs

All dialogs use modal overlay pattern (same as existing repo-dialog/group-dialog).

### 3.1 Commit Dialog (520x320px)

**File:** `commit-dialog.slint`

- Multi-line commit message field (~5 lines)
- First line length indicator (>50 yellow, >72 red)
- "Amend last commit" checkbox — fills message from previous commit
- Staged files counter: "4 files staged"
- Buttons: Cancel, Commit, Commit & Push
- Commit disabled if message empty or 0 staged files
- Commit & Push sends two Commands sequentially

### 3.2 Clone Dialog (500x260px)

**File:** `clone-dialog.slint`

- URL field (placeholder: "https://... or git@...")
- Directory field + Browse button (zenity)
- Auto-fill directory name from URL (last segment without .git)
- Error message (red)
- Buttons: Cancel, Clone
- Clone disabled until both fields filled
- On success — auto-open cloned repo

### 3.3 Push Dialog (400x200px)

**File:** `push-dialog.slint`

- Remote dropdown (from remotes list, default: origin)
- Text: "Push branch **main** → **origin/main**"
- "Force push" checkbox + red warning text when enabled
- Buttons: Cancel, Push

### 3.4 Pull Dialog (400x220px)

**File:** `pull-dialog.slint`

- Remote dropdown (default: origin)
- Radio: Merge / Rebase
- Ahead/behind indicator: "3 commits behind origin/main"
- Buttons: Cancel, Pull

### 3.5 Checkout Branch Dialog (400x300px)

**File:** `checkout-dialog.slint`

- Search/filter field
- Filterable branch list (local + remote)
- Current branch marked "(current)"
- Remote branches: "will create local branch"
- Buttons: Cancel, Checkout

### 3.6 Confirmation Dialog (360x160px)

**File:** `confirm-dialog.slint`

Universal template for:
- **Discard Changes:** "Discard changes in N files? This cannot be undone." → Cancel / Discard
- **Force Push:** "Force push will overwrite remote history. Continue?" → Cancel / Force Push

---

## 4. Backend Changes (gitstream-git)

### New module: mutation.rs

```rust
stage_files(repo_path, paths) -> Result<()>           // git add
unstage_files(repo_path, paths) -> Result<()>          // git restore --staged
discard_files(repo_path, paths) -> Result<()>          // git restore
commit(repo_path, message, amend) -> Result<Oid>       // git commit
checkout_branch(repo_path, name) -> Result<()>         // git switch
fetch(repo_path, remote) -> Result<String>             // git fetch
pull(repo_path, remote, rebase) -> Result<String>      // git pull [--rebase]
push(repo_path, remote, force) -> Result<String>       // git push [--force]
```

### New query functions

```rust
remotes(repo_path) -> Vec<String>
remote_url(repo_path, name) -> String
```

### New in repo.rs

```rust
clone_repo(url, path) -> Result<()>
```

### Extended error.rs

```rust
pub enum GitError {
    // existing...
    AuthenticationFailed(String),
    MergeConflict,
    NothingToCommit,
    DetachedHead,
}
```

`classify_git_error(stderr: &str) -> GitError` — parses stderr for auth failures, merge conflicts, etc. Produces hint strings for UI.

---

## 5. Core Changes (gitstream-core)

### New module: executor.rs

CommandExecutor as described in Section 1.

### Updated state.rs

```rust
pub struct AppState {
    // existing fields...
    pub is_busy: bool,
    pub last_error: Option<String>,
    pub last_hint: Option<String>,
}
```

---

## 6. UI Changes (gitstream-ui)

### New .slint files

| File | Description |
|------|-------------|
| `commit-dialog.slint` | Commit dialog (520x320) |
| `clone-dialog.slint` | Clone dialog (500x260) |
| `push-dialog.slint` | Push dialog (400x200) |
| `pull-dialog.slint` | Pull dialog (400x220) |
| `checkout-dialog.slint` | Checkout branch dialog (400x300) |
| `confirm-dialog.slint` | Universal confirmation dialog (360x160) |
| `diff-side-by-side.slint` | Side-by-side diff view |

### Changes to existing files

**main-window.slint:**
- Properties for dialog visibility (`show-commit-dialog`, `show-clone-dialog`, etc.)
- Properties for dialog data (remotes, branches for checkout)
- Properties `is-busy`, `error-message`, `error-hint`
- Include new dialog components

**toolbar.slint:**
- Wire callbacks to open dialogs (Pull, Push, Commit, Discard)
- Busy indicator (spinner/animation during operation)

**diff-view.slint:**
- Toggle between unified and side-by-side
- "Ignore WS" triggers diff recalculation with `--ignore-all-space`

**file-list.slint:**
- Click file → callback → load and show diff
- Stage/Unstage/Discard actions per file

**commit-graph.slint:**
- Select commit → load commit diff into file-list and diff-view

**New: status bar** (bottom of main-window):
- Current branch name
- Ahead/behind indicator
- Busy spinner + operation text ("Pushing...")
- Error with hint (red, auto-dismiss 5s or click to dismiss)

---

## 7. Authentication Error Handling

Strategy: delegate to system git config (SSH keys, credential helpers). On failure — show clear message with actionable hint.

### Error Classification

| stderr pattern | GitError | Hint |
|---------------|----------|------|
| `Authentication failed` | AuthenticationFailed | "Check credentials: run `git config credential.helper`" |
| `Permission denied (publickey)` | AuthenticationFailed | "SSH key not found or not added to agent. Run `ssh-add`" |
| `Could not resolve host` | OperationFailed | "Check network connection and remote URL" |
| `rejected.*non-fast-forward` | OperationFailed | "Remote has new commits. Pull first, or use force push" |

---

## 8. File Structure After MVP

```
gitstream-git/src/
  ├── lib.rs
  ├── types.rs          # + DiffLine, DiffHunk, FileDiff, DiffStats, LineKind
  ├── error.rs          # + AuthenticationFailed, MergeConflict, classify_git_error
  ├── repo.rs           # + clone_repo
  ├── query.rs          # + diff_*, remotes, remote_url
  ├── mutation.rs       # NEW: stage, unstage, discard, commit, checkout, fetch, pull, push
  └── repo_store.rs

gitstream-core/src/
  ├── lib.rs
  ├── state.rs          # + is_busy, last_error, last_hint
  ├── commands.rs       # expanded Command + CommandResult enums
  └── executor.rs       # NEW: CommandExecutor

gitstream-ui/
  ├── src/
  │   ├── lib.rs
  │   └── app.rs        # refactored: thin layer, delegates to executor
  └── ui/
      ├── main-window.slint      # updated
      ├── theme.slint
      ├── widgets.slint
      ├── toolbar.slint          # updated
      ├── menu-bar.slint
      ├── repo-panel.slint
      ├── branch-panel.slint
      ├── commit-graph.slint     # updated
      ├── commit-details.slint
      ├── file-list.slint        # updated
      ├── diff-view.slint        # updated
      ├── diff-side-by-side.slint    # NEW
      ├── repo-dialog.slint
      ├── group-dialog.slint
      ├── commit-dialog.slint       # NEW
      ├── clone-dialog.slint        # NEW
      ├── push-dialog.slint         # NEW
      ├── pull-dialog.slint         # NEW
      ├── checkout-dialog.slint     # NEW
      └── confirm-dialog.slint      # NEW
```
