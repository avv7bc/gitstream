# План развития GitStream → 1.0.0

Текущая версия: **0.9.5** (синхронна в `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`).

Принцип релиза 1.0.0: не «добавить максимум фич», а **закрыть пробелы, без которых клиент нельзя считать законченным**, и гарантировать стабильность. Всё некритичное для повседневной работы уезжает в 1.x.

---

## Что блокирует 1.0.0 (must-have)

### 1. `git init` / open repository — 0.9.6 ✅ (сделано)
- ✅ `git init` в выбранной пустой/не-git папке (`do_init`) — через AddRepositoryDialog
- ✅ Open folder → проверка `.git` (`check_repo_path`), добавление в treeview
- ✅ Clone (`git clone <url>` с прогрессом, выбор папки) — `do_clone` + `CloneRepositoryDialog`

### 2. Управление remote — 0.9.7 ✅ (сделано)
- ✅ add / remove / set-url / rename remote (`add_remote`/`remove_remote`/`rename_remote`/`set_remote_url`)
- ✅ Установка upstream-трекинга для ветки (`set_branch_upstream`, диалог с выбором remote-ветки)
- ✅ `fetch --prune` (флаг `prune` в `do_fetch`)
- ✅ UI: секция Remotes в BranchPanel + контекстное меню + `RemoteDialog`/`SetUpstreamDialog`

### 3. Аутентификация — 0.9.8 ✅ (сделано)
Без этого push/pull по HTTPS/SSH ломается «молча».
- ✅ Перехват запроса credentials (login/password / token) — askpass-мост (self-exec, `askpass.rs`)
- ✅ SSH passphrase prompt (`SSH_ASKPASS` + `SSH_ASKPASS_REQUIRE=force`)
- ✅ Интеграция с git credential helper / cache (`ensure_credential_helper`, чекбокс «Запомнить»)
- ✅ Внятные ошибки auth + `AuthCancelled` (отмена диалога); таймаут на паузе во время prompt'а

### 4. Качество и стабильность — 0.9.9 🟡 (в работе)
Водораздел между 0.x и 1.0.
- ✅ **CI**: `ci.yml` — `vue-tsc --noEmit`, `vitest run`, `cargo test`, `cargo clippy --all-targets -D warnings` на каждый PR/push в main; clippy-преды вычищены
- ✅ **Vitest поднят** (config + scripts `test`/`test:watch`); первое покрытие — `highlight.ts` (escaping/v-html safety)
- ✅ Вынесена хрупкая чистая логика в `utils/` + тесты: `wordDiff` (word-level diff, парность строк, лимит), `commitFilter` (фильтр лога по message/author/SHA/date/refs); `useSideBySideDiff` теперь тонкая обёртка. Покрытие — 31 тест (3 файла)
- ✅ Аудит обработки ошибок: глобальная сеть `unhandledrejection` → Git output (никакой git-reject не пропадает молча); `console.error` в Side-by-side diff заменён на `logError`. Аудит 62 call-site проведён (agent)
- 🟡 Граничные случаи: backend-регрессы (`edge_case_tests`) — пустой репо/unborn-ветка (status/log/branches/remotes/repo_info/repo_state), staged-файл в unborn, detached HEAD, репо без remote. Остаётся ручной прогон UI (огромный лог, рендеринг этих состояний)

---

## Желательно к 1.0.0 (nice-to-have, по времени)

### 5. File history + Blame — 1.0.0-rc
- ✅ **File history** (0.9.13) — `file_log` (`git log --follow -- <path>`), `get_file_log`; `FileHistoryDialog` + `useFileHistory`; diff файла на коммите, фильтр, переход к коммиту в графе; пункт «File History» в контекстном меню FileList. Backend-тест на `--follow` через rename
- ⏳ **Blame view** — авторы по строкам (`git blame --porcelain`), переход к коммиту строки

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
| 0.9.7     | Управление remote + upstream        | ✅ done |
| 0.9.8     | Аутентификация (creds/SSH/cache)    | ✅ done |
| 0.9.9     | Тесты (Vitest) + CI + аудит ошибок  | 🟡 в работе |
| 1.0.0-rc  | File history + Blame                | nice |
| **1.0.0** | Полировка, docs, скриншоты, релиз   | —    |

---

## Осознанно отложено в 1.x
Interactive rebase UI, 3-way conflict resolver, reflog, syntax/word-level highlighting в diff, GPG signing, submodules, LFS, bisect, Git-Flow. Полезно, но не определяет «готовый клиент».
