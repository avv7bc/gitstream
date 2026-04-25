<script setup lang="ts">
import { computed, ref } from "vue";
import { useLog } from "@/composables/useLog";
import { useFiles } from "@/composables/useFiles";
import { useRepo } from "@/composables/useRepo";
import type { RefLabel } from "@/types";
import { highlight } from "@/utils/highlight";

const emit = defineEmits<{
  commit: [];
  discard: [];
}>();

const { commits, selectedCommit } = useLog();
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

const ctxIsWorkingTree = computed(() => ctxCommitOid.value === "__worktree__");

const changedCount = computed(() => files.value.length);
const isWorkingTreeSelected = computed(() => selectedCommit.value === "__worktree__");

function selectWorkingTree() {
  selectedCommit.value = "__worktree__";
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
  <div class="commit-graph" :style="{ '--author-col-w': (maxAuthorLen + 2) + 'ch' }" @contextmenu.prevent>
    <div class="panel-title-bar">
      <span class="panel-title">Graph<template v-if="repoName"> | {{ repoName }}</template></span>
      <div class="graph-toolbar">
        <input v-model="graphFilter" type="text" placeholder="Filter" class="graph-filter" />
      </div>
    </div>


    <div class="graph-body">
      <!-- Working Tree / Index row -->
      <div
        v-if="changedCount > 0"
        class="graph-row wt-row"
        :class="{ selected: isWorkingTreeSelected }"
        @mousedown.shift.prevent
        @click="selectWorkingTree"
        @dblclick="emit('commit')"
        @contextmenu="onContextMenu($event, '__worktree__')"
      >
        <div class="graph-col">
          <svg width="80" height="24" class="graph-svg">
            <line x1="8" y1="12" x2="8" y2="24" stroke="var(--blue)" stroke-width="2" />
            <circle cx="8" cy="12" r="4" fill="var(--red)" stroke="var(--bg-primary)" stroke-width="1.5" />
          </svg>
        </div>
        <div class="message-col">
          <span class="wt-label">Working Tree/Index ({{ changedCount }} changed)</span>
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
        <!-- Graph column with SVG lines -->
        <div class="graph-col">
          <svg width="80" height="24" class="graph-svg">
            <line v-if="idx > 0 || changedCount > 0" x1="8" y1="0" x2="8" y2="24" stroke="var(--blue)" stroke-width="2" />
            <circle cx="8" cy="12" r="4" :fill="isUnpushed(idx) ? 'var(--yellow)' : 'var(--blue)'" stroke="var(--bg-primary)" stroke-width="1.5" />
          </svg>
        </div>

        <!-- Message + refs -->
        <div class="message-col">
          <span
            v-for="r in commit.refs"
            :key="r.name"
            :class="refClass(r)"
            v-html="highlight(r.name, graphFilter)"
          />
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
        <button class="ctx-item" :disabled="!ctxIsWorkingTree" @click="ctxAction('commit')">Commit</button>
        <button class="ctx-item ctx-danger" :disabled="!ctxIsWorkingTree" @click="ctxAction('discard')">Discard</button>
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
  width: 80px;
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
  display: inline-block;
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
  opacity: 0.4;
  cursor: not-allowed;
}
.ctx-danger {
  color: var(--red);
}
.ctx-danger:hover {
  background: rgba(243, 139, 168, 0.1);
}
</style>
