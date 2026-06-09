<script setup lang="ts">
import { ref, computed } from "vue";
import { useBranches } from "@/composables/useBranches";
import { useRemote } from "@/composables/useRemote";
import { logError } from "@/composables/useProgress";
import { highlight } from "@/utils/highlight";
import ConfirmDialog from "@/components/dialogs/ConfirmDialog.vue";
import RenameBranchDialog from "@/components/dialogs/RenameBranchDialog.vue";
import StashSaveDialog from "@/components/dialogs/StashSaveDialog.vue";
import CreateBranchDialog from "@/components/dialogs/CreateBranchDialog.vue";
import RemoteDialog from "@/components/dialogs/RemoteDialog.vue";
import SetUpstreamDialog from "@/components/dialogs/SetUpstreamDialog.vue";
import RefIcon from "@/components/RefIcon.vue";
import type { BranchInfo, TagInfo, StashEntry, RemoteInfo } from "@/types";
import { useI18n } from "@/composables/useI18n";
const { i18n } = useI18n();

// Текст подсказки для ветки: имя + автор tip-коммита (ближайшее, что Git даёт
// о «создателе» ветки).
function branchTooltip(branch: BranchInfo): string {
  const lines = [branch.name];
  if (branch.author) lines.push(`${i18n.value.branches.author}: ${branch.author}`);
  return lines.join("\n");
}

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
  branches, tags, stashes, remotes, remoteUrls,
  checkout, createBranch, mergeBranch, rebaseOnto, renameBranch, deleteBranch, deleteRemoteBranch, pushBranch,
  deleteTag, pushTag,
  stashSave, stashApply, stashPop, stashDrop,
  addRemote, removeRemote, renameRemote, setRemoteUrl, setBranchUpstream,
} = useBranches();
const { fetchRemote } = useRemote();

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

function triggerCheckoutRemote() {
  const b = selectedRemoteBranches.value[0];
  if (!b) return;
  emit("checkoutRemote", b.name);
}

function triggerMerge() {
  const remote = selectedRemoteBranches.value[0];
  const local = selectedLocalBranches.value.find((b) => !b.is_current);
  const b = remote ?? local;
  if (!b) return;
  targetBranch.value = b;
  showMergeConfirm.value = true;
}

async function triggerRebase() {
  const remote = selectedRemoteBranches.value[0];
  const local = selectedLocalBranches.value.find((b) => !b.is_current);
  const b = remote ?? local;
  if (!b) return;
  if (!window.confirm(`Rebase current branch onto '${b.name}'?`)) return;
  try {
    await rebaseOnto(b.name);
  } catch (e) {
    logError(`Rebase failed (resolve conflicts below if any): ${e}`);
  } finally {
    emit("branchesChanged");
  }
}

defineExpose({ openCreateBranch, triggerCheckoutRemote, triggerMerge, triggerRebase });

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
    logError(`Create branch failed: ${e}`);
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
    logError(`Checkout failed: ${e}`);
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
    logError(`Merge failed (resolve conflicts below if any): ${e}`);
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
    logError(`Push failed: ${e}`);
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
    logError(`Rebase failed (resolve conflicts below if any): ${e}`);
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
    logError(`Rename failed: ${e}`);
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
  if (errors.length) logError(`Delete failed:\n${errors.join("\n")}`);
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
  if (errors.length) logError(`Delete failed:\n${errors.join("\n")}`);
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
  if (errors.length) logError(`Push tag failed:\n${errors.join("\n")}`);
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
  if (errors.length) logError(`Delete tag failed:\n${errors.join("\n")}`);
  clearSelection();
  emit("tagsChanged");
  targetTags.value = [];
}

// --- Remotes (управление: add/edit-url/rename/remove + fetch) ---
const remoteMgmtCtxMenu = ref<{ x: number; y: number } | null>(null);
const ctxRemote = ref<RemoteInfo | null>(null);
const showRemoteDialog = ref(false);
const remoteDialogMode = ref<"add" | "editUrl" | "rename">("add");
const remoteDialogTarget = ref<RemoteInfo | null>(null);
const showRemoveRemoteConfirm = ref(false);

function onRemoteMgmtContextMenu(e: MouseEvent, remote: RemoteInfo) {
  e.preventDefault();
  e.stopPropagation();
  remoteMgmtCtxMenu.value = { x: e.clientX, y: e.clientY };
  ctxRemote.value = remote;
}

function closeRemoteMgmtCtxMenu() {
  remoteMgmtCtxMenu.value = null;
  ctxRemote.value = null;
}

function openAddRemote() {
  remoteDialogMode.value = "add";
  remoteDialogTarget.value = null;
  showRemoteDialog.value = true;
}

function openEditRemoteUrl() {
  remoteDialogTarget.value = ctxRemote.value;
  remoteDialogMode.value = "editUrl";
  closeRemoteMgmtCtxMenu();
  showRemoteDialog.value = true;
}

function openRenameRemote() {
  remoteDialogTarget.value = ctxRemote.value;
  remoteDialogMode.value = "rename";
  closeRemoteMgmtCtxMenu();
  showRemoteDialog.value = true;
}

async function confirmRemoteDialog(name: string, url: string) {
  const mode = remoteDialogMode.value;
  const target = remoteDialogTarget.value;
  showRemoteDialog.value = false;
  try {
    if (mode === "add") await addRemote(name, url);
    else if (mode === "editUrl" && target) await setRemoteUrl(target.name, url);
    else if (mode === "rename" && target) await renameRemote(target.name, name);
    emit("branchesChanged");
  } catch (e) {
    logError(`Remote operation failed: ${e}`);
  }
  remoteDialogTarget.value = null;
}

function handleRemoveRemoteCtx() {
  remoteDialogTarget.value = ctxRemote.value;
  closeRemoteMgmtCtxMenu();
  if (!remoteDialogTarget.value) return;
  showRemoveRemoteConfirm.value = true;
}

async function confirmRemoveRemote() {
  const target = remoteDialogTarget.value;
  showRemoveRemoteConfirm.value = false;
  if (!target) return;
  try {
    await removeRemote(target.name);
    emit("branchesChanged");
  } catch (e) {
    logError(`Remove remote failed: ${e}`);
  }
  remoteDialogTarget.value = null;
}

async function handleFetchRemoteCtx(prune: boolean) {
  const r = ctxRemote.value;
  closeRemoteMgmtCtxMenu();
  if (!r) return;
  await fetchRemote(r.name, prune);
  emit("branchesChanged");
}

// --- Set upstream for a local branch ---
const showSetUpstreamDialog = ref(false);
const upstreamBranch = ref<BranchInfo | null>(null);

function handleSetUpstreamCtx() {
  upstreamBranch.value = ctxBranch.value;
  closeCtxMenu();
  if (!upstreamBranch.value) return;
  showSetUpstreamDialog.value = true;
}

async function confirmSetUpstream(upstream: string | null) {
  const b = upstreamBranch.value;
  showSetUpstreamDialog.value = false;
  if (!b) return;
  try {
    await setBranchUpstream(b.name, upstream);
    emit("branchesChanged");
  } catch (e) {
    logError(`Set upstream failed: ${e}`);
  }
  upstreamBranch.value = null;
}

// Имена remote-веток для выбора upstream (origin/main, ...).
const remoteBranchNames = computed(() => remoteBranches.value.map((b) => b.name));

const filteredRemotes = computed(() => {
  if (!q.value) return remoteUrls.value;
  return remoteUrls.value.filter(
    (r) => r.name.toLowerCase().includes(q.value) || r.url.toLowerCase().includes(q.value),
  );
});

// --- Remote branch context menu ---
const remoteCtxMenu = ref<{ x: number; y: number } | null>(null);
const remoteCtxBranch = ref<BranchInfo | null>(null);
const showDeleteRemoteConfirm = ref(false);
const targetRemoteBranches = ref<BranchInfo[]>([]);

const selectedRemoteBranches = computed(() =>
  remoteBranches.value.filter((b) => selectedKeys.value.has(`remote:${b.name}`)),
);

function parseRemoteBranch(fullName: string): { remote: string; branch: string } {
  const idx = fullName.indexOf("/");
  return idx > 0
    ? { remote: fullName.slice(0, idx), branch: fullName.slice(idx + 1) }
    : { remote: fullName, branch: fullName };
}

function onRemoteBranchContextMenu(e: MouseEvent, branch: BranchInfo) {
  e.preventDefault();
  e.stopPropagation();
  const key = `remote:${branch.name}`;
  if (!selectedKeys.value.has(key)) {
    selectedKeys.value = new Set([key]);
    anchorKey.value = key;
  }
  remoteCtxMenu.value = { x: e.clientX, y: e.clientY };
  remoteCtxBranch.value = branch;
}

function closeRemoteCtxMenu() {
  remoteCtxMenu.value = null;
  remoteCtxBranch.value = null;
}

function handleCheckoutRemoteCtx() {
  const b = remoteCtxBranch.value;
  closeRemoteCtxMenu();
  if (!b) return;
  emit("checkoutRemote", b.name);
}

function handleMergeRemoteCtx() {
  const b = remoteCtxBranch.value;
  closeRemoteCtxMenu();
  if (!b) return;
  targetBranch.value = b;
  showMergeConfirm.value = true;
}

async function handleRebaseRemoteCtx() {
  const b = remoteCtxBranch.value;
  closeRemoteCtxMenu();
  if (!b) return;
  if (!window.confirm(`Rebase current branch onto '${b.name}'?`)) return;
  try {
    await rebaseOnto(b.name);
  } catch (e) {
    logError(`Rebase failed (resolve conflicts below if any): ${e}`);
  } finally {
    emit("branchesChanged");
  }
}

function handleDeleteRemoteCtx() {
  targetRemoteBranches.value = [...selectedRemoteBranches.value];
  closeRemoteCtxMenu();
  if (!targetRemoteBranches.value.length) return;
  showDeleteRemoteConfirm.value = true;
}

async function confirmDeleteRemote() {
  const targets = targetRemoteBranches.value;
  showDeleteRemoteConfirm.value = false;
  if (!targets.length) return;
  const errors: string[] = [];
  for (const b of targets) {
    const { remote, branch } = parseRemoteBranch(b.name);
    try {
      await deleteRemoteBranch(remote, branch);
    } catch (e) {
      errors.push(`${b.name}: ${e}`);
    }
  }
  if (errors.length) logError(`Delete remote branch failed:\n${errors.join("\n")}`);
  clearSelection();
  emit("branchesChanged");
  targetRemoteBranches.value = [];
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
    logError(`Apply stash failed: ${e}`);
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
    logError(`Pop stash failed: ${e}`);
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
    logError(`Drop stash failed: ${e}`);
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
    logError(`Stash failed: ${e}`);
  }
}

const filter = ref("");
const expandedSections = ref({
  local: true,
  remote: true,
  remotes: false,
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
            v-tooltip="branchTooltip(branch)"
            @mousedown="selectItem(`local:${branch.name}`, $event)"
            @dblclick="handleLocalDblClick(branch)"
            @contextmenu="onBranchContextMenu($event, branch)"
          >
            <RefIcon kind="local-branch" class="bp-icon bp-icon--branch" />
            <span class="branch-arrow">›</span>
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
            v-tooltip="branchTooltip(branch)"
            @mousedown="selectItem(`remote:${branch.name}`, $event)"
            @dblclick="emit('checkoutRemote', branch.name)"
            @contextmenu="onRemoteBranchContextMenu($event, branch)"
          >
            <RefIcon kind="remote-branch" class="bp-icon bp-icon--remote" />
            <span class="branch-name" v-html="highlight(branch.name, filter)" />
          </div>
        </div>
      </div>

      <!-- Remotes (управление) -->
      <div class="section">
        <div class="section-header" @click="toggleSection('remotes')">
          <svg
            class="chevron"
            :class="{ expanded: expandedSections.remotes }"
            width="12" height="12" viewBox="0 0 12 12"
          >
            <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span class="section-title">{{ i18n.branches.remotes }}</span>
          <span class="section-count">{{ filteredRemotes.length }}</span>
          <button
            class="section-add-btn"
            :title="i18n.branches.addRemote"
            @click.stop="openAddRemote"
          >+</button>
        </div>
        <div v-if="expandedSections.remotes && filteredRemotes.length" class="section-items">
          <div
            v-for="remote in filteredRemotes"
            :key="remote.name"
            class="branch-item remote-mgmt-item"
            v-tooltip="`${remote.name}\n${remote.url}`"
            @contextmenu="onRemoteMgmtContextMenu($event, remote)"
          >
            <RefIcon kind="remote-branch" class="bp-icon bp-icon--remote" />
            <span class="branch-name" v-html="highlight(remote.name, filter)" />
            <span class="remote-url">{{ remote.url }}</span>
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
          class="ctx-item"
          :disabled="multiBranch || !hasRemote"
          @click="handleSetUpstreamCtx"
        >{{ i18n.branches.setUpstream }}</button>
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
        v-if="remoteCtxMenu"
        class="ctx-menu"
        :style="{ left: remoteCtxMenu.x + 'px', top: remoteCtxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item ctx-item--keyed" @click="handleCheckoutRemoteCtx">
          <span>{{ i18n.branches.checkout }}</span><span class="ctx-hotkey">Ctrl+G</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item ctx-item--keyed" @click="handleMergeRemoteCtx">
          <span>{{ i18n.branches.merge }}</span><span class="ctx-hotkey">Ctrl+M</span>
        </button>
        <button class="ctx-item ctx-item--keyed" @click="handleRebaseRemoteCtx">
          <span>{{ i18n.branches.rebaseOnto }}</span><span class="ctx-hotkey">Ctrl+D</span>
        </button>
        <div class="ctx-separator" />
        <button
          class="ctx-item ctx-danger"
          @click="handleDeleteRemoteCtx"
        >{{ selectedRemoteBranches.length > 1 ? `${i18n.branches.deleteRemoteBranch} (${selectedRemoteBranches.length})` : i18n.branches.deleteRemoteBranch }}</button>
      </div>
      <div
        v-if="remoteCtxMenu"
        class="ctx-backdrop"
        @click="closeRemoteCtxMenu"
        @contextmenu.prevent="closeRemoteCtxMenu"
      />

      <ConfirmDialog
        v-if="showDeleteRemoteConfirm && targetRemoteBranches.length"
        :message="targetRemoteBranches.length > 1
          ? `Delete ${targetRemoteBranches.length} remote branches?`
          : `Delete remote branch '${targetRemoteBranches[0].name}'?`"
        :items="targetRemoteBranches.length > 1 ? targetRemoteBranches.map((b) => b.name) : undefined"
        confirm-label="Delete"
        danger
        @close="showDeleteRemoteConfirm = false; targetRemoteBranches = []"
        @confirm="confirmDeleteRemote"
      />

      <!-- Remote management context menu -->
      <div
        v-if="remoteMgmtCtxMenu"
        class="ctx-menu"
        :style="{ left: remoteMgmtCtxMenu.x + 'px', top: remoteMgmtCtxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" @click="handleFetchRemoteCtx(false)">{{ i18n.branches.fetchRemote }}</button>
        <button class="ctx-item" @click="handleFetchRemoteCtx(true)">{{ i18n.branches.fetchPrune }}</button>
        <div class="ctx-separator" />
        <button class="ctx-item" @click="openEditRemoteUrl">{{ i18n.branches.editRemoteUrl }}</button>
        <button class="ctx-item" @click="openRenameRemote">{{ i18n.branches.renameRemote }}</button>
        <div class="ctx-separator" />
        <button class="ctx-item ctx-danger" @click="handleRemoveRemoteCtx">{{ i18n.branches.removeRemote }}</button>
      </div>
      <div
        v-if="remoteMgmtCtxMenu"
        class="ctx-backdrop"
        @click="closeRemoteMgmtCtxMenu"
        @contextmenu.prevent="closeRemoteMgmtCtxMenu"
      />

      <RemoteDialog
        v-if="showRemoteDialog"
        :mode="remoteDialogMode"
        :name="remoteDialogTarget?.name"
        :url="remoteDialogTarget?.url"
        @close="showRemoteDialog = false; remoteDialogTarget = null"
        @confirm="confirmRemoteDialog"
      />

      <ConfirmDialog
        v-if="showRemoveRemoteConfirm && remoteDialogTarget"
        :message="`Remove remote '${remoteDialogTarget.name}'?`"
        confirm-label="Remove"
        danger
        @close="showRemoveRemoteConfirm = false; remoteDialogTarget = null"
        @confirm="confirmRemoveRemote"
      />

      <SetUpstreamDialog
        v-if="showSetUpstreamDialog && upstreamBranch"
        :branch="upstreamBranch.name"
        :current="upstreamBranch.upstream"
        :remote-branches="remoteBranchNames"
        @close="showSetUpstreamDialog = false; upstreamBranch = null"
        @confirm="confirmSetUpstream"
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
.branch-arrow {
  flex-shrink: 0;
  font-size: 14px;
  font-weight: 700;
  line-height: 1;
  color: var(--text-muted);
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.1s;
}
.branch-item.current .branch-arrow {
  opacity: 1;
  color: var(--accent);
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

/* Remotes-секция: имя remote не растягивается, URL приглушённо справа */
.remote-mgmt-item .branch-name {
  flex: 0 0 auto;
}
.remote-url {
  flex: 1;
  min-width: 0;
  margin-left: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-align: right;
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
.ctx-item--keyed {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}
.ctx-hotkey {
  font-size: var(--font-size-xs, 11px);
  color: var(--text-muted);
  white-space: nowrap;
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
