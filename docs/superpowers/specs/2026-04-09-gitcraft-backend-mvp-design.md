# GitStream Backend MVP — Design Spec

Git GUI backend на Tauri 2 (Rust), git CLI обёртка, Vue.js composables для state management.

**Git backend:** только git CLI (`std::process::Command`), парсинг `--porcelain`/`--format` вывода
**State management:** Vue composables (ref + invoke), без Pinia
**IPC:** Tauri `#[tauri::command]` → JSON → фронтенд

---

## 1. Архитектура

### Data Flow

```
Vue Component → composable.action() → invoke('command', args)
                                            ↓
                                      #[tauri::command]
                                            ↓
                                      git CLI (process::Command)
                                            ↓
                                      parse stdout → Rust struct
                                            ↓
                                      JSON → composable ref → UI
```

После каждой мутации (stage, commit, push...) composable автоматически вызывает refresh затронутых данных.

---

## 2. Rust Backend (src-tauri/src/)

### Структура файлов

```
src-tauri/src/
├── main.rs              # Tauri bootstrap, регистрация commands
├── commands.rs          # #[tauri::command] функции (тонкий слой)
└── git/
    ├── mod.rs
    ├── types.rs         # Rust-структуры с Serialize
    ├── error.rs         # GitError, classify_git_error
    ├── query.rs         # read-only операции
    └── mutation.rs      # мутации
```

### git/types.rs

```rust
#[derive(Serialize, Clone)]
pub struct FileStatus {
    pub path: String,
    pub state: String,       // "modified", "added", "deleted", "renamed", "untracked", "conflicted"
    pub staged: String,      // "staged", "unstaged", "partial"
}

#[derive(Serialize, Clone)]
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

#[derive(Serialize, Clone)]
pub struct RefLabel {
    pub name: String,
    pub kind: String,        // "local-branch", "remote-branch", "tag", "head", "stash"
}

#[derive(Serialize, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub is_current: bool,
}

#[derive(Serialize, Clone)]
pub struct TagInfo {
    pub name: String,
    pub oid: String,
    pub message: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct DiffLine {
    pub kind: String,        // "context", "added", "removed"
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(Serialize, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Serialize, Clone)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Serialize, Clone)]
pub struct RepoInfo {
    pub path: String,
    pub current_branch: String,
    pub head_oid: String,
}
```

### git/error.rs

```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git command failed: {message}")]
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

impl Serialize for GitError { ... }

/// Парсит stderr git-команды и возвращает типизированную ошибку с hint-подсказкой.
pub fn classify_git_error(stderr: &str) -> GitError { ... }
```

Классификация ошибок по паттернам stderr:
- `Authentication failed` → AuthenticationFailed + hint про credential helper
- `Permission denied (publickey)` → AuthenticationFailed + hint про ssh-add
- `Could not resolve host` → CommandFailed + hint проверить сеть
- `rejected.*non-fast-forward` → CommandFailed + hint pull first
- `nothing to commit` → NothingToCommit
- `CONFLICT` → MergeConflict

### git/query.rs

Все функции принимают `repo_path: &Path` и возвращают `Result<T, GitError>`.

| Функция | Git-команда | Формат парсинга |
|---|---|---|
| `status(path)` → `Vec<FileStatus>` | `git status --porcelain=v2` | Porcelain v2: XY codes |
| `log(path, limit)` → `Vec<CommitInfo>` | `git log --format=<custom> -n <limit>` | `%H%x00%h%x00%s%x00%an%x00%ae%x00%aI%x00%P%x00%D` с NUL-разделителями |
| `branches(path)` → `Vec<BranchInfo>` | `git branch -a --format=<custom>` | `%(refname:short)%00%(upstream:short)%00%(upstream:track,nobracket)%00%(HEAD)` |
| `tags(path)` → `Vec<TagInfo>` | `git tag -l --format=<custom>` | `%(refname:short)%00%(*objectname:short)%00%(contents:subject)` |
| `stashes(path)` → `Vec<StashEntry>` | `git stash list --format=<custom>` | `%gd%00%gs` |
| `remotes(path)` → `Vec<String>` | `git remote` | По строкам |
| `diff_file(path, file, staged)` → `FileDiff` | `git diff [--cached] -- <file>` | Unified diff парсинг |
| `diff_commit(path, oid)` → `Vec<FileDiff>` | `git diff <oid>^..<oid>` | Unified diff парсинг |
| `show_commit(path, oid)` → `CommitInfo` | `git show --format=<custom> -s <oid>` | Тот же формат что log |
| `repo_info(path)` → `RepoInfo` | `git rev-parse --show-toplevel`, `git branch --show-current`, `git rev-parse HEAD` | По строкам |

Вспомогательная функция:
```rust
fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, GitError>
```
Запускает `git -C <repo_path> <args>`, при ненулевом exit code вызывает `classify_git_error(stderr)`.

### git/mutation.rs

Все функции принимают `repo_path: &Path` + аргументы операции.

| Функция | Git-команда |
|---|---|
| `stage(path, files)` | `git add -- <files>` |
| `unstage(path, files)` | `git restore --staged -- <files>` |
| `discard(path, files)` | `git restore -- <files>` |
| `commit(path, message, amend)` | `git commit -m <msg> [--amend]` |
| `checkout(path, branch)` | `git switch <branch>` / `git switch -c <local> <remote>` для remote |
| `fetch(path, remote)` | `git fetch <remote>` |
| `pull(path, remote, rebase)` | `git pull [--rebase] <remote>` |
| `push(path, remote, force)` | `git push [--force] <remote>` |
| `clone_repo(url, dest)` | `git clone <url> <dest>` |

Сетевые операции (fetch, pull, push, clone) выполняются через `tokio::task::spawn_blocking` чтобы не блокировать UI.

### commands.rs

Тонкий слой `#[tauri::command]` → вызов git/ функций:

```rust
#[tauri::command]
async fn get_status(repo_path: String) -> Result<Vec<FileStatus>, String> {
    git::query::status(Path::new(&repo_path)).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_log(repo_path: String, limit: Option<usize>) -> Result<Vec<CommitInfo>, String> {
    git::query::log(Path::new(&repo_path), limit.unwrap_or(500)).map_err(|e| e.to_string())
}

#[tauri::command]
async fn stage_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    git::mutation::stage(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

// ... аналогично для остальных операций
```

Полный список команд: `get_status`, `get_log`, `get_branches`, `get_tags`, `get_stashes`, `get_remotes`, `get_diff_file`, `get_diff_commit`, `get_show_commit`, `get_repo_info`, `stage_files`, `unstage_files`, `discard_files`, `do_commit`, `do_checkout`, `do_fetch`, `do_pull`, `do_push`, `do_clone`.

### main.rs

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_log,
            commands::get_branches,
            // ... все команды
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 3. Frontend Composables (src/composables/)

### useRepo

```typescript
// Текущий открытый репозиторий
const repoPath: Ref<string | null>
const repoInfo: Ref<RepoInfo | null>

async function openRepo(path: string): Promise<void>
async function refreshAll(): Promise<void>  // обновить все данные
```

### useFiles

```typescript
const files: Ref<FileStatus[]>
const selectedFile: Ref<string | null>

async function refresh(): Promise<void>
async function stageFiles(paths: string[]): Promise<void>    // + refresh
async function unstageFiles(paths: string[]): Promise<void>  // + refresh
async function discardFiles(paths: string[]): Promise<void>  // + refresh
```

### useBranches

```typescript
const branches: Ref<BranchInfo[]>
const tags: Ref<TagInfo[]>
const stashes: Ref<StashEntry[]>
const remotes: Ref<string[]>

async function refresh(): Promise<void>
async function checkout(branch: string): Promise<void>  // + refreshAll
```

### useLog

```typescript
const commits: Ref<CommitInfo[]>
const selectedCommit: Ref<string | null>

async function refresh(limit?: number): Promise<void>
async function showCommit(oid: string): Promise<CommitInfo>
```

### useDiff

```typescript
const currentDiff: Ref<FileDiff | null>

async function diffFile(path: string, staged: boolean): Promise<void>
async function diffCommit(oid: string): Promise<FileDiff[]>
```

### useCommit

```typescript
async function commit(message: string, amend: boolean): Promise<void>  // + refreshAll
```

### useRemote

```typescript
const isBusy: Ref<boolean>
const lastError: Ref<string | null>

async function fetch(remote: string): Promise<void>    // + refresh
async function pull(remote: string, rebase: boolean): Promise<void>  // + refreshAll
async function push(remote: string, force: boolean): Promise<void>   // + refresh
async function cloneRepo(url: string, dest: string): Promise<void>
```

### Связь composables и компонентов

| Компонент | Composable |
|---|---|
| BranchPanel | useBranches |
| FileList | useFiles |
| CommitGraph | useLog |
| CommitDetails | useLog.showCommit |
| DiffView | useDiff |
| CommitDialog | useCommit, useFiles |
| CloneDialog | useRemote |
| PushDialog | useRemote |
| PullDialog | useRemote |
| CheckoutDialog | useBranches |
| StatusBar | useRepo, useRemote |
| AppToolbar | triggers composable actions |

---

## 4. Интеграция в компоненты

Компоненты заменяют hardcoded моковые данные на composable refs:

```vue
<script setup>
import { useFiles } from '@/composables/useFiles'
const { files, stageFiles, unstageFiles } = useFiles()
</script>
```

При монтировании App.vue:
1. Открываем репозиторий (из аргумента CLI или последний открытый)
2. `refreshAll()` — загружает status, branches, tags, stashes, log
3. UI отображает реальные данные

---

## 5. Зависимости Cargo

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["process", "rt-multi-thread"] }
```

Без gix, без дополнительных крейтов. Только стандартная библиотека + tauri + serde + tokio для async.
