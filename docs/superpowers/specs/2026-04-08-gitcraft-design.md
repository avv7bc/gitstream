# GitStream — Design Spec

Git GUI клиент на базе Rust и Slint. Внутренний инструмент для личного использования.

**Платформы:** Linux, macOS
**Git backend:** gitoxide (gix) + git CLI fallback для сложных операций
**UI framework:** Slint
**Async runtime:** tokio

---

## 1. Общая архитектура

Трёхслойная архитектура:

```
┌─────────────────────────────────────┐
│           UI (Slint)                │
│  .slint файлы + Rust-адаптеры      │
├─────────────────────────────────────┤
│        Application Layer            │
│  команды, состояние, события        │
├─────────────────────────────────────┤
│         Git Backend                 │
│  trait GitOps                       │
│  ├── GixBackend (gitoxide)          │
│  └── CliBackend (git CLI fallback)  │
└─────────────────────────────────────┘
```

### Cargo Workspace

| Крейт | Назначение |
|---|---|
| `gitstream-git` | Trait `GitOps` + реализации (gix, CLI) |
| `gitstream-core` | Доменная логика, модели, команды |
| `gitstream-ui` | Slint UI, адаптеры, привязки данных |
| `gitstream` | Бинарник, точка входа |

### Ключевые решения

- Однопоточный UI — Slint рендерит в main thread, все git-операции выполняются в фоновых tokio тасках
- UI <-> Core общаются через `slint::invoke_from_event_loop` + каналы (tokio mpsc)
- Единый `AppState` в Core, UI подписывается на изменения через Slint properties/callbacks

---

## 2. Git Backend (`gitstream-git`)

### Центральный trait

```rust
pub trait GitOps: Send + Sync {
    // Репозиторий
    fn open(path: &Path) -> Result<Self> where Self: Sized;
    fn clone_repo(url: &str, path: &Path, opts: CloneOpts) -> Result<Self> where Self: Sized;
    fn init(path: &Path) -> Result<Self> where Self: Sized;

    // Статус и индекс
    fn status(&self) -> Result<Vec<FileStatus>>;
    fn stage(&self, paths: &[&Path]) -> Result<()>;
    fn unstage(&self, paths: &[&Path]) -> Result<()>;
    fn stage_hunks(&self, path: &Path, hunks: &[HunkSelection]) -> Result<()>;

    // Коммиты
    fn commit(&self, msg: &str, opts: CommitOpts) -> Result<Oid>;
    fn amend(&self, msg: &str) -> Result<Oid>;
    fn log(&self, opts: LogOpts) -> Result<Vec<CommitInfo>>;
    fn diff(&self, from: DiffTarget, to: DiffTarget) -> Result<DiffResult>;
    fn blame(&self, path: &Path) -> Result<Vec<BlameLine>>;

    // Ветки
    fn branches(&self, filter: BranchFilter) -> Result<Vec<BranchInfo>>;
    fn create_branch(&self, name: &str, target: &str) -> Result<()>;
    fn delete_branch(&self, name: &str, force: bool) -> Result<()>;
    fn checkout(&self, target: &str) -> Result<()>;

    // Remote
    fn fetch(&self, remote: &str, opts: FetchOpts) -> Result<()>;
    fn pull(&self, opts: PullOpts) -> Result<()>;
    fn push(&self, opts: PushOpts) -> Result<()>;

    // Сложные операции (CLI fallback)
    fn merge(&self, branch: &str, opts: MergeOpts) -> Result<MergeResult>;
    fn rebase(&self, onto: &str, opts: RebaseOpts) -> Result<RebaseResult>;
    fn cherry_pick(&self, commits: &[Oid]) -> Result<()>;
    fn stash_save(&self, msg: Option<&str>, opts: StashOpts) -> Result<()>;
    fn stash_pop(&self, index: usize) -> Result<()>;
    fn stash_list(&self) -> Result<Vec<StashEntry>>;

    // Теги
    fn tags(&self) -> Result<Vec<TagInfo>>;
    fn create_tag(&self, name: &str, target: &str, opts: TagOpts) -> Result<()>;
    fn delete_tag(&self, name: &str) -> Result<()>;
}
```

### Распределение по бэкендам

| Операция | GixBackend | CliBackend |
|---|---|---|
| open, init, clone | + | |
| status, stage, unstage, stage_hunks | + | |
| commit, amend, log, diff, blame | + | |
| branches, checkout, tags | + | |
| fetch, push, pull | + | |
| merge | | + |
| rebase (включая interactive) | | + |
| cherry-pick | | + |
| stash | | + |

Реализован как единый `CompositeBackend`, который внутри маршрутизирует вызовы на gix или CLI в зависимости от операции. Снаружи — единый интерфейс `GitOps`.

CliBackend — обёртка над `tokio::process::Command`, парсит вывод git через структурированные форматы (`--format`, `--porcelain`) где возможно. Ошибки маппятся в единый `GitError` enum.

---

## 3. Доменные модели (`gitstream-core`)

### Основные структуры данных

```rust
// Файлы и статус
pub struct FileStatus {
    pub path: PathBuf,
    pub state: FileState,        // Modified, Added, Deleted, Renamed, Conflicted, Untracked
    pub staged: StagedState,     // Staged, Unstaged, Partial
}

// Коммиты
pub struct CommitInfo {
    pub oid: Oid,
    pub short_oid: String,
    pub message: String,
    pub author: Signature,
    pub committer: Signature,
    pub parents: Vec<Oid>,
    pub refs: Vec<RefLabel>,     // ветки, теги, HEAD
}

// Граф коммитов
pub struct GraphRow {
    pub commit: CommitInfo,
    pub column: usize,
    pub lines: Vec<GraphLine>,
}

// Diff
pub struct DiffResult {
    pub files: Vec<FileDiff>,
}
pub struct FileDiff {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub hunks: Vec<Hunk>,
    pub binary: bool,
}
pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}
pub struct DiffLine {
    pub kind: LineKind,          // Context, Added, Removed
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

// Blame
pub struct BlameLine {
    pub oid: Oid,
    pub author: String,
    pub date: DateTime<Utc>,
    pub lineno: u32,
    pub content: String,
}

// Ветки
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub tip: Oid,
}

// Stash
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub oid: Oid,
}

// Merge/Rebase
pub enum MergeResult {
    Clean(Oid),
    Conflict(Vec<ConflictFile>),
}
pub struct ConflictFile {
    pub path: PathBuf,
    pub ours: String,
    pub theirs: String,
    pub base: Option<String>,
}
pub enum RebaseResult {
    Complete,
    Conflict { step: usize, total: usize, conflict: Vec<ConflictFile> },
    Aborted,
}
```

### AppState

```rust
pub struct AppState {
    pub repo_path: PathBuf,
    pub files: Vec<FileStatus>,
    pub log: Vec<GraphRow>,
    pub branches: Vec<BranchInfo>,
    pub tags: Vec<TagInfo>,
    pub stashes: Vec<StashEntry>,
    pub current_branch: String,
    pub head: Oid,
    pub selected_commit: Option<Oid>,
    pub current_diff: Option<DiffResult>,
    pub operation: Option<OngoingOperation>,
}

pub enum OngoingOperation {
    Merge { branch: String, conflicts: Vec<ConflictFile> },
    Rebase { onto: String, step: usize, total: usize, conflicts: Vec<ConflictFile> },
    CherryPick { oid: Oid, conflicts: Vec<ConflictFile> },
}
```

### Команды

Каждое действие пользователя маппится в Command enum, отправляется из UI через канал, обрабатывается в CommandExecutor, результат обновляет AppState, UI реагирует на изменения.

```rust
pub enum Command {
    Refresh,
    Stage(Vec<PathBuf>),
    Unstage(Vec<PathBuf>),
    StageHunks(PathBuf, Vec<HunkSelection>),
    Commit { message: String, amend: bool },
    Checkout(String),
    CreateBranch { name: String, start: String },
    DeleteBranch { name: String, force: bool },
    Fetch { remote: String },
    Pull(PullOpts),
    Push(PushOpts),
    Merge { branch: String, opts: MergeOpts },
    Rebase { onto: String, opts: RebaseOpts },
    CherryPick(Vec<Oid>),
    StashSave(Option<String>),
    StashPop(usize),
}
```

---

## 4. UI (`gitstream-ui`)

### Компоновка главного окна

```
┌──────────────────────────────────────────────────────┐
│  Menu Bar                                            │
├──────────────────────────────────────────────────────┤
│  Toolbar (commit, push, pull, fetch, branch, merge)  │
├────────────┬─────────────────────────────────────────┤
│            │                                         │
│  Branches  │  File List                              │
│  Tags      │  (status, staged/unstaged группы)       │
│  Stashes   │                                         │
│  Remotes   │                                         │
│            ├─────────────────────────────────────────┤
│            │                                         │
│            │  Diff / Blame View                      │
│            │  (подсветка синтаксиса, hunk staging)   │
│            │                                         │
├────────────┴─────────────────────────────────────────┤
│                                                      │
│  Commit Graph / Log                                  │
│  (граф, сообщение, автор, дата, хеш)                 │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### Slint-компоненты

| Компонент | Файл | Назначение |
|---|---|---|
| `MainWindow` | `main-window.slint` | Корневое окно, layout панелей |
| `Toolbar` | `toolbar.slint` | Кнопки основных операций |
| `BranchPanel` | `branch-panel.slint` | Дерево веток/тегов/stash/remotes |
| `FileList` | `file-list.slint` | Список файлов со статусами |
| `DiffView` | `diff-view.slint` | Diff с подсветкой, выбор hunks |
| `BlameView` | `blame-view.slint` | Построчное авторство |
| `CommitGraph` | `commit-graph.slint` | Граф коммитов с колонками |
| `CommitDialog` | `commit-dialog.slint` | Ввод сообщения, amend |
| `MergeDialog` | `merge-dialog.slint` | Выбор ветки, стратегии |
| `ConflictResolver` | `conflict-resolver.slint` | 3-way merge панель |
| `RebaseDialog` | `rebase-dialog.slint` | Интерактивный rebase |

### Связь UI <-> Core

Каждый Slint-компонент получает Rust-адаптер:
- маппит AppState -> Slint models (VecModel, MapModel)
- преобразует Slint callbacks -> Command в канал

```rust
pub struct FileListAdapter {
    model: Rc<VecModel<FileListItem>>,
    cmd_tx: mpsc::Sender<Command>,
}

impl FileListAdapter {
    pub fn update(&self, files: &[FileStatus]) { /* ... */ }
    pub fn on_stage(&self, indices: &[usize]) { /* ... */ }
}
```

### Подсветка синтаксиса

Через `syntect` (Rust-библиотека), результат рендерится как цветные span-ы в Slint.

### Тема

Тёмная по умолчанию, переключение светлая/тёмная через настройки.

---

## 5. Conflict Resolver (3-way merge)

### Компоновка

```
┌─────────────────┬─────────────────┬─────────────────┐
│     BASE        │     OURS        │    THEIRS       │
│  (общий предок) │ (текущая ветка) │ (входящие)      │
├─────────────────┴─────────────────┴─────────────────┤
│                    RESULT                            │
│  (редактируемый результат слияния)                   │
└─────────────────────────────────────────────────────┘
```

### Логика

- ConflictFile содержит три версии (base, ours, theirs)
- Каждый конфликтный блок: кнопки Accept Ours / Accept Theirs / Accept Both / Edit Manually
- Цвета: зелёный (ours), синий (theirs), красный (нерешённый)
- Нижняя панель — редактируемый результат
- Кнопка Mark Resolved после разрешения всех конфликтов в файле
- Навигация между конфликтными файлами

### Парсинг конфликтов

```rust
pub struct ConflictBlock {
    pub base: Vec<String>,
    pub ours: Vec<String>,
    pub theirs: Vec<String>,
    pub line_start: usize,
    pub resolved: Option<Vec<String>>,
}
```

Конфликтные маркеры (`<<<<<<<`, `=======`, `>>>>>>>`, `|||||||`) парсятся из файлов рабочей копии. Для diff3-формата доступна секция base.

---

## 6. Интерактивный Rebase

### UI

```
┌──────────────────────────────────────────────────┐
│  Interactive Rebase onto: <branch>               │
├──────┬───────────────────────────────────────────┤
│ pick │ a1b2c3d  Fix login validation             │
│ pick │ e4f5g6h  Add password reset               │
│ pick │ i7j8k9l  Update tests                     │
│ pick │ m0n1o2p  Cleanup imports                  │
├──────┴───────────────────────────────────────────┤
│  [Pick] [Reword] [Squash] [Fixup] [Drop]        │
│  [Move Up] [Move Down]                           │
│  [Start Rebase]              [Cancel]            │
└──────────────────────────────────────────────────┘
```

### Действия

- Pick — оставить как есть
- Reword — изменить сообщение
- Squash — объединить с предыдущим, сохранить оба сообщения
- Fixup — объединить с предыдущим, отбросить сообщение
- Drop — удалить коммит
- Drag & drop для изменения порядка

### Реализация

```rust
pub struct RebaseEntry {
    pub action: RebaseAction,
    pub oid: Oid,
    pub message: String,
}
pub enum RebaseAction { Pick, Reword, Squash, Fixup, Drop }
```

Формируем todo-список -> записываем во временный файл -> `git rebase -i` с `GIT_SEQUENCE_EDITOR` подставляющим наш файл. При Reword — перехватываем `GIT_EDITOR` для ввода нового сообщения через UI-диалог. При конфликте — переход в Conflict Resolver, затем `--continue` или `--abort`.

---

## 7. Граф коммитов

### Алгоритм построения

Lane allocation (аналог gitk):

1. Обход коммитов в топологическом порядке
2. Каждая активная линия занимает колонку (lane)
3. При merge — линии сходятся, при branch — расходятся
4. Цвет привязан к ветке (первый parent сохраняет цвет)

```rust
pub struct GraphBuilder {
    lanes: Vec<Option<Oid>>,
}
pub struct GraphLine {
    pub from_column: usize,
    pub to_column: usize,
    pub color: usize,
    pub style: LineStyle,     // Straight, MergeLeft, MergeRight, Fork
}
```

### Рендеринг

Граф рисуется через кастомный Slint-компонент с Path элементами. Таблица коммитов рядом, синхронизированный скролл.

### Колонки таблицы

Graph | Message (+ ref labels) | Author | Date | Hash

### Фильтрация и поиск

- По автору, сообщению, дате, пути файла
- Pickaxe search (-S / -G)
- Фильтр по ветке

### Производительность

- Ленивая загрузка порциями по 500 при скролле
- Инкрементальное построение графа
- Кэширование, пересчёт при fetch/commit

---

## 8. Настройки и горячие клавиши

### Конфигурация

```
~/.config/gitstream/
├── config.toml
└── keybindings.toml
```

### config.toml

```toml
[ui]
theme = "dark"
font_family = "JetBrains Mono"
font_size = 13
date_format = "relative"
tab_size = 4

[git]
auto_fetch_interval = 300
default_pull_mode = "rebase"
sign_commits = false

[diff]
context_lines = 3
word_diff = true

[window]
restore_layout = true
```

### Горячие клавиши (дефолтные, переопределяемые)

| Действие | Клавиша |
|---|---|
| Commit | Ctrl+K |
| Stage | Ctrl+T |
| Unstage | Ctrl+U |
| Push | Ctrl+Shift+K |
| Pull | Ctrl+Shift+L |
| Fetch | Ctrl+F |
| New Branch | Ctrl+B |
| Find/Filter | Ctrl+P |
| Next conflict | F7 |
| Prev conflict | Shift+F7 |
| Refresh | F5 |

### keybindings.toml

```toml
[keybindings]
commit = "Ctrl+K"
stage = "Ctrl+T"
push = "Ctrl+Shift+K"
```

---

## 9. Scope

### В scope (MVP)

- Открытие / клонирование / init репозиториев
- Статус файлов, stage/unstage, partial staging (hunks, lines)
- Commit, amend
- Лог с графом коммитов, фильтрация, поиск
- Diff с подсветкой синтаксиса, word-level diff
- Blame
- Ветки: создание, удаление, checkout, переименование
- Теги: создание, удаление
- Fetch, pull, push
- Merge с разрешением конфликтов (3-way merge tool)
- Rebase, включая интерактивный
- Cherry-pick
- Stash: save, pop, list, drop
- Тёмная / светлая тема
- Настраиваемые горячие клавиши
- Linux + macOS

### НЕ в scope

- Интеграция с GitHub / GitLab / Bitbucket / Azure DevOps
- Git-Flow
- SVN / Mercurial
- Git LFS
- Submodules
- Worktrees
- GPG-подпись коммитов и тегов
- Bisect
- Reflog
- Patches, bundles, archives
- Spell-checking
- Менеджер репозиториев (работаем с одним репо)
- Встроенный терминал
- Windows
- Автообновление
- Система плагинов

---

## 10. Зависимости (ключевые крейты)

| Крейт | Назначение |
|---|---|
| `slint` | UI framework |
| `gix` | Git операции (gitoxide) |
| `tokio` | Async runtime |
| `syntect` | Подсветка синтаксиса |
| `serde` + `toml` | Конфигурация |
| `chrono` | Работа с датами |
| `thiserror` / `anyhow` | Обработка ошибок |
