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
├── commands.rs               # #[tauri::command] — IPC endpoints
└── git/
    ├── types.rs              # Serialize-структуры
    ├── error.rs              # GitError, classify_git_error
    ├── query.rs              # run_git, status, log, branches, tags,
    │                         # stashes, remotes, repo_info, diff
    └── mutation.rs           # stage, unstage, discard, commit, checkout,
                              # merge, branch/tag ops, fetch, pull, push, clone
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

## Реализовано

### Операции
- Stage / Unstage / Discard файлов
- Commit (с amend)
- Push / Pull / Fetch (с диалогами выбора remote, таймаут сетевых операций)
- Clone репозитория
- Checkout branch (локальная + remote-ветка с созданием локальной)
- Merge ветки
- Ветки: rename, delete (с force)
- Теги: create (lightweight/annotated), delete, push tag
- Settings dialog (тема, таймаут сети)

### Просмотр
- Unified + Side-by-side diff с переключателем
- Лог коммитов с деталями
- Список веток / тегов / stash
- Панель файлов со статусами
- File compare диалог

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
- **Tauri IPC** — `#[tauri::command]` endpoints (query + mutation)
- **Git CLI** — парсинг `--porcelain=v2` (status), `--format` с NUL-разделителями (log, branches, tags), unified diff
- **Composables** — реактивное состояние через Vue ref(), автообновление после мутаций
- **Обработка ошибок** — classify_git_error: auth, network, conflict, hint-подсказки
- **Сетевые операции** — async + spawn_blocking, настраиваемый таймаут (см. memory)

---

## Дорожная карта (в работе)

Порядок реализации до «полноценного Git-клиента»:

1. **Частичный stage** — stage/unstage отдельных хунков и строк (`git apply --cached`)
   прямо из DiffView
2. **Stash-мутации** — save (с сообщением, `--include-untracked`), apply, pop, drop
3. **Создание ветки** — от HEAD / выбранного коммита / другой ветки, опц. checkout
4. **Reset / Revert / Cherry-pick** — из контекстного меню CommitGraph
   (reset --soft/--mixed/--hard, revert, cherry-pick)
5. **Разрешение конфликтов** — список конфликтных файлов, accept ours/theirs,
   merge/rebase --abort/--continue
6. **Rebase** — ветка на ветку, --abort/--continue (interactive — позже)

## За пределами дорожной карты (будущее)

- Interactive rebase UI, 3-way conflict resolver
- Blame view, file history, reflog
- Управление remote (add/remove/set-url, upstream-трекинг, prune)
- Поиск/фильтрация в логе
- git init / open локальной папки
- Syntax highlighting в diff, word-level diff
- Commit graph с визуальными линиями (lane allocation)
- Сохранение дерева репозиториев в конфиг
- Аутентификация (login/password, SSH passphrase)
- GPG signing, Git-Flow, Submodules, LFS
- Bisect, patches, bundles, archive

