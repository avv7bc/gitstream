<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@/composables/useProgress";
import { useFiles } from "@/composables/useFiles";
import { useLog } from "@/composables/useLog";
import { useRepo } from "@/composables/useRepo";
import { useDiff } from "@/composables/useDiff";
import { useFileCompare } from "@/composables/useFileCompare";
import type { FileDiff } from "@/types";
import { highlight } from "@/utils/highlight";

const { files, selectedFile } = useFiles();
const { selectedCommit } = useLog();
const { repoPath } = useRepo();
const { diffFile, diffCommit } = useDiff();
const { open: openCompare } = useFileCompare();

const activeFilter = ref<string>("all");
const fileFilter = ref("");
const commitFiles = ref<FileDiff[]>([]);

const isWorkingTree = computed(() => !selectedCommit.value || selectedCommit.value === "__worktree__");

// Load commit files when a commit is selected
watch(selectedCommit, async (oid) => {
  if (!oid || oid === "__worktree__" || !repoPath.value) {
    commitFiles.value = [];
    return;
  }
  try {
    commitFiles.value = await invoke<FileDiff[]>("get_diff_commit", { repoPath: repoPath.value, oid });
  } catch {
    commitFiles.value = [];
  }
});

const filteredFiles = computed(() => {
  if (!isWorkingTree.value) return [];
  let result = files.value;
  if (activeFilter.value === "modified") result = result.filter((f) => f.state === "modified");
  else if (activeFilter.value === "staged") result = result.filter((f) => f.staged === "staged" || f.staged === "partial");
  else if (activeFilter.value === "untracked") result = result.filter((f) => f.state === "untracked");
  else if (activeFilter.value === "conflicted") result = result.filter((f) => f.state === "conflicted");
  const q = fileFilter.value.trim().toLowerCase();
  if (q) {
    result = result.filter((f) => {
      const hay = `${f.path} ${f.state} ${f.staged}`.toLowerCase();
      return hay.includes(q);
    });
  }
  return result;
});

const filteredCommitFiles = computed(() => {
  const q = fileFilter.value.trim().toLowerCase();
  if (!q) return commitFiles.value;
  return commitFiles.value.filter((f) => {
    const hay = `${f.path} ${f.insertions} ${f.deletions}`.toLowerCase();
    return hay.includes(q);
  });
});

const displayCount = computed(() => isWorkingTree.value ? filteredFiles.value.length : filteredCommitFiles.value.length);

const filters = [
  { key: "all", label: "All" },
  { key: "modified", label: "Modified" },
  { key: "staged", label: "Staged" },
  { key: "untracked", label: "Untracked" },
  { key: "conflicted", label: "Conflicted" },
];

const stateIcons: Record<string, { color: string; letter: string }> = {
  modified: { color: "var(--blue)", letter: "M" },
  added: { color: "var(--green)", letter: "A" },
  deleted: { color: "var(--red)", letter: "D" },
  renamed: { color: "var(--purple)", letter: "R" },
  conflicted: { color: "var(--red)", letter: "C" },
  untracked: { color: "var(--text-muted)", letter: "?" },
};

function fileName(path: string): string {
  return path.split("/").pop() || path;
}

function fileDir(path: string): string {
  const parts = path.split("/");
  parts.pop();
  return parts.join("/");
}

async function selectFile(path: string) {
  selectedFile.value = path;
  if (isWorkingTree.value) {
    const f = files.value.find((x) => x.path === path);
    const staged = f?.staged === "staged";
    await diffFile(path, staged);
  }
}

async function selectCommitFile(path: string) {
  selectedFile.value = path;
  if (selectedCommit.value && selectedCommit.value !== "__worktree__") {
    await diffCommit(selectedCommit.value, path);
  }
}

function compareWorkingTreeFile(path: string) {
  const f = files.value.find((x) => x.path === path);
  openCompare({ path, staged: f?.staged === "staged" });
}
function compareCommitFile(path: string) {
  if (selectedCommit.value && selectedCommit.value !== "__worktree__") {
    openCompare({ path, oid: selectedCommit.value });
  }
}
</script>

<template>
  <div class="file-list">
    <div class="file-list-header">
      <div class="header-left">
        <span class="panel-title">Files</span>
        <span class="files-hidden">{{ displayCount }} files</span>
      </div>
      <div class="header-right">
        <input
          v-model="fileFilter"
          type="text"
          placeholder="Filter"
          class="file-filter-input"
        />
        <div class="filter-tabs">
          <button
            v-for="f in filters"
            :key="f.key"
            class="filter-tab"
            :class="{ active: activeFilter === f.key }"
            @click="activeFilter = f.key"
            :title="f.label"
          >
            {{ f.label }}
          </button>
        </div>
      </div>
    </div>

    <div class="file-list-body" @mousedown="(e) => e.detail > 1 && e.preventDefault()" @contextmenu.prevent>
      <!-- Working tree files -->
      <template v-if="isWorkingTree">
        <div
          v-for="file in filteredFiles"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedFile === file.path }"
          @click="selectFile(file.path)"
          @dblclick="compareWorkingTreeFile(file.path)"
        >
          <span
            class="state-badge"
            :style="{ color: stateIcons[file.state]?.color }"
            :title="file.state"
          >
            {{ stateIcons[file.state]?.letter }}
          </span>

          <span
            v-if="file.staged === 'staged'"
            class="staged-dot"
            title="Staged"
          />
          <span
            v-else-if="file.staged === 'partial'"
            class="staged-dot partial"
            title="Partially staged"
          />

          <span class="file-name" v-html="highlight(fileName(file.path), fileFilter)" />
          <span class="file-dir" v-html="highlight(fileDir(file.path), fileFilter)" />
        </div>
      </template>

      <!-- Commit files -->
      <template v-else>
        <div
          v-for="cf in filteredCommitFiles"
          :key="cf.path"
          class="file-item"
          :class="{ selected: selectedFile === cf.path }"
          @click="selectCommitFile(cf.path)"
          @dblclick="compareCommitFile(cf.path)"
        >
          <span class="state-badge" :style="{ color: 'var(--blue)' }">M</span>
          <span class="file-name" v-html="highlight(fileName(cf.path), fileFilter)" />
          <span class="file-dir" v-html="highlight(fileDir(cf.path), fileFilter)" />
          <span class="file-stats">
            <span class="stat-add">+{{ cf.insertions }}</span>
            <span class="stat-del">-{{ cf.deletions }}</span>
          </span>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.file-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  user-select: none;
  -webkit-user-select: none;
}

.file-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-tertiary);
  gap: 8px;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.files-hidden {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
}
.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  justify-content: flex-end;
}
.file-filter-input {
  width: 140px;
  padding: 2px 6px;
  font-size: var(--font-size-xs);
  border-color: color-mix(in srgb, var(--border-subtle) 45%, transparent);
}

.panel-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-secondary);
}

.filter-tabs {
  display: flex;
  gap: 2px;
}

.filter-tab {
  padding: 2px 6px;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  border-radius: var(--radius);
}
.filter-tab:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}
.filter-tab.active {
  background: var(--bg-surface);
  color: var(--text-primary);
}

.file-list-body {
  flex: 1;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px;
  cursor: pointer;
  font-size: var(--font-size-sm);
  user-select: none;
  -webkit-user-select: none;
}
.file-item:hover {
  background: var(--bg-hover);
}
.file-item.selected {
  background: var(--bg-surface);
}

.state-badge {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  font-weight: 700;
  width: 14px;
  text-align: center;
  flex-shrink: 0;
}

.staged-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--green);
  flex-shrink: 0;
}
.staged-dot.partial {
  background: var(--yellow);
}

.file-name {
  color: var(--text-primary);
  white-space: nowrap;
}

.file-dir {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-stats {
  margin-left: auto;
  display: flex;
  gap: 6px;
  font-size: var(--font-size-xs);
  font-family: var(--font-mono);
  flex-shrink: 0;
}
.stat-add { color: var(--green); }
.stat-del { color: var(--red); }
</style>
