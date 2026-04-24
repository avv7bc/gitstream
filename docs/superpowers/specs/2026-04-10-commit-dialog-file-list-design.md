# Commit Dialog с выбором файлов — Design

## Контекст

Текущий [CommitDialog.vue](../../../src/components/dialogs/CommitDialog.vue) — минимальный: textarea + amend + счётчик staged-файлов. Файлы стейджатся отдельно через [FileList.vue](../../../src/components/FileList.vue), потом открывается диалог коммита.

Нужно объединить оба шага: в диалоге коммита пользователь видит список файлов, переключается между Staged / Local Changes, отмечает галочками, что коммитить, и пишет сообщение.

## Цели

- Один диалог от «хочу закоммитить» до коммита, без отдельного стейджинга для простых случаев.
- Сохранить существующий flow (кнопка Commit в toolbar открывает этот диалог).
- Реальный `Commit & Push` через первый remote.

## UI

```
┌─────────────────────────────────────────────────────────┐
│ Commit                                               ✕  │
├─────────────────────────────────────────────────────────┤
│ Commit local or staged changes                          │
│ Select the files you want to commit ...                 │
├─────────────────────────────────────────────────────────┤
│ ○ Staged Changes   ● Local Changes        8 files (~8)  │
│ ┌─────┬──────────────┬────────────────────────────┐     │
│ │ ☑   │ Name         │ Directory                  │     │
│ │ ☑   │ package.json │ .                          │     │
│ │ ☑   │ App.vue      │ src                        │     │
│ │ ...                                             │     │
│ └─────┴──────────────┴────────────────────────────┘     │
├─────────────────────────────────────────────────────────┤
│ Commit Message:                                         │
│ ┌─────────────────────────────────────────────────┐     │
│ │                                                 │     │
│ └─────────────────────────────────────────────────┘     │
│ ☐ Amend last commit                                     │
├─────────────────────────────────────────────────────────┤
│              [ Cancel ]  [ Commit ]  [ Commit & Push ]  │
└─────────────────────────────────────────────────────────┘
```

### Режимы

**Staged Changes** — показывает файлы из индекса. Чекбоксы read-only (все отмечены, dimmed). Список = `files` где `staged === "staged" || staged === "partial"`.

**Local Changes** — показывает все изменённые файлы (`files` без фильтра). Чекбоксы активны. По умолчанию отмечены файлы, которые уже в индексе (staged/partial).

**Автовыбор режима при открытии:**
- Если есть staged файлы → Staged Changes.
- Иначе → Local Changes. Радио «Staged» disabled.

### Счётчик
`{checkedCount} files (~{totalInMode})`. В Staged-режиме `checkedCount === totalInMode`.

### Toggle-all
Клик по заголовку колонки чекбокса переключает все в Local-режиме.

### Строки таблицы
- `basename(path)` → колонка Name.
- `dirname(path)` → колонка Directory. Если в корне — показываем `.`.

### Commit Message
Textarea как сейчас. Индикатор первой строки `N / 72` (ok/warning/error).

### Amend
Чекбокс как сейчас. При amend в Local-режиме стейджим отмеченные, разстейджим не отмеченные, потом `do_commit(..., amend=true)`.

### Кнопки
- **Cancel** — закрыть.
- **Commit** — выполнить коммит с текущей выборкой.
- **Commit & Push** — коммит + `do_push` к первому remote, без force.

Оба коммит-кнопки disabled если: пустое сообщение ИЛИ `checkedCount === 0`.

## Логика коммита

### Staged-режим
```
do_commit(message, amend)
files.refresh()
```

### Local-режим
```
checked   = отмеченные пути
unchecked = неотмеченные пути (из текущего списка файлов)

// Файлы, которые юзер снял — должны выйти из индекса (если были там)
toUnstage = unchecked ∩ {staged | partial}
if toUnstage.length: unstage_files(toUnstage)

// Отмеченные — в индекс
if checked.length: stage_files(checked)

do_commit(message, amend)
files.refresh()
```

### Commit & Push
После `do_commit` вызвать `do_push` с первым remote из `useBranches().remotes` (или аналог). Force=false.

## Изменения в коде

### Frontend
- **[src/components/dialogs/CommitDialog.vue](../../../src/components/dialogs/CommitDialog.vue)** — полностью переписать:
  - Добавить `ref<"staged" | "local">` для режима.
  - Добавить `ref<Set<string>>` для отмеченных путей.
  - Computed: `filesInMode`, `checkedCount`, `totalInMode`, `hasStaged`.
  - Таблица файлов с чекбоксами.
  - Функция `handleCommit(push: boolean)`.
- **[src/composables/useRemote.ts](../../../src/composables/useRemote.ts)** — проверить, есть ли функция `push` с параметрами `remote, branch, force`. Если нет — добавить.
- **[src/composables/useBranches.ts](../../../src/composables/useBranches.ts)** — получить список remotes (или использовать уже существующий `remotes` ref).

### Backend
Изменений не требуется. Используются существующие:
- `stage_files`
- `unstage_files`
- `do_commit`
- `do_push`

## Что намеренно пропущено (YAGNI)

- AI-генерация сообщения.
- Templates / Select dropdown для сообщений.
- More Options (GPG signing, author override и т.п.).
- Partial staging (hunk-level) из диалога.
- Выбор remote/branch для Push в этом диалоге — всегда первый remote. Если нужна гибкость — пользователь использует отдельный PushDialog после коммита.

## Edge cases

- **Нет файлов вообще** — диалог всё равно открывается, кнопки disabled. (Alt: блокировать открытие в toolbar. Оставляем как есть — простота.)
- **Нет remote при Commit & Push** — ошибка от `do_push`, обрабатывается существующим error-хандлером. Кнопку не дизейблим превентивно.
- **Файл исчез между refresh и commit** — `stage_files` вернёт ошибку, показываем через существующий error flow.
- **Amend без staged** — currently allowed by git (меняет только message); оставляем как есть.

## Тестирование

Нет unit-тестов в проекте для Vue-компонентов (проверил структуру). Ручное тестирование:
1. Открыть диалог с несколькими изменёнными файлами — видим Local Changes, все отмечены.
2. Снять пару галочек, закоммитить — в коммит попали только отмеченные.
3. Застейджить файл вручную, открыть диалог — автоматически Staged Changes.
4. Переключиться на Local, снять галочку со staged-файла, закоммитить — файл ушёл из индекса.
5. Commit & Push — коммит создан и запушен.
6. Amend в Local-режиме — перекоммитил последний с новым набором файлов.
