# Commit Dialog с выбором файлов — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Переписать CommitDialog — добавить таблицу файлов с чекбоксами, переключатель Staged/Local Changes и реальный Commit & Push.

**Architecture:** Один SFC-компонент (`CommitDialog.vue`) с локальным состоянием (режим + Set отмеченных путей). Использует существующие composables `useFiles`, `useCommit`, `useRemote`, `useBranches`. Backend не трогаем.

**Tech Stack:** Vue 3 Composition API, TypeScript. Никаких новых зависимостей.

**Spec:** [docs/superpowers/specs/2026-04-10-commit-dialog-file-list-design.md](../specs/2026-04-10-commit-dialog-file-list-design.md)

---

## File Structure

**Modify:**
- `src/components/dialogs/CommitDialog.vue` — полная замена script + template + styles.

**Read-only (используем существующее API):**
- `src/composables/useFiles.ts` — `files`, `stageFiles`, `unstageFiles`, `refresh`
- `src/composables/useCommit.ts` — `commit(message, amend)`
- `src/composables/useRemote.ts` — `push(remote, force)`
- `src/composables/useBranches.ts` — `remotes` ref
- `src/types/index.ts` — `FileStatus`, `StagedState`

**Не создаём новых файлов.** Вся логика — в одном компоненте.

**Tests:** в проекте нет настроенного тест-раннера для Vue (см. package.json). Верификация — ручная + `npm run build` для проверки TS-типов.

---

## Task 1: Базовый скрипт — состояние и computed

**Files:**
- Modify: `src/components/dialogs/CommitDialog.vue` (script блок полностью)

- [ ] **Step 1: Заменить `<script setup>` блок**

Полностью заменить содержимое тега `<script setup lang="ts">` на:

```typescript
import { ref, computed, onMounted } from "vue";
import { useCommit } from "@/composables/useCommit";
import { useFiles } from "@/composables/useFiles";
import { useRemote } from "@/composables/useRemote";
import { useBranches } from "@/composables/useBranches";
import type { FileStatus } from "@/types";

const emit = defineEmits<{ close: [] }>();

const { commit: doCommit } = useCommit();
const { files, stageFiles, unstageFiles, refresh: refreshFiles } = useFiles();
const { push } = useRemote();
const { remotes } = useBranches();

type Mode = "staged" | "local";

const message = ref("");
const amend = ref(false);
const mode = ref<Mode>("local");
const checked = ref<Set<string>>(new Set());
const busy = ref(false);

function isStagedLike(f: FileStatus): boolean {
  return f.staged === "staged" || f.staged === "partial";
}

const stagedFiles = computed(() => files.value.filter(isStagedLike));
const hasStaged = computed(() => stagedFiles.value.length > 0);

const filesInMode = computed(() =>
  mode.value === "staged" ? stagedFiles.value : files.value
);

const totalInMode = computed(() => filesInMode.value.length);

const checkedCount = computed(() => {
  if (mode.value === "staged") return stagedFiles.value.length;
  return filesInMode.value.filter((f) => checked.value.has(f.path)).length;
});

const allChecked = computed(
  () => totalInMode.value > 0 && checkedCount.value === totalInMode.value
);

const firstLineLength = computed(() => (message.value.split("\n")[0] || "").length);
const firstLineClass = computed(() => {
  if (firstLineLength.value > 72) return "error";
  if (firstLineLength.value > 50) return "warning";
  return "ok";
});

const canCommit = computed(
  () => !busy.value && message.value.trim().length > 0 && checkedCount.value > 0
);

function splitPath(path: string): { name: string; dir: string } {
  const idx = path.lastIndexOf("/");
  if (idx === -1) return { name: path, dir: "." };
  return { name: path.slice(idx + 1), dir: path.slice(0, idx) };
}

function toggleFile(path: string) {
  if (mode.value !== "local") return;
  const next = new Set(checked.value);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  checked.value = next;
}

function toggleAll() {
  if (mode.value !== "local") return;
  if (allChecked.value) {
    checked.value = new Set();
  } else {
    checked.value = new Set(filesInMode.value.map((f) => f.path));
  }
}

function setMode(next: Mode) {
  if (next === "staged" && !hasStaged.value) return;
  mode.value = next;
  if (next === "local") {
    checked.value = new Set(stagedFiles.value.map((f) => f.path));
  }
}

onMounted(() => {
  if (hasStaged.value) {
    mode.value = "staged";
  } else {
    mode.value = "local";
    checked.value = new Set();
  }
});

async function applyStagingForLocalMode() {
  const checkedPaths = filesInMode.value
    .filter((f) => checked.value.has(f.path))
    .map((f) => f.path);
  const uncheckedStaged = files.value
    .filter((f) => isStagedLike(f) && !checked.value.has(f.path))
    .map((f) => f.path);

  if (uncheckedStaged.length > 0) {
    await unstageFiles(uncheckedStaged);
  }
  if (checkedPaths.length > 0) {
    await stageFiles(checkedPaths);
  }
}

async function handleCommit(alsoPush: boolean) {
  if (!canCommit.value) return;
  busy.value = true;
  try {
    if (mode.value === "local") {
      await applyStagingForLocalMode();
    }
    await doCommit(message.value, amend.value);
    if (alsoPush) {
      const remote = remotes.value[0] ?? "origin";
      await push(remote, false);
    }
    await refreshFiles();
    emit("close");
  } finally {
    busy.value = false;
  }
}
</script>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/dialogs/CommitDialog.vue
git commit -m "feat(commit-dialog): состояние и логика выбора файлов"
```

---

## Task 2: Template — шапка, переключатель режима, таблица файлов

**Files:**
- Modify: `src/components/dialogs/CommitDialog.vue` (template блок)

- [ ] **Step 1: Заменить `<template>` блок целиком**

```vue
<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog commit-dialog">
      <div class="dialog-header">
        <h3>Commit</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="intro">
          <div class="intro-title">Commit local or staged changes</div>
          <div class="intro-sub">Select the files you want to commit and provide a commit message.</div>
        </div>

        <div class="mode-row">
          <label class="radio-label" :class="{ disabled: !hasStaged }">
            <input
              type="radio"
              name="commit-mode"
              value="staged"
              :checked="mode === 'staged'"
              :disabled="!hasStaged"
              @change="setMode('staged')"
            />
            <span>Staged Changes</span>
          </label>
          <label class="radio-label">
            <input
              type="radio"
              name="commit-mode"
              value="local"
              :checked="mode === 'local'"
              @change="setMode('local')"
            />
            <span>Local Changes</span>
          </label>
          <div class="counter">{{ checkedCount }} files (~{{ totalInMode }})</div>
        </div>

        <div class="files-table">
          <div class="files-head">
            <div class="col-check">
              <input
                type="checkbox"
                :checked="allChecked"
                :disabled="mode === 'staged' || totalInMode === 0"
                @change="toggleAll"
              />
            </div>
            <div class="col-name">Name</div>
            <div class="col-dir">Directory</div>
          </div>
          <div class="files-body">
            <div
              v-for="f in filesInMode"
              :key="f.path"
              class="file-row"
              @click="toggleFile(f.path)"
            >
              <div class="col-check">
                <input
                  type="checkbox"
                  :checked="mode === 'staged' || checked.has(f.path)"
                  :disabled="mode === 'staged'"
                  @click.stop
                  @change="toggleFile(f.path)"
                />
              </div>
              <div class="col-name">{{ splitPath(f.path).name }}</div>
              <div class="col-dir">{{ splitPath(f.path).dir }}</div>
            </div>
            <div v-if="totalInMode === 0" class="empty">No files</div>
          </div>
        </div>

        <div class="message-field">
          <label class="message-label">Commit Message:</label>
          <textarea
            v-model="message"
            placeholder="Commit message..."
            rows="5"
            class="commit-message-input"
          />
          <div class="message-indicator" :class="firstLineClass">
            {{ firstLineLength }} / 72
          </div>
        </div>

        <div class="commit-options">
          <label class="checkbox-label">
            <input type="checkbox" v-model="amend" />
            <span>Amend last commit</span>
          </label>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" :disabled="busy" @click="$emit('close')">Cancel</button>
        <button class="btn btn-primary" :disabled="!canCommit" @click="handleCommit(false)">
          Commit
        </button>
        <button class="btn btn-accent" :disabled="!canCommit" @click="handleCommit(true)">
          Commit &amp; Push
        </button>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/dialogs/CommitDialog.vue
git commit -m "feat(commit-dialog): таблица файлов и переключатель режима"
```

---

## Task 3: Стили

**Files:**
- Modify: `src/components/dialogs/CommitDialog.vue` (style блок)

- [ ] **Step 1: Заменить `<style scoped>` блок целиком**

```vue
<style scoped>
.commit-dialog {
  width: 640px;
  max-width: 90vw;
}

.intro {
  margin-bottom: 12px;
}
.intro-title {
  font-size: var(--font-size);
  font-weight: 600;
}
.intro-sub {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin-top: 2px;
}

.mode-row {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 6px;
}
.radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.radio-label.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.radio-label input {
  accent-color: var(--accent);
}
.counter {
  margin-left: auto;
  font-size: var(--font-size-sm);
  color: var(--text-muted);
}

.files-table {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-input, var(--bg));
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  max-height: 240px;
}
.files-head {
  display: grid;
  grid-template-columns: 32px 1fr 1.5fr;
  padding: 6px 8px;
  font-size: var(--font-size-sm);
  font-weight: 600;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated, var(--bg));
}
.files-body {
  overflow-y: auto;
}
.file-row {
  display: grid;
  grid-template-columns: 32px 1fr 1.5fr;
  padding: 4px 8px;
  font-size: var(--font-size-sm);
  cursor: pointer;
  user-select: none;
}
.file-row:hover {
  background: var(--bg-hover, rgba(255, 255, 255, 0.04));
}
.col-check {
  display: flex;
  align-items: center;
}
.col-check input {
  accent-color: var(--accent);
}
.col-name,
.col-dir {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.col-dir {
  color: var(--text-muted);
}
.empty {
  padding: 12px;
  text-align: center;
  font-size: var(--font-size-sm);
  color: var(--text-muted);
}

.message-field {
  position: relative;
}
.message-label {
  display: block;
  font-size: var(--font-size-sm);
  font-weight: 600;
  margin-bottom: 4px;
}
.commit-message-input {
  width: 100%;
  font-family: var(--font-mono);
  font-size: var(--font-size);
  resize: vertical;
  min-height: 100px;
  padding: 8px;
}
.message-indicator {
  position: absolute;
  bottom: 6px;
  right: 8px;
  font-size: var(--font-size-xs);
  padding: 1px 6px;
  border-radius: var(--radius);
}
.message-indicator.ok { color: var(--text-muted); }
.message-indicator.warning { color: var(--yellow); }
.message-indicator.error { color: var(--red); }

.commit-options {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 8px;
}
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.checkbox-label input {
  accent-color: var(--accent);
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/dialogs/CommitDialog.vue
git commit -m "feat(commit-dialog): стили списка файлов и режима"
```

---

## Task 4: Bump версии + сборка

**Files:**
- Modify: `src/App.vue` (title) и `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml` — по паттерну предыдущих коммитов (memory: «Version bump»).

- [ ] **Step 1: Найти текущую версию**

Запустить:
```bash
grep -n "GitStream v0" src/App.vue | head -5
```

Ожидание: найдётся строка вида `GitStream v0.1.X`.

- [ ] **Step 2: Инкрементировать patch**

Заменить `v0.1.X` на `v0.1.(X+1)` в:
- `src/App.vue` (title/toolbar)
- `package.json` (поле `version`)
- `src-tauri/tauri.conf.json` (поле `version`)
- `src-tauri/Cargo.toml` (поле `version`)

Используй grep чтобы найти все вхождения:
```bash
grep -rn "0\.1\." package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src/App.vue
```

- [ ] **Step 3: Запустить сборку TS**

Run: `npm run build`
Expected: сборка проходит без ошибок TypeScript.

Если ошибки есть — исправить в `CommitDialog.vue` и повторить сборку, пока не пройдёт.

- [ ] **Step 4: Commit**

```bash
git add src/App.vue package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore: bump version"
```

---

## Task 5: Ручная проверка сценариев

Этот таск не пишет код — он верифицирует что фича работает. Запусти `npm run tauri dev` и пройди по чек-листу.

- [ ] **Сценарий 1: Local Changes, все отмечены по умолчанию**
  1. Сделай изменения в 2-3 файлах (не стейджь).
  2. Открой Commit Dialog.
  3. Ожидание: режим **Local Changes** активен, радио «Staged» disabled, **0 файлов отмечено** (потому что ничего не в индексе).
  4. Кликни toggle-all в шапке → все отмечаются.
  5. Введи сообщение, нажми Commit.
  6. Ожидание: диалог закрывается, коммит создан, файлы пропали из списка изменений.

- [ ] **Сценарий 2: Staged-режим при открытии**
  1. Сделай изменения в 3 файлах. Застейджи 2 через FileList.
  2. Открой Commit Dialog.
  3. Ожидание: автоматически **Staged Changes**, видно 2 файла, чекбоксы отмечены и disabled, счётчик `2 files (~2)`.
  4. Commit → в коммит попали только 2 staged файла, 3-й остался в working tree.

- [ ] **Сценарий 3: Переключение Staged → Local, снятие галочки**
  1. Застейджи файл A. Измени файл B (не стейджь).
  2. Открой диалог (откроется в Staged).
  3. Переключи на Local Changes.
  4. Ожидание: видно оба файла (A и B), A отмечен, B не отмечен.
  5. Сними галочку с A, поставь на B. Commit.
  6. Ожидание: в коммит попал только B. A остался в индексе? Нет — по логике `applyStagingForLocalMode` A должен быть unstaged. Проверь через `git status`: A должен быть modified (unstaged), закоммичен только B.

- [ ] **Сценарий 4: Commit & Push**
  1. Репо с настроенным origin. Сделай изменение.
  2. Commit & Push в диалоге.
  3. Ожидание: коммит создан и запушен в origin. Проверь через `git log origin/master`.

- [ ] **Сценарий 5: Пустое сообщение / 0 отмеченных**
  1. Открой диалог, снять все галочки (в Local). Ожидание: кнопки Commit / Commit & Push **disabled**.
  2. Отметь файл, но оставь сообщение пустым. Ожидание: кнопки **disabled**.
  3. И то и другое — кнопки активны.

- [ ] **Сценарий 6: Нет файлов вообще**
  1. Чистая рабочая копия.
  2. Открой диалог (через toolbar).
  3. Ожидание: пустой список с текстом «No files», обе радио-кнопки: Staged disabled, Local активен, кнопки Commit / Commit & Push disabled.

---

## Self-Review Notes

- **Spec coverage:** все разделы покрыты — режимы (Task 1-2), toggle-all (Task 1-2), commit logic включая unstage снятых (Task 1), Commit & Push через первый remote (Task 1), стили (Task 3), edge cases (Task 5).
- **Type consistency:** `Mode` тип единый, `filesInMode` / `checkedCount` / `allChecked` используются согласованно в script и template.
- **No placeholders:** каждый шаг содержит конкретный код или команду.
