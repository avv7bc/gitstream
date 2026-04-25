<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useFiles } from "@/composables/useFiles";
import { useDraggable } from "@/composables/useDraggable";

const emit = defineEmits<{ close: [] }>();

const { files, discardFiles, unstageFiles, refresh } = useFiles();
const { dragStyle, onDragStart } = useDraggable();

const revertTo = ref<"head" | "index">("head");

// Files with changes (not clean)
const changedFiles = computed(() => files.value.filter((f) => f.state !== "untracked" || f.staged !== "unstaged"));

// Selection state: file path -> checked
const selected = ref<Record<string, boolean>>({});

onMounted(() => {
  for (const f of changedFiles.value) {
    selected.value[f.path] = true;
  }
});

const selectedPaths = computed(() =>
  Object.entries(selected.value).filter(([, v]) => v).map(([k]) => k)
);

const canDiscard = computed(() => selectedPaths.value.length > 0);

function fileName(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1];
}

function fileDir(path: string): string {
  const parts = path.split("/");
  return parts.length > 1 ? parts.slice(0, -1).join("/") : "";
}

async function handleDiscard() {
  if (!canDiscard.value) return;
  const paths = selectedPaths.value;
  if (revertTo.value === "head") {
    // Unstage first, then discard working tree changes
    await unstageFiles(paths);
    await discardFiles(paths);
  } else {
    // Revert to index = just discard working tree changes
    await discardFiles(paths);
  }
  await refresh();
  emit("close");
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog discard-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Discard</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-hint">Discard local or staged changes</p>
        <p class="dialog-subhint">Select the files for which changes should be discarded and whether to set them back to Index or HEAD state.</p>

        <div class="revert-to">
          <span class="revert-label">Revert to:</span>
          <label class="radio-label">
            <input type="radio" v-model="revertTo" value="head" />
            <span>HEAD</span>
          </label>
          <label class="radio-label">
            <input type="radio" v-model="revertTo" value="index" />
            <span>Index</span>
          </label>
        </div>

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
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-danger" :disabled="!canDiscard" @click="handleDiscard">Discard</button>
        <div class="footer-spacer"></div>
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.discard-dialog {
  width: 520px;
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

.revert-to {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.revert-label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.radio-label input {
  accent-color: var(--accent);
}

.file-table {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
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
  width: 180px;
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
