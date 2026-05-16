<script setup lang="ts">
import { ref, computed } from "vue";
import { useBranches } from "@/composables/useBranches";
import { highlight } from "@/utils/highlight";
import ConfirmDialog from "@/components/dialogs/ConfirmDialog.vue";
import RenameBranchDialog from "@/components/dialogs/RenameBranchDialog.vue";
import type { BranchInfo, TagInfo } from "@/types";

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
  checkout, mergeBranch, renameBranch, deleteBranch, pushBranch,
  deleteTag, pushTag,
} = useBranches();

// --- Context menu for local branches ---
const ctxMenu = ref<{ x: number; y: number } | null>(null);
const ctxBranch = ref<BranchInfo | null>(null);

function onBranchContextMenu(e: MouseEvent, branch: BranchInfo) {
  e.preventDefault();
  e.stopPropagation();
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
    emit("branchesChanged");
  } catch (e) {
    window.alert(`Merge failed: ${e}`);
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
  targetBranch.value = ctxBranch.value;
  closeCtxMenu();
  showDeleteConfirm.value = true;
}

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

// --- Item selection (highlight on left/right click) ---
const selectedKey = ref<string | null>(null);
function selectItem(key: string) {
  selectedKey.value = key;
}
</script>

<template>
  <div class="branch-panel" @contextmenu.prevent>
    <div class="panel-title-bar">
      <span class="panel-title">Branches</span>
    </div>
    <div class="panel-header">
      <input
        v-model="filter"
        type="text"
        placeholder="Filter..."
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
          <span class="section-title">Local Branches</span>
          <span class="section-count">{{ localBranches.length }}</span>
        </div>
        <div v-if="expandedSections.local" class="section-items">
          <div
            v-for="branch in localBranches"
            :key="branch.name"
            class="branch-item"
            :class="{ current: branch.is_current, selected: selectedKey === `local:${branch.name}` }"
            @mousedown="selectItem(`local:${branch.name}`)"
            @dblclick="handleLocalDblClick(branch)"
            @contextmenu="onBranchContextMenu($event, branch)"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" class="branch-icon">
              <path d="M5 3a2 2 0 100 4 2 2 0 000-4zM5 9a2 2 0 100 4 2 2 0 000-4z"
                    fill="none" stroke="currentColor" stroke-width="1.2"/>
              <path d="M5 7v2" fill="none" stroke="currentColor" stroke-width="1.2"/>
            </svg>
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
          <span class="section-title">Remote Branches</span>
          <span class="section-count">{{ remoteBranches.length }}</span>
        </div>
        <div v-if="expandedSections.remote && remoteBranches.length" class="section-items">
          <div
            v-for="branch in remoteBranches"
            :key="branch.name"
            class="branch-item"
            :class="{ selected: selectedKey === `remote:${branch.name}` }"
            @mousedown="selectItem(`remote:${branch.name}`)"
            @dblclick="emit('checkoutRemote', branch.name)"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" class="branch-icon remote">
              <circle cx="8" cy="4" r="2" fill="none" stroke="currentColor" stroke-width="1.2"/>
              <path d="M8 6v4M4 12h8M4 12v-2M12 12v-2" fill="none" stroke="currentColor" stroke-width="1.2"/>
            </svg>
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
          <span class="section-title">Tags</span>
          <span class="section-count">{{ filteredTags.length }}</span>
          <button
            class="section-add-btn"
            title="Add tag (on HEAD)"
            @click.stop="emit('createTag')"
          >+</button>
        </div>
        <div v-if="expandedSections.tags && filteredTags.length" class="section-items">
          <div
            v-for="tag in filteredTags"
            :key="tag.name"
            class="branch-item"
            :class="{ selected: selectedKey === `tag:${tag.name}` }"
            @mousedown="selectItem(`tag:${tag.name}`)"
            @contextmenu="onTagContextMenu($event, tag)"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" class="tag-icon">
              <path d="M2 9V2h7l5 5-7 7-5-5z" fill="none" stroke="currentColor" stroke-width="1.2"/>
              <circle cx="6" cy="6" r="1" fill="currentColor"/>
            </svg>
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
          <span class="section-title">Stashes</span>
          <span class="section-count">{{ filteredStashes.length }}</span>
        </div>
        <div v-if="expandedSections.stashes && filteredStashes.length" class="section-items">
          <div
            v-for="stash in filteredStashes"
            :key="stash.index"
            class="branch-item stash-item"
            :class="{ selected: selectedKey === `stash:${stash.index}` }"
            @mousedown="selectItem(`stash:${stash.index}`)"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" class="stash-icon">
              <rect x="3" y="3" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
              <rect x="3" y="8" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
            </svg>
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
          :disabled="ctxBranch?.is_current"
          @click="handleCheckoutCtx"
        >Check out</button>
        <div class="ctx-separator" />
        <button
          class="ctx-item"
          :disabled="ctxBranch?.is_current"
          @click="handleMergeCtx"
        >Merge</button>
        <button class="ctx-item" @click="handlePushCtx">Push</button>
        <div class="ctx-separator" />
        <button class="ctx-item" @click="handleRenameCtx">Rename</button>
        <button
          class="ctx-item ctx-danger"
          :disabled="ctxBranch?.is_current"
          @click="handleDeleteCtx"
        >Delete</button>
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
        v-if="showDeleteConfirm && targetBranch"
        :message="`Delete local branch '${targetBranch.name}'?`"
        confirm-label="Delete"
        danger
        @close="showDeleteConfirm = false; targetBranch = null"
        @confirm="confirmDelete"
      />

      <ConfirmDialog
        v-if="showForceDeleteConfirm && targetBranch"
        :message="`Branch '${targetBranch.name}' is not fully merged. Force delete anyway?`"
        confirm-label="Force Delete"
        danger
        @close="showForceDeleteConfirm = false; targetBranch = null"
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

.panel-header {
  padding: 8px;
  border-bottom: 1px solid var(--border-subtle);
}

.filter-input {
  width: 100%;
  padding: 4px 8px;
  font-size: var(--font-size-sm);
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
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
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
