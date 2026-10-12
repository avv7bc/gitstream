# GitStream — спецификация улучшений функционала и UI (2026)

Основание: отчёт [Сравнение Git клиентов с GitStream](../reports/Сравнение%20Git%20клиентов%20с%20GitStream.md) и аудит кода UI (октябрь 2026). Визуальные варианты (A «Графит», B «Студия», C «Командный центр») — на дизайн-холсте (офлайн-копия: [docs/design/index.html](design/index.html)); эта спека **не зависит от выбранного варианта**: архитектурные слои (§2) одинаковы для всех трёх, различается только набор значений токенов и композиция главного окна.

Статус: черновик для обсуждения. Версии и порядок релизов — в §5.

---

## 1. Цели и границы

**Цели**
1. Убрать «ощущение сырости»: мёртвые кнопки, четыре системы оформления диалогов, `window.confirm`, захардкоженные цвета и строки, отсутствие пустых состояний и обратной связи.
2. Добавить страховочную сеть: журнал операций с Undo/Redo и Reflog — раньше любых новых деструктивных функций.
3. Закрыть главные функциональные пробелы относительно базовой линии 2026 года: правка истории одним действием, interactive rebase, 3-way resolver, worktrees, палитра команд.

**Не входит** (P2, отдельные спеки позже): submodules, LFS, подпись коммитов, bisect, AI, PR/forge-интеграция, custom actions, патчи, стеки веток.

**Инварианты, которые сохраняются**
- Backend — только git CLI через `std::process::Command`, парсинг `--porcelain`/`--format` с NUL-разделителями. Никакого libgit2.
- Сетевые операции — async + `spawn_blocking`, таймаут, askpass-мост.
- Ошибки git-операций — в Git output (`logError`), не модальные окна. Тост (§2.4) дополняет этот канал, не заменяет.
- Все модальные окна draggable за header, Esc закрывает только верхнее.
- Горячие клавиши — по `e.code`.
- Ориентир по поведению — SmartGit.

---

## 2. Архитектурные слои («один источник правды на каждый слой»)

### 2.1 Реестр команд — `src/commands/`

Сейчас одна и та же операция описана в 3–4 местах: тулбар (`AppToolbar.vue`), хоткей (`App.vue`), контекстное меню (CSS и список пунктов скопированы в `CommitGraph`, `FileList`, `BranchPanel`, `RepositoriesPanel`), и у каждого своё условие доступности. Отсюда вечно неактивные Merge/Rebase в тулбаре и меню графа из 10 серых пунктов.

**Решение.** Единый реестр, из которого строятся тулбар, контекстные меню, палитра команд и хоткеи.

```ts
// src/commands/types.ts
export type CommandId =
  | "commit" | "commit.amend" | "push" | "push.quick" | "pull" | "fetch" | "fetch.all" | "fetch.prune"
  | "stage" | "unstage" | "discard" | "stash.save" | "stash.apply" | "stash.pop" | "stash.drop"
  | "branch.create" | "branch.checkout" | "branch.merge" | "branch.rebase" | "branch.rename"
  | "branch.delete" | "branch.setUpstream" | "branch.push"
  | "commit.cherryPick" | "commit.revert" | "commit.reset.soft" | "commit.reset.mixed" | "commit.reset.hard"
  | "commit.reword" | "commit.squash" | "commit.fixup" | "commit.drop" | "commit.moveUp" | "commit.moveDown"
  | "commit.amendStagedInto" | "tag.create" | "tag.push" | "tag.delete"
  | "file.history" | "file.blame" | "file.compare" | "file.restoreFromCommit" | "file.copyPath" | "file.savePatch"
  | "repo.add" | "repo.clone" | "repo.init" | "repo.group.add" | "repo.remove"
  | "undo" | "redo" | "reflog.open" | "palette.open" | "quickSearch.open"
  | "settings.open" | "stats.open" | "output.toggle"
  | "worktree.add" | "worktree.remove" | "worktree.open";

export type CommandScope = "global" | "commit" | "workingTree" | "file" | "commitFile"
  | "localBranch" | "remoteBranch" | "tag" | "stash" | "remote" | "repo" | "worktree";

export interface CommandContext {
  repoPath: string | null;
  selectedCommits: CommitInfo[];       // из useLog
  selectedFiles: FileStatus[];         // из useFiles
  selectedRef: SelectedRef | null;     // ветка/тег/stash/remote в BranchPanel
  repoState: RepoState;                // merge/rebase/… in progress
  isWorkingTreeSelected: boolean;
}

export interface Command {
  id: CommandId;
  /** Ключ в словаре i18n: `commands.<id>` → title и, опционально, description. */
  scopes: CommandScope[];              // где показывать в контекстном меню
  shortcut?: string;                   // "Ctrl+K", "Shift+F7" — отображение; матчинг через e.code в hotkeys.ts
  group?: string;                      // группа в меню/палитре: "sync" | "history" | "danger" | …
  danger?: boolean;
  /** Пункт виден, но неактивен (например, нужен выбранный файл). */
  enabled: (ctx: CommandContext) => boolean;
  /** Пункт скрыт полностью (не относится к выделенному объекту). */
  visible?: (ctx: CommandContext) => boolean;
  run: (ctx: CommandContext) => void | Promise<void>;
}
```

Файлы:
- `src/commands/registry.ts` — `defineCommands([...])`, `getCommand(id)`, `commandsForScope(scope, ctx)`.
- `src/commands/context.ts` — `useCommandContext()`: собирает `CommandContext` из существующих composables (computed, singleton).
- `src/commands/hotkeys.ts` — одна таблица `code → CommandId` с модификаторами; глобальный `keydown` в `App.vue` заменяется вызовом `dispatchHotkey(e)`. Хоткеи графа (F2, ↑/↓) остаются локальными.
- `src/components/ContextMenu.vue` — один компонент: `props: { scope, ctx, x, y }`, рендерит `commandsForScope`, показывает шорткат справа, разделители между `group`, красные `danger`. Невидимые пункты **скрываются**, недоступные — гасятся (как JetBrains).

Правила:
- Меню графа: для рабочей копии — только `commit`, `discard`, `stash.save`; для коммита — остальное; `commit.squash` виден при ≥2 выделенных.
- Файлы коммита получают scope `commitFile`: открыть, история, blame, сравнить с рабочей копией, восстановить версию (`git restore --source=<sha> -- <path>`), копировать путь, сохранить патч.
- Тулбар: `branch.merge`/`branch.rebase` активны, когда в BranchPanel выделена ветка, отличная от текущей (та же логика, что у Ctrl+M/Ctrl+D).
- `push` открывает PushDialog; `push.quick` (Shift+клик, Alt+PgUp) — быстрый push в upstream, если он есть, иначе открывает диалог.

Миграция: сначала реестр + ContextMenu + хоткеи (поведение не меняется), затем AppToolbar читает из реестра, затем удаляются копии CSS меню из четырёх компонентов.

### 2.2 Дизайн-токены — `src/styles/tokens.css` + `themes/*.css`

Сейчас: 13 тем задают `--bg-*`, `--text-*`, `--accent`, цвета; но ref-чипы, ahead/behind, заголовки хунков, выделение строк — захардкоженные RGBA Catppuccin; статус-бар всегда `#007acc`; `--bg-tooltip`, `--bg-elevated`, `--border-hover`, `--accent-soft`, `--peach` используются, но не определены.

**Решение.** Двухуровневая система.

Уровень 1 — палитра темы (как сейчас: `--bg-primary`, `--green`, …). Уровень 2 — **семантические токены**, определяемые в `tokens.css` один раз через уровень 1 и `color-mix()`, темы их не трогают (кроме явных переопределений):

```css
:root {
  /* поверхности */
  --surface-0: var(--bg-primary);  --surface-1: var(--bg-secondary);  --surface-2: var(--bg-tertiary);
  --surface-raised: var(--bg-elevated, var(--bg-secondary));
  --surface-hover: var(--bg-hover);  --surface-selected: var(--bg-active);
  --border: var(--border-color);     --border-subtle: var(--border-subtle-color);
  --border-hover: color-mix(in oklab, var(--border) 60%, var(--text-primary));
  /* акцент */
  --accent-fg: var(--accent);
  --accent-bg: var(--accent);
  --accent-soft: color-mix(in oklab, var(--accent) 14%, transparent);
  --on-accent: var(--bg-primary);
  /* статусы */
  --ok: var(--green); --danger: var(--red); --warn: var(--yellow); --info: var(--blue);
  --ok-soft: color-mix(in oklab, var(--ok) 14%, transparent);  /* и т. д. для danger/warn/info */
  /* refs */
  --ref-head: var(--red); --ref-local: var(--green); --ref-remote: var(--blue);
  --ref-tag: var(--yellow); --ref-stash: var(--purple); --ref-current: var(--green);
  /* граф */
  --lane-1: var(--blue); --lane-2: var(--green); --lane-3: var(--purple);
  --lane-4: var(--teal); --lane-5: var(--orange); --lane-6: var(--yellow);
  --commit-unpushed: var(--unpushed);
  /* diff */
  --diff-add-bg, --diff-add-line, --diff-del-bg, --diff-del-line, --diff-word-add, --diff-word-del (как сейчас),
  --diff-hunk-bg: color-mix(in oklab, var(--info) 10%, var(--surface-1));
  --diff-selected: color-mix(in oklab, var(--accent) 18%, transparent);
  /* статус-бар — тематический, не фиксированный */
  --statusbar-bg: var(--surface-2); --statusbar-fg: var(--text-secondary);
  --statusbar-busy-bg: var(--accent-soft); --statusbar-error-bg: var(--danger-soft);
  /* тултип, тост */
  --tooltip-bg: var(--surface-raised); --tooltip-fg: var(--text-primary);
  /* размеры */
  --row-h: 24px; --row-h-compact: 22px; --toolbar-h: 40px; --statusbar-h: 24px; --panel-title-h: 28px;
  --control-h: 28px; --control-h-lg: 32px;
  --radius-sm: 4px; --radius: 6px; --radius-lg: 8px;
  --shadow-popover: 0 8px 24px rgb(0 0 0 / .25); --shadow-dialog: 0 16px 48px rgb(0 0 0 / .35);
}
```

Правила:
- В компонентах запрещены hex/rgb(a) вне `src/styles/themes/*.css`. Проверка: stylelint `color-no-hex` + `declaration-property-value-disallowed-list` для `rgba(` в `src/components/**`. Исключение — SVG-иконки, где цвет = `currentColor`.
- Тема может переопределить любой семантический токен (например, Dark+ оставляет `--statusbar-bg: #007acc`, если захочется паритета с VSCode), остальные темы получают тематический статус-бар автоматически.
- `useTheme.ts` без изменений в API; темы переезжают из одного файла в `src/styles/themes/<name>.css` (по файлу на тему), подключение — как сейчас.
- Плотность: `data-density="compact|comfortable"` на `<html>` переключает `--row-h`, `--control-h`, отступы. Настройка в Settings → Внешний вид.

### 2.3 BaseDialog — `src/components/dialogs/BaseDialog.vue`

Сейчас четыре системы: общий `.modal-dialog` (dialogs.css), `sq-*` (Squash), `rw-*` (Reword), `fc-*` (FileCompare, имитирует отдельное окно ОС с фейковыми кнопками), `vs-*`/`st-*` (Settings/Stats, resizable). Порядок кнопок футера различается.

**Решение.** Один компонент с фиксированным контрактом.

```vue
<BaseDialog
  :open="open" :title="t.push.title" :subtitle="`main → origin/main`"
  size="sm | md | lg | xl | full"    // 400 / 520 / 720 / 960 / 90vw
  :resizable="false"                  // true для lg/xl/full (Settings, Stats, FileCompare, Resolver)
  :busy="busy"                        // блокирует кнопки, показывает спиннер у primary
  :dismissable="true"                 // false → клик по overlay не закрывает (Credential)
  @close="…"
>
  <template #body>…</template>
  <template #footer-left>                      // опционально: превью команды, вторичная опция
    <code class="cmd-preview">git push origin main</code>
  </template>
  <template #footer>                           // кнопки — ТОЛЬКО через слоты primary/secondary
    <DialogButton kind="secondary" @click="close">{{ t.common.cancel }}</DialogButton>
    <DialogButton kind="primary | danger" :default="true" @click="submit">{{ t.push.push }}</DialogButton>
  </template>
</BaseDialog>
```

Правила футера (единые для всех диалогов, не обсуждаются по месту):
- Справа: `[Отмена] [Основное действие]`. Третья кнопка (например, «Только fetch», «Commit & Push») — слева от «Отмена» как `secondary`, не между Отменой и Primary.
- Основное действие — акцентное; разрушительное — `danger` (красное); Enter выполняет `:default`, Esc = отмена. Хинты `Enter`/`Esc` рисуются в кнопках маленьким `<kbd>` (вариант A и C) или не рисуются (вариант B) — токен `--dialog-kbd-hints: 1|0`.
- `#footer-left` для превью git-команды (`cmd-preview`, моно, muted) — показываем везде, где команда детерминирована: Push, Pull, Fetch, Merge, Rebase, Reset, Stash, Tag.
- Header: title, опциональный subtitle (контекст: ветка/коммит/файл), кнопка закрытия; draggable через `useDraggable` (уже есть).
- Регистрация в modal-реестре `App.vue` автоматическая: `BaseDialog` вызывает `useModalRegistry().register(key, isOpen, close)` в `onMounted` — ручной список `modalRegistry` в `App.vue` удаляется.

Миграция (по диалогу, без изменения поведения): Confirm → Discard → Commit → Push/Pull → Checkout* → Branch/Tag/Stash → Remote/Upstream → AddRepo/Clone/Group/Rename → Credential → Squash/Reword (убрать `sq-*`/`rw-*`) → FileHistory/Blame → FileCompare (убрать фейковые min/max, статус-бар «Ready 3:1 -1~2», активный `Next Change`; Take Left/Right — убрать до появления функции) → Settings/Stats (`size="full"`, resizable).

`ConfirmDialog` получает `items?: string[]` (уже есть) и `summary?: string` («будет переписано 5 коммитов»), `confirmKind: "primary"|"danger"`, опциональный чекбокс (уже есть). Все `window.confirm` (Reset hard, Discard/Remove/Delete в FileList, три подтверждения Rebase в BranchPanel, discard выделенных строк) заменяются на него. Удаление репозитория из Repositories получает подтверждение. Discard хунка — подтверждение **не** добавляем: после появления Undo (§3.1) он восстанавливается через тост «Отменить».

### 2.4 Тосты — `src/composables/useToast.ts`, `src/components/ToastStack.vue`

Сейчас успеха не видно, а ошибка переключает нижнюю панель на Git output, скрывая diff, который смотрел пользователь.

```ts
export type ToastKind = "info" | "success" | "warning" | "error" | "progress";
export interface Toast {
  id: number; kind: ToastKind; title: string; detail?: string;
  actions?: { label: string; run: () => void; primary?: boolean }[];
  timeout?: number | null;     // null = до закрытия вручную; по умолчанию success/info 4 с, warning 8 с, error ∞
  progress?: { current: number; total?: number; cancel?: () => void };
  repoPath?: string;           // показывать только для активного репо? нет — тост глобальный, но с именем репо в title
}
export function toast(t: Omit<Toast, "id">): number;
export function updateToast(id: number, patch: Partial<Toast>): void;
export function dismissToast(id: number): void;
```

Интеграция:
- `logError(message, detail)` → пишет в Git output (как сейчас), **не** переключает вкладку; показывает `error`-тост с действиями «Git output» (`showOutput()`) и, если `diagnose_sync` вернул ситуацию, «Открыть помощника». Параметр `{ silent: true }` для случаев, где тост не нужен.
- После успешной мутации — `success`-тост с кратким итогом: «Push → origin/main · 2 коммита», «Ветка feature/x удалена» + действие «Отменить» (§3.1).
- Сетевые операции (`run_network_git`) — один `progress`-тост на операцию: обновляется из существующих событий прогресса `useProgress`, кнопка Отмена (если операция отменяема). Индикатор на узле репозитория и строка в статус-баре остаются.
- Позиция: нижний правый угол над статус-баром (A, B) или нижний центр (C) — токен `--toast-anchor`. Стек максимум 4, старые схлопываются в «ещё N».
- `UpdateBanner` становится постоянным `info`-тостом с `timeout: null`.

### 2.5 i18n

Факт из кода: словарь `ru` — по сути английский (кроме `settings`), по решению пользователя. Эту спеку он не меняет; решение — где UI на русском, где на английском — принимается при выборе визуального варианта (макеты нарисованы на русском, чтобы оценить длины строк).

Правила:
- Все строки — через `useI18n()`; ESLint `vue/no-bare-strings-in-template` (allowlist: `+`, `−`, `·`, `…`, числа, сокращения git).
- Захардкоженные сейчас строки переносятся в словари: меню графа, «Working Tree/Index», «Graph», «N files», заголовки таблицы Stats, Force в AddTag, FileCompare, Dismiss в UpdateBanner, тултипы и пустое состояние Git output, тултип статус-бара.
- Даты — один форматтер `formatDate(ts, style: "short"|"long"|"relative")`, использующий `locale`; `CommitDetails` перестаёт жёстко использовать `ru-RU`.
- Тест в CI: ключи `ru` и `en` совпадают (уже есть тип `DeepStringify`; добавить runtime-проверку в `vitest`).

### 2.6 Пустые состояния, загрузка, welcome

- `src/components/EmptyState.vue`: иконка, заголовок, подзаголовок, до двух действий (кнопки из реестра команд).
- Применение: Repositories (нет репо → «Добавить», «Клонировать»), Branches-секции («Нет тегов», «Нет stash»), FileList («Нет изменений — рабочая копия чиста»; строка `files.noChanges` уже есть), CommitDetails без выбора («Выберите коммит»), Diff без файла («Выберите файл слева»), фильтр без результатов.
- `src/components/Skeleton.vue`: строки-заглушки для графа (первая загрузка), FileHistory, Blame; `BaseDialog :busy` — прогресс в header.
- `WelcomeScreen.vue` вместо пустого окна при `repoPath === null`: Open / Clone / Init, список недавних репозиториев (из дерева Repositories, последние 8 по времени открытия — нужно сохранять `lastOpenedAt` в настройках дерева), подсказка «Перетащите папку сюда» (Tauri `onDragDropEvent`).

---

## 3. Функциональные пакеты

Формат каждого пакета: UX → Backend (Rust, IPC) → Frontend → git CLI → крайние случаи → тесты.

### 3.1 Журнал операций и Undo/Redo (P0)

**UX.** Кнопка «Отменить» в тулбаре (Ctrl+Z, когда фокус не в поле ввода) с тултипом «Отменить: Reset --hard на 2c9f2b6»; Redo (Ctrl+Shift+Z). Тост после каждой мутации с «Отменить». Панель «Операции» (вкладка нижней панели рядом с Git output или окно по `Ctrl+Alt+Z`): список операций с временем, описанием, затронутыми refs, кнопкой «Вернуться к этой точке».

**Что такое операция.** Любая мутация, меняющая refs/HEAD/индекс/рабочее дерево через GitStream: commit, amend, reword, squash, fixup, drop, reset, revert, cherry-pick, merge, rebase, checkout, branch create/rename/delete, tag create/delete, stash save/apply/pop/drop, discard (файлов и хунков), pull (integrate-часть), push с force-with-lease (на уровне локального бэкап-рефа — уже есть). Fetch и push без force не записываются (они не меняют локальных refs, кроме remote-tracking — их откат не нужен).

**Backend.** `src-tauri/src/git/oplog.rs`.

```rust
pub struct OpSnapshot {
    pub id: u64,                 // монотонный
    pub at: i64,                 // unix
    pub kind: String,            // "reset.hard", "branch.delete", …
    pub label: String,           // человекочитаемо, для тултипа/тоста (RU/EN решает фронт по kind+args)
    pub head: String,            // symbolic-ref HEAD или "detached:<sha>"
    pub head_sha: String,
    pub refs: Vec<(String, String)>, // refs/heads/*, refs/tags/*, refs/stash (+ reflog stash уже есть в git)
    pub index_tree: Option<String>,  // `git write-tree` над текущим индексом (через временный GIT_INDEX_FILE-копию)
    pub worktree_stash: Option<String>, // `git stash create` — только для операций, трогающих рабочее дерево
    pub undone: bool,
}
```

- Хранение: `.git/gitstream/oplog.jsonl` (append-only) + объекты, на которые ссылаются snapshot'ы, защищены от GC refs `refs/gitstream/oplog/<id>` (указывают на commit-объект `stash create` и/или на `commit-tree` от `index_tree`). Хранить последние 50 операций; старые refs удалять при ротации.
- `fn snapshot(repo, kind, touches_worktree) -> OpSnapshot` вызывается **внутри** `run_git_mut`-обёртки для мутаций — вводится `fn mutate(repo, op: OpKind, f: impl FnOnce() -> Result<..>)`, который делает snapshot до, выполняет, и дописывает запись. Таким образом ни одна новая мутация не может «забыть» журнал.
- `fn undo(repo, id)`: восстановление refs через `git update-ref <ref> <sha>` (и `git update-ref -d` для созданных после снимка), HEAD через `git symbolic-ref HEAD <ref>` либо `git update-ref --no-deref HEAD <sha>`; индекс — `git read-tree <index_tree>`; рабочее дерево — если был `worktree_stash`: `git checkout <stash_sha> -- .` + `git read-tree -m` для untracked из `^3` (как `stash apply`, но без конфликтов — применяем через `git stash apply <sha>` с предварительным `git reset --hard <head_sha>`). Undo сам записывается как операция (`kind: "undo"`), поэтому Redo = undo операции undo.
- Запрет: undo невозможен, если `get_repo_state` ≠ clean (идёт merge/rebase) — сначала abort. Если рабочее дерево изменилось после операции и снимок трогает рабочее дерево — предупреждение в `ConfirmDialog` с списком файлов (`git status --porcelain` vs снимок).
- IPC: `get_oplog(repo) -> Vec<OpSnapshot>`, `do_undo(repo, id)`, `do_redo(repo)`.
- Backup-ref Sync Assistant (`refs/gitstream/backup/<branch>`) становится частным случаем oplog.

**Frontend.** `useOplog.ts` (singleton, сброс при смене репо), кнопки в тулбаре через команды `undo`/`redo`, `OplogPanel.vue`, интеграция тостов: `toast({ kind: "success", title, actions: [{ label: t.common.undo, run: () => undo(lastId) }] })`.

**Тесты.** Rust: интеграционные на временном репо — reset --hard → undo восстанавливает HEAD и рабочее дерево; branch delete → undo; stash pop → undo; undo при грязном дереве. Vitest: `useOplog` сброс при смене репо.

### 3.2 Reflog (P0)

**UX.** Секция «Reflog» в окне «Операции» (или отдельный диалог из меню Local → Reflog…): список `HEAD@{n}` и, по выбору ветки, `<branch>@{n}`: sha, действие, сообщение, дата. Действия на строке: «Показать в графе», «Создать ветку здесь», «Checkout (detached)», «Reset текущую ветку сюда (ConfirmDialog)».

**Backend.** `get_reflog(repo, refname: Option<String>, limit) -> Vec<ReflogEntry>` через `git reflog show --format='%H%x00%gD%x00%gs%x00%ct%x00%s' <ref>`. Создание ветки — существующий `do_create_branch`.

### 3.3 Встроенный коммит-композер и кнопка Commit (P0)

**UX.** Под списком файлов рабочей копии — композер: поле Summary (однострочное, счётчик 50/72 с подсветкой превышения), раскрываемое Description, тоггл Amend (подставляет сообщение HEAD), чекбокс Sign-off (`-s`), кнопки `Commit в main` (primary) и `Commit & Push` (secondary). Кнопка **Коммит** (с числом staged) в тулбаре — акцентная. `Ctrl+K` — фокус в композер; `CommitDialog` остаётся доступным по `Ctrl+Shift+K` и из меню для паритета со SmartGit (выбор файлов чекбоксами). Если staged пусто, но есть изменения — кнопка «Stage all и commit» (как GitHub Desktop: коммит всех).
- Черновик сообщения сохраняется в `localStorage` по `repoPath` и восстанавливается при переключении репо.
- Если в `.git/MERGE_MSG` / `SQUASH_MSG` есть текст — подставляется.

**Frontend.** `CommitComposer.vue` внутри `FileList.vue` (нижняя часть, высота по содержимому, сворачивается стрелкой). Использует существующий `useCommit`. Шаблон сообщения: `git config commit.template` читаем через новый `get_commit_template(repo)`.

### 3.4 Push-диалог достижим; Merge/Rebase активны (P0)

- `push` команда открывает `PushDialog` (remote, ветка → remote/ветка, «Push теги», «Force with lease» с предупреждением и строкой про бэкап-реф, превью команды). `push.quick` — Shift+клик по кнопке и Alt+PgUp: push в upstream без диалога; если upstream не задан — диалог.
- Тулбар Merge/Rebase — `enabled` из реестра (§2.1).
- Fetch: split-button остаётся; пункты — Fetch `<remote>`, Fetch all, Fetch all (prune), «Параметры…» (чекбоксы prune/tags, запоминаются).

### 3.5 Контекстное меню файлов коммита (P0) — см. §2.1, scope `commitFile`.

Backend: `restore_file_from_commit(repo, sha, path)` → `git restore --source=<sha> --worktree -- <path>` (записывается в oplog), `get_file_patch(repo, sha, path) -> String` → `git format-patch -1 <sha> --stdout -- <path>`; сохранение через Tauri dialog `save`.

### 3.6 Прогноз конфликтов для merge/rebase/pull (P1, S)

**Backend.** `preview_merge(repo, ours: &str, theirs: &str) -> MergePreview { clean: bool, conflicted_files: Vec<String>, fast_forward: bool }` через `git merge-tree --write-tree --name-only <ours> <theirs>` (Git ≥ 2.38; код возврата 1 = конфликт, после пустой строки — список файлов). `fast_forward` = `git merge-base --is-ancestor <ours> <theirs>`. Для rebase — последовательно по коммитам `git rev-list --reverse <upstream>..<branch>` с ограничением 50 коммитов, иначе «прогноз недоступен». Проверять версию git один раз (`git --version`), при < 2.38 — скрывать блок.

**Frontend.** Новый `MergeDialog.vue` (сейчас merge идёт без диалога): «Merge feature/x → main», выбор ff-only / ff / no-ff / squash, блок прогноза («Конфликтов не ожидается» зелёный / «Возможен конфликт в 2 файлах: …» жёлтый), превью команды. `RebaseDialog.vue` аналогично (заменяет три `window.confirm`). В `PullDialog` — тот же блок при выборе merge/rebase (после fetch).

### 3.7 Правка истории одним действием (P1, M)

Команды на коммите в графе и в палитре: **Fixup into** (влить выделенный staged-набор или выделенный коммит в этот), **Squash into parent**, **Drop**, **Move up / Move down**, **Amend staged into this commit**.

**Backend.** Общий механизм — уже существующий «сценарный редактор» rebase -i (используется для reword): `GIT_SEQUENCE_EDITOR=<self-exec helper>` (как askpass), который получает todo-файл и переписывает его по инструкции из переменной окружения `GITSTREAM_TODO_PLAN` (JSON: список `{sha, action}` в нужном порядке). Команды:
- `do_fixup_staged(repo, sha)`: `git commit --fixup=<sha>`, затем `git rebase -i --autosquash --autostash --update-refs <sha>^` с «пустым» редактором (`GIT_SEQUENCE_EDITOR=true`). Для корневого коммита — `--root`.
- `do_rebase_plan(repo, base: Option<sha>, plan: Vec<TodoItem>)`: один вход для drop/move/squash/fixup/reword/edit — используется и для interactive rebase UI (§3.8). `--update-refs` всегда, `--autostash` всегда.
- Запрет для запушенных коммитов — **нет** (SmartGit разрешает), но ConfirmDialog с предупреждением «коммит есть в origin/main, понадобится force-push», и oplog-снимок.
- Конфликт в процессе → существующий ConflictBar (состояние rebase).

### 3.8 Interactive rebase (P1, L)

**UX.** `InteractiveRebaseDialog` (`size="lg"`): список коммитов от базы до HEAD (новые сверху), в строке — drag-handle, sha, сообщение, автор, выбор действия (pick / reword / edit / squash / fixup / drop) сегментом или селектом; drag для порядка; строки squash/fixup визуально привязаны к родителю; для reword — inline-поле сообщения. Внизу: прогноз конфликтов (§3.6, по шагам), превью «N коммитов → M», чекбокс «Обновлять ветки, указывающие на эти коммиты (--update-refs)», кнопки «Начать rebase». Вызов: контекстное меню коммита «Interactive rebase отсюда…», из ветки «Rebase interactively onto…».

**Backend.** `do_rebase_plan` (§3.7). Для `edit` — rebase останавливается, ConflictBar показывает состояние «rebase: edit» с кнопками «Amend» и «Continue».

### 3.9 3-way resolver (P1, L)

**UX.** Окно `ConflictResolverDialog` (`size="full"`, resizable): слева список конфликтных файлов со статусом (разрешён/нет); центр — три колонки **Ours | Result | Theirs** (Base — по тогглу, 4-я колонка или подсветка diff3), синхронный скролл; каждый конфликтный чанк — кнопки «← взять ours», «взять theirs →», «оба (ours сверху / theirs сверху)»; Result редактируем (textarea с моноширинным шрифтом; для начала без подсветки синтаксиса). Верхняя панель: «Применить неконфликтные» (сделано автоматически при открытии), счётчик «2 из 5 конфликтов», ‹ ›, кнопки «Сохранить и пометить разрешённым» (`git add`), «Открыть во внешнем инструменте» (`git mergetool --no-prompt -- <path>` с `merge.tool` из конфига). Кнопка «Открыть resolver» добавляется в ConflictBar.

**Backend.**
- `get_conflict_sides(repo, path) -> { base, ours, theirs, merged_draft }`: `git show :1:<path>`, `:2:`, `:3:` (при отсутствии base — пустая строка, тип конфликта add/add); черновик — `git merge-file -p --diff3 <ours> <base> <theirs>` во временных файлах; парсер маркеров `<<<<<<<`/`|||||||`/`=======`/`>>>>>>>` даёт список чанков с диапазонами в каждой из трёх сторон.
- `save_resolved(repo, path, content, mark_resolved: bool)`: запись файла, при `mark_resolved` — `git add -- <path>`. Записывается в oplog (worktree snapshot).
- Бинарные файлы — только «ours/theirs» (как сейчас).
- Rebase/cherry-pick: учитывать, что «ours»/«theirs» меняются местами — подписи колонок берём из состояния (`REBASE_HEAD` → «Применяемый коммит» / «Текущая ветка»).

**Frontend.** Переиспользовать `useSyncScroll` и виртуализацию из side-by-side; чанки — тот же тип, что hunk в `useSideBySideDiff`.

### 3.10 Worktrees (P1, M)

**UX.** В дереве Repositories worktree — дочерний узел репозитория (иконка «ветка в папке», имя ветки, путь в тултипе); двойной клик переключает на него как на репо. В BranchPanel секция «Worktrees» (как в макетах) и значок у ветки, занятой в другом worktree (checkout такой ветки → подсказка «открыть worktree»). Команды: «Добавить worktree…» (диалог: ветка существующая/новая, папка с Browse, по умолчанию `../<repo>-<branch>`), «Удалить worktree» (ConfirmDialog, `--force` если грязный — отдельный чекбокс), «Prune».

**Backend.** `get_worktrees(repo)` → `git worktree list --porcelain` (worktree / HEAD / branch / detached / locked / prunable); `do_worktree_add(repo, path, branch, new_branch: Option<String>)` → `git worktree add [-b <new>] <path> <branch>`; `do_worktree_remove(repo, path, force)`; `do_worktree_prune(repo)`. `repo_info` возвращает `common_dir` (`git rev-parse --git-common-dir`), чтобы группировать worktrees под главным репо.

### 3.11 Палитра команд и быстрый поиск (P1, M)

**UX.** `Ctrl+Shift+P` — палитра: поле ввода сверху-центр, результаты из реестра команд (title, группа, шорткат, недоступные — внизу серым с причиной), fuzzy-поиск по RU и EN названиям одновременно. `Ctrl+P` — быстрый переход: ветки, теги, коммиты (по sha/сообщению из загруженного лога), файлы рабочей копии; Enter — выбрать в соответствующей панели; `>` в начале переключает в режим команд (как VSCode). Последние 5 команд — сверху при пустом запросе. В варианте C палитра — ещё и место для **пошаговых команд** («Push › remote › ветка»): команда может объявить `steps: PaletteStep[]`, тогда вместо диалога палитра последовательно спрашивает значения; это расширение реестра, реализуется после базовой палитры.

**Frontend.** `CommandPalette.vue`, `useFuzzy.ts` (простой scorer на подпоследовательностях с бонусом за начало слова; без зависимостей). Регистрация как модальный слой (Esc закрывает).

### 3.12 Подсветка синтаксиса в diff (P1, M)

- Токенизация в Web Worker (`src/workers/highlight.ts`) на Shiki (WASM, темы маппятся на токены §2.2 через CSS-переменные `--syn-keyword`, `--syn-string`, …; тема Shiki — одна «css-variables»). Запрос — только для видимых строк виртуального списка + буфер 100 строк; кэш по `(path, sha|"wt", lineIndex)`.
- Язык по расширению; для файлов > 1 МБ или > 20 000 строк — выключено. Настройка Settings → Внешний вид → «Подсветка синтаксиса в diff».
- Word-level подсветка остаётся поверх (фоновый цвет), токены — цвет текста.

### 3.13 Drag-and-drop веток (P1, M)

Перетаскивание ветки из BranchPanel на другую ветку/на текущую ветку в BranchPanel или на коммит в графе → popover с действиями: Merge into…, Rebase … onto…, Interactive rebase…, Создать ветку здесь (для коммита), с блоком прогноза конфликтов (§3.6). Реализация — существующий `useDraggable`/HTML5 DnD как в RepositoriesPanel; popover — `ContextMenu.vue` с подготовленным контекстом.

### 3.14 Серверные фильтры лога (P1, M)

- `get_log` получает `filter: LogFilter { author, since, until, grep, path, pickaxe_s, pickaxe_g, all_branches }` → `git log --author= --since= --until= --grep= -i [-S|-G] -- <path>`. Панель фильтра над графом раскрывается кнопкой «Фильтр» (сейчас — текстовое поле, клиентский фильтр по загруженным — остаётся как «быстрый фильтр»).
- `git grep -n -- <text>` для поиска по рабочему дереву — в быстрый поиск (§3.11) как отдельная категория.

### 3.15 Мелкие P1 (S)

- Ahead/behind на кнопках Pull/Push (`git rev-list --left-right --count HEAD...@{u}`; уже есть в `repo_info` — только отображение).
- Stash выбранных файлов (`git stash push -m <msg> -- <paths>`) и только staged (`--staged`, Git ≥ 2.35) — чекбоксы в StashSaveDialog.
- Spin-off: «Перенести незапушенные коммиты в новую ветку» (`git switch -c <new>`; `git branch -f <orig> <orig>@{u}`) — команда на текущей ветке, через oplog.
- Гигиена веток: в BranchPanel фильтр-чипы «merged» / «gone» / «stale > 90 дней» (`git for-each-ref --format='%(refname:short)%00%(committerdate:unix)%00%(upstream:track)' refs/heads` + `--merged <main>`), batch delete уже есть.

---

## 4. Главное окно: что меняется независимо от варианта

- Тулбар: Fetch(split) · Pull ↓1 · Push ↑2 · **Коммит (3)** · Отменить · [палитра] · … · Stats · Settings. Меню Repository/Local/Branch остаются (A), сворачиваются в одно меню «⋯» (B) или уходят в палитру (C).
- Статус-бар: репозиторий · ветка · ↑2 ↓1 · «8 изменений» · текущая операция с обратным отсчётом · ошибка последней операции (клик → Git output) · тоггл Git output. Цвета — `--statusbar-*`.
- FileList header: один segmented-фильтр состояний (Все / Изменённые / Staged / Untracked / Конфликты) вместо четырёх букв; тоггл «дерево/список» и «показать все» — в меню «⋯»; «Stage all / Unstage all» — кнопки.
- Нижняя панель: вкладки «Изменения | Операции | Git output».
- Граф: без изменений в lane-алгоритме; чипы refs — токены `--ref-*`; аватары-инициалы (вариант B) — опция «Показывать аватары».

---

## 5. Порядок внедрения и критерии приёмки

| Релиз | Содержимое | Приёмка |
|---|---|---|
| 1.2.0 | §2.2 токены, §2.3 BaseDialog + миграция всех диалогов, §2.4 тосты, §2.5 i18n-правила, §2.6 пустые состояния/welcome, §3.4 Push-диалог и Merge/Rebase | stylelint без hex в components; `window.confirm` отсутствует (`grep` в CI); все диалоги на BaseDialog; каждая ошибка даёт тост и строку в Git output; статус-бар тематический |
| 1.3.0 | §2.1 реестр команд + ContextMenu + хоткеи, §3.5 меню файлов коммита, §3.3 композер и кнопка Commit | Один компонент меню; Merge/Rebase в тулбаре активны при выделенной ветке; коммит возможен без диалога |
| 1.4.0 | §3.1 oplog/Undo/Redo, §3.2 Reflog | Любая мутация из §3.1 отменяется кнопкой и восстанавливает refs/HEAD/индекс/дерево (интеграционные тесты) |
| 1.5.0 | §3.6 прогноз конфликтов, §3.7 правка истории одним действием, §3.11 палитра | Fixup/drop/move за одно действие; прогноз в Merge/Rebase/Pull |
| 1.6.0 | §3.8 interactive rebase, §3.10 worktrees, §3.15 мелочи | — |
| 1.7.0 | §3.9 3-way resolver, §3.12 подсветка, §3.13 DnD веток, §3.14 фильтры лога | — |

Каждый релиз: `npm run build`, `cargo clippy`, vitest, ручная проверка на репо с кириллическими путями и на detached HEAD. Версия бампается только при коммите по явному указанию.

---

## 6. Принятые решения (вместо открытых вопросов)

- Undo реализуется снимками refs + `stash create`, а не через reflog-парсинг (reflog не покрывает индекс и untracked, и ненадёжен после `gc`).
- Диалоги остаются модальными (паритет со SmartGit); «палитра как диалог» — только в варианте C и только после базовой палитры.
- Interactive rebase и 3-way resolver строятся на существующем side-by-side движке и сценарном редакторе rebase, а не на новых зависимостях.
- Подсветка синтаксиса — Shiki в worker'е; альтернатива (Lezer) не даёт выигрыша при наборе языков проекта.
- Язык UI макетов — русский; язык реального UI остаётся как сейчас до отдельного решения.
- Минимальная поддерживаемая версия git — 2.38 для прогноза конфликтов и `--update-refs`; при более старой git функции скрываются, а не падают.
