<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

const props = defineProps<{
  message: string;
  confirmLabel?: string;
  danger?: boolean;
  checkboxLabel?: string;
  items?: string[];
}>();

const emit = defineEmits<{
  close: [];
  confirm: [checkboxChecked: boolean];
}>();

const checked = ref(false);
const dialogRef = ref<HTMLElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

// Забираем фокус с элемента под диалогом: иначе Enter активирует его
// (или кнопку подтверждения) — для quit-диалога это закрывало программу.
onMounted(() => {
  dialogRef.value?.focus();
});

// Enter = подтвердить (как в PullDialog). Если фокус на конкретной кнопке
// (Tab-навигация) — активируется она, иначе срабатывает кнопка подтверждения.
function onEnterKey(e: KeyboardEvent) {
  if ((e.target as HTMLElement)?.tagName === "BUTTON") return;
  e.preventDefault();
  onConfirm();
}

function onConfirm() {
  emit("confirm", props.checkboxLabel ? checked.value : false);
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog confirm-dialog" :style="dragStyle" tabindex="-1" ref="dialogRef" @keydown.enter="onEnterKey">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.confirm.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="confirm-message">{{ message }}</p>
        <ul v-if="items && items.length" class="confirm-list">
          <li v-for="item in items" :key="item">{{ item }}</li>
        </ul>
        <label v-if="checkboxLabel" class="confirm-checkbox">
          <input type="checkbox" v-model="checked" />
          {{ checkboxLabel }}
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.confirm.cancel }}</button>
        <button
          class="btn"
          :class="danger ? 'btn-danger' : 'btn-primary'"
          @click="onConfirm"
        >
          {{ confirmLabel || i18n.dialog.confirm.ok }}
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

.confirm-list {
  margin: 8px 0 0;
  padding: 6px 8px;
  max-height: 160px;
  overflow-y: auto;
  list-style: none;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}
.confirm-list li {
  padding: 1px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
