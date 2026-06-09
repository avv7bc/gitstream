<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

const emit = defineEmits<{
  close: [];
  confirm: [url: string, dest: string, name: string];
}>();

const url = ref("");
const parentDir = ref(localStorage.getItem("gitstream:last-clone-parent") ?? "");
const folderName = ref("");
const folderTouched = ref(false);
const urlRef = ref<HTMLInputElement | null>(null);

const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

// Имя папки из URL: последний сегмент пути без .git
function deriveName(u: string): string {
  const trimmed = u.trim().replace(/\/+$/, "");
  if (!trimmed) return "";
  const last = trimmed.split(/[/\\:]/).filter(Boolean).pop() ?? "";
  return last.replace(/\.git$/i, "");
}

function onUrlInput() {
  if (!folderTouched.value) folderName.value = deriveName(url.value);
}

function onFolderInput() {
  folderTouched.value = true;
}

const destPath = computed(() => {
  const p = parentDir.value.trim().replace(/[/\\]+$/, "");
  const n = folderName.value.trim();
  if (!p || !n) return "";
  const sep = p.includes("\\") ? "\\" : "/";
  return `${p}${sep}${n}`;
});

const canClone = computed(
  () => url.value.trim().length > 0 && parentDir.value.trim().length > 0 && folderName.value.trim().length > 0,
);

async function browseFolder() {
  const selected = await open({ directory: true, multiple: false, title: "Select destination directory" });
  if (selected) parentDir.value = selected as string;
}

function submit() {
  if (!canClone.value) return;
  localStorage.setItem("gitstream:last-clone-parent", parentDir.value.trim());
  emit("confirm", url.value.trim(), destPath.value, folderName.value.trim());
}

onMounted(() => {
  urlRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog clone-repo-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.clone.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-hint">{{ i18n.dialog.clone.hint }}</p>
        <p class="dialog-subhint">{{ i18n.dialog.clone.subhint }}</p>

        <div class="form-group">
          <label class="form-label" for="clone-url-input">{{ i18n.dialog.clone.urlLabel }}</label>
          <input
            id="clone-url-input"
            ref="urlRef"
            v-model="url"
            class="form-input"
            type="text"
            placeholder="https://github.com/user/repo.git"
            @input="onUrlInput"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>

        <div class="form-group">
          <label class="form-label" for="clone-parent-input">{{ i18n.dialog.clone.dirLabel }}</label>
          <div class="input-row">
            <input
              id="clone-parent-input"
              v-model="parentDir"
              class="form-input"
              type="text"
              placeholder="/path/to/parent"
              @keydown.enter="submit"
              @keydown.escape="$emit('close')"
            />
            <button class="btn btn-browse" title="Browse..." @click="browseFolder">
              <svg width="16" height="16" viewBox="0 0 16 16"><path d="M2 4h5l1.5-2H14v10H2V4z" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
            </button>
          </div>
        </div>

        <div class="form-group">
          <label class="form-label" for="clone-name-input">{{ i18n.dialog.clone.nameLabel }}</label>
          <input
            id="clone-name-input"
            v-model="folderName"
            class="form-input"
            type="text"
            placeholder="repo"
            @input="onFolderInput"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
          <p v-if="destPath" class="path-status status-ok">{{ destPath }}</p>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.clone.cancel }}</button>
        <button class="btn btn-primary" :disabled="!canClone" @click="submit">{{ i18n.dialog.clone.ok }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.clone-repo-dialog {
  width: 480px;
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

.input-row {
  display: flex;
  gap: 6px;
}

.input-row .form-input {
  flex: 1;
}

.btn-browse {
  padding: 6px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text-secondary);
  cursor: pointer;
  flex-shrink: 0;
}
.btn-browse:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.path-status {
  font-size: var(--font-size-xs);
  margin-top: 6px;
  word-break: break-all;
}
.status-ok {
  color: var(--green);
}
</style>
