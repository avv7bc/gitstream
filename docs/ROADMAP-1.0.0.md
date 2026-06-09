# План развития GitStream → 1.0.0

Текущая версия: **0.9.5** (синхронна в `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`).

Принцип релиза 1.0.0: не «добавить максимум фич», а **закрыть пробелы, без которых клиент нельзя считать законченным**, и гарантировать стабильность. Всё некритичное для повседневной работы уезжает в 1.x.

---

## Что блокирует 1.0.0 (must-have)

### 1. `git init` / open repository — 0.9.6 ✅ (сделано)
- ✅ `git init` в выбранной пустой/не-git папке (`do_init`) — через AddRepositoryDialog
- ✅ Open folder → проверка `.git` (`check_repo_path`), добавление в treeview
- ✅ Clone (`git clone <url>` с прогрессом, выбор папки) — `do_clone` + `CloneRepositoryDialog`

### 2. Управление remote — 0.9.7
- add / remove / set-url / rename remote
- Установка upstream-трекинга для ветки (`--set-upstream`)
- `fetch --prune`
- UI: секция Remotes в BranchPanel + диалог

### 3. Аутентификация — 0.9.8
Без этого push/pull по HTTPS/SSH ломается «молча».
- Перехват запроса credentials (login/password / token)
- SSH passphrase prompt
- Интеграция с git credential helper / cache
- Внятные ошибки auth (частично уже есть в `classify_git_error`)

### 4. Качество и стабильность — 0.9.9
Водораздел между 0.x и 1.0.
- **Фронтенд-тесты**: добавить Vitest, покрыть composables (`useFiles`, `useLog`, `useDiff`, парсинг porcelain/format-вывода — самое хрупкое)
- **CI**: добавить workflow `ci.yml` — `vue-tsc --noEmit`, `vitest run`, `cargo test`, `cargo clippy` на каждый PR (сейчас только `release.yml`)
- Граничные случаи: пустой репозиторий, detached HEAD, репо без коммитов, репо без remote, огромный лог
- Аудит обработки ошибок: ни одна git-команда не должна падать без сообщения в Git output

---

## Желательно к 1.0.0 (nice-to-have, по времени)

### 5. File history + Blame — 1.0.0-rc
- **File history** — лог коммитов конкретного файла (`git log --follow -- <path>`), переход к коммиту
- **Blame view** — авторы по строкам, переход к коммиту строки

Можно вынести в 1.1, если поджимает время — на статус 1.0 не влияют.

---

## Релиз 1.0.0
- Финальный проход по README / CLAUDE.md (синхронизировать с реальностью)
- Скриншоты для README и стора
- Проверка авто-обновления на смене 0.x → 1.0.0
- Поднять версию до `1.0.0` во всех трёх местах, тег, релиз через CI

---

## Сводная таблица версий

| Версия    | Содержание                          | Тип  |
|-----------|-------------------------------------|------|
| 0.9.6     | git init / open / clone             | ✅ done |
| 0.9.7     | Управление remote + upstream        | must |
| 0.9.8     | Аутентификация (creds/SSH/cache)    | must |
| 0.9.9     | Тесты (Vitest) + CI + аудит ошибок  | must |
| 1.0.0-rc  | File history + Blame                | nice |
| **1.0.0** | Полировка, docs, скриншоты, релиз   | —    |

---

## Осознанно отложено в 1.x
Interactive rebase UI, 3-way conflict resolver, reflog, syntax/word-level highlighting в diff, GPG signing, submodules, LFS, bisect, Git-Flow. Полезно, но не определяет «готовый клиент».
