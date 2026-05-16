# Network Timeout Setting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Сетевые git-операции (fetch/pull/push/clone) получают настраиваемый таймаут (дефолт 10 с) с принудительным kill зависшего процесса; значение редактируется в диалоге «Параметры» (новая категория «Сеть») и сохраняется в JSON-файле настроек Tauri.

**Architecture:** Backend: новый async-раннер `run_network_git` на `tokio::process` убивает процесс по таймауту; модуль `settings.rs` хранит `AppSettings` в `app_config_dir/settings.json` через IPC `get_settings`/`set_settings`; `mutation.rs` отдаёт чистые построители аргументов. Frontend: композабл `useSettings` грузит/сохраняет настройки через IPC, `useRemote` пробрасывает `timeout_secs`, `SettingsDialog` показывает числовой контрол в категории «Сеть».

**Tech Stack:** Rust + Tauri 2, tokio (process/time), serde_json; Vue 3 Composition API + TypeScript; проверка — `cargo test` и `npm run build`.

Спека: `docs/superpowers/specs/2026-05-16-network-timeout-setting-design.md`

---

## File Structure

- `src-tauri/Cargo.toml` — расширить features tokio.
- `src-tauri/src/settings.rs` — **новый**: `AppSettings`, `get_settings`, `set_settings`.
- `src-tauri/src/main.rs` — `mod settings;`, регистрация двух команд.
- `src-tauri/src/git/mutation.rs` — добавить построители аргументов сетевых операций + их тесты; удалить старые сетевые обёртки `fetch/pull/push/push_branch/push_tag/clone_repo`.
- `src-tauri/src/commands.rs` — заменить `run_with_timeout`/`NETWORK_TIMEOUT` на `run_network_git`; новые сигнатуры сетевых команд с `timeout_secs`.
- `src/composables/useSettings.ts` — **новый**: реактивный `networkTimeoutSecs` + IPC-persist.
- `src/composables/useRemote.ts` — проброс `timeoutSecs` в сетевые invoke.
- `src/components/dialogs/SettingsDialog.vue` — категория «Сеть» + числовой контрол.
- `src-tauri/tauri.conf.json` — bump версии (правило проекта).

---

## Task 1: Backend — модуль настроек `settings.rs`

**Files:**
- Modify: `src-tauri/Cargo.toml:16`
- Create: `src-tauri/src/settings.rs`

- [ ] **Step 1: Расширить tokio features**

В `src-tauri/Cargo.toml` заменить строку зависимости tokio:

```toml
tokio = { version = "1", features = ["rt", "rt-multi-thread", "macros", "time", "process", "io-util"] }
```

- [ ] **Step 2: Написать падающий тест модуля настроек**

Создать `src-tauri/src/settings.rs` с реализацией-заглушкой и тестами. Сначала весь файл (тесты внизу падают, т.к. логика ещё не написана — на этом шаге пишем ТОЛЬКО типы + тест, реализация в Step 4):

```rust
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 10;

fn default_network_timeout_secs() -> u64 {
    DEFAULT_NETWORK_TIMEOUT_SECS
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppSettings {
    #[serde(default = "default_network_timeout_secs")]
    pub network_timeout_secs: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            network_timeout_secs: DEFAULT_NETWORK_TIMEOUT_SECS,
        }
    }
}

/// Чтение настроек из конкретного файла. Отсутствие файла или битый JSON →
/// дефолты + перезапись валидным дефолтом.
fn read_settings_at(path: &PathBuf) -> AppSettings {
    match fs::read_to_string(path) {
        Ok(raw) => match serde_json::from_str::<AppSettings>(&raw) {
            Ok(s) => s,
            Err(_) => {
                let def = AppSettings::default();
                let _ = write_settings_at(path, &def);
                def
            }
        },
        Err(_) => {
            let def = AppSettings::default();
            let _ = write_settings_at(path, &def);
            def
        }
    }
}

/// Запись настроек в конкретный файл (создаёт родительский каталог).
fn write_settings_at(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
    Ok(read_settings_at(&path))
}

#[tauri::command]
pub fn set_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app)?;
    write_settings_at(&path, &settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "gitstream_settings_test_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn missing_file_returns_defaults_and_creates_file() {
        let p = temp_path();
        assert!(!p.exists());
        let s = read_settings_at(&p);
        assert_eq!(s, AppSettings::default());
        assert_eq!(s.network_timeout_secs, 10);
        assert!(p.exists(), "file should be created with defaults");
        fs::remove_file(&p).ok();
    }

    #[test]
    fn round_trip_write_then_read() {
        let p = temp_path();
        let s = AppSettings { network_timeout_secs: 42 };
        write_settings_at(&p, &s).unwrap();
        let back = read_settings_at(&p);
        assert_eq!(back, s);
        fs::remove_file(&p).ok();
    }

    #[test]
    fn corrupt_json_returns_defaults_and_rewrites() {
        let p = temp_path();
        fs::write(&p, "{ not json").unwrap();
        let s = read_settings_at(&p);
        assert_eq!(s, AppSettings::default());
        // файл перезаписан валидным JSON
        let raw = fs::read_to_string(&p).unwrap();
        let parsed: AppSettings = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed, AppSettings::default());
        fs::remove_file(&p).ok();
    }
}
```

(Здесь Step 2 и Step 4 совмещены в одном файле: тесты и реализация написаны вместе. Это допустимо для нового модуля — ниже сразу запускаем тесты.)

- [ ] **Step 3: Подключить модуль, чтобы он компилировался**

В `src-tauri/src/main.rs` после строки `mod git;` добавить:

```rust
mod settings;
```

- [ ] **Step 4: Запустить тесты модуля настроек**

Run: `cd src-tauri && cargo test settings::tests -- --nocapture`
Expected: PASS — 3 теста (`missing_file_returns_defaults_and_creates_file`, `round_trip_write_then_read`, `corrupt_json_returns_defaults_and_rewrites`).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/settings.rs src-tauri/src/main.rs
git commit -m "feat: модуль настроек (settings.json в app config dir)"
```

---

## Task 2: Backend — регистрация IPC-команд настроек

**Files:**
- Modify: `src-tauri/src/main.rs:48-49` (список `generate_handler!`)

- [ ] **Step 1: Зарегистрировать команды**

В `src-tauri/src/main.rs` в макросе `tauri::generate_handler![ ... ]` добавить две строки перед `commands::check_repo_path,`:

```rust
            settings::get_settings,
            settings::set_settings,
            commands::check_repo_path,
```

- [ ] **Step 2: Проверить сборку backend**

Run: `cd src-tauri && cargo build`
Expected: успешная сборка без ошибок (предупреждения о неиспользуемых старых функциях допустимы — будут убраны в Task 3).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: регистрация IPC-команд get_settings/set_settings"
```

---

## Task 3: Backend — построители аргументов сетевых операций

**Files:**
- Modify: `src-tauri/src/git/mutation.rs:61-89` (fetch/pull/push/push_branch), `:131-156` (push_tag/clone_repo), тестовый модуль в конце файла

- [ ] **Step 1: Написать падающие тесты построителей**

В `src-tauri/src/git/mutation.rs` внутри существующего блока `#[cfg(test)] mod tag_tests { ... }` добавить новые тесты перед закрывающей `}` модуля:

```rust
    #[test]
    fn fetch_args_basic() {
        assert_eq!(fetch_args("origin"), vec!["fetch", "origin"]);
    }

    #[test]
    fn pull_args_rebase_toggle() {
        assert_eq!(pull_args("origin", false), vec!["pull", "origin"]);
        assert_eq!(pull_args("origin", true), vec!["pull", "--rebase", "origin"]);
    }

    #[test]
    fn push_args_force_toggle() {
        assert_eq!(push_args("origin", false), vec!["push", "origin"]);
        assert_eq!(push_args("origin", true), vec!["push", "--force", "origin"]);
    }

    #[test]
    fn push_branch_args_force_toggle() {
        assert_eq!(push_branch_args("origin", "main", false), vec!["push", "origin", "main"]);
        assert_eq!(
            push_branch_args("origin", "main", true),
            vec!["push", "--force", "origin", "main"]
        );
    }

    #[test]
    fn push_tag_args_delete_toggle() {
        assert_eq!(
            push_tag_args("origin", "v1.0", false),
            vec!["push", "origin", "refs/tags/v1.0"]
        );
        assert_eq!(
            push_tag_args("origin", "v1.0", true),
            vec!["push", "origin", ":refs/tags/v1.0"]
        );
    }

    #[test]
    fn clone_args_basic() {
        assert_eq!(
            clone_args("https://x/y.git", "/tmp/y"),
            vec!["clone", "https://x/y.git", "/tmp/y"]
        );
    }
```

- [ ] **Step 2: Запустить тесты — убедиться, что падают**

Run: `cd src-tauri && cargo test --lib mutation 2>&1 | tail -20`
Expected: ошибка компиляции `cannot find function 'fetch_args'` (и аналогичные) — функции ещё не определены.

- [ ] **Step 3: Реализовать построители, удалить старые сетевые обёртки**

В `src-tauri/src/git/mutation.rs` **удалить** функции `fetch`, `pull`, `push`, `push_branch` (строки ~61-89), `push_tag` (~143-156) и `clone_repo` (~158-170). На их место (можно сгруппировать в одном месте, например после `delete_tag`) добавить чистые построители:

```rust
pub fn fetch_args(remote: &str) -> Vec<String> {
    vec!["fetch".into(), remote.into()]
}

pub fn pull_args(remote: &str, rebase: bool) -> Vec<String> {
    if rebase {
        vec!["pull".into(), "--rebase".into(), remote.into()]
    } else {
        vec!["pull".into(), remote.into()]
    }
}

pub fn push_args(remote: &str, force: bool) -> Vec<String> {
    if force {
        vec!["push".into(), "--force".into(), remote.into()]
    } else {
        vec!["push".into(), remote.into()]
    }
}

pub fn push_branch_args(remote: &str, branch: &str, force: bool) -> Vec<String> {
    if force {
        vec!["push".into(), "--force".into(), remote.into(), branch.into()]
    } else {
        vec!["push".into(), remote.into(), branch.into()]
    }
}

pub fn push_tag_args(remote: &str, name: &str, delete: bool) -> Vec<String> {
    let refspec = if delete {
        format!(":refs/tags/{}", name)
    } else {
        format!("refs/tags/{}", name)
    };
    vec!["push".into(), remote.into(), refspec]
}

pub fn clone_args(url: &str, dest: &str) -> Vec<String> {
    vec!["clone".into(), url.into(), dest.into()]
}
```

Если после удаления `clone_repo` импорт `use std::process::Command;` (строка 2) больше нигде в файле не используется (проверить: `grep -n "Command" src-tauri/src/git/mutation.rs` — останутся только вхождения в `#[cfg(test)]`, где `use std::process::Command;` импортируется локально внутри `mod tag_tests`) — удалить верхний `use std::process::Command;`. Аналогично проверить `classify_git_error` в импорте строки 4: если он больше не используется в основном коде (был только в `clone_repo`), убрать его из `use super::error::{classify_git_error, GitError};` → `use super::error::GitError;`.

- [ ] **Step 4: Запустить тесты построителей**

Run: `cd src-tauri && cargo test --lib mutation 2>&1 | tail -20`
Expected: PASS — все новые `*_args` тесты зелёные; существующие tag-тесты (`creates_lightweight_tag` и т.д.) по-прежнему зелёные. Сборка `mutation.rs` даёт ошибки в `commands.rs` (он ещё вызывает удалённые функции) — это нормально, чинится в Task 4. Поэтому здесь запускаем именно `cargo test --lib`, который может не собраться целиком; если так — перейти к Task 4 и запустить тесты после него.

> Примечание: если из-за разрыва в `commands.rs` крейт не компилируется и тесты `mutation` не запускаются, Task 3 и Task 4 коммитятся вместе после Task 4 Step 4. Тогда Step 5 ниже пропустить и закоммитить в Task 4.

- [ ] **Step 5: Commit (если крейт компилируется)**

```bash
git add src-tauri/src/git/mutation.rs
git commit -m "refactor: построители аргументов сетевых git-операций"
```

---

## Task 4: Backend — раннер `run_network_git` + новые сигнатуры команд

**Files:**
- Modify: `src-tauri/src/commands.rs:1-6` (импорты/константы), `:83-178` (раннер и сетевые команды)

- [ ] **Step 1: Написать падающий тест раннера**

В конец `src-tauri/src/commands.rs` добавить тестовый модуль:

```rust
#[cfg(test)]
mod network_timeout_tests {
    use super::*;
    use std::fs;
    use std::process::Command as StdCommand;

    fn temp_repo_with_dead_remote() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_net_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            StdCommand::new("git").current_dir(&dir).args(args).output().unwrap();
        };
        run(&["init", "-q"]);
        // 192.0.2.0/24 — TEST-NET-1 (RFC 5737), не маршрутизируется → подключение зависает
        run(&["remote", "add", "origin", "https://192.0.2.1/dead.git"]);
        dir
    }

    #[tokio::test]
    async fn fetch_times_out_and_kills_process() {
        let dir = temp_repo_with_dead_remote();
        let args = crate::git::mutation::fetch_args("origin");
        let start = std::time::Instant::now();
        let res = run_network_git(Some(dir.as_path()), &args, Some(1), "fetch").await;
        let elapsed = start.elapsed();

        assert!(res.is_err(), "ожидали ошибку таймаута, получили {:?}", res);
        let msg = res.unwrap_err();
        assert!(
            msg.contains("timeout") || msg.contains("таймаут") || msg.contains("превысил"),
            "сообщение об ошибке должно сообщать о таймауте: {}",
            msg
        );
        // Раннер должен вернуться вскоре после 1 с (а не висеть на git): даём запас 10 с.
        assert!(
            elapsed < std::time::Duration::from_secs(10),
            "раннер не вернулся быстро после таймаута: {:?}",
            elapsed
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn zero_timeout_falls_back_to_default() {
        // Some(0) не должен означать «без таймаута»: трактуется как дефолт.
        // Проверяем лишь, что значение секунд вычисляется как дефолт.
        assert_eq!(
            effective_timeout_secs(Some(0)),
            DEFAULT_NETWORK_TIMEOUT_SECS
        );
        assert_eq!(effective_timeout_secs(None), DEFAULT_NETWORK_TIMEOUT_SECS);
        assert_eq!(effective_timeout_secs(Some(25)), 25);
    }
}
```

- [ ] **Step 2: Запустить тест — убедиться, что не компилируется**

Run: `cd src-tauri && cargo test --lib network_timeout_tests 2>&1 | tail -20`
Expected: ошибка компиляции (`run_network_git`, `effective_timeout_secs`, `DEFAULT_NETWORK_TIMEOUT_SECS` не определены; `commands.rs` ещё ссылается на удалённые `mutation::fetch` и т.п.).

- [ ] **Step 3: Заменить шапку `commands.rs` и реализовать раннер**

В `src-tauri/src/commands.rs` заменить строки 1-6:

```rust
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;

use crate::git::{query, mutation, types::*};

pub(crate) const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 10;

pub(crate) fn effective_timeout_secs(timeout_secs: Option<u64>) -> u64 {
    timeout_secs
        .filter(|&s| s > 0)
        .unwrap_or(DEFAULT_NETWORK_TIMEOUT_SECS)
}

/// Запускает `git` для сетевой операции с таймаутом. По истечении таймаута
/// процесс принудительно убивается. `repo_path = None` для `clone`.
async fn run_network_git(
    repo_path: Option<&Path>,
    args: &[String],
    timeout_secs: Option<u64>,
    label: &str,
) -> Result<String, String> {
    let secs = effective_timeout_secs(timeout_secs);

    let mut cmd = TokioCommand::new("git");
    if let Some(p) = repo_path {
        cmd.arg("-C").arg(p);
    }
    cmd.args(args.iter().map(|s| s.as_str()));
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to run git: {} (Is git installed and in PATH?)", e)
    })?;

    match tokio::time::timeout(Duration::from_secs(secs), child.wait()).await {
        Ok(Ok(status)) => {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut o) = child.stdout.take() {
                let _ = o.read_to_string(&mut stdout).await;
            }
            if let Some(mut e) = child.stderr.take() {
                let _ = e.read_to_string(&mut stderr).await;
            }
            if status.success() {
                Ok(stdout)
            } else {
                Err(crate::git::error::classify_git_error(&stderr).to_string())
            }
        }
        Ok(Err(e)) => Err(format!("git wait failed: {}", e)),
        Err(_) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            Err(format!(
                "Network timeout: {} превысил {} сек",
                label, secs
            ))
        }
    }
}
```

(Удаляются: `use tokio::time::timeout;`, `const NETWORK_TIMEOUT`, старая функция `run_with_timeout` — её заменит `run_network_git`.)

- [ ] **Step 4: Переписать сетевые команды с `timeout_secs`**

Заменить блок `run_with_timeout` + команды (`do_fetch`, `do_pull`, `do_push`, `do_push_branch`, `do_push_tag`, `do_clone`) на:

```rust
#[tauri::command]
pub async fn do_fetch(
    repo_path: String,
    remote: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::fetch_args(&remote);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "fetch").await
}

#[tauri::command]
pub async fn do_pull(
    repo_path: String,
    remote: String,
    rebase: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::pull_args(&remote, rebase);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "pull").await
}

#[tauri::command]
pub async fn do_push(
    repo_path: String,
    remote: String,
    force: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_args(&remote, force);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}

#[tauri::command]
pub async fn do_push_branch(
    repo_path: String,
    remote: String,
    branch: String,
    force: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_branch_args(&remote, &branch, force);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}

#[tauri::command]
pub async fn do_push_tag(
    repo_path: String,
    remote: String,
    name: String,
    delete: bool,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::push_tag_args(&remote, &name, delete);
    run_network_git(Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}

#[tauri::command]
pub async fn do_clone(
    url: String,
    dest: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::clone_args(&url, &dest);
    run_network_git(None, &args, timeout_secs, "clone").await
}
```

Остальные команды (`do_checkout_remote`, `do_merge`, … `do_create_tag`, `do_delete_tag`, `check_repo_path`) не трогать.

- [ ] **Step 5: Запустить тесты бэкенда**

Run: `cd src-tauri && cargo test 2>&1 | tail -30`
Expected: PASS — `network_timeout_tests::zero_timeout_falls_back_to_default`, `network_timeout_tests::fetch_times_out_and_kills_process` (≈1-2 с, не зависает), `settings::tests::*`, `mutation` `*_args` и tag-тесты — все зелёные.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/git/mutation.rs
git commit -m "feat: таймаут сетевых git-операций с kill процесса (run_network_git)"
```

---

## Task 5: Frontend — композабл `useSettings`

**Files:**
- Create: `src/composables/useSettings.ts`

- [ ] **Step 1: Создать композабл**

Создать `src/composables/useSettings.ts`:

```ts
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface AppSettings {
  network_timeout_secs: number;
}

const DEFAULT_TIMEOUT = 10;
const MIN_TIMEOUT = 1;
const MAX_TIMEOUT = 600;

const networkTimeoutSecs = ref<number>(DEFAULT_TIMEOUT);
let loaded = false;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

function clamp(v: number): number {
  if (!Number.isFinite(v)) return DEFAULT_TIMEOUT;
  const n = Math.round(v);
  if (n < MIN_TIMEOUT) return MIN_TIMEOUT;
  if (n > MAX_TIMEOUT) return MAX_TIMEOUT;
  return n;
}

async function loadSettings() {
  if (loaded) return;
  loaded = true;
  try {
    const s = await invoke<AppSettings>("get_settings");
    networkTimeoutSecs.value = clamp(s.network_timeout_secs);
  } catch {
    networkTimeoutSecs.value = DEFAULT_TIMEOUT;
  }
}

// Дебаунс-запись, чтобы не писать файл на каждый ввод цифры.
watch(networkTimeoutSecs, (v) => {
  const safe = clamp(v);
  if (safe !== v) {
    networkTimeoutSecs.value = safe;
    return; // повторный watch с уже валидным значением запишет файл
  }
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    invoke("set_settings", { settings: { network_timeout_secs: safe } }).catch(() => {});
  }, 400);
});

void loadSettings();

export function useSettings() {
  return { networkTimeoutSecs };
}
```

- [ ] **Step 2: Проверить типизацию/сборку**

Run: `npm run build`
Expected: сборка проходит (vue-tsc без ошибок). Композабл пока не используется — это нормально.

- [ ] **Step 3: Commit**

```bash
git add src/composables/useSettings.ts
git commit -m "feat: композабл useSettings (IPC-persist таймаута)"
```

---

## Task 6: Frontend — проброс `timeoutSecs` в `useRemote`

**Files:**
- Modify: `src/composables/useRemote.ts:1-40`

- [ ] **Step 1: Заменить содержимое `useRemote.ts`**

Заменить весь файл `src/composables/useRemote.ts`:

```ts
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRepo } from "./useRepo";
import { useSettings } from "./useSettings";

const isBusy = ref(false);
const lastError = ref<string | null>(null);

export function useRemote() {
  const { repoPath } = useRepo();
  const { networkTimeoutSecs } = useSettings();

  async function wrapAsync(fn: () => Promise<unknown>) {
    isBusy.value = true;
    lastError.value = null;
    try {
      await fn();
    } catch (e) {
      lastError.value = String(e);
    } finally {
      isBusy.value = false;
    }
  }

  async function fetchRemote(remote: string) {
    await wrapAsync(() =>
      invoke("do_fetch", {
        repoPath: repoPath.value!,
        remote,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  async function pull(remote: string, rebase: boolean) {
    await wrapAsync(() =>
      invoke("do_pull", {
        repoPath: repoPath.value!,
        remote,
        rebase,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  async function push(remote: string, force: boolean) {
    await wrapAsync(() =>
      invoke("do_push", {
        repoPath: repoPath.value!,
        remote,
        force,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  async function cloneRepo(url: string, dest: string) {
    await wrapAsync(() =>
      invoke("do_clone", {
        url,
        dest,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  return { isBusy, lastError, fetchRemote, pull, push, cloneRepo };
}
```

> Tauri автоматически конвертирует camelCase аргумент `timeoutSecs` (frontend) в snake_case `timeout_secs` (Rust-команда), как и существующий `repoPath` → `repo_path`.

- [ ] **Step 2: Проверить сборку**

Run: `npm run build`
Expected: сборка проходит без ошибок типов.

- [ ] **Step 3: Commit**

```bash
git add src/composables/useRemote.ts
git commit -m "feat: проброс timeoutSecs в сетевые IPC-вызовы"
```

---

## Task 7: Frontend — UI настройки в `SettingsDialog.vue`

**Files:**
- Modify: `src/components/dialogs/SettingsDialog.vue:131-153` (категории/settings), `:269-284` (рендер контрола), `:525-553` (стили)

- [ ] **Step 1: Импорт композабла и данные категории**

В `src/components/dialogs/SettingsDialog.vue` в `<script setup>`:

После строки `import { useTheme, type ThemeMode } from "@/composables/useTheme";` добавить:

```ts
import { useSettings } from "@/composables/useSettings";
```

После строки `const { mode } = useTheme();` добавить:

```ts
const { networkTimeoutSecs } = useSettings();
```

Заменить массив `categories` (строки 135-137):

```ts
const categories = [
  { id: "appearance", label: "Внешний вид" },
  { id: "network", label: "Сеть" },
];
```

В массив `settings` добавить второй элемент (после объекта `color-theme`, перед закрывающей `]`):

```ts
  {
    id: "network-timeout",
    category: "network",
    label: "Network: Timeout (сек)",
    description:
      "Максимальное время сетевой git-операции (fetch, pull, push, clone). По истечении операция прерывается, а зависший процесс git принудительно завершается.",
  },
```

- [ ] **Step 2: Рендер числового контрола**

В `<template>` после блока `<div v-if="s.id === 'color-theme'" ...> ... </div>` (заканчивается на строке ~283) добавить:

```html
            <div v-if="s.id === 'network-timeout'" class="vs-setting-control">
              <input
                v-model.number="networkTimeoutSecs"
                type="number"
                min="1"
                max="600"
                step="1"
                class="vs-number"
              />
            </div>
```

- [ ] **Step 3: Стиль `.vs-number`**

В блоке `<style scoped>` после правила `.vs-select option { ... }` (конец файла, перед `</style>`) добавить:

```css
.vs-number {
  width: 120px;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  padding: 6px 10px;
  font-size: var(--font-size-sm);
  border-radius: var(--radius);
  outline: none;
}
.vs-number:focus {
  border-color: var(--accent);
}
```

- [ ] **Step 4: Проверить сборку**

Run: `npm run build`
Expected: сборка проходит без ошибок типов.

- [ ] **Step 5: Commit**

```bash
git add src/components/dialogs/SettingsDialog.vue
git commit -m "feat: настройка таймаута в категории «Сеть» (SettingsDialog)"
```

---

## Task 8: Bump версии и финальная проверка

**Files:**
- Modify: `src-tauri/tauri.conf.json:4`

- [ ] **Step 1: Инкремент patch-версии (правило проекта)**

В `src-tauri/tauri.conf.json` изменить `"version": "0.1.9"` → `"version": "0.1.10"`.

- [ ] **Step 2: Полная проверка backend**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: все тесты зелёные (settings, mutation `*_args` + tag, network_timeout).

- [ ] **Step 3: Полная проверка frontend**

Run: `npm run build`
Expected: сборка успешна, ошибок типов нет.

- [ ] **Step 4: Ручная проверка (smoke), описание для исполнителя**

Запустить `npm run tauri dev`, открыть диалог «Параметры» → в сайдбаре под «Внешний вид» видна категория «Сеть» → выбрать её → поле «Network: Timeout (сек)» со значением 10. Изменить на 3, закрыть/открыть приложение — значение сохранилось (читается из `settings.json` в app config dir). Выполнить push на недоступный remote — операция прерывается ~через выбранное число секунд с сообщением о таймауте, фоновый процесс git не остаётся (проверить `ps aux | grep git`).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: bump версии 0.1.10"
```

---

## Self-Review

**Spec coverage:**
- Требование 1 (таймаут только сетевым) → Task 4 (только сетевые команды переписаны).
- Требование 2 (kill процесса) → Task 4 Step 3 (`start_kill` + `kill_on_drop`), тест `fetch_times_out_and_kills_process`.
- Требование 3 (дефолт 10) → `DEFAULT_NETWORK_TIMEOUT_SECS`/`AppSettings::default` (Task 1, Task 4).
- Требование 4 (`timeout_secs` из UI) → Task 4 сигнатуры, Task 6 проброс.
- Требование 5 (persist в Tauri JSON) → Task 1 (settings.rs), Task 2 (регистрация).
- Требование 6 (категория «Сеть» ниже «Внешний вид») → Task 7.
- Слой 2 спеки (построители аргументов, удаление старых обёрток) → Task 3.
- Обработка ошибок (классификация, спавн, kill, битый JSON, Some(0)) → Task 1 тесты + Task 4 раннер/тесты.

**Placeholder scan:** нет TBD/TODO; весь код приведён полностью.

**Type consistency:** `AppSettings { network_timeout_secs: u64 }` одинаково в settings.rs и фронте (`network_timeout_secs`); `timeout_secs: Option<u64>` в командах ↔ `timeoutSecs` в invoke (Tauri camelCase→snake_case); `effective_timeout_secs`/`DEFAULT_NETWORK_TIMEOUT_SECS`/`run_network_git` определены в Task 4 и используются согласованно; `*_args` имена совпадают между Task 3 реализацией, тестами и вызовами в Task 4.

**Известный риск (отражён в плане):** при удалении старых сетевых обёрток (Task 3) крейт временно не компилируется, пока Task 4 не перепишет `commands.rs`. План явно допускает совместный коммит Task 3+4, если `cargo test --lib mutation` не запускается из-за разрыва.
