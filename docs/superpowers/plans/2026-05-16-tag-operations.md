# Tag Operations (Add / Delete / Push) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add full git tag lifecycle to GitStream — create (lightweight/annotated, force), delete (local + optionally remote), and push a single tag — modeled on common desktop git clients.

**Architecture:** Extend existing modules rather than add new subsystems. Rust tag mutations go in `mutation.rs` (local ops sync, push async via `spawn_blocking` like `do_push`). Tag composable functions extend `useBranches.ts` (tags already live there). New `AddTagDialog.vue`; `ConfirmDialog.vue` gains an optional checkbox for "also delete on remote". Entry points: a "Create Tag here…" item in the existing `CommitGraph` context menu, and a `+` button + per-tag context menu in the `BranchPanel` Tags section.

**Tech Stack:** Rust (Tauri 2, `std::process::Command` git wrapper), Vue 3 Composition API + TypeScript, Vite.

**Spec:** `docs/superpowers/specs/2026-05-16-add-tag-design.md`

**Verification note:** The codebase has no automated test harness for the frontend (`npm run build` runs `vue-tsc --noEmit && vite build`). Rust has no existing tests, but `cargo test` needs no infra, so backend tasks add `#[cfg(test)]` tests that drive real git in a throwaway repo (no new dependencies — temp dirs created via `std::env::temp_dir()`). Frontend tasks are verified by `npm run build` (type-check + build) plus a stated manual check.

---

### Task 1: Backend — tag mutation functions

**Files:**
- Modify: `src-tauri/src/git/mutation.rs` (append after `delete_branch`, around line 103)

- [ ] **Step 1: Write the failing tests**

Append to `src-tauri/src/git/mutation.rs`:

```rust
#[cfg(test)]
mod tag_tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn temp_repo() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_tag_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            Command::new("git").current_dir(&dir).args(args).output().unwrap();
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t.t"]);
        run(&["config", "user.name", "t"]);
        fs::write(dir.join("a.txt"), "hello").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "init"]);
        dir
    }

    fn list_tags(dir: &std::path::Path) -> String {
        let out = Command::new("git")
            .current_dir(dir)
            .args(["tag", "-l"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    #[test]
    fn creates_lightweight_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        assert!(list_tags(&dir).contains("v1.0"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn creates_annotated_tag_with_message() {
        let dir = temp_repo();
        create_tag(&dir, "v2.0", Some("release two"), None, false).unwrap();
        let out = Command::new("git")
            .current_dir(&dir)
            .args(["cat-file", "-t", "v2.0"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "tag");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn force_overwrites_existing_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        assert!(create_tag(&dir, "v1.0", None, None, false).is_err());
        create_tag(&dir, "v1.0", None, None, true).unwrap();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn deletes_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        delete_tag(&dir, "v1.0").unwrap();
        assert!(!list_tags(&dir).contains("v1.0"));
        fs::remove_dir_all(&dir).ok();
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test tag_tests 2>&1 | tail -20`
Expected: FAIL — compile error `cannot find function create_tag` / `delete_tag`.

- [ ] **Step 3: Implement the mutation functions**

Insert into `src-tauri/src/git/mutation.rs` immediately after the `delete_branch` function (after line 103, before `clone_repo`):

```rust
pub fn create_tag(
    repo_path: &Path,
    name: &str,
    message: Option<&str>,
    target: Option<&str>,
    force: bool,
) -> Result<(), GitError> {
    let mut args: Vec<&str> = vec!["tag"];
    if force {
        args.push("-f");
    }
    if let Some(msg) = message {
        args.push("-a");
        args.push("-m");
        args.push(msg);
    }
    args.push(name);
    if let Some(t) = target {
        args.push(t);
    }
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn delete_tag(repo_path: &Path, name: &str) -> Result<(), GitError> {
    run_git_mut(repo_path, &["tag", "-d", name])?;
    Ok(())
}

pub fn push_tag(
    repo_path: &Path,
    remote: &str,
    name: &str,
    delete: bool,
) -> Result<String, GitError> {
    let refspec = if delete {
        format!(":refs/tags/{}", name)
    } else {
        format!("refs/tags/{}", name)
    };
    run_git_mut(repo_path, &["push", remote, &refspec])
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test tag_tests 2>&1 | tail -20`
Expected: PASS — `test result: ok. 4 passed`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/git/mutation.rs
git commit -m "feat: add create_tag/delete_tag/push_tag git mutations"
```

---

### Task 2: Backend — Tauri commands + registration

**Files:**
- Modify: `src-tauri/src/commands.rs` (add after `do_delete_branch`, line 135)
- Modify: `src-tauri/src/main.rs` (add to `generate_handler!`, after line 42)

- [ ] **Step 1: Add the command endpoints**

Insert into `src-tauri/src/commands.rs` immediately after `do_delete_branch` (after line 135, before `do_clone`). `do_create_tag`/`do_delete_tag` are sync (local ops); `do_push_tag` is async via the existing `run_with_timeout` helper, matching `do_push`:

```rust
#[tauri::command]
pub fn do_create_tag(
    repo_path: String,
    name: String,
    message: Option<String>,
    target: Option<String>,
    force: bool,
) -> Result<(), String> {
    mutation::create_tag(
        Path::new(&repo_path),
        &name,
        message.as_deref(),
        target.as_deref(),
        force,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn do_delete_tag(repo_path: String, name: String) -> Result<(), String> {
    mutation::delete_tag(Path::new(&repo_path), &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn do_push_tag(
    repo_path: String,
    remote: String,
    name: String,
    delete: bool,
) -> Result<String, String> {
    run_with_timeout(move || {
        mutation::push_tag(Path::new(&repo_path), &remote, &name, delete)
            .map_err(|e| e.to_string())
    }, "push").await
}
```

- [ ] **Step 2: Register the commands**

In `src-tauri/src/main.rs`, in the `tauri::generate_handler![ ... ]` list, add three lines after `commands::do_delete_branch,` (line 42):

```rust
            commands::do_create_tag,
            commands::do_delete_tag,
            commands::do_push_tag,
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo build 2>&1 | tail -10`
Expected: builds with no errors (warnings about unused are acceptable only if pre-existing).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "feat: expose do_create_tag/do_delete_tag/do_push_tag commands"
```

---

### Task 3: Frontend — composable functions

**Files:**
- Modify: `src/composables/useBranches.ts` (add functions before `return`, line 62; add to return object)

- [ ] **Step 1: Add the three functions**

In `src/composables/useBranches.ts`, insert after `pushBranch` (after line 60, before the `return {` on line 62):

```ts
  async function createTag(
    name: string,
    message: string | null,
    target: string | null,
    force: boolean,
  ) {
    if (!repoPath.value) return;
    await invoke("do_create_tag", {
      repoPath: repoPath.value,
      name,
      message,
      target,
      force,
    });
  }

  async function deleteTag(name: string) {
    if (!repoPath.value) return;
    await invoke("do_delete_tag", { repoPath: repoPath.value, name });
  }

  async function pushTag(remote: string, name: string, del: boolean) {
    if (!repoPath.value) return;
    await invoke("do_push_tag", {
      repoPath: repoPath.value,
      remote,
      name,
      delete: del,
    });
  }
```

- [ ] **Step 2: Export them**

Replace the return block (lines 62-66) with:

```ts
  return {
    branches, tags, stashes, remotes,
    refresh, checkout, checkoutRemote,
    mergeBranch, renameBranch, deleteBranch, pushBranch,
    createTag, deleteTag, pushTag,
  };
```

- [ ] **Step 3: Verify types**

Run: `npm run build 2>&1 | tail -15`
Expected: build succeeds (no `vue-tsc` errors).

- [ ] **Step 4: Commit**

```bash
git add src/composables/useBranches.ts
git commit -m "feat: add createTag/deleteTag/pushTag to useBranches composable"
```

---

### Task 4: Frontend — AddTagDialog component

**Files:**
- Create: `src/components/dialogs/AddTagDialog.vue`

Modeled on `RenameBranchDialog.vue` (draggable header via `useDraggable`, Esc-close, autofocus, disabled primary until valid).

- [ ] **Step 1: Create the component**

Create `src/components/dialogs/AddTagDialog.vue` with this exact content:

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{
  target: { oid: string; subject: string } | null;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [payload: { name: string; message: string | null; force: boolean }];
}>();

const name = ref("");
const message = ref("");
const force = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();

// git ref name rules (subset): non-empty, no whitespace or ~^:?*[\, no leading '-'
const INVALID = /[\s~^:?*[\\]/;
const canCreate = computed(() => {
  const t = name.value.trim();
  return t.length > 0 && !t.startsWith("-") && !INVALID.test(t);
});

const targetLabel = computed(() =>
  props.target
    ? `${props.target.oid.slice(0, 9)} ${props.target.subject}`
    : "HEAD",
);

function submit() {
  if (!canCreate.value) return;
  emit("confirm", {
    name: name.value.trim(),
    message: message.value.trim() ? message.value : null,
    force: force.value,
  });
}

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog add-tag-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Add Tag</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-subhint">Tag will point to: <b>{{ targetLabel }}</b></p>

        <div class="form-group">
          <label class="form-label" for="add-tag-name">Tag Name:</label>
          <input
            id="add-tag-name"
            ref="inputRef"
            v-model="name"
            class="form-input"
            type="text"
            placeholder="v1.0.0"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>

        <div class="form-group">
          <label class="form-label" for="add-tag-msg">Message (empty = lightweight):</label>
          <textarea
            id="add-tag-msg"
            v-model="message"
            class="form-input add-tag-msg"
            rows="3"
            @keydown.escape="$emit('close')"
          />
        </div>

        <label class="add-tag-force">
          <input type="checkbox" v-model="force" />
          Force (overwrite if tag exists)
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button class="btn btn-primary" :disabled="!canCreate" @click="submit">
          Create Tag
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-tag-dialog {
  width: 420px;
}
.dialog-subhint {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin-bottom: 12px;
}
.add-tag-msg {
  resize: vertical;
  font-family: inherit;
}
.add-tag-force {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  margin-top: 4px;
}
</style>
```

- [ ] **Step 2: Verify build**

Run: `npm run build 2>&1 | tail -15`
Expected: build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/components/dialogs/AddTagDialog.vue
git commit -m "feat: add AddTagDialog component"
```

---

### Task 5: Frontend — optional checkbox in ConfirmDialog

Adds an optional checkbox to `ConfirmDialog` so tag deletion can offer "Also delete on remote". The `confirm` emit gains a boolean payload (the checkbox state). Existing callers (`confirmMerge`, `confirmDelete`, etc.) take no args and ignore the extra value — runtime- and type-safe.

**Files:**
- Modify: `src/components/dialogs/ConfirmDialog.vue`

- [ ] **Step 1: Replace the component**

Overwrite `src/components/dialogs/ConfirmDialog.vue` with:

```vue
<script setup lang="ts">
import { ref } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{
  message: string;
  confirmLabel?: string;
  danger?: boolean;
  checkboxLabel?: string;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [checkboxChecked: boolean];
}>();

const checked = ref(false);
const { dragStyle, onDragStart } = useDraggable();

function onConfirm() {
  emit("confirm", props.checkboxLabel ? checked.value : false);
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog confirm-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Confirm</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="confirm-message">{{ message }}</p>
        <label v-if="checkboxLabel" class="confirm-checkbox">
          <input type="checkbox" v-model="checked" />
          {{ checkboxLabel }}
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button
          class="btn"
          :class="danger ? 'btn-danger' : 'btn-primary'"
          @click="onConfirm"
        >
          {{ confirmLabel || "Confirm" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.confirm-dialog {
  width: 360px;
}

.confirm-message {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.confirm-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}
</style>
```

- [ ] **Step 2: Verify build (confirms existing callers still type-check)**

Run: `npm run build 2>&1 | tail -15`
Expected: build succeeds — no `vue-tsc` errors in `BranchPanel.vue` / `App.vue` (existing `@confirm="confirmMerge"` handlers remain assignable).

- [ ] **Step 3: Commit**

```bash
git add src/components/dialogs/ConfirmDialog.vue
git commit -m "feat: add optional checkbox to ConfirmDialog"
```

---

### Task 6: Frontend — "Create Tag here…" in CommitGraph context menu

**Files:**
- Modify: `src/components/CommitGraph.vue` (emit list line 9-12; `ctxAction` line 40-44; context menu template line 233-234)

- [ ] **Step 1: Extend the emit signature**

In `src/components/CommitGraph.vue`, replace the `defineEmits` block (lines 9-12):

```ts
const emit = defineEmits<{
  commit: [];
  discard: [];
  createTag: [target: { oid: string; subject: string }];
}>();
```

- [ ] **Step 2: Add the context-menu action handler**

Replace `ctxAction` (lines 40-44) with:

```ts
function ctxAction(action: "commit" | "discard") {
  closeCtxMenu();
  if (action === "commit") emit("commit");
  else emit("discard");
}

function ctxCreateTag() {
  const oid = ctxCommitOid.value;
  closeCtxMenu();
  if (!oid || oid === "__worktree__") return;
  const c = commits.value.find((x) => x.oid === oid);
  emit("createTag", { oid, subject: c?.message ?? "" });
}
```

(`commits` is already destructured from `useLog()` at line 14; `CommitInfo.message` holds the subject line.)

- [ ] **Step 3: Add the menu item**

In the context-menu template, replace the two buttons at lines 233-234 with:

```vue
        <button class="ctx-item" :disabled="!ctxIsWorkingTree" @click="ctxAction('commit')">Commit</button>
        <button class="ctx-item ctx-danger" :disabled="!ctxIsWorkingTree" @click="ctxAction('discard')">Discard</button>
        <div class="ctx-separator" />
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxCreateTag">Create Tag here…</button>
```

(`.ctx-separator` is already styled and used by `BranchPanel`'s shared `.ctx-menu` CSS; reuse is consistent.)

- [ ] **Step 4: Verify build**

Run: `npm run build 2>&1 | tail -15`
Expected: build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/components/CommitGraph.vue
git commit -m "feat: add 'Create Tag here' to CommitGraph context menu"
```

---

### Task 7: Frontend — Tags section button + per-tag context menu in BranchPanel

**Files:**
- Modify: `src/components/BranchPanel.vue` (script: emits, imports, tag ctx state, handlers; template: Tags section header `+`, tag item `@contextmenu`, tag ctx menu, delete confirm with checkbox)

- [ ] **Step 1: Extend emits and imports**

In `src/components/BranchPanel.vue`, replace the `defineEmits` block (lines 9-13):

```ts
const emit = defineEmits<{
  checkoutRemote: [remoteBranch: string];
  checkedOut: [];
  branchesChanged: [];
  tagsChanged: [];
  createTag: [];
}>();
```

Replace the type import (line 7):

```ts
import type { BranchInfo, TagInfo } from "@/types";
```

Add `createTag, deleteTag, pushTag` to the `useBranches()` destructure (lines 21-24):

```ts
const {
  branches, tags, stashes, remotes,
  checkout, mergeBranch, renameBranch, deleteBranch, pushBranch,
  deleteTag, pushTag,
} = useBranches();
```

(Note: `createTag` is invoked from `App.vue`, not here — the `+` button and CommitGraph both route through `App.vue`'s AddTagDialog. `BranchPanel` only emits `createTag`.)

- [ ] **Step 2: Add tag context-menu state and handlers**

In `src/components/BranchPanel.vue`, insert after `confirmForceDelete` (after line 155, before `const filter = ref("")` on line 157):

```ts
// --- Tag context menu ---
const tagCtxMenu = ref<{ x: number; y: number } | null>(null);
const ctxTag = ref<TagInfo | null>(null);
const showDeleteTagConfirm = ref(false);
const targetTag = ref<TagInfo | null>(null);

function onTagContextMenu(e: MouseEvent, tag: TagInfo) {
  e.preventDefault();
  e.stopPropagation();
  tagCtxMenu.value = { x: e.clientX, y: e.clientY };
  ctxTag.value = tag;
}

function closeTagCtxMenu() {
  tagCtxMenu.value = null;
  ctxTag.value = null;
}

const hasRemote = computed(() => remotes.value.length > 0);

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

function handleDeleteTagCtx() {
  targetTag.value = ctxTag.value;
  closeTagCtxMenu();
  showDeleteTagConfirm.value = true;
}

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

- [ ] **Step 3: Add the `+` button to the Tags section header**

In the template, replace the Tags `section-header` block (lines 294-304) with:

```vue
        <div class="section-header" @click="toggleSection('tags')">
          <svg
            class="chevron"
            :class="{ expanded: expandedSections.tags }"
            width="12" height="12" viewBox="0 0 12 12"
          >
            <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span class="section-title">Tags</span>
          <span class="section-count">{{ filteredTags.length }}</span>
          <button
            class="section-add-btn"
            title="Add tag (on HEAD)"
            @click.stop="emit('createTag')"
          >+</button>
        </div>
```

- [ ] **Step 4: Add `@contextmenu` to tag items**

In the template, replace the tag item `<div>` opening tag (lines 306-312) with:

```vue
          <div
            v-for="tag in filteredTags"
            :key="tag.name"
            class="branch-item"
            :class="{ selected: selectedKey === `tag:${tag.name}` }"
            @mousedown="selectItem(`tag:${tag.name}`)"
            @contextmenu="onTagContextMenu($event, tag)"
          >
```

- [ ] **Step 5: Add tag context menu + delete confirm to the Teleport**

In the `<Teleport to="body">` block, insert after the `RenameBranchDialog` element (after line 422, before `</Teleport>` on line 423):

```vue
      <div
        v-if="tagCtxMenu"
        class="ctx-menu"
        :style="{ left: tagCtxMenu.x + 'px', top: tagCtxMenu.y + 'px' }"
        @click.stop
      >
        <button
          class="ctx-item"
          :disabled="!hasRemote"
          @click="handlePushTagCtx"
        >Push Tag</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item ctx-danger"
          @click="handleDeleteTagCtx"
        >Delete Tag</button>
      </div>
      <div
        v-if="tagCtxMenu"
        class="ctx-backdrop"
        @click="closeTagCtxMenu"
        @contextmenu.prevent="closeTagCtxMenu"
      />

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

- [ ] **Step 6: Add `.section-add-btn` style**

In the `<style scoped>` block append (after the `.panel-title` rule near line 444, anywhere inside the scoped style):

```css
.section-add-btn {
  margin-left: auto;
  width: 16px;
  height: 16px;
  line-height: 14px;
  text-align: center;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 3px;
  font-size: 13px;
}
.section-add-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}
```

- [ ] **Step 7: Verify build**

Run: `npm run build 2>&1 | tail -15`
Expected: build succeeds.

- [ ] **Step 8: Commit**

```bash
git add src/components/BranchPanel.vue
git commit -m "feat: add tag '+' button and context menu (push/delete) to BranchPanel"
```

---

### Task 8: Frontend — wire AddTagDialog into App.vue

**Files:**
- Modify: `src/App.vue` (imports ~line 20; state ~line 54; handler near other handlers; BranchPanel/CommitGraph bindings lines 256-277; dialog render near line 326)

- [ ] **Step 1: Import the dialog and useBranches**

In `src/App.vue` add to the dialog imports (after line 20, `import FileCompareDialog ...`):

```ts
import AddTagDialog from "./components/dialogs/AddTagDialog.vue";
```

Verify `useBranches` is imported; if not present in the script, add near the other composable imports (e.g. after line 25 `import { useLog } ...`):

```ts
import { useBranches } from "@/composables/useBranches";
```

- [ ] **Step 2: Add state and handlers**

In `src/App.vue`, after `const showSettingsDialog = ref(false);` (line 54) add:

```ts
const showAddTagDialog = ref(false);
const addTagTarget = ref<{ oid: string; subject: string } | null>(null);
const { createTag } = useBranches();

function openAddTag(target: { oid: string; subject: string } | null) {
  addTagTarget.value = target;
  showAddTagDialog.value = true;
}

async function handleCreateTag(payload: {
  name: string;
  message: string | null;
  force: boolean;
}) {
  const target = addTagTarget.value?.oid ?? null;
  showAddTagDialog.value = false;
  try {
    await createTag(payload.name, payload.message, target, payload.force);
    await refreshAll();
  } catch (e) {
    showError(String(e));
  }
  addTagTarget.value = null;
}
```

(`showError` and `refreshAll` already exist — defined at lines 57 and 35.)

- [ ] **Step 3: Wire BranchPanel and CommitGraph events**

Replace the `<BranchPanel ... />` element (lines 256-260):

```vue
          <BranchPanel
            @checkout-remote="checkoutRemoteTarget = $event"
            @checked-out="refreshAll()"
            @branches-changed="refreshAll()"
            @tags-changed="refreshAll()"
            @create-tag="openAddTag(null)"
          />
```

Replace the `<CommitGraph ... />` element (lines 274-277):

```vue
            <CommitGraph
              @commit="showCommitDialog = true"
              @discard="showDiscardDialog = true"
              @create-tag="openAddTag($event)"
            />
```

- [ ] **Step 4: Render the dialog**

In `src/App.vue`, after the `<SettingsDialog ... />` line (line 327) add:

```vue
    <AddTagDialog
      v-if="showAddTagDialog"
      :target="addTagTarget"
      @close="showAddTagDialog = false; addTagTarget = null"
      @confirm="handleCreateTag"
    />
```

- [ ] **Step 5: Verify build**

Run: `npm run build 2>&1 | tail -15`
Expected: build succeeds.

- [ ] **Step 6: Manual smoke check**

Run: `npm run tauri dev` (or the project's normal dev command). Verify, in a repo with at least one commit and an `origin` remote:
1. Right-click a commit in CommitGraph → "Create Tag here…" → dialog shows target `<short-oid> <subject>` → create lightweight tag → appears in Tags section after refresh.
2. Tags section `+` button → dialog shows target `HEAD` → enter name + message (annotated) → tag appears.
3. Right-click a tag → "Push Tag" → no error (or readable error if no remote).
4. Right-click a tag → "Delete Tag" → ConfirmDialog shows "Also delete on remote 'origin'" checkbox → confirm without checkbox → tag removed locally.
5. Esc closes AddTagDialog; header drag moves it.

- [ ] **Step 7: Commit**

```bash
git add src/App.vue
git commit -m "feat: wire AddTagDialog and tag refresh into App"
```

---

### Task 9: Version bump + final verification

**Files:**
- Modify: `src-tauri/tauri.conf.json` (line 4, `"version"`)

- [ ] **Step 1: Bump patch version**

In `src-tauri/tauri.conf.json` change line 4 from `"version": "0.1.7",` to:

```json
  "version": "0.1.8",
```

(Per project rule: increment patch after each code change.)

- [ ] **Step 2: Full verification**

Run: `npm run build 2>&1 | tail -5 && cd src-tauri && cargo test tag_tests 2>&1 | tail -5 && cargo build 2>&1 | tail -5`
Expected: `vite build` succeeds, `test result: ok. 4 passed`, `cargo build` finishes with no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: bump version to 0.1.8"
```

---

## Self-Review

**Spec coverage:**
- Create (lightweight/annotated/force) — Task 1 (mutation) + 2 (cmd) + 3 (composable) + 4 (dialog) + 6/7 (entry points) + 8 (wiring). ✓
- Delete (local + optional remote) — Task 1, 2, 3, 5 (checkbox), 7 (handler `confirmDeleteTag`). ✓
- Push single tag — Task 1 (`push_tag`), 2 (`do_push_tag` async), 3 (`pushTag`), 7 (`handlePushTagCtx`). ✓
- Entry point: CommitGraph commit context menu — Task 6 + 8. ✓
- Entry point: Tags section button + per-tag menu — Task 7. ✓
- Reference UX model (no push in Add dialog; message-driven annotated; Force) — Task 4. ✓
- Async network push via spawn_blocking — Task 2 uses existing `run_with_timeout`. ✓
- Draggable dialog — Task 4 uses `useDraggable`. ✓
- Error handling via alert / showError + classify_git_error — Tasks 7, 8. ✓
- Edge case: working-tree row disables "Create Tag here" — Task 6 (`:disabled="ctxIsWorkingTree"` + guard in `ctxCreateTag`). ✓
- Edge case: no remote hides remote checkbox / disables Push — Task 7 (`hasRemote`, `:disabled`, conditional `checkbox-label`). ✓
- Edge case: name validation on frontend — Task 4 (`canCreate` regex). ✓
- Version bump rule — Task 9. ✓

**Placeholder scan:** No TBD/TODO; every code step contains complete code. No "add error handling" hand-waving — explicit try/catch shown.

**Type consistency:** `createTag(name, message, target, force)` signature identical in Task 3 (definition) and Task 8 (call). `confirm` payload `{ name, message, force }` emitted in Task 4 matches `handleCreateTag` param in Task 8. `ConfirmDialog` `confirm: [checkboxChecked: boolean]` (Task 5) matches `confirmDeleteTag(alsoRemote: boolean)` (Task 7) and is back-compat with arg-less handlers. `createTag`/`deleteTag`/`pushTag` exported in Task 3 and consumed in Tasks 7/8. `do_push_tag` returns `Result<String, String>` consistent with `push_tag` returning `Result<String, GitError>`.
