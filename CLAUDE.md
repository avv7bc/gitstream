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
│   ├── AppToolbar.vue        # Тулбар: Repository▾/Local▾/Branch▾ + Pull/Push/Fetch
│   ├── RepositoriesPanel.vue # Treeview репозиториев (drag-and-drop)
│   ├── BranchPanel.vue       # Ветки, теги, stashes
│   ├── CommitGraph.vue       # Лог коммитов
│   ├── CommitDetails.vue     # Детали выбранного коммита
│   ├── FileList.vue          # Список файлов со статусами
│   ├── DiffPanel.vue         # Unified diff
│   ├── SideBySideDiffView.vue # Side-by-side diff + выбор строк
│   ├── ConflictBar.vue       # Управление merge/rebase/cherry-pick/revert
│   ├── StatusBar.vue         # Строка состояния
│   └── dialogs/              # Модальные диалоги (см. ниже)
├── composables/              # State management
│   ├── useRepo.ts            # Текущий репозиторий
│   ├── useFiles.ts           # Статус файлов, stage/unstage/discard, частичный stage
│   ├── useBranches.ts        # Ветки, теги, stashes, remotes
│   ├── useLog.ts             # Лог коммитов, reset/revert/cherry-pick
│   ├── useDiff.ts            # Diff файлов и коммитов
│   ├── useCommit.ts          # Создание коммитов, squash, reword
│   ├── useRemote.ts          # Fetch, pull, push
│   ├── useConflicts.ts       # Состояние merge/rebase + accept ours/theirs
│   ├── useI18n.ts            # Локализация RU/EN
│   ├── useSettings.ts        # Настройки приложения
│   └── ...                   # useDraggable, useTheme, useUpdate, useVirtualList и др.
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
                              # merge, branch/tag ops, fetch, pull, push
```

**Layout UI:**

```
┌──────────────────────────────────────────────────────┐
│  Toolbar: Repository▾ Local▾ | Pull Push Fetch | Branch▾ │
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
- **Частичный stage** — выбор строк/хунков в Side-by-side diff, stage/unstage/discard выбранного (`git apply --cached`)
- `git rm` для tracked / disk-delete для untracked
- **Множественное выделение** (Shift / Ctrl-клик) в панели файлов и BranchPanel — групповые операции над выбранным: stage/unstage/commit/discard/remove/delete файлов, batch delete веток, batch delete/push тегов
- **Ctrl+A** — выделить все файлы в FileList (working tree и commit-файлы)
- Commit (с amend), Reword (HEAD — amend; не-HEAD — rebase -i со сценарным редактором)
- Squash нескольких коммитов в один
- Push / Pull / Fetch (с диалогами выбора remote, таймаут сетевых операций, индикатор remote + обратный отсчёт)
- Checkout branch (локальная + remote-ветка с созданием локальной)
- Merge ветки
- Rebase ветки на ветку (с continue/abort через ConflictBar)
- Ветки: create (от HEAD / коммита / другой ветки), rename, delete (с force), batch delete выделенных локальных веток
- Теги: create (lightweight/annotated), delete, push tag, batch delete / push выделенных тегов
- Stash: save (с сообщением, `--include-untracked`), apply, pop, drop
- Reset (soft / mixed / hard), Revert, Cherry-pick — из контекстного меню CommitGraph
- Разрешение конфликтов: accept ours/theirs, abort/continue для merge/rebase/cherry-pick/revert
- Settings dialog (тема, шрифты, таймаут сети, язык RU/EN)

### Горячие клавиши
- **Ctrl+K** — открыть диалог коммита
- **Ctrl+T** — stage выбранных файлов
- **Shift+Ctrl+T** — unstage выбранных файлов
- **Ctrl+M** — merge ветки
- **Ctrl+D** — rebase ветки
- **F7** — открыть Create Branch
- **Shift+F7** — открыть Create Tag

### Просмотр
- Unified + Side-by-side diff с переключателем, синхронным скроллом, виртуальным списком
- Превью бинарных файлов и изображений в diff-панели
- Лог коммитов с деталями (виртуальный список, бесконечная подгрузка при скролле)
- Поддержка root-коммитов (без родителей) в diff, деталях и списке файлов
- Список веток / тегов / stash с поиском; шеврон-индикатор текущей ветки
- Панель файлов со статусами; автовыбор первого файла при загрузке коммита
- File compare диалог (произвольные ревизии)
- Stats — статистика репозитория
- i18n (русский / английский)
- Авто-обновление приложения; заголовок окна `GitStream v{version}`

### Repositories (treeview)
- Drag-and-drop: репозитории в папки, папки в папки, на верхний уровень
- Двойной клик — переключение на репозиторий
- Контекстное меню: Add Repository, Add Group, Delete

### Диалоги
- **Commit** — сообщение, amend, индикатор длины строки, Commit & Push
- **Push / Pull** — выбор remote, force/rebase, behind-счётчик
- **Checkout** (локальная) и **Checkout Remote** (с созданием локальной)
- **Create Branch** — имя, start point, опц. checkout
- **Rename Branch** — переименование локальной ветки
- **Add Tag** — lightweight / annotated, target, force
- **Stash Save** — сообщение, include-untracked
- **Squash / Reword** — операции над коммитами
- **Discard** — подтверждение с выбором файлов
- **File Compare** — diff между двумя ревизиями файла
- **Stats** — статистика репозитория
- **Settings** — тема, шрифты, таймаут сети, язык
- **Add Repository / Add Group / Rename Node** — управление treeview
- **Confirm** — универсальный диалог подтверждения, опц. список затрагиваемых объектов
- Все диалоги draggable за header и закрываются по Esc

### Архитектура
- **Tauri IPC** — `#[tauri::command]` endpoints (query + mutation)
- **Git CLI** — парсинг `--porcelain=v2` (status), `--format` с NUL-разделителями (log, branches, tags), unified diff
- **Composables** — реактивное состояние через Vue ref(), автообновление после мутаций
- **ConflictBar** — реагирует на состояние репозитория (merge/rebase/cherry-pick/revert), accept ours/theirs, continue/abort
- **Обработка ошибок** — classify_git_error: auth, network, conflict, hint-подсказки
- **Сетевые операции** — async + spawn_blocking, настраиваемый таймаут (см. memory)
- **i18n** — кастомный composable `useI18n`, словари RU/EN

---

## Дорожная карта

Базовая дорожная карта (частичный stage, stash, create branch, reset/revert/cherry-pick,
разрешение конфликтов, rebase) закрыта. Следующий слой возможных направлений:

- **Blame view** — авторы по строкам файла, переход к коммиту
- **File history** — лог коммитов конкретного файла
- **Reflog** — просмотр reflog с возможностью восстановления
- **Управление remote** — add/remove/set-url, upstream-трекинг, prune
- **Поиск/фильтрация в логе** — по сообщению, автору, файлу, хэшу
- **git init / open** — инициализация репозитория, открытие локальной папки
- **Syntax highlighting в diff**, word-level diff
- **Commit graph с визуальными линиями** (lane allocation)
- **Interactive rebase UI**, 3-way conflict resolver
- **Аутентификация** — login/password, SSH passphrase, credentials cache
- **Прочее** — GPG signing, Git-Flow, Submodules, LFS, bisect, patches, bundles, archive

