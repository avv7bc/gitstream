<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

interface Props {
  oldName: string;
  nodeType: "repo" | "folder";
}

const props = defineProps<Props>();

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

const dialogTitle = computed(() => {
  return props.nodeType === "repo" ? i18n.value.dialog.renameNode.titleRepo : i18n.value.dialog.renameNode.titleGroup;
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
        <h3>{{ dialogTitle }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-subhint" v-if="nodeType === 'repo'">
          {{ i18n.dialog.renameNode.subhintRepo }} <b>{{ oldName }}</b>
        </p>
        <p class="dialog-subhint" v-else>
          {{ i18n.dialog.renameNode.subhintGroup }} <b>{{ oldName }}</b>
        </p>
        <div class="form-group">
          <label class="form-label" for="rename-input">{{ i18n.dialog.renameNode.newNameLabel }}</label>
          <input
            id="rename-input"
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
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.renameNode.cancel }}</button>
        <button
          class="btn btn-primary"
          :disabled="!canRename"
          @click="$emit('confirm', newName.trim())"
        >
          {{ i18n.dialog.renameNode.rename }}
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
