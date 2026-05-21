<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

const props = defineProps<{ oldName: string }>();

defineEmits<{
  close: [];
  confirm: [newName: string];
}>();

const newName = ref(props.oldName);
const inputRef = ref<HTMLInputElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();
const canRename = computed(() => {
  const trimmed = newName.value.trim();
  return trimmed.length > 0 && trimmed !== props.oldName;
});

onMounted(() => {
  inputRef.value?.focus();
  inputRef.value?.select();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog rename-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.renameBranch.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-subhint">{{ i18n.dialog.renameBranch.subhint }} <b>{{ oldName }}</b></p>
        <div class="form-group">
          <label class="form-label" for="rename-branch-input">{{ i18n.dialog.renameBranch.newNameLabel }}</label>
          <input
            id="rename-branch-input"
            ref="inputRef"
            v-model="newName"
            class="form-input"
            type="text"
            @keydown.enter="canRename && $emit('confirm', newName.trim())"
            @keydown.escape="$emit('close')"
          />
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.renameBranch.cancel }}</button>
        <button
          class="btn btn-primary"
          :disabled="!canRename"
          @click="$emit('confirm', newName.trim())"
        >
          {{ i18n.dialog.renameBranch.rename }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.rename-dialog {
  width: 400px;
}
.dialog-subhint {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin-bottom: 12px;
}
</style>
