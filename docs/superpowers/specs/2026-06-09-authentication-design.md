# Дизайн: Аутентификация (GitStream 0.9.8)

Закрывает пункт 3 дорожной карты 1.0.0: перехват запроса credentials (HTTPS login/token,
SSH passphrase), интеграция с git credential helper, внятные ошибки auth.

## Проблема

[`run_network_git`](../../../src-tauri/src/commands.rs) запускает git **без** `GIT_ASKPASS`/
`GIT_TERMINAL_PROMPT`. При запросе пароля git зависает на отсутствующем TTY и падает по
таймауту (10 c). Никакого prompt'а в GUI нет. `classify_git_error` ловит auth-ошибки только
постфактум, текстом в Git output.

## Решения (согласовано с пользователем)

- **Объём:** HTTPS (login/token) + SSH passphrase — полный паритет со SmartGit.
- **Хранение:** делегируем git'у. askpass — только источник ввода; персистентность даёт
  настроенный `credential.helper` (git сам вызывает `store`/`approve` после успеха).
- **SSH «запомнить»:** спрашиваем каждый раз; ssh-agent кэширует сам (best-effort, не в 0.9.8).

## Механизм — self-exec askpass-мост

GitStream выступает и приложением, и askpass-helper'ом (приём VSCode), без отдельного бинаря.

1. **Режим askpass.** В начале `main()` проверяется env `GITSTREAM_ASKPASS_PIPE`. Если задан →
   Tauri не поднимается; процесс: читает prompt из `argv[1]`, коннектится к `127.0.0.1:PORT`,
   шлёт `nonce\nprompt\n`, получает ответ, печатает в stdout, выходит (cancel → код 1, git
   прерывается).
2. **IPC-сервер родителя.** В `setup()` поднимается `std::net::TcpListener` на `127.0.0.1:0`
   + случайный hex-nonce. Поток принимает соединения; на каждое: валидирует nonce, парсит
   prompt → `{kind, host, key_path}`, эмитит событие `askpass_request` во фронт, блокируется
   на `mpsc::recv()` до ответа команды `askpass_respond`, пишет `OK\n<value>\n` либо `CANCEL\n`.

**Протокол сокета:** клиент → `nonce\nprompt\n`; сервер → `OK\n<value>\n` или `CANCEL\n`.

*Отвергнуто:* отдельный helper-скрипт (лишний файл/права); Unix-socket (на Windows нужен
named pipe — TCP+nonce кроссплатформенно).

## Встройка в `run_network_git`

Для fetch/pull/push/clone (через `app.try_state::<AskpassState>()`) выставляются env:
`GIT_ASKPASS`/`SSH_ASKPASS` = `current_exe()`, `SSH_ASKPASS_REQUIRE=force`,
`GIT_TERMINAL_PROMPT=0`, `GITSTREAM_ASKPASS_PIPE`, `GITSTREAM_ASKPASS_NONCE`.

**Пауза таймаута.** Текущий единый `tokio::time::timeout(child.wait())` убил бы git, пока
юзер печатает пароль. Замена — poll-цикл `child.try_wait()` каждые ~150 мс: бюджет таймаута
расходуется только когда нет активного askpass-запроса (`AskpassState.active: AtomicUsize > 0`).

## Парсер prompt'а (Rust, юнит-тест)

`Username for '…'` → Username (host); `Password for '…'` → Password (host);
`Enter passphrase for key '…'` → Passphrase (key_path); `(yes/no…)` → Confirm; иначе → Generic.

## Хранение / «запомнить»

Команда `ensure_credential_helper()`: если `git config --global credential.helper` пуст —
ставит дефолт платформы (`cache` Linux, `osxkeychain` macOS, `manager` Windows). Идемпотентно,
не перезаписывает существующий. Чекбокс «Запомнить» в диалоге (default on) дёргает её при вводе
HTTPS-логина/пароля. Persistent-хранилище на Linux (libsecret) — за рамками 0.9.8.

## Ошибки

`GitError::AuthCancelled` + классификация `could not read username/password`,
`unable to read askpass`. Surface через Git output (per memory), не модалкой. Сам диалог ввода —
модалка ввода, не ошибки.

## Файлы

- **new** `src-tauri/src/askpass.rs` — `maybe_run_askpass()` (клиент), `start(app) -> AskpassState`
  (сервер), `AskpassState::respond`, `parse_prompt` + тесты.
- `src-tauri/src/main.rs` — ранний `askpass::maybe_run_askpass()`, `app.manage(askpass::start(...))`,
  регистрация `askpass_respond` + `ensure_credential_helper`.
- `src-tauri/src/commands.rs` — env в `run_network_git`, poll-таймаут, команды `askpass_respond`,
  `ensure_credential_helper`.
- `src-tauri/src/git/error.rs` — `AuthCancelled`.
- `src-tauri/src/git/mutation.rs` — `ensure_credential_helper()`.
- `src-tauri/Cargo.toml` — `getrandom` (nonce).
- **new** `src/composables/useAuth.ts` — слушает `askpass_request`, очередь, respond/cancel.
- **new** `src/components/dialogs/CredentialDialog.vue` — draggable, поля по kind, маска,
  чекбокс «Запомнить», yes/no для Confirm.
- `src/App.vue` — монтаж диалога; `src/locales/{ru,en}.ts` — блок `credential`.

## Поток данных

git → нужен cred → exec GitStream(askpass) с prompt в argv → TCP+nonce → родитель эмитит
`askpass_request` → CredentialDialog → submit → `askpass_respond` → родитель пишет в сокет →
askpass печатает в stdout → git использует; при настроенном helper git сохраняет сам.

## Тесты / проверка

- Rust unit: `parse_prompt` (все kinds), nonce-валидация.
- Manual: HTTPS clone приватного репо по токену; SSH clone с passphrase-ключом; отмена диалога.
- `vue-tsc --noEmit`, `cargo build`, `cargo clippy`.
