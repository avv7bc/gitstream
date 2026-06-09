<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import AppToolbar from "./components/AppToolbar.vue";
import RepositoriesPanel from "./components/RepositoriesPanel.vue";
import BranchPanel from "./components/BranchPanel.vue";
import CommitGraph from "./components/CommitGraph.vue";
import CommitDetails from "./components/CommitDetails.vue";
import FileList from "./components/FileList.vue";
import SideBySideDiffView from "./components/SideBySideDiffView.vue";
import StatusBar from "./components/StatusBar.vue";
import ConflictBar from "./components/ConflictBar.vue";
import CommitDialog from "./components/dialogs/CommitDialog.vue";
import PushDialog from "./components/dialogs/PushDialog.vue";
import PullDialog from "./components/dialogs/PullDialog.vue";
import CheckoutDialog from "./components/dialogs/CheckoutDialog.vue";
import CheckoutRemoteDialog from "./components/dialogs/CheckoutRemoteDialog.vue";
import ConfirmDialog from "./components/dialogs/ConfirmDialog.vue";
import DiscardDialog from "./components/dialogs/DiscardDialog.vue";
import SettingsDialog from "./components/dialogs/SettingsDialog.vue";
import CredentialDialog from "./components/dialogs/CredentialDialog.vue";
import FileHistoryDialog from "./components/dialogs/FileHistoryDialog.vue";
import { useFileHistory } from "./composables/useFileHistory";
import BlameDialog from "./components/dialogs/BlameDialog.vue";
import { useBlame } from "./composables/useBlame";
import StatsDialog from "./components/dialogs/StatsDialog.vue";
import SquashDialog from "./components/dialogs/SquashDialog.vue";
import RewordDialog from "./components/dialogs/RewordDialog.vue";
import FileCompareDialog from "./components/dialogs/FileCompareDialog.vue";
import AddTagDialog from "./components/dialogs/AddTagDialog.vue";
import StashSaveDialog from "./components/dialogs/StashSaveDialog.vue";
import UpdateBanner from "./components/UpdateBanner.vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useUpdate } from "@/composables/useUpdate";
import type { CommitInfo } from "@/types";
import { useFileCompare } from "@/composables/useFileCompare";
import { useRepo } from "@/composables/useRepo";
import { useFiles } from "@/composables/useFiles";
import { useBranches } from "@/composables/useBranches";
import { useLog } from "@/composables/useLog";
import { useDiff } from "@/composables/useDiff";
import { useRemote } from "@/composables/useRemote";
import { useConflicts } from "@/composables/useConflicts";
import { toggleLog, logError } from "@/composables/useProgress";

const { repoPath, onRepoOpened, restoreLastRepo } = useRepo();
const { refresh: refreshFiles, selectedFile, files, stageFiles, unstageFiles } = useFiles();
const { refresh: refreshBranches, createTag, stashSave } = useBranches();
const { refresh: refreshLog, selectedCommit, commits, squashCommits, rewordCommit } = useLog();
const { clearDiff } = useDiff();
const { pull, push, fetchRemote } = useRemote();
const { target: compareTarget, close: closeFileCompare } = useFileCompare();
const { refresh: refreshConflicts } = useConflicts();
const { updateInfo, checkForUpdate } = useUpdate();

const selectedFileStatus = computed(() =>
  files.value.find((f) => f.path === selectedFile.value) ?? null,
);
const canStageGlobal = computed(
  () => !!selectedFileStatus.value && selectedFileStatus.value.staged !== "staged",
);
const canUnstageGlobal = computed(
  () =>
    !!selectedFileStatus.value &&
    (selectedFileStatus.value.staged === "staged" ||
      selectedFileStatus.value.staged === "partial"),
);

async function refreshAll() {
  await Promise.all([
    refreshFiles(),
    refreshBranches(),
    refreshLog(),
    refreshConflicts(),
  ]);
}

// После checkout: сбрасываем selectedCommit, чтобы refreshLog выбрал HEAD
// новой ветки. Сбрасываем всегда (включая worktree-режим): если новая ветка
// чистая (нет изменений рабочего дерева), worktree-режим даёт пустую панель
// Files — пользователю не видно никаких файлов.
async function onBranchCheckedOut() {
  selectedCommit.value = null;
  await refreshAll();
}

// Открыт другой репозиторий: выделение коммита/файла и дифф — это
// модульные синглтоны, относящиеся к предыдущему репо. Сбрасываем их
// до загрузки данных нового, иначе панель Changes показывает дифф из
// старого репозитория (FileList обновляется через refreshFiles, а
// currentDiff/selectedFile — нет).
onRepoOpened(async () => {
  selectedCommit.value = null;
  selectedFile.value = null;
  clearDiff();
  await refreshAll();
});

watch(repoPath, (val) => {
  if (!val) {
    // Репозиторий закрыт/удалён: чистим выделение и дифф (модульные
    // синглтоны от предыдущего репо), иначе панель Changes показывает
    // старый дифф. refreshAll сбросит списки файлов/веток/лога.
    selectedCommit.value = null;
    selectedFile.value = null;
    clearDiff();
    refreshAll();
  }
});

const { open: fileHistoryOpen, close: closeFileHistory } = useFileHistory();
const { open: blameOpen, close: closeBlame } = useBlame();
const showCommitDialog = ref(false);
const showStashSaveDialog = ref(false);
const showPushDialog = ref(false);
const showPullDialog = ref(false);
const showCheckoutDialog = ref(false);
const checkoutRemoteTarget = ref<string | null>(null);
const showConfirmDialog = ref(false);
const showDiscardDialog = ref(false);
const showSettingsDialog = ref(false);
const showStatsDialog = ref(false);
const showAddTagDialog = ref(false);
const addTagTarget = ref<{ oid: string; subject: string } | null>(null);
const squashPayload = ref<{ oids: string[]; commits: CommitInfo[] } | null>(null);
const rewordPayload = ref<{ oid: string; message: string; isHead: boolean } | null>(null);
const graphRef = ref<InstanceType<typeof CommitGraph> | null>(null);

function onSquash(payload: { oids: string[] }) {
  const squashCommitsList = payload.oids
    .map(oid => commits.value.find(c => c.oid === oid))
    .filter(Boolean) as CommitInfo[];
  squashPayload.value = { oids: payload.oids, commits: squashCommitsList };
}

function onReword(payload: { oid: string; message: string; isHead: boolean }) {
  rewordPayload.value = payload;
}

async function doReword(message: string) {
  if (!rewordPayload.value) return;
  const { oid } = rewordPayload.value;
  rewordPayload.value = null;
  try {
    const newOid = await rewordCommit(oid, message);
    await refreshAll();
    if (newOid) selectedCommit.value = newOid;
  } catch (e) {
    logError(String(e));
  } finally {
    graphRef.value?.focus();
  }
}

async function doSquash(message: string) {
  if (!squashPayload.value) return;
  const { oids } = squashPayload.value;
  squashPayload.value = null;
  try {
    await squashCommits(oids, message);
    await refreshAll();
  } catch (e) {
    logError(String(e));
  }
}

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
    logError(String(e));
  }
  addTagTarget.value = null;
}

async function handleStashSave(payload: { message: string | null; includeUntracked: boolean }) {
  showStashSaveDialog.value = false;
  try {
    await stashSave(payload.message, payload.includeUntracked);
    await refreshAll();
  } catch (e) {
    logError(String(e));
  }
}

function handlePullRequest(remote: string, rebase: boolean) {
  showPullDialog.value = false;
  requestAnimationFrame(() => {
    requestAnimationFrame(async () => {
      document.body.style.cursor = "wait";
      try {
        await pull(remote, rebase);
        await refreshAll();
      } finally {
        document.body.style.cursor = "";
      }
    });
  });
}
function handleFetchRequest(remote: string) {
  showPullDialog.value = false;
  requestAnimationFrame(() => {
    requestAnimationFrame(async () => {
      document.body.style.cursor = "wait";
      try {
        await fetchRemote(remote);
        await refreshAll();
      } finally {
        document.body.style.cursor = "";
      }
    });
  });
}
function handlePushRequest(remote: string, force: boolean) {
  showPushDialog.value = false;
  requestAnimationFrame(() => {
    requestAnimationFrame(async () => {
      document.body.style.cursor = "wait";
      try {
        await push(remote, force);
        await refreshAll();
      } finally {
        document.body.style.cursor = "";
      }
    });
  });
}

const confirmMessage = ref("");

// --- Template ref for RepositoriesPanel ---
const repositoriesPanelRef = ref<InstanceType<typeof RepositoriesPanel> | null>(null);
const branchPanelRef = ref<InstanceType<typeof BranchPanel> | null>(null);

function handleAddRepository() {
  repositoriesPanelRef.value?.triggerAddRepository();
}

function handleAddGroup() {
  repositoriesPanelRef.value?.triggerAddGroup();
}

function handleCloneRepository() {
  repositoriesPanelRef.value?.triggerCloneRepository();
}

// --- Modal registry + open-order stack: Esc closes the topmost (last opened) ---
const modalRegistry: { key: string; isOpen: () => boolean; close: () => void }[] = [
  { key: "commit", isOpen: () => showCommitDialog.value, close: () => { showCommitDialog.value = false; refreshAll(); } },
  { key: "stashSave", isOpen: () => showStashSaveDialog.value, close: () => { showStashSaveDialog.value = false; } },
  { key: "push", isOpen: () => showPushDialog.value, close: () => { showPushDialog.value = false; } },
  { key: "pull", isOpen: () => showPullDialog.value, close: () => { showPullDialog.value = false; } },
  { key: "checkout", isOpen: () => showCheckoutDialog.value, close: () => { showCheckoutDialog.value = false; } },
  { key: "checkoutRemote", isOpen: () => !!checkoutRemoteTarget.value, close: () => { checkoutRemoteTarget.value = null; } },
  { key: "confirm", isOpen: () => showConfirmDialog.value, close: () => { showConfirmDialog.value = false; } },
  { key: "discard", isOpen: () => showDiscardDialog.value, close: () => { showDiscardDialog.value = false; refreshAll(); } },
  { key: "settings", isOpen: () => showSettingsDialog.value, close: () => { showSettingsDialog.value = false; } },
  { key: "stats", isOpen: () => showStatsDialog.value, close: () => { showStatsDialog.value = false; } },
  { key: "addTag", isOpen: () => showAddTagDialog.value, close: () => { showAddTagDialog.value = false; addTagTarget.value = null; } },
  { key: "squash", isOpen: () => !!squashPayload.value, close: () => { squashPayload.value = null; } },
  { key: "reword", isOpen: () => !!rewordPayload.value, close: () => { rewordPayload.value = null; } },
  { key: "fileCompare", isOpen: () => !!compareTarget.value, close: () => { closeFileCompare(); } },
  { key: "fileHistory", isOpen: () => fileHistoryOpen.value, close: () => { closeFileHistory(); } },
  { key: "blame", isOpen: () => blameOpen.value, close: () => { closeBlame(); } },
];

const modalOpenOrder = ref<string[]>([]);
watch(
  () => modalRegistry.map((m) => m.isOpen()),
  (states) => {
    modalRegistry.forEach((m, i) => {
      const idx = modalOpenOrder.value.indexOf(m.key);
      if (states[i] && idx === -1) modalOpenOrder.value.push(m.key);
      else if (!states[i] && idx !== -1) modalOpenOrder.value.splice(idx, 1);
    });
  },
);

// Capture phase: runs before per-dialog handlers so the topmost modal swallows Esc.
function onEscapeCapture(e: KeyboardEvent) {
  if (e.key !== "Escape") return;
  // Compute open modals directly — never rely on the tracked stack for the bail-out,
  // otherwise a stale stack would skip stopPropagation and let every dialog close at once.
  const open = modalRegistry.filter((m) => m.isOpen());
  if (open.length === 0) return;
  // Swallow the event so no other (per-dialog / window) handler closes a second window.
  e.preventDefault();
  e.stopImmediatePropagation();
  // Topmost = most recently opened (tracked order); fall back to last open in registry.
  let top = open[open.length - 1];
  for (let i = modalOpenOrder.value.length - 1; i >= 0; i--) {
    const found = open.find((m) => m.key === modalOpenOrder.value[i]);
    if (found) { top = found; break; }
  }
  top.close();
}

function onKeydown(e: KeyboardEvent) {
  if (e.altKey && e.key === "PageDown") {
    e.preventDefault();
    if (repoPath.value) showPullDialog.value = true;
  }
  if (e.altKey && e.key === "PageUp") {
    e.preventDefault();
    if (repoPath.value) handlePushRequest("origin", false);
  }
  if ((e.ctrlKey || e.metaKey || e.altKey) && !e.shiftKey && e.code === "KeyP") {
    e.preventDefault();
    showSettingsDialog.value = !showSettingsDialog.value;
  }
  if (e.key === "F7" && !e.shiftKey) {
    e.preventDefault();
    if (repoPath.value) branchPanelRef.value?.openCreateBranch(null);
  }
  if (e.key === "F7" && e.shiftKey) {
    e.preventDefault();
    if (repoPath.value) openAddTag(null);
  }
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && (e.key === "t" || e.key === "T" || e.code === "KeyT")) {
    e.preventDefault();
    if (canStageGlobal.value && selectedFile.value) stageFiles([selectedFile.value]);
  }
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "t" || e.key === "T" || e.code === "KeyT")) {
    e.preventDefault();
    if (canUnstageGlobal.value && selectedFile.value) unstageFiles([selectedFile.value]);
  }
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && (e.key === "k" || e.key === "K" || e.code === "KeyK")) {
    e.preventDefault();
    if (repoPath.value) showCommitDialog.value = true;
  }
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && (e.key === "g" || e.key === "G" || e.code === "KeyG")) {
    e.preventDefault();
    if (repoPath.value) branchPanelRef.value?.triggerCheckoutRemote();
  }
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && (e.key === "m" || e.key === "M" || e.code === "KeyM")) {
    e.preventDefault();
    if (repoPath.value) branchPanelRef.value?.triggerMerge();
  }
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && (e.key === "d" || e.key === "D" || e.code === "KeyD")) {
    e.preventDefault();
    if (repoPath.value) branchPanelRef.value?.triggerRebase();
  }
  if ((e.altKey || e.ctrlKey || e.metaKey) && !e.shiftKey && e.code === "KeyO") {
    e.preventDefault();
    toggleLog();
  }
}
let pollTimer: ReturnType<typeof setTimeout> | null = null;
let pollStopped = false;

// Самопланирующийся поллинг: следующий цикл стартует только после того,
// как завершился предыдущий. Иначе перекрывающиеся обновления резолвятся
// не по порядку и список веток «прыгает» (мигание).
async function pollTick() {
  if (pollStopped) return;
  if (repoPath.value) {
    try {
      await Promise.all([refreshFiles(), refreshLog(), refreshBranches()]);
    } catch {
      /* фоновый поллинг не должен ломать UI */
    }
  }
  if (pollStopped) return;
  pollTimer = setTimeout(pollTick, 1000);
}

function onContextMenu(e: MouseEvent) {
  e.preventDefault();
}

onMounted(() => {
  getCurrentWindow().setTitle(`GitStream v${__APP_VERSION__}`);
  window.addEventListener("keydown", onEscapeCapture, true);
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("contextmenu", onContextMenu);
  restoreLastRepo();
  pollTimer = setTimeout(pollTick, 1000);
  checkForUpdate();
});
onUnmounted(() => {
  window.removeEventListener("keydown", onEscapeCapture, true);
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("contextmenu", onContextMenu);
  pollStopped = true;
  if (pollTimer) clearTimeout(pollTimer);
});

// --- Resizable panel sizes (persisted to localStorage) ---
const LAYOUT_KEY = "gitstream-layout";

function loadLayout() {
  try {
    const raw = localStorage.getItem(LAYOUT_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return {};
}

const saved = loadLayout();
const sidebarWidth = ref<number>(saved.sidebarWidth ?? 220);
const rightPaneWidth = ref<number>(saved.rightPaneWidth ?? 340);
const topRowHeight = ref<number | null>(saved.topRowHeight ?? null);
const detailsHeight = ref<number | null>(saved.detailsHeight ?? null);
const reposHeight = ref<number | null>(saved.reposHeight ?? null);

function saveLayout() {
  localStorage.setItem(LAYOUT_KEY, JSON.stringify({
    sidebarWidth: sidebarWidth.value,
    rightPaneWidth: rightPaneWidth.value,
    topRowHeight: topRowHeight.value,
    detailsHeight: detailsHeight.value,
    reposHeight: reposHeight.value,
  }));
}

const horizontalHandles = new Set(["sidebar", "right-pane"]);
let dragging: string | null = null;
let startPos = 0;
let startSize = 0;

function onMouseDown(e: MouseEvent, handle: string) {
  dragging = handle;
  startPos = horizontalHandles.has(handle) ? e.clientX : e.clientY;

  if (handle === "sidebar") startSize = sidebarWidth.value;
  else if (handle === "right-pane") startSize = rightPaneWidth.value;
  else if (handle === "top-row") {
    const el = document.querySelector(".top-row") as HTMLElement | null;
    startSize = topRowHeight.value ?? (el ? el.offsetHeight : 400);
  } else if (handle === "right-row") {
    const el = document.querySelector(".details-pane") as HTMLElement | null;
    startSize = detailsHeight.value ?? (el ? el.offsetHeight : 200);
  } else if (handle === "repos") {
    const el = document.querySelector(".repos-section") as HTMLElement | null;
    startSize = reposHeight.value ?? (el ? el.offsetHeight : 180);
  }

  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
  document.body.style.cursor = horizontalHandles.has(handle) ? "col-resize" : "row-resize";
  document.body.style.userSelect = "none";
}

function onMouseMove(e: MouseEvent) {
  if (!dragging) return;
  if (dragging === "sidebar") {
    sidebarWidth.value = Math.max(120, startSize + (e.clientX - startPos));
  } else if (dragging === "right-pane") {
    rightPaneWidth.value = Math.max(180, startSize - (e.clientX - startPos));
  } else if (dragging === "top-row") {
    topRowHeight.value = Math.max(100, startSize + (e.clientY - startPos));
  } else if (dragging === "right-row") {
    detailsHeight.value = Math.max(60, startSize + (e.clientY - startPos));
  } else if (dragging === "repos") {
    reposHeight.value = Math.max(60, startSize + (e.clientY - startPos));
  }
}

function onMouseUp() {
  dragging = null;
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  saveLayout();
}
</script>

<template>
  <div class="app-layout">
    <!-- Toolbar -->
    <AppToolbar
      @commit="showCommitDialog = true"
      @push="handlePushRequest('origin', false)"
      @pull="showPullDialog = true"
      @checkout="showCheckoutDialog = true"
      @settings="showSettingsDialog = true"
      @stats="showStatsDialog = true"
      @add-repository="handleAddRepository"
      @clone-repository="handleCloneRepository"
      @add-group="handleAddGroup"
      @discard="showDiscardDialog = true"
      @stash="showStashSaveDialog = true"
    />

    <ConflictBar @changed="refreshAll()" />

    <!-- Main body -->
    <div class="app-body">
      <!-- LEFT SIDEBAR -->
      <aside class="left-sidebar" :style="{ width: sidebarWidth + 'px' }">
        <div
          class="repos-section"
          :style="reposHeight ? { flex: 'none', height: reposHeight + 'px' } : {}"
        >
          <RepositoriesPanel ref="repositoriesPanelRef" />
        </div>
        <div class="resize-handle-h" @mousedown="onMouseDown($event, 'repos')" />
        <div class="branches-section">
          <BranchPanel
            ref="branchPanelRef"
            @checkout-remote="checkoutRemoteTarget = $event"
            @checked-out="onBranchCheckedOut()"
            @branches-changed="refreshAll()"
            @tags-changed="refreshAll()"
            @create-tag="openAddTag(null)"
          />
        </div>
      </aside>

      <div class="resize-handle-v" @mousedown="onMouseDown($event, 'sidebar')" />

      <!-- CENTER + RIGHT -->
      <div class="main-area">
        <!-- TOP ROW: Graph | (Commit Details + Files) -->
        <div
          class="top-row"
          :style="topRowHeight ? { flex: 'none', height: topRowHeight + 'px' } : {}"
        >
          <div class="graph-pane">
            <CommitGraph
              ref="graphRef"
              @commit="showCommitDialog = true"
              @discard="showDiscardDialog = true"
              @create-tag="openAddTag($event)"
              @changed="refreshAll()"
              @squash="onSquash($event)"
              @reword="onReword($event)"
            />
          </div>

          <div class="resize-handle-v" @mousedown="onMouseDown($event, 'right-pane')" />

          <div class="right-pane" :style="{ width: rightPaneWidth + 'px' }">
            <div
              class="details-pane"
              :style="detailsHeight ? { flex: 'none', height: detailsHeight + 'px' } : {}"
            >
              <CommitDetails />
            </div>
            <div class="resize-handle-h" @mousedown="onMouseDown($event, 'right-row')" />
            <div class="files-pane">
              <FileList @commit="showCommitDialog = true" />
            </div>
          </div>
        </div>

        <div class="resize-handle-h" @mousedown="onMouseDown($event, 'top-row')" />

        <!-- BOTTOM ROW: Diff -->
        <div class="bottom-row">
          <div class="diff-pane">
            <SideBySideDiffView />
          </div>
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <StatusBar />

    <!-- Dialogs -->
    <CommitDialog v-if="showCommitDialog" @close="showCommitDialog = false; refreshAll()" />
    <PushDialog v-if="showPushDialog" @close="showPushDialog = false" @push="handlePushRequest" />
    <PullDialog v-if="showPullDialog" @close="showPullDialog = false" @pull="handlePullRequest" @fetch="handleFetchRequest" />
    <CheckoutDialog v-if="showCheckoutDialog" @close="showCheckoutDialog = false" @checked-out="onBranchCheckedOut()" />
    <CheckoutRemoteDialog
      v-if="checkoutRemoteTarget"
      :remote-branch="checkoutRemoteTarget"
      @close="checkoutRemoteTarget = null"
      @checked-out="onBranchCheckedOut()"
    />
    <ConfirmDialog
      v-if="showConfirmDialog"
      :message="confirmMessage"
      @close="showConfirmDialog = false"
    />
    <DiscardDialog v-if="showDiscardDialog" @close="showDiscardDialog = false; refreshAll()" />
    <StashSaveDialog v-if="showStashSaveDialog" @close="showStashSaveDialog = false" @confirm="handleStashSave" />
    <SettingsDialog v-if="showSettingsDialog" @close="showSettingsDialog = false" />
    <StatsDialog v-if="showStatsDialog" @close="showStatsDialog = false" />
    <AddTagDialog
      v-if="showAddTagDialog"
      :target="addTagTarget"
      @close="showAddTagDialog = false; addTagTarget = null"
      @confirm="handleCreateTag"
    />
    <SquashDialog
      v-if="squashPayload"
      :oids="squashPayload.oids"
      :commits="squashPayload.commits"
      @confirm="doSquash($event)"
      @close="squashPayload = null"
    />
    <RewordDialog
      v-if="rewordPayload"
      :oid="rewordPayload.oid"
      :message="rewordPayload.message"
      :is-head="rewordPayload.isHead"
      @confirm="doReword($event)"
      @close="rewordPayload = null"
    />
    <FileCompareDialog v-if="compareTarget" />
    <FileHistoryDialog v-if="fileHistoryOpen" />
    <BlameDialog v-if="blameOpen" />
    <CredentialDialog />
    <UpdateBanner
      v-if="updateInfo"
      :info="updateInfo"
      @dismiss="updateInfo = null"
    />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-primary);
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* Left sidebar */
.left-sidebar {
  min-width: 120px;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-subtle);
  overflow: hidden;
  flex-shrink: 0;
}

.repos-section {
  flex: 1;
  overflow: hidden;
}

.branches-section {
  flex: 2;
  overflow: hidden;
  border-top: 1px solid var(--border-subtle);
}

/* Main area */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Top row */
.top-row {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.graph-pane {
  flex: 1;
  overflow: hidden;
}

/* Right pane: details + files stacked */
.right-pane {
  min-width: 180px;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border-subtle);
  overflow: hidden;
  flex-shrink: 0;
}

.details-pane {
  flex: 2;
  overflow: hidden;
}

.files-pane {
  flex: 3;
  overflow: hidden;
  border-top: 1px solid var(--border-subtle);
}

/* Bottom row */
.bottom-row {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.diff-pane {
  flex: 1;
  overflow: hidden;
}

/* ---- Resize handles ---- */
.resize-handle-v {
  width: 4px;
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  z-index: 10;
}
.resize-handle-v::after {
  content: "";
  position: absolute;
  inset: 0 -2px;
}
.resize-handle-v:hover {
  background: var(--accent);
}

.resize-handle-h {
  height: 4px;
  cursor: row-resize;
  flex-shrink: 0;
  position: relative;
  z-index: 10;
}
.resize-handle-h::after {
  content: "";
  position: absolute;
  inset: -2px 0;
}
.resize-handle-h:hover {
  background: var(--accent);
}
</style>
