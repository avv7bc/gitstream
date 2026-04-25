<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useCommit } from "@/composables/useCommit";
import { useFiles } from "@/composables/useFiles";
import { useRemote } from "@/composables/useRemote";
import { useBranches } from "@/composables/useBranches";
import { useDraggable } from "@/composables/useDraggable";
import type { FileStatus } from "@/types";

const emit = defineEmits<{ close: [] }>();

const { commit: doCommit } = useCommit();
const { files, stageFiles, unstageFiles, refresh } = useFiles();
const { push } = useRemote();
const { remotes } = useBranches();

const message = ref("");
const amend = ref(false);
const busy = ref(false);

const { dragStyle, onDragStart } = useDraggable();

const changedFiles = computed(() => files.value);

const selected = ref<Record<string, boolean>>({});

onMounted(() => {
  for (const f of changedFiles.value) {
    selected.value[f.path] = true;
  }
});

const selectedPaths = computed(() =>
  Object.entries(selected.value).filter(([, v]) => v).map(([k]) => k)
);

const firstLineLength = computed(() => (message.value.split("\n")[0] || "").length);
const firstLineClass = computed(() => {
  if (firstLineLength.value > 72) return "error";
  if (firstLineLength.value > 50) return "warning";
  return "ok";
});

const canCommit = computed(
  () => !busy.value && message.value.trim().length > 0 && selectedPaths.value.length > 0
);

function fileName(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1];
}

function fileDir(path: string): string {
  const parts = path.split("/");
  return parts.length > 1 ? parts.slice(0, -1).join("/") : "";
}

function isStagedLike(f: FileStatus): boolean {
  return f.staged === "staged" || f.staged === "partial";
}

async function reconcileIndex() {
  const checked = new Set(selectedPaths.value);

  const toUnstage = changedFiles.value
    .filter((f) => isStagedLike(f) && !checked.has(f.path))
    .map((f) => f.path);

  const toStage = changedFiles.value
    .filter((f) => checked.has(f.path) && !isStagedLike(f))
    .map((f) => f.path);

  if (toUnstage.length > 0) await unstageFiles(toUnstage);
  if (toStage.length > 0) await stageFiles(toStage);
}

async function handleCommit(alsoPush: boolean) {
  if (!canCommit.value) return;
  busy.value = true;
  try {
    await reconcileIndex();
    await doCommit(message.value, amend.value);
    if (alsoPush) {
      const remote = remotes.value[0] ?? "origin";
      await push(remote, false);
    }
    await refresh();
    emit("close");
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog commit-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Commit</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-hint">Commit local or staged changes</p>
        <p class="dialog-subhint">Select the files to commit and provide a commit message.</p>

        <div class="file-table">
          <div class="file-table-header">
            <span class="col-check"></span>
            <span class="col-name">Name</span>
            <span class="col-dir">Directory</span>
          </div>
          <div class="file-table-body">
            <label
              v-for="f in changedFiles"
              :key="f.path"
              class="file-row"
            >
              <input type="checkbox" v-model="selected[f.path]" class="col-check" />
              <span class="col-name">{{ fileName(f.path) }}</span>
              <span class="col-dir">{{ fileDir(f.path) }}</span>
            </label>
            <div v-if="changedFiles.length === 0" class="empty">No changes</div>
          </div>
        </div>

        <div class="message-field">
          <textarea
            v-model="message"
            placeholder="Commit message..."
            rows="5"
            class="commit-message-input"
          />
          <div class="message-indicator" :class="firstLineClass">
            {{ firstLineLength }} / 72
          </div>
        </div>

        <div class="commit-options">
          <label class="checkbox-label">
            <input type="checkbox" v-model="amend" />
            <span>Amend last commit</span>
          </label>
          <span class="selected-count">{{ selectedPaths.length }} selected</span>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-primary" :disabled="!canCommit" @click="handleCommit(false)">
          Commit
        </button>
        <button class="btn btn-accent" :disabled="!canCommit" @click="handleCommit(true)">
          Commit &amp; Push
        </button>
        <div class="footer-spacer"></div>
        <button class="btn btn-secondary" :disabled="busy" @click="$emit('close')">Cancel</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.commit-dialog {
  width: 560px;
}

.dialog-hint {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.dialog-subhint {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin-bottom: 16px;
  line-height: 1.4;
}

.file-table {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  margin-bottom: 12px;
}

.file-table-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-subtle);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
}

.file-table-body {
  max-height: 200px;
  overflow-y: auto;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.file-row:hover {
  background: var(--bg-hover);
}

.col-check {
  width: 20px;
  flex-shrink: 0;
  accent-color: var(--accent);
}

.col-name {
  width: 200px;
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.col-dir {
  flex: 1;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-xs);
}

.empty {
  padding: 12px;
  text-align: center;
  font-size: var(--font-size-sm);
  color: var(--text-muted);
}

.message-field {
  position: relative;
}

.commit-message-input {
  width: 100%;
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  resize: vertical;
  min-height: 100px;
  padding: 8px;
}

.message-indicator {
  position: absolute;
  bottom: 6px;
  right: 8px;
  font-size: var(--font-size-xs);
  padding: 1px 6px;
  border-radius: var(--radius);
}
.message-indicator.ok { color: var(--text-muted); }
.message-indicator.warning { color: var(--yellow); }
.message-indicator.error { color: var(--red); }

.commit-options {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.checkbox-label input {
  accent-color: var(--accent);
}

.selected-count {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
}

.footer-spacer {
  flex: 1;
}

.dialog-footer {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
