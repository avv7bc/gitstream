<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

defineEmits<{
  close: [];
  confirm: [name: string];
}>();

const groupName = ref("");
const inputRef = ref<HTMLInputElement | null>(null);
const canAdd = computed(() => groupName.value.trim().length > 0);
const { dragStyle, onDragStart } = useDraggable();

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1" ref="overlayRef">
    <div class="modal-dialog add-group-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Add Group</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-hint">Enter the group name</p>
        <p class="dialog-subhint">After having created the group, you can move repositories inside it.</p>
        <div class="form-group">
          <label class="form-label" for="group-name-input">Group Name:</label>
          <input
            id="group-name-input"
            ref="inputRef"
            v-model="groupName"
            class="form-input"
            type="text"
            @keydown.enter="canAdd && $emit('confirm', groupName.trim())"
            @keydown.escape="$emit('close')"
          />
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button
          class="btn btn-primary"
          :disabled="!canAdd"
          @click="$emit('confirm', groupName.trim())"
        >
          Add
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-group-dialog {
  width: 400px;
}

.dialog-hint {
  font-size: 15px;
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
</style>
