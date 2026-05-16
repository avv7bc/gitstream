<script setup lang="ts">
import { ref } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{
  message: string;
  confirmLabel?: string;
  danger?: boolean;
  checkboxLabel?: string;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [checkboxChecked: boolean];
}>();

const checked = ref(false);
const { dragStyle, onDragStart } = useDraggable();

function onConfirm() {
  emit("confirm", props.checkboxLabel ? checked.value : false);
}
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
        <label v-if="checkboxLabel" class="confirm-checkbox">
          <input type="checkbox" v-model="checked" />
          {{ checkboxLabel }}
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button
          class="btn"
          :class="danger ? 'btn-danger' : 'btn-primary'"
          @click="onConfirm"
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
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.confirm-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}
</style>
