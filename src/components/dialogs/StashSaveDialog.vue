<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const emit = defineEmits<{
  close: [];
  confirm: [payload: { message: string | null; includeUntracked: boolean }];
}>();

const message = ref("");
const includeUntracked = ref(true);
const inputRef = ref<HTMLInputElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();

function submit() {
  emit("confirm", {
    message: message.value.trim() ? message.value.trim() : null,
    includeUntracked: includeUntracked.value,
  });
}

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog stash-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Stash Changes</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label class="form-label" for="stash-msg">Message (optional):</label>
          <input
            id="stash-msg"
            ref="inputRef"
            v-model="message"
            class="form-input"
            type="text"
            placeholder="WIP: ..."
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>

        <label class="stash-opt">
          <input type="checkbox" v-model="includeUntracked" />
          Include untracked files
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button class="btn btn-primary" @click="submit">Stash</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stash-dialog {
  width: 420px;
}
.stash-opt {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  margin-top: 4px;
}
</style>
