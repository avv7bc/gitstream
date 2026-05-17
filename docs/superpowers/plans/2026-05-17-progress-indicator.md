# Индикатор выполнения операций (>100 ms) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Любая IPC-операция дольше 100 мс показывает крутилку с подписью в StatusBar, автоматически для всех вызовов.

**Architecture:** Новый модуль `useProgress.ts` экспортирует drop-in замену `invoke` (таймер 100 мс + карта активных операций) и composable `useProgress()`. Все composables меняют только строку импорта. StatusBar читает глобальное состояние.

**Tech Stack:** Vue 3 Composition API, TypeScript, Tauri 2 (`@tauri-apps/api/core`). Тест-фреймворка в проекте нет — верификация через `npm run build` (vue-tsc + vite) и ручной сценарий.

Спецификация: `docs/superpowers/specs/2026-05-17-progress-indicator-design.md`

---

## Структура файлов

- **Создать:** `src/composables/useProgress.ts` — обёртка `invoke`, глобальное состояние, словарь подписей.
- **Изменить (только строка импорта):** `src/composables/` — `useBranches.ts`, `useCommit.ts`, `useConflicts.ts`, `useDiff.ts`, `useFiles.ts`, `useLog.ts`, `useRemote.ts`, `useRepo.ts`, `useSettings.ts`.
- **Изменить:** `src/components/StatusBar.vue` — крутилка + подпись из `useProgress()`, инкремент версии.

---

### Task 1: Модуль useProgress.ts

**Files:**
- Create: `src/composables/useProgress.ts`

- [ ] **Step 1: Создать модуль**

Создать `src/composables/useProgress.ts` со следующим содержимым целиком:

```ts
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { computed, ref } from "vue";

const THRESHOLD_MS = 100;

const COMMAND_LABELS: Record<string, string> = {
  do_fetch: "Fetch…",
  do_pull: "Pull…",
  do_push: "Push…",
  do_clone: "Клонирование…",
  get_status: "Статус файлов…",
  get_log: "Загрузка лога…",
  stage_files: "Stage…",
  unstage_files: "Unstage…",
  discard_files: "Discard…",
};

const FALLBACK_LABEL = "Работаем…";

interface ActiveOp {
  cmd: string;
  label: string;
}

const active = ref(new Map<number, ActiveOp>());
let seq = 0;

export const isWorking = computed(() => active.value.size > 0);

export const progressLabel = computed(() => {
  const size = active.value.size;
  if (size === 0) return "";
  if (size > 1) return `Операций: ${size}`;
  const first = active.value.values().next().value as ActiveOp | undefined;
  return first?.label ?? FALLBACK_LABEL;
});

export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const id = ++seq;
  const label = COMMAND_LABELS[cmd] ?? FALLBACK_LABEL;

  const timer = setTimeout(() => {
    const next = new Map(active.value);
    next.set(id, { cmd, label });
    active.value = next;
  }, THRESHOLD_MS);

  try {
    return await tauriInvoke<T>(cmd, args);
  } finally {
    clearTimeout(timer);
    if (active.value.has(id)) {
      const next = new Map(active.value);
      next.delete(id);
      active.value = next;
    }
  }
}

export function useProgress() {
  return { isWorking, progressLabel };
}
```

Примечание: `active` — `ref<Map>` с пересозданием Map при изменении, чтобы `computed` гарантированно реагировал (Vue не отслеживает мутацию Map по месту в `ref`).

- [ ] **Step 2: Проверка типов**

Run: `npm run build`
Expected: PASS (сборка без ошибок vue-tsc).

- [ ] **Step 3: Commit**

```bash
git add src/composables/useProgress.ts
git commit -m "feat: модуль useProgress — обёртка invoke с индикатором >100ms"
```

---

### Task 2: Перевести composables на обёртку

**Files:**
- Modify: `src/composables/useBranches.ts`, `useCommit.ts`, `useConflicts.ts`, `useDiff.ts`, `useFiles.ts`, `useLog.ts`, `useRemote.ts`, `useRepo.ts`, `useSettings.ts`

- [ ] **Step 1: Заменить строку импорта в каждом файле**

В каждом из 9 файлов заменить:

```ts
import { invoke } from "@tauri-apps/api/core";
```

на:

```ts
import { invoke } from "@/composables/useProgress";
```

Точная форма строки в `useCommit.ts` — без `;`-различий проверить по факту; заменять весь импорт `invoke` из `@tauri-apps/api/core`. Тела вызовов `invoke(...)` НЕ менять. Если в файле импортируются и другие сущности из `@tauri-apps/api/core` — оставить их отдельным импортом, убрав только `invoke`.

- [ ] **Step 2: Убедиться, что не осталось прямых импортов invoke из tauri core**

Run: `grep -rn 'invoke.*@tauri-apps/api/core' src/`
Expected: пусто (единственное упоминание `@tauri-apps/api/core` для invoke — внутри `useProgress.ts`, где импортируется как `tauriInvoke`).

- [ ] **Step 3: Проверка типов и сборки**

Run: `npm run build`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/composables/
git commit -m "refactor: composables используют tracked invoke из useProgress"
```

---

### Task 3: StatusBar — крутилка и подпись

**Files:**
- Modify: `src/components/StatusBar.vue`

- [ ] **Step 1: Подключить useProgress в script**

В `<script setup lang="ts">` добавить импорт и использование рядом с существующими:

```ts
import { useProgress } from "@/composables/useProgress";

const { isWorking, progressLabel } = useProgress();
```

Существующий `const { isBusy, lastError } = useRemote();` оставить без изменений (`isBusy` нужен для блокировки кнопок в других местах; здесь больше не используется в шаблоне).

- [ ] **Step 2: Заменить содержимое .statusbar-center**

Заменить блок:

```html
    <div class="statusbar-center">
      <span class="status-message">{{ lastError ?? (isBusy ? 'Working...' : '') }}</span>
    </div>
```

на:

```html
    <div class="statusbar-center">
      <span v-if="lastError" class="status-message">{{ lastError }}</span>
      <span v-else-if="isWorking" class="status-message progress">
        <svg class="codicon spin" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M8 1.5a6.5 6.5 0 1 0 6.5 6.5h-1.5a5 5 0 1 1-5-5V1.5z"/>
        </svg>
        <span>{{ progressLabel }}</span>
      </span>
    </div>
```

- [ ] **Step 3: Добавить стили для крутилки**

В блок `<style scoped>` добавить в конец:

```css
.status-message.progress {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.codicon.spin {
  animation: gs-spin 0.8s linear infinite;
}

@keyframes gs-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
```

- [ ] **Step 4: Инкремент версии**

В шаблоне найти `<span class="version">0.1.34</span>` и заменить на `<span class="version">0.1.35</span>`.

(Memory: после каждого изменения кода патч-версия в заголовке инкрементируется.)

- [ ] **Step 5: Проверка сборки**

Run: `npm run build`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/components/StatusBar.vue
git commit -m "feat: крутилка и подпись операции в StatusBar (v0.1.35)"
```

---

### Task 4: Ручная проверка сценариев

**Files:** нет (ручной прогон)

- [ ] **Step 1: Запустить приложение**

Run: `npm run tauri dev`
Expected: приложение стартует без ошибок консоли.

- [ ] **Step 2: Долгая операция показывает крутилку**

Открыть репозиторий с большой историей и выполнить Fetch/Pull (или лог большого репо).
Expected: в центре StatusBar появляется вращающаяся иконка + подпись (`Fetch…` / `Pull…` / `Загрузка лога…`).

- [ ] **Step 3: Быстрая операция не мерцает**

Сделать stage/unstage одного файла в небольшом репозитории.
Expected: индикатор не появляется (операция < 100 мс) либо появляется только на действительно долгих.

- [ ] **Step 4: Ошибка гасит индикатор**

Спровоцировать ошибку (например push без доступа).
Expected: крутилка исчезает, в центре показывается `lastError`.

- [ ] **Step 5: Финальная проверка сборки**

Run: `npm run build`
Expected: PASS.

---

## Self-Review

- **Покрытие спеца:** модуль (Task 1) ↔ «Состояние и API модуля», «Архитектура»; импорты (Task 2) ↔ «Подключение»; StatusBar (Task 3) ↔ раздел «StatusBar» + критерий версии; ручной прогон (Task 4) ↔ «Критерии готовности» и «Граничные случаи». Гэпов нет.
- **Плейсхолдеры:** отсутствуют — весь код приведён целиком.
- **Согласованность типов:** `invoke<T>`, `useProgress() → { isWorking, progressLabel }`, `ActiveOp`, `THRESHOLD_MS`, `COMMAND_LABELS`, `FALLBACK_LABEL` — имена консистентны между Task 1 и Task 3.
- **Замечание о реактивности Map** учтено: `active` пересоздаётся, а не мутируется по месту, чтобы `computed` срабатывал.
