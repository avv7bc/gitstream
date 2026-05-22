# Multi-Select Batch Delete (BranchPanel) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the user select multiple tags and local branches in `BranchPanel.vue` via shift / ctrl+click and delete the selection in one operation.

**Architecture:** Frontend-only change. `BranchPanel.vue` replaces its single `selectedKey` with a `selectedKeys` set plus a shift-anchor; context menus and delete handlers become batch-aware (loop over the selection, calling the existing single-item backend commands). `ConfirmDialog.vue` gains an optional `items` list prop to show all affected names. No backend changes.

**Tech Stack:** Vue 3 Composition API + TypeScript, Vite.

**Spec:** `docs/superpowers/specs/2026-05-22-multi-select-batch-delete-design.md`

**Verification note:** The codebase has no automated test harness for the frontend — `npm run build` runs `vue-tsc --noEmit && vite build` (type-check + build). Every task is verified by `npm run build` plus a stated manual check in `npm run tauri:dev`.

---

### Task 1: ConfirmDialog — optional `items` list

**Files:**
- Modify: `src/components/dialogs/ConfirmDialog.vue`

- [ ] **Step 1: Add the `items` prop**

In `src/components/dialogs/ConfirmDialog.vue`, replace the `defineProps` block:

```ts
const props = defineProps<{
  message: string;
  confirmLabel?: string;
  danger?: boolean;
  checkboxLabel?: string;
}>();
```

with:

```ts
const props = defineProps<{
  message: string;
  confirmLabel?: string;
  danger?: boolean;
  checkboxLabel?: string;
  items?: string[];
}>();
```

- [ ] **Step 2: Render the list in the dialog body**

Replace the `<div class="dialog-body">` block:

```vue
      <div class="dialog-body">
        <p class="confirm-message">{{ message }}</p>
        <label v-if="checkboxLabel" class="confirm-checkbox">
          <input type="checkbox" v-model="checked" />
          {{ checkboxLabel }}
        </label>
      </div>
```

with:

```vue
      <div class="dialog-body">
        <p class="confirm-message">{{ message }}</p>
        <ul v-if="items && items.length" class="confirm-list">
          <li v-for="item in items" :key="item">{{ item }}</li>
        </ul>
        <label v-if="checkboxLabel" class="confirm-checkbox">
          <input type="checkbox" v-model="checked" />
          {{ checkboxLabel }}
        </label>
      </div>
```

- [ ] **Step 3: Add list styles**

Append to the `<style scoped>` block, after `.confirm-checkbox { ... }`:

```css
.confirm-list {
  margin: 8px 0 0;
  padding: 6px 8px;
  max-height: 160px;
  overflow-y: auto;
  list-style: none;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}
.confirm-list li {
  padding: 1px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
```

- [ ] **Step 4: Verify the build**

Run: `npm run build`
Expected: PASS (no type errors). Existing `ConfirmDialog` callers pass no `items` and are unaffected.

- [ ] **Step 5: Commit**

```bash
git add src/components/dialogs/ConfirmDialog.vue
git commit -m "feat(dialog): optional items list in ConfirmDialog"
```

---

### Task 2: BranchPanel — multi-selection model

Replaces the single-item `selectedKey` with a `selectedKeys` set and a shift-anchor, and adds derived computeds used by later tasks. After this task, shift/ctrl+click changes the highlight but no batch action exists yet.

**Files:**
- Modify: `src/components/BranchPanel.vue`

- [ ] **Step 1: Replace the selection state and `selectItem`**

In `src/components/BranchPanel.vue`, replace this block (currently the last lines of `<script setup>`):

```ts
// --- Item selection (highlight on left/right click) ---
const selectedKey = ref<string | null>(null);
function selectItem(key: string) {
  selectedKey.value = key;
}
```

with:

```ts
// --- Item selection (single + multi via shift / ctrl) ---
const selectedKeys = ref<Set<string>>(new Set());
const anchorKey = ref<string | null>(null);

function sectionOf(key: string): string {
  return key.slice(0, key.indexOf(":"));
}

// Ordered item keys for the section a given key belongs to.
function sectionKeys(section: string): string[] {
  switch (section) {
    case "local":
      return localBranches.value.map((b) => `local:${b.name}`);
    case "remote":
      return remoteBranches.value.map((b) => `remote:${b.name}`);
    case "tag":
      return filteredTags.value.map((t) => `tag:${t.name}`);
    case "stash":
      return filteredStashes.value.map((s) => `stash:${s.index}`);
    default:
      return [];
  }
}

function selectItem(key: string, event: MouseEvent) {
  // Shift+click: range from the anchor, but only within one section.
  if (
    event.shiftKey &&
    anchorKey.value &&
    sectionOf(anchorKey.value) === sectionOf(key)
  ) {
    const keys = sectionKeys(sectionOf(key));
    const from = keys.indexOf(anchorKey.value);
    const to = keys.indexOf(key);
    if (from !== -1 && to !== -1) {
      const [lo, hi] = from <= to ? [from, to] : [to, from];
      selectedKeys.value = new Set(keys.slice(lo, hi + 1));
      return;
    }
  }
  // Ctrl/Cmd+click: toggle a single key, move the anchor.
  if (event.ctrlKey || event.metaKey) {
    const next = new Set(selectedKeys.value);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    selectedKeys.value = next;
    anchorKey.value = key;
    return;
  }
  // Plain click: replace selection.
  selectedKeys.value = new Set([key]);
  anchorKey.value = key;
}

```

Derived selections (`clearSelection`, `selectedTags`, etc.) are deliberately **not** added here — `tsconfig` has `noUnusedLocals: true`, so each is introduced in the task that first uses it (Tasks 3 and 4).

- [ ] **Step 2: Update the local-branch item bindings**

In the template, in the Local Branches `v-for`, replace:

```vue
            :class="{ current: branch.is_current, selected: selectedKey === `local:${branch.name}` }"
            @mousedown="selectItem(`local:${branch.name}`)"
```

with:

```vue
            :class="{ current: branch.is_current, selected: selectedKeys.has(`local:${branch.name}`) }"
            @mousedown="selectItem(`local:${branch.name}`, $event)"
```

- [ ] **Step 3: Update the remote-branch item bindings**

In the Remote Branches `v-for`, replace:

```vue
            :class="{ selected: selectedKey === `remote:${branch.name}` }"
            @mousedown="selectItem(`remote:${branch.name}`)"
```

with:

```vue
            :class="{ selected: selectedKeys.has(`remote:${branch.name}`) }"
            @mousedown="selectItem(`remote:${branch.name}`, $event)"
```

- [ ] **Step 4: Update the tag item bindings**

In the Tags `v-for`, replace:

```vue
            :class="{ selected: selectedKey === `tag:${tag.name}` }"
            @mousedown="selectItem(`tag:${tag.name}`)"
```

with:

```vue
            :class="{ selected: selectedKeys.has(`tag:${tag.name}`) }"
            @mousedown="selectItem(`tag:${tag.name}`, $event)"
```

- [ ] **Step 5: Update the stash item bindings**

In the Stashes `v-for`, replace:

```vue
            :class="{ selected: selectedKey === `stash:${stash.index}` }"
            @mousedown="selectItem(`stash:${stash.index}`)"
```

with:

```vue
            :class="{ selected: selectedKeys.has(`stash:${stash.index}`) }"
            @mousedown="selectItem(`stash:${stash.index}`, $event)"
```

- [ ] **Step 6: Verify the build**

Run: `npm run build`
Expected: PASS. Every binding added here is used: `selectItem` by the four templates, `selectedKeys` by the `:class` bindings, `anchorKey`/`sectionOf`/`sectionKeys` inside `selectItem`.

- [ ] **Step 7: Manual check**

Run `npm run tauri:dev`, open a repo with several branches/tags. Verify: plain click selects one item; ctrl/cmd+click adds/removes individual items; shift+click selects a contiguous range within one section; shift+click across sections behaves like a plain click. No text gets selected while shift-clicking.

- [ ] **Step 8: Commit**

```bash
git add src/components/BranchPanel.vue
git commit -m "feat(branches): multi-select model with shift/ctrl click"
```

---

### Task 3: BranchPanel — batch tag delete & push

Makes the tag context menu operate on the whole tag selection.

**Files:**
- Modify: `src/components/BranchPanel.vue`

- [ ] **Step 1: Add the `clearSelection` helper and `selectedTags` computed**

Append directly after the `selectItem` function added in Task 2 (the end of `<script setup>`):

```ts
function clearSelection() {
  selectedKeys.value = new Set();
  anchorKey.value = null;
}

// Tags currently selected, in display order.
const selectedTags = computed(() =>
  filteredTags.value.filter((t) => selectedKeys.value.has(`tag:${t.name}`)),
);
```

- [ ] **Step 2: Replace the single-tag target ref**

Replace:

```ts
const ctxTag = ref<TagInfo | null>(null);
const showDeleteTagConfirm = ref(false);
const targetTag = ref<TagInfo | null>(null);
```

with:

```ts
const ctxTag = ref<TagInfo | null>(null);
const showDeleteTagConfirm = ref(false);
const targetTags = ref<TagInfo[]>([]);
```

- [ ] **Step 3: Make the tag context menu selection-aware**

Replace `onTagContextMenu`:

```ts
function onTagContextMenu(e: MouseEvent, tag: TagInfo) {
  e.preventDefault();
  e.stopPropagation();
  tagCtxMenu.value = { x: e.clientX, y: e.clientY };
  ctxTag.value = tag;
}
```

with:

```ts
function onTagContextMenu(e: MouseEvent, tag: TagInfo) {
  e.preventDefault();
  e.stopPropagation();
  // Right-clicking an unselected tag selects just that tag first.
  const key = `tag:${tag.name}`;
  if (!selectedKeys.value.has(key)) {
    selectedKeys.value = new Set([key]);
    anchorKey.value = key;
  }
  tagCtxMenu.value = { x: e.clientX, y: e.clientY };
  ctxTag.value = tag;
}
```

- [ ] **Step 4: Make push-tag batch over the selection**

Replace `handlePushTagCtx`:

```ts
async function handlePushTagCtx() {
  const t = ctxTag.value;
  closeTagCtxMenu();
  if (!t) return;
  try {
    await pushTag(remotes.value[0] ?? "origin", t.name, false);
    emit("tagsChanged");
  } catch (e) {
    window.alert(`Push tag failed: ${e}`);
  }
}
```

with:

```ts
async function handlePushTagCtx() {
  const targets = selectedTags.value;
  closeTagCtxMenu();
  if (!targets.length) return;
  const remote = remotes.value[0] ?? "origin";
  const errors: string[] = [];
  for (const t of targets) {
    try {
      await pushTag(remote, t.name, false);
    } catch (e) {
      errors.push(`${t.name}: ${e}`);
    }
  }
  if (errors.length) window.alert(`Push tag failed:\n${errors.join("\n")}`);
  emit("tagsChanged");
}
```

- [ ] **Step 5: Make delete-tag batch over the selection**

Replace `handleDeleteTagCtx`:

```ts
function handleDeleteTagCtx() {
  targetTag.value = ctxTag.value;
  closeTagCtxMenu();
  showDeleteTagConfirm.value = true;
}
```

with:

```ts
function handleDeleteTagCtx() {
  targetTags.value = [...selectedTags.value];
  closeTagCtxMenu();
  if (!targetTags.value.length) return;
  showDeleteTagConfirm.value = true;
}
```

Replace `confirmDeleteTag`:

```ts
async function confirmDeleteTag(alsoRemote: boolean) {
  const t = targetTag.value;
  showDeleteTagConfirm.value = false;
  if (!t) {
    targetTag.value = null;
    return;
  }
  try {
    await deleteTag(t.name);
    if (alsoRemote && remotes.value.length > 0) {
      await pushTag(remotes.value[0], t.name, true);
    }
    emit("tagsChanged");
  } catch (e) {
    window.alert(`Delete tag failed: ${e}`);
  }
  targetTag.value = null;
}
```

with:

```ts
async function confirmDeleteTag(alsoRemote: boolean) {
  const targets = targetTags.value;
  showDeleteTagConfirm.value = false;
  if (!targets.length) return;
  const errors: string[] = [];
  for (const t of targets) {
    try {
      await deleteTag(t.name);
      if (alsoRemote && remotes.value.length > 0) {
        await pushTag(remotes.value[0], t.name, true);
      }
    } catch (e) {
      errors.push(`${t.name}: ${e}`);
    }
  }
  if (errors.length) window.alert(`Delete tag failed:\n${errors.join("\n")}`);
  clearSelection();
  emit("tagsChanged");
  targetTags.value = [];
}
```

- [ ] **Step 6: Update the tag context-menu labels**

In the template, in the tag context menu (the `v-if="tagCtxMenu"` block), replace:

```vue
        <button
          class="ctx-item"
          :disabled="!hasRemote"
          @click="handlePushTagCtx"
        >{{ i18n.branches.pushTag }}</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item ctx-danger"
          @click="handleDeleteTagCtx"
        >{{ i18n.branches.deleteTag }}</button>
```

with:

```vue
        <button
          class="ctx-item"
          :disabled="!hasRemote"
          @click="handlePushTagCtx"
        >{{ selectedTags.length > 1 ? `Push ${selectedTags.length} Tags` : i18n.branches.pushTag }}</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item ctx-danger"
          @click="handleDeleteTagCtx"
        >{{ selectedTags.length > 1 ? `Delete ${selectedTags.length} Tags` : i18n.branches.deleteTag }}</button>
```

- [ ] **Step 7: Update the delete-tag ConfirmDialog**

Replace:

```vue
      <ConfirmDialog
        v-if="showDeleteTagConfirm && targetTag"
        :message="`Delete tag '${targetTag.name}'?`"
        confirm-label="Delete"
        danger
        :checkbox-label="hasRemote ? `Also delete on remote '${remotes[0]}'` : undefined"
        @close="showDeleteTagConfirm = false; targetTag = null"
        @confirm="confirmDeleteTag"
      />
```

with:

```vue
      <ConfirmDialog
        v-if="showDeleteTagConfirm && targetTags.length"
        :message="targetTags.length > 1
          ? `Delete ${targetTags.length} tags?`
          : `Delete tag '${targetTags[0].name}'?`"
        :items="targetTags.length > 1 ? targetTags.map((t) => t.name) : undefined"
        confirm-label="Delete"
        danger
        :checkbox-label="hasRemote ? `Also delete on remote '${remotes[0]}'` : undefined"
        @close="showDeleteTagConfirm = false; targetTags = []"
        @confirm="confirmDeleteTag"
      />
```

- [ ] **Step 8: Verify the build**

Run: `npm run build`
Expected: PASS. No remaining references to `targetTag` (singular).

- [ ] **Step 9: Manual check**

Run `npm run tauri:dev`. In a repo with several tags: ctrl/shift+click to select 3 tags, right-click one — the menu reads "Delete 3 Tags". Confirm — the dialog lists all 3 names; deleting removes all 3. Right-click a tag outside the selection — selection collapses to it, menu reads "Delete Tag". Verify "Also delete on remote" still works for a single tag when a remote exists.

- [ ] **Step 10: Commit**

```bash
git add src/components/BranchPanel.vue
git commit -m "feat(branches): batch delete and push for selected tags"
```

---

### Task 4: BranchPanel — batch local-branch delete

Makes the local-branch context menu operate on the whole branch selection; single-target actions are disabled while multiple branches are selected.

**Files:**
- Modify: `src/components/BranchPanel.vue`

- [ ] **Step 1: Add batch target refs and derived branch selections**

Replace:

```ts
const targetBranch = ref<BranchInfo | null>(null);
```

with:

```ts
const targetBranch = ref<BranchInfo | null>(null);
const targetBranches = ref<BranchInfo[]>([]);
const notMergedBranches = ref<BranchInfo[]>([]);
```

(`targetBranch` is still used by the Merge and Rename flows; the new refs cover batch delete.)

Then append, after the `selectedTags` computed added in Task 3 (the end of `<script setup>`):

```ts
// Local branches currently selected, in display order.
const selectedLocalBranches = computed(() =>
  localBranches.value.filter((b) => selectedKeys.value.has(`local:${b.name}`)),
);
// Selected local branches eligible for deletion — the current branch can't be deleted.
const deletableBranches = computed(() =>
  selectedLocalBranches.value.filter((b) => !b.is_current),
);
const multiBranch = computed(() => selectedLocalBranches.value.length > 1);
```

- [ ] **Step 2: Make the branch context menu selection-aware**

Replace `onBranchContextMenu`:

```ts
function onBranchContextMenu(e: MouseEvent, branch: BranchInfo) {
  e.preventDefault();
  e.stopPropagation();
  ctxMenu.value = { x: e.clientX, y: e.clientY };
  ctxBranch.value = branch;
}
```

with:

```ts
function onBranchContextMenu(e: MouseEvent, branch: BranchInfo) {
  e.preventDefault();
  e.stopPropagation();
  // Right-clicking an unselected branch selects just that branch first.
  const key = `local:${branch.name}`;
  if (!selectedKeys.value.has(key)) {
    selectedKeys.value = new Set([key]);
    anchorKey.value = key;
  }
  ctxMenu.value = { x: e.clientX, y: e.clientY };
  ctxBranch.value = branch;
}
```

- [ ] **Step 3: Make delete batch over the selection**

Replace `handleDeleteCtx`:

```ts
function handleDeleteCtx() {
  targetBranch.value = ctxBranch.value;
  closeCtxMenu();
  showDeleteConfirm.value = true;
}
```

with:

```ts
function handleDeleteCtx() {
  targetBranches.value = [...deletableBranches.value];
  closeCtxMenu();
  if (!targetBranches.value.length) return;
  showDeleteConfirm.value = true;
}
```

- [ ] **Step 4: Rewrite `confirmDelete` for the batch**

Replace `confirmDelete`:

```ts
async function confirmDelete() {
  const b = targetBranch.value;
  showDeleteConfirm.value = false;
  if (!b) return;
  try {
    await deleteBranch(b.name, false);
    emit("branchesChanged");
    targetBranch.value = null;
  } catch (e) {
    const msg = String(e);
    if (msg.includes("not fully merged")) {
      showForceDeleteConfirm.value = true;
    } else {
      window.alert(`Delete failed: ${e}`);
      targetBranch.value = null;
    }
  }
}
```

with:

```ts
async function confirmDelete() {
  const targets = targetBranches.value;
  showDeleteConfirm.value = false;
  if (!targets.length) return;
  const errors: string[] = [];
  const notMerged: BranchInfo[] = [];
  for (const b of targets) {
    try {
      await deleteBranch(b.name, false);
    } catch (e) {
      if (String(e).includes("not fully merged")) notMerged.push(b);
      else errors.push(`${b.name}: ${e}`);
    }
  }
  if (errors.length) window.alert(`Delete failed:\n${errors.join("\n")}`);
  emit("branchesChanged");
  if (notMerged.length) {
    notMergedBranches.value = notMerged;
    showForceDeleteConfirm.value = true;
  } else {
    targetBranches.value = [];
    clearSelection();
  }
}
```

- [ ] **Step 5: Rewrite `confirmForceDelete` for the batch**

Replace `confirmForceDelete`:

```ts
async function confirmForceDelete() {
  const b = targetBranch.value;
  showForceDeleteConfirm.value = false;
  if (!b) return;
  try {
    await deleteBranch(b.name, true);
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Delete failed: ${e}`);
  }
  targetBranch.value = null;
}
```

with:

```ts
async function confirmForceDelete() {
  const targets = notMergedBranches.value;
  showForceDeleteConfirm.value = false;
  const errors: string[] = [];
  for (const b of targets) {
    try {
      await deleteBranch(b.name, true);
    } catch (e) {
      errors.push(`${b.name}: ${e}`);
    }
  }
  if (errors.length) window.alert(`Delete failed:\n${errors.join("\n")}`);
  emit("branchesChanged");
  notMergedBranches.value = [];
  targetBranches.value = [];
  clearSelection();
}
```

- [ ] **Step 6: Disable single-target menu items in multi-select**

In the template, in the branch context menu (the first `v-if="ctxMenu"` block), replace the whole list of buttons:

```vue
        <button
          class="ctx-item"
          :disabled="ctxBranch?.is_current"
          @click="handleCheckoutCtx"
        >{{ i18n.branches.checkout }}</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item"
          :disabled="ctxBranch?.is_current"
          @click="handleMergeCtx"
        >{{ i18n.branches.merge }}</button>
        <button
          class="ctx-item"
          :disabled="ctxBranch?.is_current"
          @click="handleRebaseCtx"
        >{{ i18n.branches.rebaseOnto }}</button>
        <button class="ctx-item" @click="handlePushCtx">{{ i18n.branches.push }}</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item"
          @click="handleCreateBranchCtx"
        >{{ i18n.branches.createFrom }}</button>
        <button class="ctx-item" @click="handleRenameCtx">{{ i18n.branches.rename }}</button>
        <button
          class="ctx-item ctx-danger"
          :disabled="ctxBranch?.is_current"
          @click="handleDeleteCtx"
        >{{ i18n.branches.delete }}</button>
```

with:

```vue
        <button
          class="ctx-item"
          :disabled="ctxBranch?.is_current || multiBranch"
          @click="handleCheckoutCtx"
        >{{ i18n.branches.checkout }}</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item"
          :disabled="ctxBranch?.is_current || multiBranch"
          @click="handleMergeCtx"
        >{{ i18n.branches.merge }}</button>
        <button
          class="ctx-item"
          :disabled="ctxBranch?.is_current || multiBranch"
          @click="handleRebaseCtx"
        >{{ i18n.branches.rebaseOnto }}</button>
        <button
          class="ctx-item"
          :disabled="multiBranch"
          @click="handlePushCtx"
        >{{ i18n.branches.push }}</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item"
          :disabled="multiBranch"
          @click="handleCreateBranchCtx"
        >{{ i18n.branches.createFrom }}</button>
        <button
          class="ctx-item"
          :disabled="multiBranch"
          @click="handleRenameCtx"
        >{{ i18n.branches.rename }}</button>
        <button
          class="ctx-item ctx-danger"
          :disabled="!deletableBranches.length"
          @click="handleDeleteCtx"
        >{{ deletableBranches.length > 1 ? `Delete ${deletableBranches.length} Branches` : i18n.branches.delete }}</button>
```

- [ ] **Step 7: Update the delete-branch ConfirmDialog**

Replace:

```vue
      <ConfirmDialog
        v-if="showDeleteConfirm && targetBranch"
        :message="`Delete local branch '${targetBranch.name}'?`"
        confirm-label="Delete"
        danger
        @close="showDeleteConfirm = false; targetBranch = null"
        @confirm="confirmDelete"
      />
```

with:

```vue
      <ConfirmDialog
        v-if="showDeleteConfirm && targetBranches.length"
        :message="targetBranches.length > 1
          ? `Delete ${targetBranches.length} local branches?`
          : `Delete local branch '${targetBranches[0].name}'?`"
        :items="targetBranches.length > 1 ? targetBranches.map((b) => b.name) : undefined"
        confirm-label="Delete"
        danger
        @close="showDeleteConfirm = false; targetBranches = []"
        @confirm="confirmDelete"
      />
```

- [ ] **Step 8: Update the force-delete ConfirmDialog**

Replace:

```vue
      <ConfirmDialog
        v-if="showForceDeleteConfirm && targetBranch"
        :message="`Branch '${targetBranch.name}' is not fully merged. Force delete anyway?`"
        confirm-label="Force Delete"
        danger
        @close="showForceDeleteConfirm = false; targetBranch = null"
        @confirm="confirmForceDelete"
      />
```

with:

```vue
      <ConfirmDialog
        v-if="showForceDeleteConfirm && notMergedBranches.length"
        :message="notMergedBranches.length > 1
          ? `${notMergedBranches.length} branches are not fully merged. Force delete anyway?`
          : `Branch '${notMergedBranches[0].name}' is not fully merged. Force delete anyway?`"
        :items="notMergedBranches.length > 1 ? notMergedBranches.map((b) => b.name) : undefined"
        confirm-label="Force Delete"
        danger
        @close="showForceDeleteConfirm = false; notMergedBranches = []; targetBranches = []"
        @confirm="confirmForceDelete"
      />
```

- [ ] **Step 9: Verify the build**

Run: `npm run build`
Expected: PASS. `targetBranch` is still referenced by the Merge confirm dialog and `RenameBranchDialog`, so it stays declared.

- [ ] **Step 10: Manual check**

Run `npm run tauri:dev`. In a repo with several local branches:
- Select 3 non-current branches, right-click one — menu reads "Delete 3 Branches"; Checkout/Merge/Rebase/Create/Rename/Push are disabled. Confirm — dialog lists all 3; all are deleted.
- Include the current branch in the selection — it is excluded; the count and dialog list only the others.
- Select 2 branches with unmerged commits — after the first confirm, a force-delete dialog lists both; confirming force-deletes them.
- Right-click a branch outside the selection — selection collapses to it; menu actions behave as single-target again.

- [ ] **Step 11: Commit**

```bash
git add src/components/BranchPanel.vue
git commit -m "feat(branches): batch delete for selected local branches"
```

---

### Task 5: Version bump

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Bump the patch version in all three files**

In `package.json`, change `"version": "0.5.10",` to `"version": "0.5.11",`.

In `src-tauri/Cargo.toml`, change `version = "0.5.10"` to `version = "0.5.11"`.

In `src-tauri/tauri.conf.json`, change `"version": "0.5.10",` to `"version": "0.5.11",`.

- [ ] **Step 2: Verify the build**

Run: `npm run build`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to 0.5.11"
```

---

## Notes for the implementer

- `selectItem`, `sectionKeys`, and the derived computeds reference `localBranches` / `remoteBranches` / `filteredTags` / `filteredStashes`, which are declared earlier in `<script setup>`. The selection block must stay below those computed declarations (it replaces the existing `selectItem` block, which is already the last code in the script).
- Always reassign `selectedKeys.value` to a fresh `Set` (never mutate in place) so Vue reactivity fires.
- The `.sections` container already has `@selectstart.prevent` and `.branch-item` has `user-select: none`, so shift-clicking will not select page text.
- Batch operations call the existing single-item composable functions (`deleteTag`, `pushTag`, `deleteBranch`) in a loop; no `useBranches.ts` or Rust changes are needed.
