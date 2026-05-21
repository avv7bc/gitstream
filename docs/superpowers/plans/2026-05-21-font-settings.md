# Font Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 4 font settings (Workbench Font Family, Workbench Font Size, Editor Font Family, Editor Font Size) to the Settings dialog, styled like VS Code, with live CSS preview and persistence via Tauri.

**Architecture:** Settings are persisted via Rust `AppSettings` in `settings.json`. The frontend applies font settings as CSS custom properties on `:root` immediately on load and on every change (live preview). A shared `scheduleSave()` debounce in `useSettings.ts` saves all settings together to avoid partial overwrites. Diff components switch to a new `--font-size-diff` variable so the editor font size controls them independently.

**Tech Stack:** Rust (serde_json), TypeScript, Vue 3 Composition API, CSS custom properties, Tauri IPC invoke.

---

## Files Modified

| File | Change |
|---|---|
| `src-tauri/src/settings.rs` | Add 4 fields to `AppSettings`, update `Default`, add Rust tests |
| `src/composables/useSettings.ts` | Add 4 refs, `applyCssFonts()`, refactor persist to `scheduleSave()` |
| `src/styles/main.css` | Add `--font-size-diff: 13px` to `:root` |
| `src/components/DiffPanel.vue` | `.diff-panel` font-size: `--font-size-sm` → `--font-size-diff` |
| `src/components/DiffLinesPair.vue` | `.diff-line-container` font-size: `--font-size-sm` → `--font-size-diff` |
| `src/components/SideBySideDiffView.vue` | `.diff-side` font-size: `--font-size-sm` → `--font-size-diff` |
| `src/components/dialogs/SettingsDialog.vue` | Add 4 settings items + text/number controls + CSS |

---

### Task 1: Rust — расширить AppSettings

**Files:**
- Modify: `src-tauri/src/settings.rs`

- [ ] **Шаг 1: Добавить поля в `AppSettings` и обновить `Default`**

Заменить весь блок от строки 6 до строки 24 в `settings.rs`:

```rust
const DEFAULT_NETWORK_TIMEOUT_SECS: u64 = 10;
const DEFAULT_WORKBENCH_FONT_FAMILY: &str =
    "Ubuntu, -apple-system, BlinkMacSystemFont, sans-serif";
const DEFAULT_WORKBENCH_FONT_SIZE: u8 = 15;
const DEFAULT_EDITOR_FONT_FAMILY: &str = "Ubuntu Mono, Courier New, monospace";
const DEFAULT_EDITOR_FONT_SIZE: u8 = 13;

fn default_network_timeout_secs() -> u64 { DEFAULT_NETWORK_TIMEOUT_SECS }
fn default_workbench_font_family() -> String { DEFAULT_WORKBENCH_FONT_FAMILY.to_string() }
fn default_workbench_font_size() -> u8 { DEFAULT_WORKBENCH_FONT_SIZE }
fn default_editor_font_family() -> String { DEFAULT_EDITOR_FONT_FAMILY.to_string() }
fn default_editor_font_size() -> u8 { DEFAULT_EDITOR_FONT_SIZE }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppSettings {
    #[serde(default = "default_network_timeout_secs")]
    pub network_timeout_secs: u64,
    #[serde(default = "default_workbench_font_family")]
    pub workbench_font_family: String,
    #[serde(default = "default_workbench_font_size")]
    pub workbench_font_size: u8,
    #[serde(default = "default_editor_font_family")]
    pub editor_font_family: String,
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: u8,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            network_timeout_secs: DEFAULT_NETWORK_TIMEOUT_SECS,
            workbench_font_family: DEFAULT_WORKBENCH_FONT_FAMILY.to_string(),
            workbench_font_size: DEFAULT_WORKBENCH_FONT_SIZE,
            editor_font_family: DEFAULT_EDITOR_FONT_FAMILY.to_string(),
            editor_font_size: DEFAULT_EDITOR_FONT_SIZE,
        }
    }
}
```

- [ ] **Шаг 2: Добавить тесты для новых полей**

Добавить в блок `#[cfg(test)] mod tests` после существующего теста `corrupt_json_returns_defaults_and_rewrites`:

```rust
#[test]
fn font_fields_default_correctly() {
    let s = AppSettings::default();
    assert_eq!(s.workbench_font_family, "Ubuntu, -apple-system, BlinkMacSystemFont, sans-serif");
    assert_eq!(s.workbench_font_size, 15);
    assert_eq!(s.editor_font_family, "Ubuntu Mono, Courier New, monospace");
    assert_eq!(s.editor_font_size, 13);
}

#[test]
fn old_settings_json_without_font_fields_uses_defaults() {
    let p = temp_path();
    // JSON без новых полей — как у существующих пользователей
    fs::write(&p, r#"{"network_timeout_secs":30}"#).unwrap();
    let s = read_settings_at(&p);
    assert_eq!(s.network_timeout_secs, 30);
    assert_eq!(s.workbench_font_size, 15);
    assert_eq!(s.editor_font_size, 13);
    fs::remove_file(&p).ok();
}

#[test]
fn font_fields_round_trip() {
    let p = temp_path();
    let s = AppSettings {
        network_timeout_secs: 10,
        workbench_font_family: "Segoe UI, sans-serif".to_string(),
        workbench_font_size: 14,
        editor_font_family: "Fira Code, monospace".to_string(),
        editor_font_size: 16,
    };
    write_settings_at(&p, &s).unwrap();
    let back = read_settings_at(&p);
    assert_eq!(back, s);
    fs::remove_file(&p).ok();
}
```

- [ ] **Шаг 3: Запустить тесты**

```bash
cd /home/avv/projects/gitstream && cargo test -p gitstream-lib -- settings 2>&1 | tail -20
```

Если пакет называется иначе:
```bash
cd /home/avv/projects/gitstream/src-tauri && cargo test -- settings 2>&1 | tail -20
```

Ожидаемый результат: все тесты `settings::tests::*` — PASSED.

- [ ] **Шаг 4: Коммит**

```bash
cd /home/avv/projects/gitstream
git add src-tauri/src/settings.rs
git commit -m "feat(settings): add font fields to AppSettings"
```

---

### Task 2: Frontend composable — useSettings.ts

**Files:**
- Modify: `src/composables/useSettings.ts`

- [ ] **Шаг 1: Заменить содержимое файла**

```typescript
import { ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";

interface AppSettings {
  network_timeout_secs: number;
  workbench_font_family: string;
  workbench_font_size: number;
  editor_font_family: string;
  editor_font_size: number;
}

const TIMEOUT_OPTIONS = [5, 10, 30, 60];
const DEFAULT_TIMEOUT = 10;
const DEFAULT_WB_FONT_FAMILY = "Ubuntu, -apple-system, BlinkMacSystemFont, sans-serif";
const DEFAULT_WB_FONT_SIZE = 15;
const DEFAULT_ED_FONT_FAMILY = "Ubuntu Mono, Courier New, monospace";
const DEFAULT_ED_FONT_SIZE = 13;

const networkTimeoutSecs = ref<number>(DEFAULT_TIMEOUT);
const workbenchFontFamily = ref<string>(DEFAULT_WB_FONT_FAMILY);
const workbenchFontSize = ref<number>(DEFAULT_WB_FONT_SIZE);
const editorFontFamily = ref<string>(DEFAULT_ED_FONT_FAMILY);
const editorFontSize = ref<number>(DEFAULT_ED_FONT_SIZE);

let loaded = false;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

function clampTimeout(v: number): number {
  if (!Number.isFinite(v)) return DEFAULT_TIMEOUT;
  if (TIMEOUT_OPTIONS.includes(v)) return v;
  return TIMEOUT_OPTIONS.reduce((best, o) =>
    Math.abs(o - v) < Math.abs(best - v) ? o : best
  );
}

function clampFontSize(v: number, min: number, max: number, def: number): number {
  if (!Number.isFinite(v) || v < min || v > max) return def;
  return Math.round(v);
}

function applyCssFonts() {
  const root = document.documentElement.style;
  root.setProperty("--font-sans", workbenchFontFamily.value || DEFAULT_WB_FONT_FAMILY);
  const n = workbenchFontSize.value;
  root.setProperty("--font-size", `${n}px`);
  root.setProperty("--font-size-sm", `${n - 2}px`);
  root.setProperty("--font-size-xs", `${n - 3}px`);
  root.setProperty("--font-mono", editorFontFamily.value || DEFAULT_ED_FONT_FAMILY);
  root.setProperty("--font-size-diff", `${editorFontSize.value}px`);
}

function scheduleSave() {
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    invoke("set_settings", {
      settings: {
        network_timeout_secs: networkTimeoutSecs.value,
        workbench_font_family: workbenchFontFamily.value,
        workbench_font_size: workbenchFontSize.value,
        editor_font_family: editorFontFamily.value,
        editor_font_size: editorFontSize.value,
      },
    }).catch(() => {});
  }, 400);
}

async function loadSettings() {
  if (loaded) return;
  loaded = true;
  try {
    const s = await invoke<AppSettings>("get_settings");
    networkTimeoutSecs.value = clampTimeout(s.network_timeout_secs);
    workbenchFontFamily.value = s.workbench_font_family || DEFAULT_WB_FONT_FAMILY;
    workbenchFontSize.value = clampFontSize(s.workbench_font_size, 11, 20, DEFAULT_WB_FONT_SIZE);
    editorFontFamily.value = s.editor_font_family || DEFAULT_ED_FONT_FAMILY;
    editorFontSize.value = clampFontSize(s.editor_font_size, 10, 24, DEFAULT_ED_FONT_SIZE);
  } catch {
    // defaults already set
  }
  applyCssFonts();
}

watch(networkTimeoutSecs, (v) => {
  const safe = clampTimeout(v);
  if (safe !== v) { networkTimeoutSecs.value = safe; return; }
  scheduleSave();
});

watch(workbenchFontFamily, () => { applyCssFonts(); scheduleSave(); });
watch(workbenchFontSize, (v) => {
  const safe = clampFontSize(v, 11, 20, DEFAULT_WB_FONT_SIZE);
  if (safe !== v) { workbenchFontSize.value = safe; return; }
  applyCssFonts(); scheduleSave();
});
watch(editorFontFamily, () => { applyCssFonts(); scheduleSave(); });
watch(editorFontSize, (v) => {
  const safe = clampFontSize(v, 10, 24, DEFAULT_ED_FONT_SIZE);
  if (safe !== v) { editorFontSize.value = safe; return; }
  applyCssFonts(); scheduleSave();
});

void loadSettings();

export function useSettings() {
  return {
    networkTimeoutSecs,
    workbenchFontFamily,
    workbenchFontSize,
    editorFontFamily,
    editorFontSize,
  };
}
```

- [ ] **Шаг 2: Коммит**

```bash
cd /home/avv/projects/gitstream
git add src/composables/useSettings.ts
git commit -m "feat(settings): add font refs and CSS application in useSettings"
```

---

### Task 3: CSS — добавить переменную --font-size-diff

**Files:**
- Modify: `src/styles/main.css`

- [ ] **Шаг 1: Добавить переменную в блок `:root`**

В `src/styles/main.css` найти строку `--font-size-xs: 12px;` и добавить после неё:

```css
  --font-size-diff: 13px;
```

Итоговый блок шрифтов в `:root`:
```css
:root {
  --font-mono: "Ubuntu Mono", "Courier New", monospace;
  --font-sans: "Ubuntu", -apple-system, BlinkMacSystemFont, sans-serif;
  --font-size: 15px;
  --font-size-sm: 13px;
  --font-size-xs: 12px;
  --font-size-diff: 13px;
  /* ... остальные переменные */
}
```

- [ ] **Шаг 2: Коммит**

```bash
cd /home/avv/projects/gitstream
git add src/styles/main.css
git commit -m "feat(settings): add --font-size-diff CSS variable"
```

---

### Task 4: Diff-компоненты — переключить на --font-size-diff

**Files:**
- Modify: `src/components/DiffPanel.vue:54`
- Modify: `src/components/DiffLinesPair.vue:67`
- Modify: `src/components/SideBySideDiffView.vue:396`

- [ ] **Шаг 1: DiffPanel.vue**

Строка 54, заменить:
```css
  font-size: var(--font-size-sm);
```
на:
```css
  font-size: var(--font-size-diff);
```

(Только в блоке `.diff-panel` — строка 53-55. Строку 65 `.hunk-header` с `--font-size-xs` не трогать.)

- [ ] **Шаг 2: DiffLinesPair.vue**

Строка 67, в блоке `.diff-line-container` заменить:
```css
  font-size: var(--font-size-sm);
```
на:
```css
  font-size: var(--font-size-diff);
```

- [ ] **Шаг 3: SideBySideDiffView.vue**

Строка 396, в блоке `.diff-side` заменить:
```css
  font-size: var(--font-size-sm);
```
на:
```css
  font-size: var(--font-size-diff);
```

- [ ] **Шаг 4: Коммит**

```bash
cd /home/avv/projects/gitstream
git add src/components/DiffPanel.vue src/components/DiffLinesPair.vue src/components/SideBySideDiffView.vue
git commit -m "feat(settings): switch diff components to --font-size-diff"
```

---

### Task 5: SettingsDialog — добавить 4 новых параметра

**Files:**
- Modify: `src/components/dialogs/SettingsDialog.vue`

- [ ] **Шаг 1: Импортировать новые refs из useSettings**

В блоке `<script setup>` найти:
```ts
const { networkTimeoutSecs } = useSettings();
```
Заменить на:
```ts
const { networkTimeoutSecs, workbenchFontFamily, workbenchFontSize, editorFontFamily, editorFontSize } = useSettings();
```

- [ ] **Шаг 2: Добавить 4 новых элемента в массив `settings`**

Найти в массиве `settings` элемент с `id: "network-timeout"` и добавить после него 4 новых элемента:

```ts
const settings: SettingItem[] = [
  {
    id: "color-theme",
    category: "appearance",
    label: "Workbench: Color Theme",
    description: "Задаёт цветовую тему интерфейса. Тема «System» подстраивается под настройки операционной системы.",
  },
  {
    id: "workbench-font-family",
    category: "appearance",
    label: "Workbench: Font Family",
    description: "Шрифт интерфейса: панели, тулбар, диалоги. Задаётся как CSS font-family. Пример: «Segoe UI, sans-serif».",
  },
  {
    id: "workbench-font-size",
    category: "appearance",
    label: "Workbench: Font Size",
    description: "Базовый размер шрифта интерфейса в пикселях (11–20). Пропорционально масштабирует все уровни шрифта UI.",
  },
  {
    id: "editor-font-family",
    category: "appearance",
    label: "Editor: Font Family",
    description: "Шрифт diff-вьюера. Рекомендуется моноширинный. Пример: «Fira Code, monospace».",
  },
  {
    id: "editor-font-size",
    category: "appearance",
    label: "Editor: Font Size",
    description: "Размер шрифта diff-вьюера в пикселях (10–24).",
  },
  {
    id: "network-timeout",
    category: "network",
    label: "Network: Timeout (сек)",
    description:
      "Максимальное время сетевой git-операции (fetch, pull, push, clone). По истечении операция прерывается, а зависший процесс git принудительно завершается.",
  },
];
```

- [ ] **Шаг 3: Добавить контролы в template**

В `<template>`, после блока `v-if="s.id === 'color-theme'"`, добавить 4 новых `v-if`-блока:

```html
<div v-if="s.id === 'workbench-font-family'" class="vs-setting-control">
  <input
    v-model="workbenchFontFamily"
    type="text"
    class="vs-input"
    placeholder="Ubuntu, -apple-system, sans-serif"
  />
</div>
<div v-if="s.id === 'workbench-font-size'" class="vs-setting-control">
  <input
    v-model.number="workbenchFontSize"
    type="number"
    class="vs-number"
    min="11"
    max="20"
  />
</div>
<div v-if="s.id === 'editor-font-family'" class="vs-setting-control">
  <input
    v-model="editorFontFamily"
    type="text"
    class="vs-input"
    placeholder="Ubuntu Mono, Courier New, monospace"
  />
</div>
<div v-if="s.id === 'editor-font-size'" class="vs-setting-control">
  <input
    v-model.number="editorFontSize"
    type="number"
    class="vs-number"
    min="10"
    max="24"
  />
</div>
```

- [ ] **Шаг 4: Добавить CSS для новых контролов**

В `<style scoped>`, после блока `.vs-select option { ... }`, добавить:

```css
.vs-input {
  width: 320px;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  padding: 6px 10px;
  font-size: var(--font-size-sm);
  border-radius: var(--radius);
  outline: none;
  font-family: var(--font-sans);
}
.vs-input:focus {
  border-color: var(--accent);
}

.vs-number {
  width: 100px;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  padding: 6px 10px;
  font-size: var(--font-size-sm);
  border-radius: var(--radius);
  outline: none;
  font-family: var(--font-sans);
  text-align: center;
}
.vs-number:focus {
  border-color: var(--accent);
}
```

- [ ] **Шаг 5: Коммит**

```bash
cd /home/avv/projects/gitstream
git add src/components/dialogs/SettingsDialog.vue
git commit -m "feat(settings): add font family and font size controls to SettingsDialog"
```

---

### Task 6: Финальная сборка и проверка

- [ ] **Шаг 1: Запустить Rust-тесты**

```bash
cd /home/avv/projects/gitstream/src-tauri && cargo test -- settings 2>&1 | tail -30
```

Ожидаемый результат: все 6 тестов `settings::tests::*` PASSED.

- [ ] **Шаг 2: Запустить TypeScript-компиляцию**

```bash
cd /home/avv/projects/gitstream && npx vue-tsc --noEmit 2>&1 | tail -20
```

Ожидаемый результат: no errors.

- [ ] **Шаг 3: Запустить dev-сборку для финальной проверки**

```bash
cd /home/avv/projects/gitstream && npm run tauri dev 2>&1 &
```

Проверить вручную:
- Открыть Settings → Внешний вид
- Изменить Workbench Font Size → шрифт панелей меняется мгновенно
- Изменить Editor Font Family → шрифт diff-вьюера меняется
- Закрыть и переоткрыть приложение → настройки сохранились
