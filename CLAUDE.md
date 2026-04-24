# GitStream

Git-клиент с графическим интерфейсом на **Vue.js + Tauri**. Фокус на повседневных Git-операциях с неблокирующим UI.

---

## О проекте

GitStream — самостоятельный Git GUI. Приоритет — удобство ежедневной работы с Git: коммиты, ветки, push/pull, просмотр diff.

**Технологии:**
- **Frontend:** Vue.js 3 (Composition API), TypeScript, Vite
- **Backend:** Tauri 2 (Rust)
- **Git:** git CLI (`std::process::Command`), парсинг `--porcelain`/`--format` вывода
- **State management:** Vue composables (ref + invoke), без Pinia

**Архитектура:**

```
┌─────────────────────────────────────┐
│        UI (Vue.js + TypeScript)     │
│  SFC-компоненты, composables        │
├─────────────────────────────────────┤
│        Tauri IPC (invoke)           │
│  #[tauri::command] → JSON           │
├─────────────────────────────────────┤
│        Backend (Rust / Tauri)       │
│  git CLI обёртка                    │
│  query.rs — read-only операции      │
│  mutation.rs — мутации              │
└─────────────────────────────────────┘
```

**Структура проекта:**

```
src/                          # Vue.js frontend
├── components/               # UI-компоненты
│   ├── AppToolbar.vue        # Панель инструментов
│   ├── RepositoriesPanel.vue # Treeview репозиториев (drag-and-drop)
│   ├── BranchPanel.vue       # Ветки, теги, stashes
│   ├── CommitGraph.vue       # Лог коммитов
│   ├── CommitDetails.vue     # Детали выбранного коммита
│   ├── FileList.vue          # Список файлов со статусами
│   ├── DiffView.vue          # Unified/Side-by-side diff
│   ├── StatusBar.vue         # Строка состояния
│   └── dialogs/              # Модальные диалоги
│       ├── CommitDialog.vue
│       ├── CloneDialog.vue
│       ├── PushDialog.vue
│       ├── PullDialog.vue
│       ├── CheckoutDialog.vue
│       └── ConfirmDialog.vue
├── composables/              # State management
│   ├── useRepo.ts            # Текущий репозиторий
│   ├── useFiles.ts           # Статус файлов, stage/unstage/discard
│   ├── useBranches.ts        # Ветки, теги, stashes, remotes
│   ├── useLog.ts             # Лог коммитов
│   ├── useDiff.ts            # Diff файлов и коммитов
│   ├── useCommit.ts          # Создание коммитов
│   └── useRemote.ts          # Fetch, pull, push, clone
├── types/index.ts            # TypeScript-типы (snake_case, как в Rust JSON)
└── styles/                   # CSS (тёмная тема, Catppuccin-inspired)

src-tauri/src/                # Rust backend
├── main.rs                   # Tauri bootstrap, регистрация commands
├── commands.rs               # #[tauri::command] — 18 IPC endpoints
└── git/
    ├── types.rs              # Serialize-структуры
    ├── error.rs              # GitError, classify_git_error
    ├── query.rs              # run_git, status, log, branches, tags,
    │                         # stashes, remotes, repo_info, diff
    └── mutation.rs           # stage, unstage, discard, commit,
                              # checkout, fetch, pull, push, clone
```

**Layout UI:**

```
┌──────────────────────────────────────────────────────┐
│  Toolbar (Pull, Push, Fetch, Commit, Checkout...)    │
├─────────────┬─────────────────────┬──────────────────┤
│ Repositories│                     │ Commit Details   │
│ (treeview,  │  Commit Graph       │ (SHA, author,    │
│  drag-drop) │  (лог коммитов)     │  date, message)  │
├─────────────┤                     ├──────────────────┤
│ Branches    │                     │ Files            │
│ Tags        │                     │ (статусы файлов) │
│ Stashes     │                     │                  │
├─────────────┴─────────────────────┴──────────────────┤
│  Diff View (Unified / Side-by-side)                  │
├──────────────────────────────────────────────────────┤
│  Status Bar (branch, ahead/behind, status)           │
└──────────────────────────────────────────────────────┘
```

Все панели с drag-resize ручками.

---

## Scope MVP (реализовано)

### Операции
- Stage / Unstage / Discard файлов
- Commit (с amend)
- Push / Pull / Fetch (с диалогами выбора remote)
- Clone репозитория
- Checkout branch

### Просмотр
- Unified + Side-by-side diff с переключателем
- Лог коммитов с деталями
- Список веток / тегов / stash
- Панель файлов со статусами

### Repositories (treeview)
- Drag-and-drop: репозитории в папки, папки в папки, на верхний уровень
- Двойной клик — переключение на репозиторий
- Контекстное меню: Add Repository, Add Group, Delete

### Диалоги MVP
- **Commit Dialog** — сообщение, amend, индикатор длины строки, Commit & Push
- **Clone Dialog** — URL + путь, автоимя папки из URL
- **Push Dialog** — выбор remote, force push с предупреждением
- **Pull Dialog** — выбор remote, merge/rebase, behind-счётчик
- **Checkout Branch Dialog** — поиск и фильтрация веток
- **Confirm Dialog** — универсальный диалог подтверждения
- Все диалоги закрываются по Esc

### Архитектура
- **Tauri IPC** — 18 команд: get_status, get_log, get_branches, get_tags, get_stashes, get_remotes, get_diff_file, get_diff_commit, get_repo_info, stage_files, unstage_files, discard_files, do_commit, do_checkout, do_fetch, do_pull, do_push, do_clone
- **Git CLI** — парсинг `--porcelain=v2` (status), `--format` с NUL-разделителями (log, branches, tags), unified diff
- **Composables** — реактивное состояние через Vue ref(), автообновление после мутаций
- **Обработка ошибок** — classify_git_error: auth, network, conflict, hint-подсказки

---

## За пределами MVP (будущее)

- Merge / Rebase диалоги и interactive rebase UI
- 3-way conflict resolver
- Blame view
- Stash операции (save/apply/pop/drop)
- Settings dialog (user.name, email, тема)
- Syntax highlighting в diff
- Commit graph с визуальными линиями (lane allocation)
- Сохранение дерева репозиториев в конфиг
- Git-Flow поддержка
- Submodules, LFS, GPG signing
- Bisect, patches, bundles, archive

