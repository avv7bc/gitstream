# CommitGraph Ref-Chips Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.
>
> **HARD CONSTRAINT — NO GIT COMMITS.** The user has a standing rule: never `git commit`/`merge`/`push` without explicit consent. Every task here implements + verifies ONLY. Do NOT run `git add`/`git commit`. Leave all changes uncommitted in the worktree. The controller asks for commit consent once at the end.

**Goal:** Ref-метки в CommitGraph как на референсе: иконки в чипах, выделение текущей ветки, бейджи `+N`/`−N` (ahead/behind) у HEAD-коммита текущей ветки.

**Architecture:** Бэкенд помечает `HEAD -> X` новым kind `current-branch`. Иконки выносятся в общий `RefIcon.vue` (единый источник; BranchPanel рефакторится на него — попутно устранение дублирования SVG). Бейджи ahead/behind — синтетические чипы из существующего `useBranches()`.

**Tech Stack:** Rust (Tauri), Vue 3 + TS, Vite. Работа в worktree `/home/avv/projects/gitstream/.worktrees/ref-chips` (ветка `feature/commitgraph-ref-chips`).

**Spec:** `docs/superpowers/specs/2026-05-16-commitgraph-ref-chips-design.md`

**Verify:** FE — `npm run build` (vue-tsc + vite); BE — `cargo test`. Нет FE-тест-раннера.

---

### Task 1: Backend — kind `current-branch` + тип + Rust-тест

**Files:**
- Modify: `src-tauri/src/git/query.rs` (`parse_ref_labels`, ~строки 84–99; добавить `#[cfg(test)]` модуль в конец файла)
- Modify: `src/types/index.ts` (union `RefLabel.kind`)

- [ ] **Step 1: Тест (TDD, red)** — В конец `src-tauri/src/git/query.rs` добавить:

```rust
#[cfg(test)]
mod ref_label_tests {
    use super::*;

    fn kinds(raw: &str) -> Vec<(String, String)> {
        parse_ref_labels(raw)
            .into_iter()
            .map(|r| (r.name, r.kind))
            .collect()
    }

    #[test]
    fn current_branch_from_head_arrow() {
        assert_eq!(
            kinds("HEAD -> main"),
            vec![("main".to_string(), "current-branch".to_string())]
        );
    }

    #[test]
    fn standalone_head_is_head() {
        assert_eq!(
            kinds("HEAD"),
            vec![("HEAD".to_string(), "head".to_string())]
        );
    }

    #[test]
    fn tag_remote_local_kinds() {
        assert_eq!(kinds("tag: v1.0"), vec![("v1.0".to_string(), "tag".to_string())]);
        assert_eq!(kinds("origin/main"), vec![("origin/main".to_string(), "remote-branch".to_string())]);
        assert_eq!(kinds("dev"), vec![("dev".to_string(), "local-branch".to_string())]);
    }

    #[test]
    fn combined_decoration() {
        assert_eq!(
            kinds("HEAD -> main, tag: v1, origin/main"),
            vec![
                ("main".to_string(), "current-branch".to_string()),
                ("v1".to_string(), "tag".to_string()),
                ("origin/main".to_string(), "remote-branch".to_string()),
            ]
        );
    }
}
```

- [ ] **Step 2: Run, verify FAIL** — `cd src-tauri && cargo test ref_label_tests 2>&1 | tail -15` — Expected: FAIL (`current_branch_from_head_arrow` ожидает `current-branch`, код выдаёт `local-branch`).

- [ ] **Step 3: Реализация** — Заменить тело `parse_ref_labels` (строки 84–99) на:

```rust
fn parse_ref_labels(raw: &str) -> Vec<RefLabel> {
    if raw.trim().is_empty() { return Vec::new(); }
    raw.split(", ").filter_map(|r| {
        let r = r.trim();
        if r.is_empty() { return None; }
        if r == "HEAD" { return Some(RefLabel { name: "HEAD".to_string(), kind: "head".to_string() }); }
        if let Some(rest) = r.strip_prefix("HEAD -> ") {
            return Some(RefLabel { name: rest.to_string(), kind: "current-branch".to_string() });
        }
        if let Some(t) = r.strip_prefix("tag: ") {
            Some(RefLabel { name: t.to_string(), kind: "tag".to_string() })
        } else if r.contains('/') {
            Some(RefLabel { name: r.to_string(), kind: "remote-branch".to_string() })
        } else {
            Some(RefLabel { name: r.to_string(), kind: "local-branch".to_string() })
        }
    }).collect()
}
```

- [ ] **Step 4: Run, verify PASS** — `cd src-tauri && cargo test ref_label_tests 2>&1 | tail -8` — Expected: `test result: ok. 4 passed`.

- [ ] **Step 5: Тип во фронте** — В `src/types/index.ts` найти `RefLabel` и заменить строку поля `kind` на:

```ts
  kind: "local-branch" | "remote-branch" | "tag" | "head" | "stash" | "current-branch";
```

- [ ] **Step 6: Verify FE build** — `npm run build 2>&1 | tail -5` — Expected: `✓ built`.

- [ ] **Step 7: НЕ коммитить.** Сообщить статус и список изменённых файлов.

---

### Task 2: Новый компонент `RefIcon.vue`

**Files:**
- Create: `src/components/RefIcon.vue`

Единый источник иконок. SVG взяты из `BranchPanel.vue` дословно. Цвет — по kind (единая палитра; `currentColor` для штрихов).

- [ ] **Step 1: Создать файл** `src/components/RefIcon.vue` с точным содержимым:

```vue
<script setup lang="ts">
import type { RefLabel } from "@/types";

defineProps<{ kind: RefLabel["kind"] }>();
</script>

<template>
  <svg
    v-if="kind === 'tag'"
    class="ref-icon ref-icon--tag"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <path d="M2 9V2h7l5 5-7 7-5-5z" fill="none" stroke="currentColor" stroke-width="1.2" />
    <circle cx="6" cy="6" r="1" fill="currentColor" />
  </svg>
  <svg
    v-else-if="kind === 'stash'"
    class="ref-icon ref-icon--stash"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <rect x="3" y="3" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2" />
    <rect x="3" y="8" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2" />
  </svg>
  <svg
    v-else-if="kind === 'remote-branch'"
    class="ref-icon ref-icon--remote"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <circle cx="8" cy="4" r="2" fill="none" stroke="currentColor" stroke-width="1.2" />
    <path d="M8 6v4M4 12h8M4 12v-2M12 12v-2" fill="none" stroke="currentColor" stroke-width="1.2" />
  </svg>
  <svg
    v-else
    class="ref-icon ref-icon--branch"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <path
      d="M5 3a2 2 0 100 4 2 2 0 000-4zM5 9a2 2 0 100 4 2 2 0 000-4z"
      fill="none" stroke="currentColor" stroke-width="1.2"
    />
    <path d="M5 7v2" fill="none" stroke="currentColor" stroke-width="1.2" />
  </svg>
</template>

<style scoped>
.ref-icon {
  flex-shrink: 0;
  vertical-align: middle;
}
</style>
```

(Цвет НЕ задаётся в компоненте — наследуется `currentColor` от родителя: в чипах CommitGraph его задаёт `.ref-*`, в BranchPanel — Task 3.)

- [ ] **Step 2: Verify build** — `npm run build 2>&1 | tail -3` — Expected: `✓ built` (компонент пока не используется — это нормально).

- [ ] **Step 3: НЕ коммитить.** Сообщить статус.

---

### Task 3: Рефактор `BranchPanel.vue` на `RefIcon`

**Files:**
- Modify: `src/components/BranchPanel.vue` (4 инлайн-SVG → `<RefIcon>`; импорт; CSS иконок)

- [ ] **Step 1: Импорт** — В блоке `<script setup>` `BranchPanel.vue`, рядом с другими импортами компонентов (напр. после `import RenameBranchDialog ...`), добавить:

```ts
import RefIcon from "@/components/RefIcon.vue";
```

- [ ] **Step 2: Local branch icon** — Заменить блок:

```vue
            <svg width="14" height="14" viewBox="0 0 16 16" class="branch-icon">
              <path d="M5 3a2 2 0 100 4 2 2 0 000-4zM5 9a2 2 0 100 4 2 2 0 000-4z"
                    fill="none" stroke="currentColor" stroke-width="1.2"/>
              <path d="M5 7v2" fill="none" stroke="currentColor" stroke-width="1.2"/>
            </svg>
```

на:

```vue
            <RefIcon kind="local-branch" class="bp-icon bp-icon--branch" />
```

- [ ] **Step 3: Remote branch icon** — Заменить блок:

```vue
            <svg width="14" height="14" viewBox="0 0 16 16" class="branch-icon remote">
              <circle cx="8" cy="4" r="2" fill="none" stroke="currentColor" stroke-width="1.2"/>
              <path d="M8 6v4M4 12h8M4 12v-2M12 12v-2" fill="none" stroke="currentColor" stroke-width="1.2"/>
            </svg>
```

на:

```vue
            <RefIcon kind="remote-branch" class="bp-icon bp-icon--remote" />
```

- [ ] **Step 4: Tag icon** — Заменить блок:

```vue
            <svg width="14" height="14" viewBox="0 0 16 16" class="tag-icon">
              <path d="M2 9V2h7l5 5-7 7-5-5z" fill="none" stroke="currentColor" stroke-width="1.2"/>
              <circle cx="6" cy="6" r="1" fill="currentColor"/>
            </svg>
```

на:

```vue
            <RefIcon kind="tag" class="bp-icon bp-icon--tag" />
```

- [ ] **Step 5: Stash icon** — Заменить блок:

```vue
            <svg width="14" height="14" viewBox="0 0 16 16" class="stash-icon">
              <rect x="3" y="3" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
              <rect x="3" y="8" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
            </svg>
```

на:

```vue
            <RefIcon kind="stash" class="bp-icon bp-icon--stash" />
```

- [ ] **Step 6: CSS** — В `<style scoped>` `BranchPanel.vue` найти существующие правила (строки ~635–649):

```css
.branch-icon {
  flex-shrink: 0;
  color: var(--green);
}
.branch-icon.remote {
  color: var(--text-muted);
}
.tag-icon {
  flex-shrink: 0;
  color: var(--yellow);
}
.stash-icon {
  flex-shrink: 0;
  color: var(--purple);
}
```

и заменить их на (цвет теперь применяется к корню `RefIcon` через класс-обёртку; `flex-shrink` уже в самом `RefIcon`):

```css
.bp-icon--branch {
  color: var(--green);
}
.bp-icon--remote {
  color: var(--text-muted);
}
.bp-icon--tag {
  color: var(--yellow);
}
.bp-icon--stash {
  color: var(--purple);
}
```

(`color` наследуется в `<svg>` `RefIcon` через `currentColor` — обёрточный класс на корне компонента задаёт его. Поведение/цвета BranchPanel сохраняются 1:1.)

- [ ] **Step 7: Verify build + grep** — `npm run build 2>&1 | tail -3` (Expected `✓ built`) и `grep -n "branch-icon\|tag-icon\|stash-icon\|<svg" src/components/BranchPanel.vue` — Expected: НЕТ остатков `.branch-icon/.tag-icon/.stash-icon` и НЕТ инлайн `<svg ... class="...-icon">` для веток/тегов/stash (chevron-стрелки секций `<svg class="chevron">` остаются — их НЕ трогать).

- [ ] **Step 8: НЕ коммитить.** Сообщить статус, список изменённых файлов, подтвердить что chevron-иконки секций не затронуты.

---

### Task 4: CommitGraph — иконки в чипах, выделение текущей ветки, бейджи ahead/behind

**Files:**
- Modify: `src/components/CommitGraph.vue` (импорт RefIcon + useBranches; разметка чипа; бейджи; CSS)

- [ ] **Step 1: Импорты + данные ветки** — В `<script setup>` `CommitGraph.vue`:
  - рядом с `import { useLog } from "@/composables/useLog";` добавить:
    ```ts
    import { useBranches } from "@/composables/useBranches";
    import RefIcon from "@/components/RefIcon.vue";
    ```
  - рядом с `const { commits, selectedCommit } = useLog();` добавить:
    ```ts
    const { branches } = useBranches();
    const currentBranch = computed(() => branches.value.find((b) => b.is_current));
    function isCurrentBranchRow(c: { refs: { kind: string }[] }): boolean {
      return c.refs.some((r) => r.kind === "current-branch");
    }
    ```
  - убедиться, что `computed` импортирован из `vue` (он уже используется в файле — `filteredCommits` и т.д.; если нет в импорте — добавить в существующий `import { ... } from "vue"`).

- [ ] **Step 2: Разметка чипа + бейджи** — Заменить блок (`message-col`, строки ~219–227):

```vue
        <div class="message-col">
          <span
            v-for="r in commit.refs"
            :key="r.name"
            :class="refClass(r)"
            v-html="highlight(r.name, graphFilter)"
          />
          <span class="commit-message" v-html="highlight(commit.message, graphFilter)" />
        </div>
```

на:

```vue
        <div class="message-col">
          <template v-if="isCurrentBranchRow(commit)">
            <span
              v-if="currentBranch && currentBranch.ahead > 0"
              class="ref-label ref-ahead"
              :title="`${currentBranch.ahead} ahead`"
            >+{{ currentBranch.ahead }}</span>
            <span
              v-if="currentBranch && currentBranch.behind > 0"
              class="ref-label ref-behind"
              :title="`${currentBranch.behind} behind`"
            >&minus;{{ currentBranch.behind }}</span>
          </template>
          <span
            v-for="r in commit.refs"
            :key="r.name"
            :class="refClass(r)"
          >
            <RefIcon :kind="r.kind" />
            <span v-html="highlight(r.name, graphFilter)" />
          </span>
          <span class="commit-message" v-html="highlight(commit.message, graphFilter)" />
        </div>
```

(`refClass` уже возвращает `ref-label ref-${r.kind}` — для `current-branch` автоматически `ref-label ref-current-branch`; менять функцию не нужно.)

- [ ] **Step 3: CSS** — В `<style scoped>` `CommitGraph.vue` после блока `.ref-stash { ... }` (≈строка 441) добавить:

```css
.ref-current-branch {
  background: rgba(166, 227, 161, 0.35);
  color: var(--green);
  font-weight: 800;
}
.ref-ahead {
  background: rgba(166, 227, 161, 0.2);
  color: var(--green);
}
.ref-behind {
  background: rgba(243, 139, 168, 0.2);
  color: var(--red);
}
.ref-label {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}
.ref-label .ref-icon {
  width: 11px;
  height: 11px;
}
```

(Последние два правила переопределяют `display:inline-block` базового `.ref-label`, чтобы иконка и текст были по центру; `.ref-ahead/.ref-behind` — текстовые бейджи без иконки, flex с одним ребёнком безопасен.)

- [ ] **Step 4: Verify build** — `npm run build 2>&1 | tail -3` — Expected: `✓ built`, без vue-tsc ошибок.

- [ ] **Step 5: НЕ коммитить.** Сообщить статус, изменённые файлы.

---

### Task 5: Bump версии + финальная проверка

**Files:**
- Modify: `src-tauri/tauri.conf.json` (`version`)

- [ ] **Step 1: Bump** — В `src-tauri/tauri.conf.json` заменить `"version": "0.1.8",` на `"version": "0.1.9",` (правило проекта: инкремент patch после изменений кода).

- [ ] **Step 2: Полная проверка** — выполнить из корня worktree:
  `npm run build 2>&1 | tail -3 && cd src-tauri && cargo test 2>&1 | tail -5 && cargo build 2>&1 | tail -3`
  Expected: `✓ built`; `cargo test` все тесты `ok` (включая `ref_label_tests` 4 passed и ранее существующие `tag_tests`); `cargo build` без ошибок.

- [ ] **Step 3: НЕ коммитить.** Сообщить итоговый статус и полный список изменённых/созданных файлов.

---

## Self-Review

**Покрытие спеки:**
- kind `current-branch` (бэкенд + тип) — Task 1. ✓
- Иконки в чипах через общий компонент — Task 2 (RefIcon) + Task 4 (использование). ✓
- Единый источник, BranchPanel рефакторится — Task 3. ✓
- Выделение текущей ветки — Task 1 (kind) + Task 4 (`.ref-current-branch`). ✓
- Бейджи `+N`/`−N` из useBranches у HEAD-коммита текущей ветки — Task 4. ✓
- Detached HEAD без бейджей — обеспечено `isCurrentBranchRow` (нет `current-branch` ref). ✓
- Нет upstream → ahead/behind 0 → `v-if` скрывает бейджи. ✓
- Rust-тест parse_ref_labels — Task 1. ✓
- Bump версии — Task 5. ✓

**Плейсхолдеры:** нет — весь код приведён дословно.

**Согласованность типов:** `RefLabel.kind` union (Task 1) включает `current-branch`, используется `RefIcon` props (Task 2) и `refClass`/`isCurrentBranchRow` (Task 4). `RefIcon` `kind` prop типизирован `RefLabel["kind"]`. `currentBranch` — из `useBranches().branches` (`BranchInfo` с `ahead/behind/is_current`), как в `StatusBar`. `refClass` не меняется (уже даёт `ref-current-branch`).

**Изменение поведения (намеренное):** в BranchPanel удаляются классы `.branch-icon/.tag-icon/.stash-icon`; цвета 1:1 переносятся на `.bp-icon--*`. Прочих визуальных изменений в BranchPanel нет; chevron-иконки секций не трогаются.
