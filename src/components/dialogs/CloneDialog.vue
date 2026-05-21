<script setup lang="ts">
import { ref, computed } from "vue";
import { useRemote } from "@/composables/useRemote";
import { useRepo } from "@/composables/useRepo";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

const emit = defineEmits<{ close: [] }>();

const { cloneRepo, isBusy } = useRemote();
const { openRepo } = useRepo();
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

const url = ref("");
const directory = ref("");
const error = ref("");

const autoName = computed(() => {
  if (!url.value) return "";
  const match = url.value.match(/\/([^/]+?)(\.git)?$/);
  return match ? match[1] : "";
});

const canClone = computed(() => !isBusy.value && url.value.trim() && (directory.value.trim() || autoName.value));

async function handleClone() {
  const dest = directory.value || autoName.value;
  try {
    await cloneRepo(url.value, dest);
    await openRepo(dest);
    emit("close");
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog clone-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.clone.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label class="form-label">{{ i18n.dialog.clone.urlLabel }}</label>
          <input
            v-model="url"
            type="text"
            :placeholder="i18n.dialog.clone.urlPlaceholder"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label class="form-label">{{ i18n.dialog.clone.directoryLabel }}</label>
          <div class="input-with-btn">
            <input
              v-model="directory"
              type="text"
              :placeholder="autoName || i18n.dialog.clone.directoryPlaceholder"
              class="form-input"
            />
            <button class="btn btn-secondary browse-btn">{{ i18n.dialog.clone.browse }}</button>
          </div>
        </div>

        <div v-if="error" class="error-message">{{ error }}</div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.clone.cancel }}</button>
        <button class="btn btn-primary" :disabled="!canClone" @click="handleClone">{{ i18n.dialog.clone.clone }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.clone-dialog {
  width: 500px;
}

.input-with-btn {
  display: flex;
  gap: 8px;
}
.input-with-btn .form-input {
  flex: 1;
}
.browse-btn {
  flex-shrink: 0;
}

.error-message {
  color: var(--red);
  font-size: var(--font-size-sm);
  margin-top: 8px;
}
</style>
