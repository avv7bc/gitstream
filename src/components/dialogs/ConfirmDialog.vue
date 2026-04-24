<script setup lang="ts">
import { useDraggable } from "@/composables/useDraggable";

defineProps<{
  message: string;
  confirmLabel?: string;
  danger?: boolean;
}>();

defineEmits<{
  close: [];
  confirm: [];
}>();

const { dragStyle, onDragStart } = useDraggable();
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog confirm-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Confirm</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="confirm-message">{{ message }}</p>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button
          class="btn"
          :class="danger ? 'btn-danger' : 'btn-primary'"
          @click="$emit('confirm')"
        >
          {{ confirmLabel || "Confirm" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.confirm-dialog {
  width: 360px;
}

.confirm-message {
  font-size: var(--font-size);
  color: var(--text-secondary);
  line-height: 1.5;
}
</style>
