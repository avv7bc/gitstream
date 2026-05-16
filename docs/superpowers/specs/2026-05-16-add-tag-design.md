# Дизайн: Удобная работа с тегами (Add Tag)

**Дата:** 2026-05-16
**Статус:** Утверждён к планированию

## Цель

Сделать работу с git-тегами в GitStream полноценной: создание (lightweight/annotated, с force),
удаление и push тегов на remote. Поведение моделируется по типовым десктоп-клиентам git.

Сейчас теги только отображаются в `BranchPanel` (read-only, без контекстного меню),
а в бэкенде нет ни одной мутации для тегов (`mutation.rs` содержит только ветки/коммиты/сеть).

## Объём (scope)

В рамках задачи:

- **Create** — создание тега (lightweight/annotated, force-перезапись)
- **Delete** — удаление локального тега, опционально и на remote
- **Push** — push отдельного тега на remote

Вне объёма: подпись GPG, массовый push `--tags`, переименование тега, просмотр содержимого
аннотации в отдельной панели.

## Модель UX

**Add Tag (диалог):**

- Поле **Tag Name**
- Поле **Message** (многострочное, необязательное): пусто → lightweight tag, есть текст → annotated tag
- Чекбокс **Force** (перезаписать/переместить тег, если имя уже занято)
- Информационная строка: `Tag will point to: <short-oid> <subject>` либо `Tag will point to: HEAD`
- В самом диалоге **push нет**

**Операции с тегом** (контекстное меню на теге в секции Tags):

- **Push Tag** → push конкретного тега на remote
- **Delete Tag** → подтверждение с чекбоксом «Also delete on remote `<remote>`»

## Точки входа

1. **Контекстное меню коммита в CommitGraph** — ПКМ по реальному коммиту → «Create Tag here…»
   (тег на конкретный коммит). Пункт неактивен для working-tree-строки.
2. **Секция Tags в BranchPanel** — кнопка `+` в заголовке секции → Add Tag с `target=null`
   (тег на HEAD). Контекстное меню на теге → Push Tag / Delete Tag.

Кнопка в Toolbar — вне объёма (по решению пользователя).

## Архитектура

Выбран подход «расширить существующие модули» (следует established pattern проекта —
как `renameBranch`/`deleteBranch`/`pushBranch`), вместо отдельного `useTags.ts`.
Причина: теги уже живут в `useBranches` (`tags.value` используется в `BranchPanel`),
отдельный composable создал бы рассинхрон и дублирование refresh-логики.

### Backend (Rust)

**`src-tauri/src/git/mutation.rs`** — три функции:

```rust
// lightweight: message=None  → git tag <name> [<target>]
// annotated:   message=Some  → git tag -a <name> -m <msg> [<target>]
// force:       добавляет -f
pub fn create_tag(repo_path: &Path, name: &str, message: Option<&str>,
                   target: Option<&str>, force: bool) -> Result<(), GitError>;

pub fn delete_tag(repo_path: &Path, name: &str) -> Result<(), GitError>;
// git tag -d <name>

pub fn push_tag(repo_path: &Path, remote: &str, name: &str, delete: bool)
    -> Result<(), GitError>;
// push:   git push <remote> refs/tags/<name>
// delete: git push <remote> :refs/tags/<name>
```

**`src-tauri/src/commands.rs`** — три endpoint'а:

- `do_create_tag` — sync (локальная операция)
- `do_delete_tag` — sync (локальная операция)
- `do_push_tag` — `async`, выполняется через `spawn_blocking` (правило: сетевые git-операции
  должны быть async, чтобы не блокировать UI; по образцу `do_push`/`do_pull`)

Регистрация всех трёх в `main.rs`.

Целевой коммит: из CommitGraph передаётся `oid`; из секции Tags `target=None` (HEAD).

### Frontend

**`src/composables/useBranches.ts`** — три функции (экспортировать в return):

```ts
async function createTag(name, message: string|null, target: string|null, force: boolean) {
  if (!repoPath.value) return;
  await invoke("do_create_tag", { repoPath: repoPath.value, name, message, target, force });
}
async function deleteTag(name) {
  if (!repoPath.value) return;
  await invoke("do_delete_tag", { repoPath: repoPath.value, name });
}
async function pushTag(remote, name, del: boolean) {
  if (!repoPath.value) return;
  await invoke("do_push_tag", { repoPath: repoPath.value, remote, name, delete: del });
}
```

`TagInfo` в `src/types/index.ts` менять не нужно (уже соответствует Rust-структуре).

**`src/components/dialogs/AddTagDialog.vue`** (по образцу `RenameBranchDialog.vue`,
draggable через `useDraggable` — правило draggable dialogs):

- Props: `target: { oid: string; subject: string } | null` (null = HEAD)
- Поля: Tag Name (input, автофокус), Message (textarea, опц.), Force (чекбокс),
  инфо-строка о целевом коммите
- Валидация имени: непусто, без `~^:?*[` и пробелов, не начинается с `-`
- Кнопки Cancel / Create Tag (disabled пока имя невалидно), закрытие по Esc,
  emit `confirm` / `close`

**Удаление тега** — переиспользовать `ConfirmDialog.vue`, добавив опциональный prop
для дополнительного чекбокса (label + v-model), используемого как «Also delete on remote
`<remote>`». Если remote нет — чекбокс скрыт. Новый отдельный диалог не создаётся.

**`src/components/BranchPanel.vue`:**

- Кнопка `+` в заголовке секции Tags → открыть AddTagDialog с `target=null`
- Контекстное меню на теге (по образцу меню веток): `Push Tag`, `Delete Tag` (danger)
- `Push Tag`: если remotes > 1 — выбор remote (как в существующем потоке push веток),
  иначе первый/`origin`
- Новый emit `tagsChanged` → `refreshAll()`

**`src/components/CommitGraph.vue`:**

- В существующее `ctx-menu` добавить пункт `Create Tag here…`, активен только для
  реальных коммитов (`!ctxIsWorkingTree`)
- Новый emit `createTag: [oid, subject]` (по образцу `commit`/`discard`)

**`src/App.vue`:**

- Рефы `showAddTagDialog`, `addTagTarget`
- Условный рендер `AddTagDialog`
- Подписки `@create-tag` (от CommitGraph) и `@tags-changed` (от BranchPanel) → `refreshAll()`

## Data flow

**Создание из CommitGraph:**

```
CommitGraph: ПКМ по коммиту → "Create Tag here…"
  → emit createTag(oid, subject)
  → App.vue: showAddTagDialog=true, addTagTarget={oid,subject}
  → AddTagDialog confirm(name, message, force)
  → useBranches.createTag(...) → invoke do_create_tag → git tag ...
  → App.vue: refreshAll()  (теги перечитываются через get_tags)
```

**Удаление + also-on-remote:**

```
BranchPanel: ПКМ по тегу → "Delete Tag"
  → ConfirmDialog (+ чекбокс "Also delete on remote <r>")
  → confirm: await deleteTag(name); if(checked) await pushTag(remote, name, true)
  → emit tagsChanged → App.vue refreshAll()
```

Порядок при удалении: сначала локальное удаление, затем (если отмечено) remote.

## Обработка ошибок

- try/catch с `window.alert` (как в `confirmRename`); текст ошибки git проходит через
  существующий `classify_git_error` → читаемое сообщение
- Push-операции async на бэке — UI не блокируется

## Краевые случаи

- Имя уже существует, `force=off` → ошибка git → alert
- Невалидное имя → блокируется на фронте (disabled), бэк как страховка вернёт ошибку git
- Нет remote → «Push Tag» неактивен, чекбокс «also on remote» скрыт
- Push несуществующего на remote тега при удалении → no-op/ошибка git → alert;
  локальное удаление уже выполнено
- Тег на working-tree (`__worktree__`) → «Create Tag here» неактивен
- Annotated vs lightweight определяется только наличием message

## Тестирование

- **Backend (Rust):** следовать стилю существующих тестов в `mutation.rs`/`query.rs`
  (если есть; иначе проверка во временном репозитории) — create lightweight/annotated/force,
  delete, push и delete-on-remote
- **Frontend:** ручная проверка обоих entry points, валидации имени, Esc/drag диалога,
  refresh после операций; проект без фронтенд-тест-раннера — проверка `npm run build` + типы
  (правило self-check)
- Инкремент patch-версии в заголовке «GitStream v0.1.x» (правило version bump)

## Definition of Done

- Оба entry point создают тег (lightweight/annotated/force); тег виден в секции Tags
  после refresh
- Контекстное меню тега выполняет Push Tag и Delete Tag (с опцией also-on-remote)
- `npm run build` и `cargo build` проходят без ошибок
