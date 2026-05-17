# Контекстное меню и мультивыделение в панели файлов

Дата: 2026-05-17

## Цель

Добавить в панель файлов (`FileList.vue`) контекстное меню по правой кнопке мыши
с действиями над одним или несколькими выделенными файлами рабочего дерева.

## Требования

- ПКМ по файлу открывает контекстное меню.
- Множественное выделение мышью:
  - **Ctrl+клик** — точечно добавить/убрать файл из выделения.
  - **Shift+клик** — выделить диапазон от якоря до текущего файла.
  - обычный клик — выделить один файл (как сейчас).
- Меню действует на всё текущее выделение (1 или N файлов).
- Если ПКМ по файлу вне выделения — он становится единственным выделением,
  затем открывается меню.
- Пункты меню (enable/disable по состоянию выделенных файлов):
  - **Stage** — активно, если среди выделенных есть unstaged/untracked.
  - **Unstage** — активно, если есть staged/partial.
  - **Commit** — открыть диалог коммита (без авто-stage выделенных).
  - **Discard** — отменить изменения (с подтверждением).
  - **Remove** — `git rm` (удалить с диска + застейджить удаление; с подтверждением).
  - **Delete** — удалить файл(ы) только с диска, git не трогаем (с подтверждением).
- Действует только для списка файлов рабочего дерева. Для файлов выбранного
  коммита мультивыделение и меню не добавляются.

## Архитектура

Состояние выделения — локально в `FileList.vue` (не выносится в composable;
другим компонентам сейчас не нужно). Контекстное меню — инлайн через
`<Teleport to="body">` по образцу уже существующего меню в `CommitGraph.vue`
(переиспользуются CSS-классы `ctx-menu`, `ctx-item`, `ctx-separator`,
`ctx-danger`, `ctx-backdrop`).

### Backend (Rust)

`src-tauri/src/git/mutation.rs`:

- `remove(repo_path: &Path, files: &[String]) -> Result<(), GitError>` —
  `git rm -- <files>`. Если файл untracked (`git rm` падает) — фоллбэк на
  удаление файла с диска (`std::fs::remove_file`).
- `delete(repo_path: &Path, files: &[String]) -> Result<(), GitError>` —
  `std::fs::remove_file` для каждого пути; git не трогаем (tracked-файл
  станет «deleted, unstaged»).

`src-tauri/src/commands.rs` — два `#[tauri::command]`:
`remove_files`, `delete_files` (сигнатура как у `discard_files`:
`repo_path: String, files: Vec<String>`). Регистрация в `main.rs`.

### Composable

`src/composables/useFiles.ts` — добавить `removeFiles(paths)` и
`deleteFiles(paths)` по образцу `discardFiles` (invoke + `refreshAfterMutation`).
`selectedFile` остаётся «активным» файлом для diff (последний кликнутый).

### FileList.vue

- `selectedPaths = ref<string[]>([])`, `anchorPath = ref<string | null>(null)`.
- Обработчик клика по файлу:
  - без модификаторов: `selectedPaths = [path]`, `anchorPath = path`,
    `selectFile(path)` (diff как сейчас).
  - Ctrl: тоггл `path` в `selectedPaths`, `anchorPath = path`.
  - Shift: диапазон по индексам в `filteredFiles` от `anchorPath` до `path`.
  - `selectedFile` (для diff) = последний кликнутый.
- `@contextmenu` по файлу: если `path` не в `selectedPaths` —
  `selectedPaths = [path]`; затем открыть меню в координатах курсора.
- Подсветка `.selected` — для всех путей из `selectedPaths`.
- Меню закрывается: клик по `ctx-backdrop`, Esc, после выполнения действия.

### Действия меню

| Пункт   | Условие активности                       | Действие |
|---------|------------------------------------------|----------|
| Stage   | есть unstaged/untracked в выделении      | `stageFiles(selectedPaths)` |
| Unstage | есть staged/partial в выделении          | `unstageFiles(selectedPaths)` |
| Commit  | всегда                                   | `emit("commit")` (открыть `CommitDialog`, как в CommitGraph) |
| Discard | всегда                                   | `window.confirm` → `discardFiles` |
| Remove  | всегда                                   | `window.confirm` → `removeFiles` |
| Delete  | всегда                                   | `window.confirm` → `deleteFiles` |

В тексте подтверждения при N>1 показывать количество файлов.

## Обработка ошибок

Ошибки команд — через `window.alert` (паттерн `CommitGraph.vue`).
`git rm` на untracked-файле — фоллбэк к удалению с диска внутри backend.

## Тестирование

- Rust unit-тесты для `remove` и `delete` в `mutation.rs` (по образцу
  существующих: временный репозиторий, проверка состояния индекса/ФС;
  кейсы: tracked-файл, untracked-файл, несколько файлов).
- Фронтенд: ручная проверка сценариев выделения/меню + `npm run build`
  и type-check (self-check перед сдачей).

## Прочее

- Бамп patch-версии в заголовке «GitStream v0.1.x» после изменения кода.
- Без AI-атрибуции в коммитах.
- Коммит только с явного согласия пользователя.
