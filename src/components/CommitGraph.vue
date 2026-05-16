<script setup lang="ts">
import { computed, ref } from "vue";
import { useLog } from "@/composables/useLog";
import { useBranches } from "@/composables/useBranches";
import RefIcon from "@/components/RefIcon.vue";
import { useFiles } from "@/composables/useFiles";
import { useRepo } from "@/composables/useRepo";
import type { RefLabel, GraphLine } from "@/types";
import { highlight } from "@/utils/highlight";

const emit = defineEmits<{
  commit: [];
  discard: [];
  createTag: [target: { oid: string; subject: string }];
}>();

const { commits, selectedCommit } = useLog();
const { branches } = useBranches();
const currentBranch = computed(() => branches.value.find((b) => b.is_current));
function isCurrentBranchRow(c: { refs: { kind: string }[] }): boolean {
  return c.refs.some((r) => r.kind === "current-branch");
}
const GRAPH_PALETTE = ["--blue", "--green", "--purple", "--teal", "--orange", "--yellow"];
const GRAPH_COL_W = 14;
const GRAPH_PAD = 8;
const GRAPH_ROW_H = 24;

function laneX(c: number): number {
  return GRAPH_PAD + c * GRAPH_COL_W;
}
function laneColor(colorIdx: number): string {
  return `var(${GRAPH_PALETTE[colorIdx % GRAPH_PALETTE.length]})`;
}
function linePath(l: GraphLine): string {
  const x1 = laneX(l.from_column);
  const x2 = laneX(l.to_column);
  const mid = GRAPH_ROW_H / 2;
  if (l.style === "straight") {
    return `M ${x1} 0 L ${x1} ${GRAPH_ROW_H}`;
  }
  if (l.style === "fork") {
    const cy = (mid + GRAPH_ROW_H) / 2;
    return `M ${x1} ${mid} C ${x1} ${cy} ${x2} ${cy} ${x2} ${GRAPH_ROW_H}`;
  }
  const cy = mid / 2;
  return `M ${x1} 0 C ${x1} ${cy} ${x2} ${cy} ${x2} ${mid}`;
}
const graphMaxCol = computed(() => {
  let m = 0;
  for (const c of commits.value) {
    if (c.column > m) m = c.column;
    for (const l of c.lines) {
      if (l.from_column > m) m = l.from_column;
      if (l.to_column > m) m = l.to_column;
    }
  }
  return m;
});
const graphColW = computed(() => Math.max(80, laneX(graphMaxCol.value) + 16));
const wtCol = computed(() => commits.value[0]?.column ?? 0);
const { files } = useFiles();
const { repoPath } = useRepo();

const graphFilter = ref("");

const repoName = computed(() => {
  if (!repoPath.value) return "";
  const parts = repoPath.value.replace(/\/+$/, "").split("/");
  return parts[parts.length - 1];
});

const ctxMenu = ref<{ x: number; y: number } | null>(null);
const ctxCommitOid = ref<string | null>(null);

function onContextMenu(e: MouseEvent, oid: string) {
  e.preventDefault();
  ctxMenu.value = { x: e.clientX, y: e.clientY };
  ctxCommitOid.value = oid;
}

function closeCtxMenu() {
  ctxMenu.value = null;
  ctxCommitOid.value = null;
}

function ctxAction(action: "commit" | "discard") {
  closeCtxMenu();
  if (action === "commit") emit("commit");
  else emit("discard");
}

function emitAddTag(oid: string | null) {
  if (!oid || oid === "__worktree__") return;
  const c = commits.value.find((x) => x.oid === oid);
  emit("createTag", { oid, subject: c?.message ?? "" });
}

function ctxCreateTag() {
  const oid = ctxCommitOid.value;
  closeCtxMenu();
  emitAddTag(oid);
}

const ctxIsWorkingTree = computed(() => ctxCommitOid.value === "__worktree__");

const changedCount = computed(() => files.value.length);
const isWorkingTreeSelected = computed(() => selectedCommit.value === "__worktree__");

function selectWorkingTree() {
  selectedCommit.value = "__worktree__";
}

function getCurrentIndex(): number {
  // Working Tree is at position -1
  if (selectedCommit.value === "__worktree__") {
    return -1; // Working Tree is always shown at position -1
  }

  // Find commit in filtered list
  const idx = filteredCommits.value.findIndex((c) => c.oid === selectedCommit.value);
  return idx >= 0 ? idx : 0; // fallback to 0 if not found
}

function navigateCommits(direction: 'up' | 'down'): void {
  const currentIdx = getCurrentIndex();
  const hasWorkingTree = true;
  const maxIdx = filteredCommits.value.length - 1;

  let newIdx: number;

  if (direction === 'up') {
    newIdx = currentIdx - 1;
    // Boundary: don't go above -1 (Working Tree) or 0 (first commit)
    if (newIdx < (hasWorkingTree ? -1 : 0)) {
      return; // no-op, stay at boundary
    }
  } else {
    // direction === 'down'
    newIdx = currentIdx + 1;
    // Boundary: don't go beyond last commit
    if (newIdx > maxIdx) {
      return; // no-op, stay at boundary
    }
  }

  // Apply new selection
  if (newIdx === -1) {
    selectWorkingTree();
  } else {
    selectedCommit.value = filteredCommits.value[newIdx].oid;
  }
}

function handleKeyDown(e: KeyboardEvent): void {
  if (e.shiftKey && e.key === 'F7') {
    e.preventDefault();
    emitAddTag(selectedCommit.value);
    return;
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    navigateCommits('up');
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    navigateCommits('down');
  }
}

const firstRemoteIdx = computed(() => {
  return filteredCommits.value.findIndex((c) =>
    c.refs.some((r) => r.kind === "remote-branch")
  );
});

function isUnpushed(idx: number): boolean {
  const ri = firstRemoteIdx.value;
  return ri === -1 ? false : idx < ri;
}

const maxAuthorLen = computed(() => {
  let m = 0;
  for (const c of commits.value) {
    if (c.author.length > m) m = c.author.length;
  }
  return m;
});

const filteredCommits = computed(() => {
  const q = graphFilter.value.toLowerCase();
  if (!q) return commits.value;
  return commits.value.filter((c) =>
    c.message.toLowerCase().includes(q) ||
    c.author.toLowerCase().includes(q) ||
    c.author_email.toLowerCase().includes(q) ||
    c.oid.toLowerCase().includes(q) ||
    c.short_oid.toLowerCase().includes(q) ||
    c.date.toLowerCase().includes(q) ||
    c.refs.some((r) => r.name.toLowerCase().includes(q))
  );
});

function refClass(r: RefLabel): string {
  return `ref-label ref-${r.kind}`;
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    const hours = String(d.getHours()).padStart(2, "0");
    const mins = String(d.getMinutes()).padStart(2, "0");
    return `${year}-${month}-${day} ${hours}:${mins}`;
  } catch {
    return iso;
  }
}
</script>

<template>
  <div class="commit-graph" :style="{ '--author-col-w': (maxAuthorLen + 2) + 'ch', '--graph-col-w': graphColW + 'px' }" @contextmenu.prevent>
    <div class="panel-title-bar">
      <span class="panel-title">Graph<template v-if="repoName"> | {{ repoName }}</template></span>
      <div class="graph-toolbar">
        <input v-model="graphFilter" type="text" placeholder="Filter" class="graph-filter" />
      </div>
    </div>


    <div class="graph-body" tabindex="0" role="listbox" @keydown="handleKeyDown">
      <!-- Working Tree / Index row -->
      <div
        class="graph-row wt-row"
        :class="{ selected: isWorkingTreeSelected }"
        @mousedown.shift.prevent
        @click="selectWorkingTree"
        @dblclick="emit('commit')"
        @contextmenu="onContextMenu($event, '__worktree__')"
      >
        <div class="graph-col">
          <svg :width="graphColW" height="24" class="graph-svg">
            <line :x1="laneX(wtCol)" y1="12" :x2="laneX(wtCol)" y2="24" stroke="var(--red)" stroke-width="2" />
            <circle :cx="laneX(wtCol)" cy="12" r="5" fill="var(--green)" stroke="var(--bg-primary)" stroke-width="1.5" />
          </svg>
        </div>
        <div class="message-col">
          <span class="wt-label">Working Tree/Index ({{ changedCount > 0 ? changedCount + ' changed' : 'clean' }})</span>
        </div>
        <span class="author-col"></span>
        <span class="date-col"></span>
      </div>

      <div
        v-for="(commit, idx) in filteredCommits"
        :key="commit.oid"
        class="graph-row"
        :class="{ selected: selectedCommit === commit.oid, unpushed: isUnpushed(idx) }"
        @mousedown.shift.prevent
        @click="selectedCommit = commit.oid"
        @contextmenu="onContextMenu($event, commit.oid)"
      >
        <!-- Graph column with SVG lane lines -->
        <div class="graph-col">
          <svg :width="graphColW" height="24" class="graph-svg">
            <path
              v-for="(ln, li) in commit.lines"
              :key="li"
              :d="linePath(ln)"
              :stroke="laneColor(ln.color)"
              stroke-width="2"
              fill="none"
            />
            <circle
              :cx="laneX(commit.column)"
              cy="12"
              r="5"
              :fill="isUnpushed(idx) ? 'var(--yellow)' : laneColor(commit.column)"
              stroke="var(--bg-primary)"
              stroke-width="1.5"
            />
          </svg>
        </div>

        <!-- Message + refs -->
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

        <span class="author-col" v-html="highlight(commit.author, graphFilter)" />
        <span class="date-col" v-html="highlight(formatDate(commit.date), graphFilter)" />
      </div>
    </div>

    <!-- Context menu -->
    <Teleport to="body">
      <div
        v-if="ctxMenu"
        class="ctx-menu"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" :disabled="!ctxIsWorkingTree" @click="ctxAction('commit')">
          <span class="ctx-label">Commit</span>
        </button>
        <button class="ctx-item ctx-danger" :disabled="!ctxIsWorkingTree" @click="ctxAction('discard')">
          <span class="ctx-label">Discard</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxCreateTag">
          <span class="ctx-label">Add Tag</span>
          <span class="ctx-shortcut">Shift+F7</span>
        </button>
      </div>
      <div v-if="ctxMenu" class="ctx-backdrop" @click="closeCtxMenu" @contextmenu.prevent="closeCtxMenu" />
    </Teleport>
  </div>
</template>

<style scoped>
.commit-graph {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  container-type: inline-size;
}

.panel-title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
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
.graph-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
}
.graph-filter {
  padding: 1px 6px;
  font-size: var(--font-size-xs);
  width: 120px;
  height: 20px;
}

.graph-header {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  user-select: none;
}

.graph-body {
  flex: 1;
  overflow-y: auto;
  user-select: none;
  -webkit-user-select: none;
}

.graph-row {
  display: flex;
  align-items: center;
  padding: 0 8px;
  cursor: pointer;
  font-size: var(--font-size-sm);
  height: 24px;
  user-select: none;
}
.graph-row:hover {
  background: var(--bg-hover);
}
.graph-row.selected {
  background: var(--bg-surface);
}
.graph-row.unpushed .commit-message {
  color: #e8e8e8;
}
.graph-row.unpushed .author-col,
.graph-row.unpushed .date-col,
.graph-row.unpushed .hash-col {
  color: #c8c8c8;
}

.graph-col {
  width: var(--graph-col-w, 80px);
  flex-shrink: 0;
  overflow: hidden;
}
.graph-svg {
  display: block;
}

.message-col {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 4px;
  overflow: hidden;
  margin-right: 12px;
}
.commit-message {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.author-col {
  display: inline-block;
  width: var(--author-col-w, 0);
  max-width: 240px;
  flex-shrink: 0;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-right: 8px;
  font-size: var(--font-size-xs);
}

.date-col {
  width: 130px;
  flex-shrink: 0;
  color: var(--text-muted);
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-xs);
}

@container (max-width: 500px) {
  .author-col { display: none; }
  .date-col { width: 110px; }
}
@container (max-width: 380px) {
  .date-col { display: none; }
}

.hash-col {
  width: 70px;
  flex-shrink: 0;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-align: right;
}

/* Working tree row */
.wt-label {
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
}

.graph-col-head {
  font-size: var(--font-size-xs);
}

/* Ref labels */
.ref-label {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 0 5px;
  border-radius: 3px;
  font-size: var(--font-size-xs);
  font-weight: 600;
  line-height: 16px;
  white-space: nowrap;
  flex-shrink: 0;
}
.ref-local-branch {
  background: rgba(166, 227, 161, 0.2);
  color: var(--green);
}
.ref-remote-branch {
  background: rgba(137, 180, 250, 0.15);
  color: var(--blue);
}
.ref-tag {
  background: rgba(249, 226, 175, 0.15);
  color: var(--yellow);
}
.ref-head {
  background: rgba(243, 139, 168, 0.2);
  color: var(--red);
  font-weight: 800;
}
.ref-stash {
  background: rgba(203, 166, 247, 0.15);
  color: var(--purple);
}
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
.ref-label .ref-icon {
  width: 11px;
  height: 11px;
}

/* Context menu */
.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
}
.ctx-menu {
  position: fixed;
  z-index: 1000;
  min-width: 140px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
}
.ctx-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  width: 100%;
  padding: 6px 12px;
  text-align: left;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  background: none;
  border: none;
  cursor: pointer;
}
.ctx-label {
  white-space: nowrap;
}
.ctx-shortcut {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  white-space: nowrap;
}
.ctx-item:hover:not(:disabled) {
  background: var(--bg-hover);
}
.ctx-item:disabled {
  color: var(--text-muted);
  opacity: 0.4;
  cursor: not-allowed;
}
.ctx-danger {
  color: var(--red);
}
.ctx-danger:hover {
  background: rgba(243, 139, 168, 0.1);
}
.ctx-separator {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}
</style>
