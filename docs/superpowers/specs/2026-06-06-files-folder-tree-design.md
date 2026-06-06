# Панель-дерево папок в FileList

Дата: 2026-06-06
Ориентир: SmartGit (Files view с деревом папок слева и списком файлов справа).

## Цель

Добавить в панель Files кнопку, включающую отдельную панель-дерево папок слева
от списка файлов. Дерево содержит только папки, в которых есть файлы из текущего
списка; выбор папки фильтрует список файлов (с вложенными подпапками).

## Решения (по итогам брейншторма)

- **Режим:** отдельная панель-дерево слева от списка (не toggle разметки одного
  списка, не замена списка). Кнопка показывает/скрывает дерево.
- **Содержимое дерева:** только папки, реально содержащие файлы из текущего
  (уже отфильтрованного) списка. Клик по папке → файлы этой папки + всех вложенных.
- **Охват:** оба режима — working tree (изменённые файлы) и файлы выбранного
  коммита.
- **Состояние кнопки:** сохраняется между запусками в настройках (`files_tree_view`).
- **Разделитель:** drag-resize (как остальные панели приложения). Ширина дерева
  хранится в localStorage рядом с layout.

## Архитектура

Бэкенд-команды не нужны — дерево строится на фронте из путей, которые уже есть
в `files` / `commitFiles`.

### 1. Настройка `files_tree_view`

- **Rust** `src-tauri/src/settings.rs`: поле `files_tree_view: bool` в `AppSettings`
  с `#[serde(default)]` (default `false`) + в `impl Default`. `#[serde(default)]`
  обеспечивает чтение старых конфигов без поля.
- **TS** `src/composables/useSettings.ts`:
  - `interface AppSettings` — добавить `files_tree_view: boolean`.
  - `const filesTreeView = ref<boolean>(false)`.
  - В `loadSettings` — `filesTreeView.value = !!s.files_tree_view`.
  - В `scheduleSave` — добавить `files_tree_view: filesTreeView.value` в объект.
  - `watch(filesTreeView, scheduleSave)`.
  - Экспортировать `filesTreeView`.

### 2. Кнопка-переключатель

В шапке `FileList.vue` (`.header-right`, рядом с табами фильтра) — icon-button
«дерево папок». Активное состояние подсвечивается стилем как `.filter-tab.active`.
Клик: `filesTreeView.value = !filesTreeView.value`. Tooltip из i18n `files.treeView`.

### 3. Раскладка

`file-list-body` при `filesTreeView === true` становится горизонтальным flex:
`[ дерево ] [ resizer ] [ список файлов ]`. Список файлов — существующая разметка
`.file-item` без изменений. При `false` — текущий плоский список на всю ширину.

Resizer: паттерн из `App.vue` (mousedown → mousemove меняет ширину → mouseup
сохраняет). Ширина дерева `treeWidth` (ref, default ~220px, min ~120px),
персист в localStorage (ключ напр. `gitstream.filesTreeWidth`).

### 4. Построение дерева

`computed folderTree` из активного списка путей
(`filteredFiles` или `filteredCommitFiles` — учитывает текст-фильтр и табы):

- Разбить каждый путь по `/`, отбросить имя файла, накопить уникальные сегменты
  директорий в дерево узлов `{ name, path, children, fileCount }`.
- Узлы — только директории, содержащие файлы (прямо или во вложенных).
- Корневой синтетический узел `/ (все)` (`path === ""`) — показывает все файлы,
  выбран по умолчанию.
- Сортировка детей по имени (папки по алфавиту).

Состояние раскрытия: `expandedDirs` (ref `Set<string>`), по умолчанию все
раскрыты (или раскрывать лениво — но проще: раскрыты). Иконка ▾/▸.

### 5. Фильтрация по выбранной папке

- `selectedDir` (ref `string`, default `""` = корень/все).
- `dirFilteredFiles` / `dirFilteredCommitFiles`: если `selectedDir === ""` →
  весь список; иначе файлы, чей путь начинается с `selectedDir + "/"`.
- Шаблон рендерит `dirFiltered*` вместо `filtered*` (при выключенном дереве
  `selectedDir === ""`, поэтому поведение идентично текущему).
- `displayCount` и Ctrl+A select-all (`onListKeydown`) используют `dirFiltered*`.
- Сброс `selectedDir = ""` там же, где сейчас сбрасывается `selectedPaths`:
  - `watch(selectedCommit, ...)` (смена коммита/worktree),
  - `watch([activeFilter, fileFilter], ...)` (смена таба/текст-фильтра),
  - при смене репозитория (через существующий механизм, файлы перезагружаются).
- При выключении дерева (`watch(filesTreeView)`) — тоже сбросить `selectedDir = ""`.

### 6. i18n

В `src/locales/ru.ts` и `en.ts`, секция `files`:
- `treeView` — tooltip кнопки («Дерево папок» / «Folder tree»).
- `allFiles` — метка корневого узла («Все файлы» / «All files»).

## Затрагиваемые файлы

- `src-tauri/src/settings.rs` — поле настройки.
- `src/composables/useSettings.ts` — ref + load/save.
- `src/components/FileList.vue` — кнопка, дерево, resizer, фильтрация (основное).
- `src/locales/ru.ts`, `src/locales/en.ts` — строки.

## Вне области

- Никаких новых бэкенд-команд.
- Дерево не показывает неизменённые файлы репозитория — только из текущего списка.
- Множественный выбор папок не поддерживается (одна выбранная папка).

## Проверка

- `npm run build` / type-check проходит.
- Версия патч-бамп в package.json + Cargo.toml + tauri.conf.json.
