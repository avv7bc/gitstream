<script setup lang="ts">
import { ref, computed } from "vue";
import { useBranches } from "@/composables/useBranches";
import { highlight } from "@/utils/highlight";
import ConfirmDialog from "@/components/dialogs/ConfirmDialog.vue";
import RenameBranchDialog from "@/components/dialogs/RenameBranchDialog.vue";
import StashSaveDialog from "@/components/dialogs/StashSaveDialog.vue";
import CreateBranchDialog from "@/components/dialogs/CreateBranchDialog.vue";
import RefIcon from "@/components/RefIcon.vue";
import type { BranchInfo, TagInfo, StashEntry } from "@/types";
import { useI18n } from "@/composables/useI18n";
const { i18n } = useI18n();

const emit = defineEmits<{
  checkoutRemote: [remoteBranch: string];
  checkedOut: [];
  branchesChanged: [];
  tagsChanged: [];
  createTag: [];
}>();

async function handleLocalDblClick(branch: { name: string; is_current: boolean }) {
  if (branch.is_current) return;
  await checkout(branch.name);
  emit("checkedOut");
}

const {
  branches, tags, stashes, remotes,
  checkout, createBranch, mergeBranch, rebaseOnto, renameBranch, deleteBranch, pushBranch,
  deleteTag, pushTag,
  stashSave, stashApply, stashPop, stashDrop,
} = useBranches();

// --- Create branch ---
const showCreateBranchDialog = ref(false);
const createBranchStart = ref<string | null>(null);

function openCreateBranch(start: string | null) {
  createBranchStart.value = start;
  showCreateBranchDialog.value = true;
}

function handleCreateBranchCtx() {
  const b = ctxBranch.value;
  closeCtxMenu();
  openCreateBranch(b ? b.name : null);
}

async function confirmCreateBranch(payload: {
  name: string;
  startPoint: string | null;
  checkout: boolean;
}) {
  showCreateBranchDialog.value = false;
  try {
    await createBranch(payload.name, payload.startPoint, payload.checkout);
    if (payload.checkout) emit("checkedOut");
    else emit("branchesChanged");
  } catch (e) {
    window.alert(`Create branch failed: ${e}`);
  }
}

// --- Context menu for local branches ---
const ctxMenu = ref<{ x: number; y: number } | null>(null);
const ctxBranch = ref<BranchInfo | null>(null);

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

function closeCtxMenu() {
  ctxMenu.value = null;
  ctxBranch.value = null;
}

function resolveRemoteFor(branch: BranchInfo): string {
  if (branch.upstream) {
    const idx = branch.upstream.indexOf("/");
    if (idx > 0) return branch.upstream.slice(0, idx);
  }
  return remotes.value[0] ?? "origin";
}

// --- Actions ---
const showMergeConfirm = ref(false);
const showDeleteConfirm = ref(false);
const showForceDeleteConfirm = ref(false);
const showRenameDialog = ref(false);
const targetBranch = ref<BranchInfo | null>(null);
const targetBranches = ref<BranchInfo[]>([]);
const notMergedBranches = ref<BranchInfo[]>([]);

async function handleCheckoutCtx() {
  const b = ctxBranch.value;
  closeCtxMenu();
  if (!b || b.is_current) return;
  try {
    await checkout(b.name);
    emit("checkedOut");
  } catch (e) {
    window.alert(`Checkout failed: ${e}`);
  }
}

function handleMergeCtx() {
  targetBranch.value = ctxBranch.value;
  closeCtxMenu();
  showMergeConfirm.value = true;
}

async function confirmMerge() {
  const b = targetBranch.value;
  showMergeConfirm.value = false;
  if (!b) return;
  try {
    await mergeBranch(b.name);
  } catch (e) {
    window.alert(`Merge failed (resolve conflicts below if any): ${e}`);
  } finally {
    emit("branchesChanged");
  }
  targetBranch.value = null;
}

async function handlePushCtx() {
  const b = ctxBranch.value;
  closeCtxMenu();
  if (!b) return;
  try {
    await pushBranch(b.name, resolveRemoteFor(b), false);
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Push failed: ${e}`);
  }
}

async function handleRebaseCtx() {
  const b = ctxBranch.value;
  closeCtxMenu();
  if (!b || b.is_current) return;
  if (!window.confirm(`Rebase current branch onto '${b.name}'?`)) return;
  try {
    await rebaseOnto(b.name);
  } catch (e) {
    window.alert(`Rebase failed (resolve conflicts below if any): ${e}`);
  } finally {
    emit("branchesChanged");
  }
}

function handleRenameCtx() {
  targetBranch.value = ctxBranch.value;
  closeCtxMenu();
  showRenameDialog.value = true;
}

async function confirmRename(newName: string) {
  const b = targetBranch.value;
  showRenameDialog.value = false;
  if (!b) return;
  try {
    await renameBranch(b.name, newName);
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Rename failed: ${e}`);
  }
  targetBranch.value = null;
}

function handleDeleteCtx() {
  targetBranches.value = [...deletableBranches.value];
  closeCtxMenu();
  if (!targetBranches.value.length) return;
  showDeleteConfirm.value = true;
}

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

// --- Tag context menu ---
const tagCtxMenu = ref<{ x: number; y: number } | null>(null);
const ctxTag = ref<TagInfo | null>(null);
const showDeleteTagConfirm = ref(false);
const targetTags = ref<TagInfo[]>([]);

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

function closeTagCtxMenu() {
  tagCtxMenu.value = null;
  ctxTag.value = null;
}

const hasRemote = computed(() => remotes.value.length > 0);

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

function handleDeleteTagCtx() {
  targetTags.value = [...selectedTags.value];
  closeTagCtxMenu();
  if (!targetTags.value.length) return;
  showDeleteTagConfirm.value = true;
}

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

// --- Stash context menu ---
const stashCtxMenu = ref<{ x: number; y: number } | null>(null);
const ctxStash = ref<StashEntry | null>(null);
const showStashSaveDialog = ref(false);
const showDropStashConfirm = ref(false);
const targetStash = ref<StashEntry | null>(null);

function onStashContextMenu(e: MouseEvent, stash: StashEntry) {
  e.preventDefault();
  e.stopPropagation();
  stashCtxMenu.value = { x: e.clientX, y: e.clientY };
  ctxStash.value = stash;
}

function closeStashCtxMenu() {
  stashCtxMenu.value = null;
  ctxStash.value = null;
}

async function handleStashApplyCtx() {
  const s = ctxStash.value;
  closeStashCtxMenu();
  if (!s) return;
  try {
    await stashApply(s.index);
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Apply stash failed: ${e}`);
  }
}

async function handleStashPopCtx() {
  const s = ctxStash.value;
  closeStashCtxMenu();
  if (!s) return;
  try {
    await stashPop(s.index);
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Pop stash failed: ${e}`);
  }
}

function handleStashDropCtx() {
  targetStash.value = ctxStash.value;
  closeStashCtxMenu();
  showDropStashConfirm.value = true;
}

async function confirmDropStash() {
  const s = targetStash.value;
  showDropStashConfirm.value = false;
  if (!s) {
    targetStash.value = null;
    return;
  }
  try {
    await stashDrop(s.index);
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Drop stash failed: ${e}`);
  }
  targetStash.value = null;
}

async function confirmStashSave(payload: {
  message: string | null;
  includeUntracked: boolean;
}) {
  showStashSaveDialog.value = false;
  try {
    await stashSave(payload.message, payload.includeUntracked);
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Stash failed: ${e}`);
  }
}

const filter = ref("");
const expandedSections = ref({
  local: true,
  remote: true,
  tags: false,
  stashes: false,
});

const q = computed(() => filter.value.toLowerCase());

const localBranches = computed(() => {
  const list = branches.value.filter((b) => !b.is_remote);
  if (!q.value) return list;
  return list.filter((b) =>
    b.name.toLowerCase().includes(q.value) ||
    (b.upstream && b.upstream.toLowerCase().includes(q.value))
  );
});
const remoteBranches = computed(() => {
  const list = branches.value.filter((b) => b.is_remote);
  if (!q.value) return list;
  return list.filter((b) => b.name.toLowerCase().includes(q.value));
});
const filteredTags = computed(() => {
  if (!q.value) return tags.value;
  return tags.value.filter((t) =>
    t.name.toLowerCase().includes(q.value) ||
    t.oid.toLowerCase().includes(q.value) ||
    (t.message && t.message.toLowerCase().includes(q.value))
  );
});
const filteredStashes = computed(() => {
  if (!q.value) return stashes.value;
  return stashes.value.filter((s) =>
    s.message.toLowerCase().includes(q.value) ||
    s.date.toLowerCase().includes(q.value)
  );
});

function toggleSection(key: keyof typeof expandedSections.value) {
  expandedSections.value[key] = !expandedSections.value[key];
}

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
  // Only the primary (left) button changes selection. A right-button
  // mousedown fires before `contextmenu`; if it ran the plain-click path
  // it would collapse a multi-selection before the context menu opens.
  if (event.button !== 0) return;
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

function clearSelection() {
  selectedKeys.value = new Set();
  anchorKey.value = null;
}

// Tags currently selected, in display order.
const selectedTags = computed(() =>
  filteredTags.value.filter((t) => selectedKeys.value.has(`tag:${t.name}`)),
);

// Local branches currently selected, in display order.
const selectedLocalBranches = computed(() =>
  localBranches.value.filter((b) => selectedKeys.value.has(`local:${b.name}`)),
);
// Selected local branches eligible for deletion — the current branch can't be deleted.
const deletableBranches = computed(() =>
  selectedLocalBranches.value.filter((b) => !b.is_current),
);
const multiBranch = computed(() => selectedLocalBranches.value.length > 1);
</script>

<template>
  <div class="branch-panel" @contextmenu.prevent>
    <div class="panel-title-bar">
      <span class="panel-title">{{ i18n.branches.title }}</span>
      <input
        v-model="filter"
        type="text"
        :placeholder="i18n.branches.filter"
        class="filter-input"
      />
    </div>

    <div class="sections" @selectstart.prevent>
      <!-- Local Branches -->
      <div class="section">
        <div class="section-header" @click="toggleSection('local')">
          <svg
            class="chevron"
            :class="{ expanded: expandedSections.local }"
            width="12" height="12" viewBox="0 0 12 12"
          >
            <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span class="section-title">{{ i18n.branches.localBranches }}</span>
          <span class="section-count">{{ localBranches.length }}</span>
          <button
            class="section-action"
            :title="i18n.branches.createBranch"
            @click.stop="openCreateBranch(null)"
          >+</button>
        </div>
        <div v-if="expandedSections.local" class="section-items">
          <div
            v-for="branch in localBranches"
            :key="branch.name"
            class="branch-item"
            :class="{ current: branch.is_current, selected: selectedKeys.has(`local:${branch.name}`) }"
            @mousedown="selectItem(`local:${branch.name}`, $event)"
            @dblclick="handleLocalDblClick(branch)"
            @contextmenu="onBranchContextMenu($event, branch)"
          >
            <RefIcon kind="local-branch" class="bp-icon bp-icon--branch" />
            <span class="branch-name" v-html="highlight(branch.name, filter)" />
            <span v-if="branch.ahead > 0" class="ahead-badge" :title="`${branch.ahead} ahead`">
              {{ branch.ahead }}
            </span>
            <span v-if="branch.behind > 0" class="behind-badge" :title="`${branch.behind} behind`">
              {{ branch.behind }}
            </span>
          </div>
        </div>
      </div>

      <!-- Remote Branches -->
      <div class="section">
        <div class="section-header" @click="toggleSection('remote')">
          <svg
            class="chevron"
            :class="{ expanded: expandedSections.remote }"
            width="12" height="12" viewBox="0 0 12 12"
          >
            <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span class="section-title">{{ i18n.branches.remoteBranches }}</span>
          <span class="section-count">{{ remoteBranches.length }}</span>
        </div>
        <div v-if="expandedSections.remote && remoteBranches.length" class="section-items">
          <div
            v-for="branch in remoteBranches"
            :key="branch.name"
            class="branch-item"
            :class="{ selected: selectedKeys.has(`remote:${branch.name}`) }"
            @mousedown="selectItem(`remote:${branch.name}`, $event)"
            @dblclick="emit('checkoutRemote', branch.name)"
          >
            <RefIcon kind="remote-branch" class="bp-icon bp-icon--remote" />
            <span class="branch-name" v-html="highlight(branch.name, filter)" />
          </div>
        </div>
      </div>

      <!-- Tags -->
      <div class="section">
        <div class="section-header" @click="toggleSection('tags')">
          <svg
            class="chevron"
            :class="{ expanded: expandedSections.tags }"
            width="12" height="12" viewBox="0 0 12 12"
          >
            <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span class="section-title">{{ i18n.branches.tags }}</span>
          <span class="section-count">{{ filteredTags.length }}</span>
          <button
            class="section-add-btn"
            :title="i18n.branches.addTag"
            @click.stop="emit('createTag')"
          >+</button>
        </div>
        <div v-if="expandedSections.tags && filteredTags.length" class="section-items">
          <div
            v-for="tag in filteredTags"
            :key="tag.name"
            class="branch-item"
            :class="{ selected: selectedKeys.has(`tag:${tag.name}`) }"
            @mousedown="selectItem(`tag:${tag.name}`, $event)"
            @contextmenu="onTagContextMenu($event, tag)"
          >
            <RefIcon kind="tag" class="bp-icon bp-icon--tag" />
            <span class="branch-name" v-html="highlight(tag.name, filter)" />
          </div>
        </div>
      </div>

      <!-- Stashes -->
      <div class="section">
        <div class="section-header" @click="toggleSection('stashes')">
          <svg
            class="chevron"
            :class="{ expanded: expandedSections.stashes }"
            width="12" height="12" viewBox="0 0 12 12"
          >
            <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span class="section-title">{{ i18n.branches.stashes }}</span>
          <span class="section-count">{{ filteredStashes.length }}</span>
          <button
            class="section-action"
            :title="i18n.branches.stashChanges"
            @click.stop="showStashSaveDialog = true"
          >+</button>
        </div>
        <div v-if="expandedSections.stashes && filteredStashes.length" class="section-items">
          <div
            v-for="stash in filteredStashes"
            :key="stash.index"
            class="branch-item stash-item"
            :class="{ selected: selectedKeys.has(`stash:${stash.index}`) }"
            @mousedown="selectItem(`stash:${stash.index}`, $event)"
            @contextmenu="onStashContextMenu($event, stash)"
          >
            <RefIcon kind="stash" class="bp-icon bp-icon--stash" />
            <div class="stash-info">
              <span class="branch-name" v-html="highlight(stash.message, filter)" />
              <span class="stash-date" v-html="highlight(stash.date, filter)" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Context menu for local branches -->
    <Teleport to="body">
      <div
        v-if="ctxMenu"
        class="ctx-menu"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @click.stop
      >
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
      </div>
      <div
        v-if="ctxMenu"
        class="ctx-backdrop"
        @click="closeCtxMenu"
        @contextmenu.prevent="closeCtxMenu"
      />

      <ConfirmDialog
        v-if="showMergeConfirm && targetBranch"
        :message="`Merge branch '${targetBranch.name}' into the current branch?`"
        confirm-label="Merge"
        @close="showMergeConfirm = false; targetBranch = null"
        @confirm="confirmMerge"
      />

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

      <RenameBranchDialog
        v-if="showRenameDialog && targetBranch"
        :old-name="targetBranch.name"
        @close="showRenameDialog = false; targetBranch = null"
        @confirm="confirmRename"
      />

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
        >{{ selectedTags.length > 1 ? `Push ${selectedTags.length} Tags` : i18n.branches.pushTag }}</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item ctx-danger"
          @click="handleDeleteTagCtx"
        >{{ selectedTags.length > 1 ? `Delete ${selectedTags.length} Tags` : i18n.branches.deleteTag }}</button>
      </div>
      <div
        v-if="tagCtxMenu"
        class="ctx-backdrop"
        @click="closeTagCtxMenu"
        @contextmenu.prevent="closeTagCtxMenu"
      />

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

      <div
        v-if="stashCtxMenu"
        class="ctx-menu"
        :style="{ left: stashCtxMenu.x + 'px', top: stashCtxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" @click="handleStashApplyCtx">{{ i18n.branches.apply }}</button>
        <button class="ctx-item" @click="handleStashPopCtx">{{ i18n.branches.pop }}</button>
        <div class="ctx-separator" />
        <button class="ctx-item ctx-danger" @click="handleStashDropCtx">{{ i18n.branches.drop }}</button>
      </div>
      <div
        v-if="stashCtxMenu"
        class="ctx-backdrop"
        @click="closeStashCtxMenu"
        @contextmenu.prevent="closeStashCtxMenu"
      />

      <ConfirmDialog
        v-if="showDropStashConfirm && targetStash"
        :message="`Drop stash '${targetStash.message}'?`"
        confirm-label="Drop"
        danger
        @close="showDropStashConfirm = false; targetStash = null"
        @confirm="confirmDropStash"
      />

      <StashSaveDialog
        v-if="showStashSaveDialog"
        @close="showStashSaveDialog = false"
        @confirm="confirmStashSave"
      />

      <CreateBranchDialog
        v-if="showCreateBranchDialog"
        :default-start="createBranchStart"
        @close="showCreateBranchDialog = false"
        @confirm="confirmCreateBranch"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.panel-title-bar {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
  flex-shrink: 0;
}
.panel-title {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.branch-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.filter-input {
  width: 120px;
  margin-left: auto;
  padding: 2px 6px;
  font-size: var(--font-size-xs);
  border-color: var(--border);
}

.sections {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.section {
  margin-bottom: 2px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  user-select: none;
}
.section-header:hover {
  background: var(--bg-hover);
}

.chevron {
  transition: transform 0.15s;
  flex-shrink: 0;
}
.chevron.expanded {
  transform: rotate(90deg);
}

.section-title {
  flex: 1;
}

.section-count {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  font-weight: 400;
}

.section-action {
  margin-left: 6px;
  width: 16px;
  height: 16px;
  line-height: 14px;
  border-radius: var(--radius);
  border: 1px solid var(--border-subtle);
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;
}

.section-action:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.section-items {
  padding: 2px 0;
}

.branch-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px 3px 24px;
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  user-select: none;
}
.branch-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.branch-item.selected {
  background: var(--bg-surface);
  color: var(--text-primary);
}
.branch-item.current {
  color: var(--accent);
  font-weight: 600;
}

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

.branch-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ahead-badge, .behind-badge {
  font-size: var(--font-size-xs);
  padding: 0 4px;
  border-radius: 8px;
  line-height: 16px;
}
.ahead-badge {
  background: rgba(166, 227, 161, 0.15);
  color: var(--green);
}
.ahead-badge::before { content: "\2191"; }
.behind-badge {
  background: rgba(243, 139, 168, 0.15);
  color: var(--red);
}
.behind-badge::before { content: "\2193"; }

.stash-item {
  align-items: flex-start;
}
.stash-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow: hidden;
}
.stash-date {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
}

.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
}
.ctx-menu {
  position: fixed;
  z-index: 1000;
  min-width: 160px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
}
.ctx-item {
  display: block;
  width: 100%;
  padding: 6px 12px;
  text-align: left;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  background: none;
  border: none;
  cursor: pointer;
}
.ctx-item:hover:not(:disabled) {
  background: var(--bg-hover);
}
.ctx-item:disabled {
  color: var(--text-muted);
  cursor: default;
}
.ctx-danger {
  color: var(--red);
}
.ctx-danger:hover:not(:disabled) {
  background: rgba(243, 139, 168, 0.1);
}
.ctx-separator {
  height: 0;
  border-top: 1px solid var(--border);
  margin: 0;
}

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
</style>
