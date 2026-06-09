# Дизайн: File History (GitStream 1.0.0-rc, часть 1)

Первая половина пункта «File history + Blame» дорожной карты 1.0.0. Blame — отдельный
следующий инкремент (см. в конце).

## Цель

Показать историю коммитов конкретного файла (`git log --follow -- <path>`), его diff на
выбранном коммите и дать переход к коммиту в основном графе.

## Backend

- **Рефактор:** разбор записи коммита формата `%x1e%H%x00…%B` сейчас встроен в `log()`.
  Вынести в хелпер `parse_commit_record(record, &remotes, &unpushed) -> Option<CommitInfo>`,
  чтобы `log` и новый `file_log` не дублировали парсинг.
- **`file_log(repo_path, path, limit) -> Vec<CommitInfo>`** — `git log --follow
  --format=<тот же формат> -<limit> -- <path>`. `--follow` отслеживает переименования.
  Пустой результат (файл новый/без истории) — `Ok(vec![])`, не ошибка.
- Команда `get_file_log(repo_path, path, limit)`. Diff файла на коммите — переиспользуем
  существующий `get_diff_commit_file`.

## Frontend

- **`useFileHistory.ts`** — состояние: `path`, `commits`, `selectedOid`, `fileDiff`, `open`.
  `openFor(path)` грузит лог; выбор коммита грузит diff через `useDiff.diffCommit(oid, path)`.
  Singleton-стейт, сбрасывается при смене репозитория (как прочие composables).
- **`FileHistoryDialog.vue`** — draggable (per memory), Esc закрывает. Слева список коммитов
  (short_oid / автор / дата / subject, фильтр-поле как в графе через `filterCommits`),
  справа/снизу — diff файла на выбранном коммите (переиспользовать рендеринг diff-строк).
  Двойной клик по коммиту или кнопка «Перейти к коммиту» → `goToCommit(oid)`.
- **Вход:** пункт «File History» в контекстном меню `FileList.vue` (working-tree файл).
  i18n RU/EN (`files.historyCtx`, блок `dialog.fileHistory`).

## Переход к коммиту

`goToCommit(oid)` устанавливает `useLog.selectedCommit = oid` и эмитит запрос скролла к нему
в `CommitGraph` (через общий ref/событие), затем закрывает диалог. Если коммита нет в
загруженном окне лога — подгрузка не требуется в MVP: выделяем, граф доскроллит при наличии.

## Ошибки

Через try/catch → `logError` (Git output), не модалкой. Глобальная сеть из 0.9.11 — backstop.

## Файлы

- `src-tauri/src/git/query.rs` — `parse_commit_record`, `file_log`.
- `src-tauri/src/commands.rs` — `get_file_log`.
- `src-tauri/src/main.rs` — регистрация команды.
- **new** `src/composables/useFileHistory.ts`.
- **new** `src/components/dialogs/FileHistoryDialog.vue`.
- `src/components/FileList.vue` — пункт меню; `src/App.vue` — монтаж; `src/locales/{ru,en}.ts`.

## Тесты / проверка

- Rust: `file_log` на репо с переименованием (`--follow` находит историю через rename);
  пустой результат для файла без истории.
- Переиспользование `parse_commit_record` не ломает существующие log-тесты.
- `vue-tsc`, `cargo test`, `cargo clippy -D warnings`, `vitest`, `vite build`.

## Следующий инкремент: Blame

`blame(repo_path, path, rev?) -> Vec<BlameLine>` через `git blame --porcelain` (oid/short/
author/date/line_no/content) + юнит-тест парсера porcelain; `BlameView` с gutter, клик по
строке → переход к коммиту.
