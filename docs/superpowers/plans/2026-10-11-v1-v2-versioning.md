# Разделение версий v1 / v2 — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Версия 1 продолжает жить в ветке `release/1.x`, а `main` становится веткой версии 2. Пользователи каждой версии получают уведомления об обновлениях только своей линии, а о версии 2 — одно отдельное сообщение.

**Architecture:** Проверка обновлений перестаёт опираться на GitHub `releases/latest`. Она загружает список релизов и выбирает новейший внутри своей мажорной версии; эта логика вынесена в чистую функцию `pick_update` (`src-tauri/src/update.rs`), её покрывают юнит-тесты. Релизный workflow собирает prerelease-теги и снимает метку «latest» с релизов v1, когда уже вышла стабильная v2. Совместимость настроек обеспечивает правило «только добавлять поля», его закрепляет тест.

**Tech Stack:** Rust (Tauri 2, reqwest, serde, crate `semver`), Vue 3 + TypeScript, vitest, GitHub Actions (`tauri-apps/tauri-action@v0`, `gh` CLI на раннерах).

**Правила проекта, которые действуют на каждом шаге:**
- Коммит, push, создание ветки, тег — **только после явного согласия пользователя в этот раз**. Шаги «Commit» в плане означают «подготовить коммит и спросить».
- В коммитах нет строки `Co-Authored-By`.
- Версию в `package.json` / `tauri.conf.json` не поднимать, пока пользователь не скажет.
- Перед отчётом о готовности: `npx vue-tsc --noEmit`, `npm test`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (в `src-tauri`).

---

## Файлы

| Файл | Ветка | Что меняется |
|---|---|---|
| `src-tauri/Cargo.toml` | release/1.x → main | + зависимость `semver = "1"` |
| `src-tauri/src/update.rs` | release/1.x → main | **новый**: `ReleaseEntry`, `pick_update`, тесты |
| `src-tauri/src/main.rs` | release/1.x → main | `mod update;` |
| `src-tauri/src/git/types.rs:182-186` | release/1.x → main | `UpdateInfo` + `PartialEq`; новый `UpdateCheck` |
| `src-tauri/src/commands.rs:31-40, 849-893, 1008-1041` | release/1.x → main | `check_for_update` → список релизов + `pick_update`; удалить `version_gt` и его тесты |
| `src-tauri/src/settings.rs` (тест-модуль) | release/1.x → main | тест: неизвестные поля не сбрасывают настройки |
| `src/types/index.ts:191-194` | release/1.x → main | + `UpdateCheck` |
| `src/composables/useUpdate.ts` | release/1.x → main | две ссылки: `updateInfo`, `nextMajorInfo`; запоминание «не показывать» |
| `src/composables/useUpdate.test.ts` | release/1.x → main | **новый** |
| `src/components/UpdateBanner.vue` | release/1.x → main | проп `kind`, строки через i18n |
| `src/locales/ru.ts:154-156`, `src/locales/en.ts:149-151` | release/1.x → main | ключи `update.*` |
| `src/App.vue:711-715` | release/1.x → main | второй баннер для следующей мажорной версии |
| `.github/workflows/ci.yml:3-6` | release/1.x → main | CI на `release/**` |
| `.github/workflows/release.yml` | release/1.x → main | prerelease-теги, снятие метки «latest» со старой линии |
| `CLAUDE.md` | main (и release/1.x) | раздел «Версии и ветки» |
| `docs/SPEC-redesign-2026.md` §5 | main | релизы 1.2–1.7 → 2.0–2.3 |

Порядок: Задача 1 — подготовка `main`; Задача 2 — создание `release/1.x`; Задачи 3–9 делаются **в `release/1.x`**, потому что это исправления, нужные v1; Задача 10 переносит их в `main`; Задачи 11–12 — только `main`.

---

### Задача 1: Зафиксировать текущее состояние `main`

**Files:** удаление `flatpak/` (уже в индексе), `docs/SPEC-redesign-2026.md`, `docs/superpowers/plans/2026-10-11-v1-v2-versioning.md`, `docs/ROADMAP-2.0.md`.

- [ ] **Step 1: Решить с пользователем судьбу `reports/` и `research_notes/`**

Варианты: закоммитить в `docs/research/` или добавить в `.gitignore`. Без ответа — не коммитить их.

- [ ] **Step 2: Проверить состояние**

Run: `git status --short`
Expected: `D  flatpak/...` (5 файлов) и новые файлы в `docs/`.

- [ ] **Step 3: Commit (после согласия)**

```bash
git add docs/SPEC-redesign-2026.md docs/ROADMAP-2.0.md docs/superpowers/plans/2026-10-11-v1-v2-versioning.md
git commit -m "chore: убрать Flatpak; спека редизайна и план версий v1/v2"
```

---

### Задача 2: Создать ветку `release/1.x`

Ветка создаётся от `main` **после** Задачи 1, чтобы в v1 тоже не было Flatpak.

- [ ] **Step 1: Создать ветку (после согласия)**

```bash
git switch -c release/1.x
git push -u origin release/1.x
```

Expected: `branch 'release/1.x' set up to track 'origin/release/1.x'`.

- [ ] **Step 2: Защитить ветку на GitHub (делает пользователь в веб-интерфейсе)**

Settings → Branches → Add rule: `release/*`, запрет force-push и удаления.

---

### Задача 3: Зависимость `semver` и тип `UpdateCheck` (ветка `release/1.x`)

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/git/types.rs:182-186`

- [ ] **Step 1: Добавить crate**

```bash
cd src-tauri && cargo add semver@1
```

Expected: в `[dependencies]` появилась строка `semver = "1..."`.

- [ ] **Step 2: Заменить `UpdateInfo` в `types.rs`**

```rust
#[derive(Debug, serde::Serialize, Clone, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub release_url: String,
}

/// Результат проверки обновлений.
#[derive(Debug, serde::Serialize, Clone, PartialEq, Default)]
pub struct UpdateCheck {
    /// Новейшая версия в той же мажорной линии, что и текущая, — обычный баннер обновления.
    pub update: Option<UpdateInfo>,
    /// Новейшая стабильная версия следующей мажорной линии — разовое уведомление «Доступна версия N».
    pub next_major: Option<UpdateInfo>,
}
```

- [ ] **Step 3: Проверить сборку**

Run: `cd src-tauri && cargo check`
Expected: успешно; предупреждение о неиспользуемом `UpdateCheck` допустимо до Задачи 5.

---

### Задача 4: Чистая функция `pick_update` (TDD)

**Files:**
- Create: `src-tauri/src/update.rs`
- Modify: `src-tauri/src/main.rs` (рядом с `mod settings;`)

- [ ] **Step 1: Создать модуль с тестами и заглушкой**

`src-tauri/src/update.rs`:

```rust
//! Выбор обновления по списку релизов GitHub.
//!
//! v1 и v2 живут параллельно, поэтому `releases/latest` использовать нельзя:
//! после выхода v2 он предложит пользователям v1 «обновиться» на v2, а патч v1,
//! выпущенный позже, станет «latest» и скроет обновления v2.

use semver::Version;

use crate::git::types::{UpdateCheck, UpdateInfo};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReleaseEntry {
    pub tag_name: String,
    pub html_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
}

pub fn pick_update(_releases: &[ReleaseEntry], _current: &str) -> UpdateCheck {
    UpdateCheck::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(tag: &str, prerelease: bool) -> ReleaseEntry {
        ReleaseEntry {
            tag_name: tag.into(),
            html_url: format!("https://github.com/avv7bc/gitstream/releases/tag/{tag}"),
            draft: false,
            prerelease,
        }
    }

    #[test]
    fn v1_user_gets_newest_v1_and_separate_v2_notice() {
        let r = [rel("v2.0.0", false), rel("v1.1.35", false), rel("v1.1.34", false)];
        let c = pick_update(&r, "1.1.33");
        assert_eq!(c.update.unwrap().version, "1.1.35");
        assert_eq!(c.next_major.unwrap().version, "2.0.0");
    }

    #[test]
    fn v2_user_ignores_v1_patch_published_later() {
        let r = [rel("v1.1.40", false), rel("v2.0.1", false), rel("v2.0.0", false)];
        let c = pick_update(&r, "2.0.0");
        assert_eq!(c.update.unwrap().version, "2.0.1");
        assert!(c.next_major.is_none());
    }

    #[test]
    fn stable_user_skips_prereleases_of_any_line() {
        let r = [rel("v2.0.0-alpha.1", true), rel("v1.1.35-rc.1", true), rel("v1.1.34", false)];
        let c = pick_update(&r, "1.1.33");
        assert_eq!(c.update.unwrap().version, "1.1.34");
        assert!(c.next_major.is_none());
    }

    #[test]
    fn prerelease_user_gets_next_prerelease_then_final() {
        let r = [rel("v2.0.0-alpha.2", true)];
        assert_eq!(pick_update(&r, "2.0.0-alpha.1").update.unwrap().version, "2.0.0-alpha.2");

        let r = [rel("v2.0.0", false), rel("v2.0.0-alpha.2", true)];
        assert_eq!(pick_update(&r, "2.0.0-alpha.2").update.unwrap().version, "2.0.0");
    }

    #[test]
    fn up_to_date_returns_nothing() {
        let r = [rel("v1.1.33", false), rel("v1.1.32", false)];
        assert_eq!(pick_update(&r, "1.1.33"), UpdateCheck::default());
    }

    #[test]
    fn drafts_unparsable_tags_and_empty_urls_are_ignored() {
        let mut draft = rel("v1.1.99", false);
        draft.draft = true;
        let mut no_url = rel("v1.1.98", false);
        no_url.html_url.clear();
        let r = [draft, no_url, rel("nightly", false)];
        assert_eq!(pick_update(&r, "1.1.33"), UpdateCheck::default());
    }

    #[test]
    fn unparsable_current_version_returns_nothing() {
        let r = [rel("v1.1.34", false)];
        assert_eq!(pick_update(&r, "dev"), UpdateCheck::default());
    }
}
```

В `src-tauri/src/main.rs` после `mod settings;` добавить строку:

```rust
mod update;
```

- [ ] **Step 2: Убедиться, что тесты падают**

Run: `cd src-tauri && cargo test update::tests`
Expected: FAIL в `v1_user_gets_newest_v1_and_separate_v2_notice`, `v2_user_ignores_v1_patch_published_later`, `stable_user_skips_prereleases_of_any_line`, `prerelease_user_gets_next_prerelease_then_final` (`called Option::unwrap() on a None value`); остальные три проходят.

- [ ] **Step 3: Реализовать `pick_update`**

Заменить заглушку:

```rust
fn parse(tag: &str) -> Option<Version> {
    Version::parse(tag.trim_start_matches('v')).ok()
}

pub fn pick_update(releases: &[ReleaseEntry], current: &str) -> UpdateCheck {
    let Some(cur) = parse(current) else {
        return UpdateCheck::default();
    };
    let on_prerelease = !cur.pre.is_empty();

    let mut best_same: Option<(Version, &ReleaseEntry)> = None;
    let mut best_next: Option<(Version, &ReleaseEntry)> = None;

    for r in releases.iter().filter(|r| !r.draft && !r.html_url.is_empty()) {
        let Some(v) = parse(&r.tag_name) else { continue };
        let is_pre = r.prerelease || !v.pre.is_empty();

        if v.major == cur.major {
            // Prerelease предлагаем только тем, кто сам сидит на prerelease.
            if is_pre && !on_prerelease {
                continue;
            }
            if v > cur && best_same.as_ref().is_none_or(|(b, _)| v > *b) {
                best_same = Some((v, r));
            }
        } else if v.major > cur.major
            && !is_pre
            && best_next.as_ref().is_none_or(|(b, _)| v > *b)
        {
            best_next = Some((v, r));
        }
    }

    let info = |(v, r): (Version, &ReleaseEntry)| UpdateInfo {
        version: v.to_string(),
        release_url: r.html_url.clone(),
    };
    UpdateCheck {
        update: best_same.map(info),
        next_major: best_next.map(info),
    }
}
```

- [ ] **Step 4: Убедиться, что тесты проходят**

Run: `cd src-tauri && cargo test update::tests`
Expected: `test result: ok. 7 passed`.

- [ ] **Step 5: Commit (после согласия)**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/update.rs src-tauri/src/main.rs src-tauri/src/git/types.rs
git commit -m "feat(update): выбор обновления внутри своей мажорной линии (pick_update)"
```

---

### Задача 5: `check_for_update` через список релизов

**Files:**
- Modify: `src-tauri/src/commands.rs:31-40` (удалить `version_gt`), `:849-893` (команда), `:1008-1041` (удалить `mod update_tests`)

- [ ] **Step 1: Убедиться, что `version_gt` больше нигде не используется**

Run: `grep -rn "version_gt" src-tauri/src`
Expected: только определение (стр. 31), вызов в `check_for_update` (стр. 886) и `mod update_tests`.

- [ ] **Step 2: Удалить функцию `version_gt` (стр. 31-40) и модуль `update_tests` целиком (стр. 1008-1041)**

Их покрытие заменяют тесты `update::tests`.

- [ ] **Step 3: Заменить команду**

```rust
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateCheck, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("gitstream-app")
        .build()
        .map_err(|e| e.to_string())?;

    // Список, а не releases/latest: v1 и v2 выпускаются параллельно (см. update.rs).
    let resp = match client
        .get("https://api.github.com/repos/avv7bc/gitstream/releases?per_page=50")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Ok(UpdateCheck::default()),
    };

    if !resp.status().is_success() {
        return Ok(UpdateCheck::default());
    }

    let releases: Vec<crate::update::ReleaseEntry> = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Ok(UpdateCheck::default()),
    };

    let current = app.package_info().version.to_string();
    Ok(crate::update::pick_update(&releases, &current))
}
```

`UpdateCheck` импортируется через уже существующий `use crate::git::{..., types::*};` (стр. 11).

- [ ] **Step 4: Проверить**

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: без предупреждений; все тесты зелёные (тестов `update_tests::*` больше нет, есть `update::tests::*`).

- [ ] **Step 5: Commit (после согласия)**

```bash
git add src-tauri/src/commands.rs
git commit -m "fix(update): проверка обновлений по списку релизов, а не releases/latest"
```

---

### Задача 6: Фронтенд — `useUpdate` с уведомлением о новой мажорной версии (TDD)

**Files:**
- Modify: `src/types/index.ts:191-194`
- Modify: `src/composables/useUpdate.ts`
- Create: `src/composables/useUpdate.test.ts`

- [ ] **Step 1: Тип в `src/types/index.ts`** (сразу после `UpdateInfo`)

```ts
export interface UpdateCheck {
  update: UpdateInfo | null;
  next_major: UpdateInfo | null;
}
```

- [ ] **Step 2: Написать падающий тест**

`src/composables/useUpdate.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { UpdateCheck } from "@/types";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const v2 = { version: "2.0.0", release_url: "https://github.com/avv7bc/gitstream/releases/tag/v2.0.0" };
const v1 = { version: "1.1.35", release_url: "https://github.com/avv7bc/gitstream/releases/tag/v1.1.35" };

beforeEach(() => {
  vi.resetModules();
  invoke.mockReset();
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => { values.set(key, value); },
    removeItem: (key: string) => { values.delete(key); },
  });
});

async function load() {
  const mod = await import("./useUpdate");
  return mod.useUpdate();
}

describe("useUpdate", () => {
  it("раскладывает обновление своей линии и следующую мажорную версию", async () => {
    invoke.mockResolvedValue({ update: v1, next_major: v2 } satisfies UpdateCheck);
    const u = await load();
    await u.checkForUpdate();
    expect(u.updateInfo.value).toEqual(v1);
    expect(u.nextMajorInfo.value).toEqual(v2);
  });

  it("после «не показывать» мажорная версия не появляется при следующей проверке", async () => {
    invoke.mockResolvedValue({ update: null, next_major: v2 } satisfies UpdateCheck);
    const u = await load();
    await u.checkForUpdate();
    u.dismissNextMajor();
    expect(u.nextMajorInfo.value).toBeNull();
    await u.checkForUpdate();
    expect(u.nextMajorInfo.value).toBeNull();
  });

  it("отказ от v2 не скрывает v3", async () => {
    invoke.mockResolvedValueOnce({ update: null, next_major: v2 } satisfies UpdateCheck);
    const u = await load();
    await u.checkForUpdate();
    u.dismissNextMajor();
    const v3 = { version: "3.0.0", release_url: "https://github.com/avv7bc/gitstream/releases/tag/v3.0.0" };
    invoke.mockResolvedValueOnce({ update: null, next_major: v3 } satisfies UpdateCheck);
    await u.checkForUpdate();
    expect(u.nextMajorInfo.value).toEqual(v3);
  });

  it("ошибка сети не трогает состояние", async () => {
    invoke.mockRejectedValue(new Error("offline"));
    const u = await load();
    await u.checkForUpdate();
    expect(u.updateInfo.value).toBeNull();
    expect(u.nextMajorInfo.value).toBeNull();
  });
});
```

- [ ] **Step 3: Убедиться, что тест падает**

Run: `npx vitest run src/composables/useUpdate.test.ts`
Expected: FAIL — `nextMajorInfo` / `dismissNextMajor` не определены.

- [ ] **Step 4: Реализовать `src/composables/useUpdate.ts`**

```ts
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateCheck, UpdateInfo } from "@/types";

// Мажорная версия, о которой пользователь попросил больше не напоминать.
const DISMISSED_MAJOR_KEY = "gitstream:dismissed-next-major";

const updateInfo = ref<UpdateInfo | null>(null);
const nextMajorInfo = ref<UpdateInfo | null>(null);

function majorOf(version: string): string {
  return version.split(".")[0];
}

function readDismissedMajor(): string | null {
  try {
    return localStorage.getItem(DISMISSED_MAJOR_KEY);
  } catch {
    return null;
  }
}

export function useUpdate() {
  async function checkForUpdate() {
    try {
      const res = await invoke<UpdateCheck>("check_for_update");
      updateInfo.value = res.update;
      const next = res.next_major;
      nextMajorInfo.value = next && readDismissedMajor() !== majorOf(next.version) ? next : null;
    } catch {
      // silent fail — no network or API error
    }
  }

  function dismissNextMajor() {
    const next = nextMajorInfo.value;
    if (next) {
      try {
        localStorage.setItem(DISMISSED_MAJOR_KEY, majorOf(next.version));
      } catch {
        // storage недоступен — просто скрываем до перезапуска
      }
    }
    nextMajorInfo.value = null;
  }

  return { updateInfo, nextMajorInfo, checkForUpdate, dismissNextMajor };
}
```

- [ ] **Step 5: Убедиться, что тест проходит**

Run: `npx vitest run src/composables/useUpdate.test.ts`
Expected: `4 passed`.

- [ ] **Step 6: Commit (после согласия)**

```bash
git add src/types/index.ts src/composables/useUpdate.ts src/composables/useUpdate.test.ts
git commit -m "feat(update): отдельное уведомление о новой мажорной версии с «не показывать»"
```

---

### Задача 7: `UpdateBanner` — два вида и строки через i18n

**Files:**
- Modify: `src/locales/en.ts:149-151`, `src/locales/ru.ts:154-156`
- Modify: `src/components/UpdateBanner.vue`
- Modify: `src/App.vue:711-715`

- [ ] **Step 1: Ключи локалей**

В обоих файлах заменить блок `update` (русская локаль по решению пользователя использует английский UI вне `settings`, поэтому текст одинаковый):

```ts
  update: {
    download: "Download",
    available: "GitStream update available (v{version})",
    nextMajor: "GitStream {major} is available",
    nextMajorDetail: "New interface and features. Version {current} keeps receiving fixes.",
    changelog: "Changelog ›",
    dismiss: "Dismiss",
    dontShow: "Don't show again",
  },
```

- [ ] **Step 2: `UpdateBanner.vue` — script и template**

```vue
<script setup lang="ts">
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "@/types";
import { useI18n } from "@/composables/useI18n";

const props = withDefaults(defineProps<{ info: UpdateInfo; kind?: "update" | "nextMajor" }>(), {
  kind: "update",
});
const emit = defineEmits<{ dismiss: [] }>();
const { i18n } = useI18n();

const currentMajor = __APP_VERSION__.split(".")[0];

const title = computed(() =>
  props.kind === "update"
    ? i18n.value.update.available.replace("{version}", props.info.version)
    : i18n.value.update.nextMajor.replace("{major}", props.info.version.split(".")[0]),
);

async function openUrl(url: string) {
  try {
    await invoke("open_url", { url });
  } catch {
    // fallback: window.open может не работать в Tauri, но как запасной вариант
    window.open(url, "_blank");
  }
}

async function download() {
  await openUrl(props.info.release_url);
  emit("dismiss");
}
</script>

<template>
  <div class="update-banner">
    <div class="update-icon">↑</div>
    <div class="update-content">
      <div class="update-title">{{ title }}</div>
      <div v-if="kind === 'nextMajor'" class="update-detail">
        {{ i18n.update.nextMajorDetail.replace("{current}", currentMajor) }}
      </div>
      <button class="update-changelog" @click="openUrl(info.release_url)">
        {{ i18n.update.changelog }}
      </button>
    </div>
    <div class="update-actions">
      <button class="btn btn-primary" @click="download">{{ i18n.update.download }}</button>
      <button class="btn btn-secondary" @click="emit('dismiss')">
        {{ kind === "nextMajor" ? i18n.update.dontShow : i18n.update.dismiss }}
      </button>
    </div>
  </div>
</template>
```

В `<style scoped>` добавить:

```css
.update-detail {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  margin-bottom: 4px;
  line-height: 1.35;
}
```

`__APP_VERSION__` уже объявлен глобально через Vite `define` (заголовок окна использует его же); если `vue-tsc` ругается — проверить `src/env.d.ts` на `declare const __APP_VERSION__: string;`.

- [ ] **Step 3: `App.vue` — показать второй баннер, если нет обычного обновления**

В `script setup` (стр. 54):

```ts
const { updateInfo, nextMajorInfo, checkForUpdate, dismissNextMajor } = useUpdate();
```

В шаблоне (стр. 711-715) заменить на:

```vue
    <UpdateBanner
      v-if="updateInfo"
      :info="updateInfo"
      @dismiss="updateInfo = null"
    />
    <UpdateBanner
      v-else-if="nextMajorInfo"
      :info="nextMajorInfo"
      kind="nextMajor"
      @dismiss="dismissNextMajor"
    />
```

- [ ] **Step 4: Проверить**

Run: `npx vue-tsc --noEmit && npm test`
Expected: без ошибок типов, все тесты зелёные.

- [ ] **Step 5: Ручная проверка (без сети в GitHub)**

Код не трогаем — в DevTools Tauri (`npm run tauri dev`, Ctrl+Shift+I) выполнить:

```js
const u = (await import("/src/composables/useUpdate.ts")).useUpdate();
u.nextMajorInfo.value = { version: "2.0.0", release_url: "https://github.com/avv7bc/gitstream/releases" };
```

Expected: справа внизу баннер «GitStream 2 is available», строка про продолжение исправлений для версии 1, кнопки «Download» и «Don't show again».

- [ ] **Step 6: Commit (после согласия)**

```bash
git add src/locales/en.ts src/locales/ru.ts src/components/UpdateBanner.vue src/App.vue
git commit -m "feat(update): баннер новой мажорной версии; строки баннера через i18n"
```

---

### Задача 8: Тест совместимости настроек

v1 и v2 читают один `settings.json`. v2 будет добавлять поля; v1 не должна считать такой файл битым и сбрасывать его (сейчас битый JSON → дефолты + перезапись, `settings.rs:69-86`).

**Files:**
- Modify: `src-tauri/src/settings.rs` (тест-модуль, рядом с `corrupt_json_returns_defaults_and_rewrites`)

- [ ] **Step 1: Добавить тест**

```rust
    #[test]
    fn unknown_fields_from_newer_version_are_ignored_not_reset() {
        // v2 пишет поля, которых v1 не знает; v1 должна прочитать известные
        // поля как есть, а не принять файл за битый и сбросить в дефолты.
        let path = temp_path();
        fs::write(
            &path,
            r#"{"network_timeout_secs":25,"language":"ru","ui_preset":"graphite","density":"compact"}"#,
        )
        .unwrap();
        let s = read_settings_at(&path);
        assert_eq!(s.network_timeout_secs, 25);
        assert_eq!(s.language, "ru");
        let _ = fs::remove_file(&path);
    }
```

(`temp_path()` уже есть в тест-модуле, стр. 118.)

- [ ] **Step 2: Запустить**

Run: `cd src-tauri && cargo test settings::`
Expected: PASS сразу — serde по умолчанию игнорирует неизвестные поля. Тест фиксирует поведение: если кто-то добавит `#[serde(deny_unknown_fields)]`, он упадёт.

- [ ] **Step 3: Commit (после согласия)**

```bash
git add src-tauri/src/settings.rs
git commit -m "test(settings): поля новой версии не сбрасывают настройки старой"
```

---

### Задача 9: CI и релизный workflow

**Files:**
- Modify: `.github/workflows/ci.yml:3-6`
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: `ci.yml` — запуск на ветках релизов**

```yaml
on:
  push:
    branches: [main, 'release/**']
  pull_request:
```

- [ ] **Step 2: `release.yml` — prerelease-теги**

Блок `on`:

```yaml
on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'
      - 'v[0-9]+.[0-9]+.[0-9]+-*'
```

В шаге `tauri-apps/tauri-action@v0` заменить `prerelease: false` на:

```yaml
          prerelease: ${{ contains(github.ref_name, '-') }}
```

- [ ] **Step 3: `release.yml` — снять «latest» с релизов старой линии**

После job `release` добавить job:

```yaml
  latest-flag:
    needs: release
    runs-on: ubuntu-22.04
    steps:
      - name: Keep "latest" on the newest major line
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAG: ${{ github.ref_name }}
        run: |
          major="${TAG#v}"; major="${major%%.*}"
          newest=$(gh release list -R "$GITHUB_REPOSITORY" --exclude-pre-releases --exclude-drafts \
                     --limit 100 --json tagName -q '.[].tagName' \
                   | sed -E 's/^v([0-9]+)\..*/\1/' | sort -n | tail -1)
          if [ -n "$newest" ] && [ "$major" -lt "$newest" ]; then
            gh release edit "$TAG" -R "$GITHUB_REPOSITORY" --latest=false
            echo "$TAG: старая линия ($major < $newest), метка latest снята"
          fi
```

Зачем: клиенты ≤ 1.1.33 по-прежнему смотрят `releases/latest`. Пока стабильной v2 нет, `latest` — это новейший v1, как и раньше. После выхода 2.0.0 `latest` остаётся на v2, даже если позже выйдет 1.1.40.

- [ ] **Step 4: Проверить синтаксис**

Run: `npx --yes yaml-lint .github/workflows/ci.yml .github/workflows/release.yml` (или `python3 -c "import yaml,sys;[yaml.safe_load(open(f)) for f in sys.argv[1:]]" .github/workflows/*.yml`)
Expected: без ошибок.

- [ ] **Step 5: Commit (после согласия)**

```bash
git add .github/workflows/ci.yml .github/workflows/release.yml
git commit -m "ci: CI на release/**, prerelease-теги, latest остаётся на новейшей мажорной линии"
```

- [ ] **Step 6: Релиз v1 с новой проверкой обновлений (1.1.N) — только по команде пользователя**

Поднять patch-версию в `package.json`, `src-tauri/Cargo.toml` и `src-tauri/tauri.conf.json` до следующего patch (`1.1.N`) (Cargo.lock обновит `cargo check`), коммит, тег `v1.1.N`, push ветки и тега. Этот релиз должен выйти **до** любого релиза v2, чтобы пользователи v1 успели получить новую логику обновлений.

---

### Задача 10: Перенести исправления в `main`

- [ ] **Step 1: Cherry-pick коммитов Задач 4–9 (после согласия)**

```bash
git switch main
git log --reverse --format=%h release/1.x ^main   # список коммитов для переноса
git cherry-pick -x <sha1> <sha2> ...
```

Коммит с подъёмом версии **не переносить**.

- [ ] **Step 2: Проверить**

Run: `npx vue-tsc --noEmit && npm test && (cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test)`
Expected: всё зелёное.

---

### Задача 11: Документация веток в `main` (и `release/1.x`)

**Files:**
- Modify: `CLAUDE.md` (после раздела «Рабочие правила»)

- [ ] **Step 1: Добавить раздел**

```markdown
## Версии и ветки

| Ветка | Версия | Что туда идёт |
|---|---|---|
| `main` | **2.x** (разработка; `2.0.0-alpha.N` → `beta` → `rc` → `2.0.0`) | Редизайн «Графит», новые функции (см. `docs/ROADMAP-2.0.md`, `docs/SPEC-redesign-2026.md`) |
| `release/1.x` | **1.x** (поддержка) | Только исправления и мелкие бэкпорты, теги `v1.1.N` |

- Баг есть в обеих версиях → исправление в `release/1.x`, затем `git cherry-pick -x` в `main`. Слияние `release/1.x` → `main` не делаем: после редизайна UI-файлы расходятся.
- Баг только в v2 → только `main`.
- Теги: `vX.Y.Z` — стабильный релиз; `vX.Y.Z-alpha.N` / `-beta.N` / `-rc.N` — prerelease (GitHub помечает автоматически).
- Проверка обновлений (`src-tauri/src/update.rs`) предлагает версии только своей мажорной линии; о следующей мажорной — отдельный баннер.
- `settings.json` общий для v1 и v2: в v2 поля только **добавляем**, тип и смысл существующих не меняем (тест `unknown_fields_from_newer_version_are_ignored_not_reset`).
- Поддержка v1: исправления 6 месяцев после выхода 2.0.0, затем ещё 6 месяцев только критичные (порча данных, безопасность).
```

- [ ] **Step 2: Commit в `main`, затем `git cherry-pick -x` в `release/1.x` (после согласия)**

```bash
git add CLAUDE.md
git commit -m "docs: версии и ветки v1/v2"
```

---

### Задача 12: Перенумеровать релизы в спеке (`main`)

**Files:**
- Modify: `docs/SPEC-redesign-2026.md` §5, таблица «Порядок внедрения»

- [ ] **Step 1: Заменить колонку «Релиз»** по `docs/ROADMAP-2.0.md`:

| Было | Стало |
|---|---|
| 1.2.0 + 1.3.0 + 1.4.0 | 2.0.0 |
| 1.5.0 | 2.1.0 |
| 1.6.0 | 2.2.0 |
| 1.7.0 | 2.3.0 |

- [ ] **Step 2: Commit (после согласия)**

```bash
git add docs/SPEC-redesign-2026.md
git commit -m "docs(spec): релизы v2 — 2.0–2.3"
```

---

## Самопроверка плана

- Разделение веток → Задачи 1, 2, 10, 11.
- Пользователи v1 не получают баннер «обнови до v2» как обновление → Задачи 4, 5, 6, 7 (тест `v1_user_gets_newest_v1_and_separate_v2_notice`).
- Патч v1 после v2 не ломает обновления v2 → Задача 4 (тест `v2_user_ignores_v1_patch_published_later`) и Задача 9, Step 3 (для старых клиентов).
- Prerelease v2 не уходят стабильным пользователям → Задача 4 (тест `stable_user_skips_prereleases_of_any_line`), Задача 9, Step 2.
- Общие настройки → Задача 8 + правило в CLAUDE.md.
- Flatpak удалён до создания `release/1.x` → Задачи 1–2.
