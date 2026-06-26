<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@/composables/useProgress";
import { open } from "@tauri-apps/plugin-dialog";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";
import type { RepoPathCheck } from "@/types";

const emit = defineEmits<{
  close: [];
  confirm: [path: string, name: string, isGitRepo: boolean];
}>();

const lastPath = localStorage.getItem("gitstream:last-repo-path") ?? "";
const repoPath = ref(lastPath);
const pathCheck = ref<RepoPathCheck | null>(null);
const checking = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

let checkTimer: ReturnType<typeof setTimeout> | null = null;

const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

const canAdd = computed(() => {
  return repoPath.value.trim().length > 0 && pathCheck.value?.exists === true;
});

const statusText = computed(() => {
  if (!repoPath.value.trim() || !pathCheck.value) return "";
  if (!pathCheck.value.exists) return i18n.value.dialog.addRepo.dirNotExist;
  if (pathCheck.value.is_git_repo) return i18n.value.dialog.addRepo.gitFound;
  return i18n.value.dialog.addRepo.notGitWillInit;
});

const statusClass = computed(() => {
  if (!pathCheck.value || !repoPath.value.trim()) return "";
  if (!pathCheck.value.exists) return "status-error";
  if (pathCheck.value.is_git_repo) return "status-ok";
  return "status-warn";
});

function onPathInput() {
  pathCheck.value = null;
  if (checkTimer) clearTimeout(checkTimer);
  const val = repoPath.value.trim();
  if (!val) return;
  checking.value = true;
  checkTimer = setTimeout(async () => {
    try {
      pathCheck.value = await invoke<RepoPathCheck>("check_repo_path", { path: val });
    } catch {
      pathCheck.value = null;
    }
    checking.value = false;
  }, 300);
}

// Снять висящий debounce-таймер, если диалог закрыли в течение 300 мс после
// ввода — иначе колбэк выстрелит на размонтированном компоненте.
onUnmounted(() => {
  if (checkTimer) clearTimeout(checkTimer);
});

async function browseFolder() {
  try {
    const selected = await open({ directory: true, multiple: false, title: "Select repository directory" });
    if (selected) {
      repoPath.value = selected as string;
      onPathInput();
    }
  } catch {
    // Отмена/ошибка системного диалога выбора папки — игнорируем.
  }
}

function submit() {
  if (!canAdd.value || !pathCheck.value) return;
  localStorage.setItem("gitstream:last-repo-path", repoPath.value.trim());
  emit("confirm", repoPath.value.trim(), pathCheck.value.display_name, pathCheck.value.is_git_repo);
}

onMounted(() => {
  inputRef.value?.focus();
  if (repoPath.value) onPathInput();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1" ref="overlayRef">
    <div class="modal-dialog add-repo-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.addRepo.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-hint">{{ i18n.dialog.addRepo.hint }}</p>
        <p class="dialog-subhint">{{ i18n.dialog.addRepo.subhint }}</p>
        <div class="form-group">
          <label class="form-label" for="repo-path-input">{{ i18n.dialog.addRepo.label }}</label>
          <div class="input-row">
            <input
              id="repo-path-input"
              ref="inputRef"
              v-model="repoPath"
              class="form-input"
              type="text"
              placeholder="/path/to/repository"
              @input="onPathInput"
              @keydown.enter="submit"
              @keydown.escape="$emit('close')"
            />
            <button class="btn btn-browse" title="Browse..." @click="browseFolder">
              <svg width="16" height="16" viewBox="0 0 16 16"><path d="M2 4h5l1.5-2H14v10H2V4z" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
            </button>
          </div>
          <p v-if="statusText" class="path-status" :class="statusClass">{{ statusText }}</p>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.addRepo.cancel }}</button>
        <button class="btn btn-primary" :disabled="!canAdd" @click="submit">{{ i18n.dialog.addRepo.ok }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-repo-dialog {
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
}
.status-ok {
  color: var(--green);
}
.status-warn {
  color: var(--yellow);
}
.status-error {
  color: var(--red);
}
</style>
