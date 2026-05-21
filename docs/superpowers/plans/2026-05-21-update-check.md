# Update Check Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** При запуске GitStream однократно проверяет GitHub Releases API и показывает ненавязчивый баннер снизу-справа, если доступна новая версия.

**Architecture:** Rust-команда `check_for_update` делает HTTP-запрос через `reqwest` к GitHub API, сравнивает semver текущей и последней версии, возвращает `UpdateInfo | null`. Vue-composable вызывает её при старте; `UpdateBanner.vue` рендерится при наличии результата. Вторая команда `open_url` открывает URL через системный браузер (xdg-open / open / cmd).

**Tech Stack:** Tauri 2, Rust (reqwest 0.12 + rustls-tls), Vue 3 Composition API, TypeScript

---

## File Map

| Action | Path | Responsibility |
|--------|------|---------------|
| Modify | `src-tauri/Cargo.toml` | Add `reqwest` dependency |
| Modify | `src-tauri/src/git/types.rs` | Add `UpdateInfo` struct |
| Modify | `src-tauri/src/commands.rs` | Add `version_gt`, `check_for_update`, `open_url`, `open_in_browser` |
| Modify | `src-tauri/src/main.rs` | Register two new commands |
| Modify | `src/types/index.ts` | Add `UpdateInfo` TypeScript interface |
| Create | `src/composables/useUpdate.ts` | Composable: invoke + reactive state |
| Create | `src/components/UpdateBanner.vue` | Fixed banner UI |
| Modify | `src/App.vue` | Import + mount banner, call `checkForUpdate` on mount |

---

## Task 1: Add reqwest + UpdateInfo struct

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/git/types.rs`

- [ ] **Step 1: Add reqwest to Cargo.toml**

В секцию `[dependencies]` добавить строку:

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

Итоговый `[dependencies]` block:
```toml
[dependencies]
tauri = { version = "2", features = ["image-png"] }
tauri-plugin-window-state = "2"
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["rt", "rt-multi-thread", "macros", "time", "process", "io-util"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

- [ ] **Step 2: Add UpdateInfo struct to types.rs**

В конец файла `src-tauri/src/git/types.rs` добавить:

```rust
#[derive(Debug, serde::Serialize, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub release_url: String,
    pub changelog_url: String,
}
```

- [ ] **Step 3: Verify compile**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```

Expected: завершение без ошибок (возможны предупреждения — не ошибки).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/git/types.rs
git commit -m "feat(update): add reqwest dep + UpdateInfo struct"
```

---

## Task 2: TDD — version_gt helper

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: Write failing unit tests**

В конец файла `src-tauri/src/commands.rs` добавить:

```rust
#[cfg(test)]
mod update_tests {
    use super::version_gt;

    #[test]
    fn newer_patch_is_gt() {
        assert!(version_gt("v0.4.2", "0.4.1"));
    }

    #[test]
    fn newer_minor_is_gt() {
        assert!(version_gt("v0.5.0", "0.4.9"));
    }

    #[test]
    fn newer_major_is_gt() {
        assert!(version_gt("v1.0.0", "0.9.9"));
    }

    #[test]
    fn same_version_is_not_gt() {
        assert!(!version_gt("v0.4.1", "0.4.1"));
    }

    #[test]
    fn older_version_is_not_gt() {
        assert!(!version_gt("v0.4.0", "0.4.1"));
    }

    #[test]
    fn no_v_prefix_works() {
        assert!(version_gt("0.4.2", "0.4.1"));
    }
}
```

- [ ] **Step 2: Run tests — ожидаем FAIL (функция не существует)**

```bash
cd src-tauri && cargo test update_tests 2>&1 | tail -10
```

Expected: ошибка компиляции `cannot find function version_gt`.

- [ ] **Step 3: Implement version_gt**

В `src-tauri/src/commands.rs` перед существующим кодом (после импортов) добавить:

```rust
fn version_gt(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.trim_start_matches('v')
            .splitn(4, '.')
            .take(3)
            .filter_map(|x| x.split('-').next().and_then(|n| n.parse().ok()))
            .collect()
    };
    parse(latest) > parse(current)
}
```

- [ ] **Step 4: Run tests — ожидаем PASS**

```bash
cd src-tauri && cargo test update_tests 2>&1 | tail -10
```

Expected:
```
test update_tests::newer_patch_is_gt ... ok
test update_tests::newer_minor_is_gt ... ok
test update_tests::newer_major_is_gt ... ok
test update_tests::same_version_is_not_gt ... ok
test update_tests::older_version_is_not_gt ... ok
test update_tests::no_v_prefix_works ... ok
test result: ok. 6 passed; 0 failed
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat(update): version_gt helper with unit tests"
```

---

## Task 3: Rust commands — check_for_update + open_url

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add check_for_update command to commands.rs**

В конец `src-tauri/src/commands.rs` (перед `#[cfg(test)]` блоком) добавить:

```rust
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("gitstream-app")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = match client
        .get("https://api.github.com/repos/avv7bc/gitstream/releases/latest")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Ok(None),
    };

    let tag_name = match json["tag_name"].as_str() {
        Some(t) => t.to_string(),
        None => return Ok(None),
    };

    let html_url = json["html_url"].as_str().unwrap_or("").to_string();
    let current = app.package_info().version.to_string();

    if version_gt(&tag_name, &current) {
        Ok(Some(UpdateInfo {
            version: tag_name.trim_start_matches('v').to_string(),
            release_url: html_url.clone(),
            changelog_url: html_url,
        }))
    } else {
        Ok(None)
    }
}
```

- [ ] **Step 2: Add open_url command to commands.rs**

Сразу после `check_for_update` добавить:

```rust
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open_in_browser(&url).map_err(|e| e.to_string())
}

fn open_in_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn()?;
    Ok(())
}
```

- [ ] **Step 3: Register commands in main.rs**

В `src-tauri/src/main.rs` в `tauri::generate_handler![...]` добавить две строки в конец списка, перед закрывающей `]`:

```rust
            commands::check_for_update,
            commands::open_url,
```

Итоговый конец списка будет выглядеть так:
```rust
            commands::get_repo_stats,
            commands::check_for_update,
            commands::open_url,
        ])
```

- [ ] **Step 4: Build проверка**

```bash
cd src-tauri && cargo build 2>&1 | grep -E '^error' | head -10
```

Expected: нет строк начинающихся с `error`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "feat(update): check_for_update + open_url Tauri commands"
```

---

## Task 4: TypeScript type + useUpdate composable

**Files:**
- Modify: `src/types/index.ts`
- Create: `src/composables/useUpdate.ts`

- [ ] **Step 1: Add UpdateInfo to src/types/index.ts**

В конец файла `src/types/index.ts` добавить:

```typescript
export interface UpdateInfo {
  version: string;
  release_url: string;
  changelog_url: string;
}
```

- [ ] **Step 2: Create src/composables/useUpdate.ts**

```typescript
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "@/types";

const updateInfo = ref<UpdateInfo | null>(null);

export function useUpdate() {
  async function checkForUpdate() {
    try {
      const info = await invoke<UpdateInfo | null>("check_for_update");
      updateInfo.value = info;
    } catch {
      // silent fail — no network or API error
    }
  }

  return { updateInfo, checkForUpdate };
}
```

- [ ] **Step 3: Type-check**

```bash
npx tsc --noEmit 2>&1 | head -20
```

Expected: нет ошибок.

- [ ] **Step 4: Commit**

```bash
git add src/types/index.ts src/composables/useUpdate.ts
git commit -m "feat(update): UpdateInfo type + useUpdate composable"
```

---

## Task 5: UpdateBanner.vue component

**Files:**
- Create: `src/components/UpdateBanner.vue`

- [ ] **Step 1: Create src/components/UpdateBanner.vue**

```vue
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "@/types";

const props = defineProps<{ info: UpdateInfo }>();
const emit = defineEmits<{ dismiss: [] }>();

async function openUrl(url: string) {
  try {
    await invoke("open_url", { url });
  } catch {
    // fallback: window.open может не работать в Tauri, но как запасной вариант
    window.open(url, "_blank");
  }
}

function download() {
  openUrl(props.info.release_url);
  emit("dismiss");
}
</script>

<template>
  <div class="update-banner">
    <div class="update-icon">↑</div>
    <div class="update-content">
      <div class="update-title">
        Обновление GitStream доступно (v{{ info.version }})
      </div>
      <button class="update-changelog" @click="openUrl(info.changelog_url)">
        Список изменений ›
      </button>
    </div>
    <div class="update-actions">
      <button class="btn btn-primary" @click="download">Загрузить</button>
      <button class="btn btn-secondary" @click="emit('dismiss')">
        Отменить
      </button>
    </div>
  </div>
</template>

<style scoped>
.update-banner {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 90;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  padding: 12px 16px;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-width: 280px;
  max-width: 340px;
}

.update-icon {
  font-size: 18px;
  color: var(--accent);
  flex-shrink: 0;
  padding-top: 2px;
}

.update-content {
  flex: 1;
  min-width: 0;
}

.update-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
  line-height: 1.3;
}

.update-changelog {
  font-size: var(--font-size-xs);
  color: var(--accent);
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}

.update-changelog:hover {
  text-decoration: underline;
}

.update-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
}

.update-actions .btn {
  font-size: var(--font-size-xs);
  padding: 4px 10px;
  white-space: nowrap;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/UpdateBanner.vue
git commit -m "feat(update): UpdateBanner component"
```

---

## Task 6: Integrate into App.vue

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Add import for UpdateBanner and useUpdate**

В секции `<script setup>` в `src/App.vue`, в блок импортов добавить две строки после существующих импортов (после строки с `AddTagDialog`):

```typescript
import UpdateBanner from "./components/UpdateBanner.vue";
import { useUpdate } from "@/composables/useUpdate";
```

- [ ] **Step 2: Destructure useUpdate и вызов при старте**

После существующих строк с `useXxx()` destructuring добавить:

```typescript
const { updateInfo, checkForUpdate } = useUpdate();
```

В существующем `onMounted` добавить вызов `checkForUpdate()` в конец блока:

```typescript
onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("contextmenu", onContextMenu);
  restoreLastRepo();
  pollTimer = setTimeout(pollTick, 1000);
  checkForUpdate();
});
```

- [ ] **Step 3: Mount UpdateBanner в template**

В конце `<template>`, перед закрывающим `</div>` и `</template>`, после последнего диалога (`<FileCompareDialog ...>` или `<ConfirmDialog ...>`), добавить:

```vue
    <UpdateBanner
      v-if="updateInfo"
      :info="updateInfo"
      @dismiss="updateInfo = null"
    />
```

- [ ] **Step 4: Build проверка**

```bash
npm run build 2>&1 | grep -E 'error|Error' | grep -v 'node_modules' | head -20
```

Expected: нет ошибок сборки.

- [ ] **Step 5: Commit**

```bash
git add src/App.vue
git commit -m "feat(update): интеграция UpdateBanner в App.vue"
```

---

## Task 7: Manual QA

- [ ] **Step 1: Запустить dev-сборку**

```bash
npm run tauri dev
```

- [ ] **Step 2: Проверить нормальный сценарий (актуальная версия)**

Текущая версия `0.4.1`. Если на GitHub нет релизов выше — баннер не должен появиться. Убедиться, что приложение запускается без ошибок в консоли.

- [ ] **Step 3: Симулировать наличие обновления**

В `src/composables/useUpdate.ts` временно заменить `invoke` на мок:

```typescript
async function checkForUpdate() {
  updateInfo.value = {
    version: "9.9.9",
    release_url: "https://github.com/avv7bc/gitstream/releases",
    changelog_url: "https://github.com/avv7bc/gitstream/releases",
  };
}
```

Перезапустить dev-сборку, убедиться:
- Баннер появляется снизу-справа
- Версия отображается корректно
- Кнопка «Загрузить» открывает браузер и закрывает баннер
- Кнопка «Список изменений ›» открывает браузер, баннер остаётся
- Кнопка «Отменить» закрывает баннер

- [ ] **Step 4: Вернуть реальный вызов**

Откатить изменения в `useUpdate.ts`:

```typescript
async function checkForUpdate() {
  try {
    const info = await invoke<UpdateInfo | null>("check_for_update");
    updateInfo.value = info;
  } catch {
    // silent fail
  }
}
```

- [ ] **Step 5: Финальный коммит (bump версии)**

Поднять patch-версию в `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` (0.4.1 → 0.4.2).

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Cargo.lock
git commit -m "feat: check for updates on startup + bump 0.4.2"
```
