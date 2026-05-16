# Таймаут сетевых git-операций с настройкой в «Параметрах»

Дата: 2026-05-16
Статус: согласовано, готово к написанию плана реализации

## Проблема

Сетевые git-операции (fetch/pull/push/clone) могут зависать (недоступный
remote, запрос аутентификации, медленная сеть). Сейчас:

- `commands.rs` содержит `run_with_timeout` с хардкод-константой
  `NETWORK_TIMEOUT = 5 сек`, оборачивающей только сетевые команды через
  `tokio::time::timeout` + `tokio::task::spawn_blocking`.
- Таймаут лишь **прекращает ожидание**, но **не убивает** процесс `git` —
  он продолжает висеть в фоне (orphan-процесс).
- Значение не настраивается пользователем и не сохраняется между запусками.

## Требования (согласованы с пользователем)

1. Таймаут применяется **только к сетевым операциям**:
   `do_fetch`, `do_pull`, `do_push`, `do_push_branch`, `do_push_tag`,
   `do_clone`.
2. По истечении таймаута зависший процесс `git` **принудительно убивается**
   (`child.kill()`), без orphan-процессов.
3. Значение таймаута — настраиваемый параметр, **по умолчанию 10 секунд**.
4. Параметр передаётся из UI как аргумент IPC-команд (`timeout_secs`).
5. Значение **сохраняется между запусками** в JSON-файле настроек Tauri
   (app config dir), не в localStorage.
6. Пользователь меняет значение в существующем диалоге «Параметры»
   (`SettingsDialog.vue`) — **новая категория «Сеть» в сайдбаре ниже
   «Внешний вид»**.

## Архитектура

### Слой 1. Backend: убиваемый раннер сетевых git-операций

`Cargo.toml`: добавить фичу `process` в tokio →
`tokio = { version = "1", features = ["rt", "time", "process"] }`.

Новый async-хелпер в `commands.rs`, заменяет текущий `run_with_timeout`:

```rust
const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 10;

async fn run_network_git(
    repo_path: Option<&Path>,
    args: &[String],
    timeout_secs: Option<u64>,
    label: &str,
) -> Result<String, String>
```

Поведение:

- `secs = timeout_secs.filter(|&s| s > 0).unwrap_or(DEFAULT_NETWORK_TIMEOUT_SECS)`
  — `None` и `Some(0)` дают дефолт 10 (нельзя случайно отключить таймаут).
- Спавн через `tokio::process::Command::new("git")`; если
  `repo_path = Some(p)` — добавляются аргументы `-C <p>`; для `clone`
  передаётся `None` (репозитория ещё нет). `stdout`/`stderr` → `piped()`.
- `tokio::time::timeout(Duration::from_secs(secs), child.wait_with_output())`.
- Успех → если `status.success()`: вернуть stdout; иначе stderr прогнать
  через существующий `classify_git_error` (классификация
  auth/network/conflict + hint сохраняется).
- Таймаут (`Err(Elapsed)`) → вызвать `child.start_kill()` /
  `child.kill().await` (ошибку kill игнорировать — процесс мог уже
  завершиться), вернуть
  `Err("Network timeout: <label> превысил <secs> сек")`.
- Ошибка спавна (git не найден) → `Err` с hint
  `"Is git installed and in PATH?"` (как в текущем `query::run_git`).

Примечание по реализации: чтобы убить процесс, `child` не должен быть
полностью перемещён в `wait_with_output()` до момента kill. Используется
паттерн tokio: держать `child`, под `timeout` ждать `child.wait()`
(берёт `&mut self`), при истечении — `child.kill().await`; вывод
`stdout`/`stderr` читается из захваченных pipe после успешного `wait`.
Конкретная форма (select! или timeout + ручное чтение pipe) выбирается на
этапе реализации; контракт функции фиксирован выше.

### Слой 2. Backend: разбиение mutation → построители аргументов

Текущие блокирующие обёртки `mutation::fetch/pull/push/push_branch/
push_tag/clone_repo` используются **только** сетевыми командами в
`commands.rs`. Сепарация `query`/`mutation` из CLAUDE.md сохраняется:

- В `mutation.rs` добавить чистые функции-построители аргументов,
  возвращающие `Vec<String>` (без выполнения процесса), например:
  `fetch_args(remote)`, `pull_args(remote, rebase)`,
  `push_args(remote, force)`, `push_branch_args(remote, branch, force)`,
  `push_tag_args(remote, name, delete)`, `clone_args(url, dest)`.
- Старые сетевые функции `fetch/pull/push/push_branch/push_tag/
  clone_repo`, ходившие через `run_git_mut`, удаляются (их единственные
  вызовы переезжают на `run_network_git` + построители).
- Несетевые мутации (`stage/unstage/discard/commit/checkout/merge/
  rename_branch/delete_branch/create_tag/delete_tag` и т.д.) **не
  затрагиваются** — остаются синхронными через `run_git_mut`.
- Существующие тесты в `mutation.rs` не ломаются (построители
  покрываются косвенно; при необходимости добавить юнит-тесты на
  корректность собранных аргументов).

### Слой 3. Backend: хранение настроек (Tauri JSON)

Новый модуль `src-tauri/src/settings.rs`:

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub network_timeout_secs: u64, // default 10
}
```

- Путь: `app.path().app_config_dir()` + `settings.json`.
- `get_settings(app) -> AppSettings`: файл отсутствует или JSON битый →
  вернуть дефолты (`network_timeout_secs: 10`) и перезаписать файл
  валидным дефолтом.
- `set_settings(app, settings: AppSettings) -> Result<(), String>`:
  создать config dir при отсутствии, сериализовать, записать атомарно
  (write в temp + rename, либо простой write — выбрать на реализации).
- Обе функции — `#[tauri::command]`, принимают `app: tauri::AppHandle`.
- Регистрация в `main.rs`: `mod settings;` + добавить
  `commands::get_settings, commands::set_settings` (или
  `settings::get_settings, settings::set_settings`) в
  `tauri::generate_handler!`.

### Слой 4. Frontend: композабл настроек

Новый `src/composables/useSettings.ts` (паттерн как `useTheme`, но через
IPC вместо localStorage):

- При инициализации: `invoke("get_settings")` → реактивный
  `networkTimeoutSecs = ref<number>(10)`.
- `watch(networkTimeoutSecs, …)` → `invoke("set_settings", { settings:
  { network_timeout_secs: value } })` (с дебаунсом или на blur — выбрать
  на реализации, чтобы не писать файл на каждый ввод цифры).
- Экспорт `useSettings()` → `{ networkTimeoutSecs }`.
- Загрузка инициируется один раз при старте приложения (как `useTheme`).

### Слой 5. Frontend: проброс таймаута в сетевые invoke

`src/composables/useRemote.ts`: во все сетевые `invoke`
(`do_fetch`, `do_pull`, `do_push`, `do_push_branch`, `do_push_tag`,
`do_clone`) добавить аргумент `timeout_secs: networkTimeoutSecs.value`
(через `useSettings`).

### Слой 6. Frontend: UI в SettingsDialog

`src/components/dialogs/SettingsDialog.vue`:

- В массив `categories` добавить **после** `appearance`:
  `{ id: "network", label: "Сеть" }`.
- В массив `settings` добавить элемент категории `network`:
  - `id: "network-timeout"`
  - `label: "Network: Timeout (сек)"`
  - `description`: пояснение, что это лимит на сетевые git-операции
    (fetch/pull/push/clone); по истечении операция прерывается, процесс
    git убивается.
- В шаблоне добавить ветку рендера контрола для
  `s.id === 'network-timeout'`: числовой `<input type="number">`,
  `min=1`, `max=600`, `step=1`, привязка к `networkTimeoutSecs` из
  `useSettings`. Невалидный/пустой ввод нормализуется к ближайшему
  допустимому (clamp 1..600), при пустом — дефолт 10.
- Новый CSS-класс `.vs-number` в духе существующего `.vs-select`
  (ширина, фон `--bg-tertiary`, рамка `--border`, фокус `--accent`).

## Изменяемые сигнатуры IPC

```rust
do_fetch(repo_path, remote, timeout_secs: Option<u64>)
do_pull(repo_path, remote, rebase, timeout_secs: Option<u64>)
do_push(repo_path, remote, force, timeout_secs: Option<u64>)
do_push_branch(repo_path, remote, branch, force, timeout_secs: Option<u64>)
do_push_tag(repo_path, remote, name, delete, timeout_secs: Option<u64>)
do_clone(url, dest, timeout_secs: Option<u64>)
```

## Обработка ошибок

| Случай | Поведение |
|---|---|
| Таймаут истёк | kill процесса; `Err("Network timeout: <label> превысил <N> сек")` |
| git не найден (спавн) | `Err` + hint `"Is git installed and in PATH?"` |
| git вернул ненулевой код | stderr → `classify_git_error` (auth/network/conflict) |
| kill не сработал (процесс мёртв) | ошибку kill игнорировать, всё равно вернуть ошибку таймаута |
| `settings.json` отсутствует/битый | дефолты (`network_timeout_secs: 10`), файл перезаписывается |
| `timeout_secs` = `None` или `Some(0)` | дефолт 10 (нельзя отключить таймаут) |

## Тестирование

- Юнит-тест `run_network_git`: операция против заведомо недоступного
  remote с маленьким `timeout_secs` → ошибка таймаута за ~N сек,
  процесс git убит (нет orphan / zombie).
- Успешный путь: локальный `fetch`/`push` в bare-репозиторий
  укладывается в таймаут → `Ok`.
- Юнит-тесты построителей аргументов в `mutation.rs`
  (`fetch_args`/`pull_args`/… дают ожидаемые `Vec<String>`).
- `settings.rs`: round-trip get→set→get; отсутствующий файл → дефолты;
  битый JSON → дефолты + перезапись.
- Существующие тесты `mutation.rs` остаются зелёными.

## Вне объёма (YAGNI)

- Таймаут для несетевых/локальных git-операций.
- Глобальный/системный конфиг, синхронизация настроек.
- Отдельная кнопка/иконка запуска «Параметров» (диалог уже есть).
- Прочие настройки кроме `network_timeout_secs`.
- Миграция темы/геометрии из localStorage в новый JSON (не трогаем).

## Затрагиваемые файлы

- `src-tauri/Cargo.toml` — фича tokio `process`
- `src-tauri/src/commands.rs` — `run_network_git`, новые сигнатуры
- `src-tauri/src/git/mutation.rs` — построители аргументов, удаление
  старых сетевых обёрток
- `src-tauri/src/settings.rs` — новый модуль (AppSettings, get/set)
- `src-tauri/src/main.rs` — `mod settings;`, регистрация команд
- `src/composables/useSettings.ts` — новый композабл
- `src/composables/useRemote.ts` — проброс `timeout_secs`
- `src/components/dialogs/SettingsDialog.vue` — категория «Сеть» + контрол

## Замечания / открытые риски

- Хранение таймаута в Tauri JSON отличается от текущего паттерна
  (тема/геометрия в localStorage). Сделано по явному выбору пользователя;
  существующие настройки не мигрируются.
- Версия в заголовке окна «GitStream v0.1.x» должна быть инкрементирована
  по правилу проекта при реализации.
