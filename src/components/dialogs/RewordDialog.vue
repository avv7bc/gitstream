<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{
  oid: string;
  message: string;
  isHead: boolean;
}>();

const emit = defineEmits<{
  confirm: [message: string];
  close: [];
}>();

const { dragStyle, onDragStart } = useDraggable();

const text = ref("");
const busy = ref(false);

onMounted(() => {
  text.value = props.message;
});

async function doConfirm() {
  if (busy.value || !text.value.trim()) return;
  busy.value = true;
  try {
    emit("confirm", text.value.trim());
  } finally {
    busy.value = false;
  }
}

function onTextareaKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") { emit("close"); return; }
  if ((e.ctrlKey || e.metaKey) && e.key === "Enter") { doConfirm(); return; }
  e.stopPropagation();
}
</script>

<template>
  <div class="modal-overlay" @mousedown.self="emit('close')">
    <div class="rw-dialog" :style="dragStyle" @mousedown.stop>
      <div class="rw-titlebar" @mousedown="onDragStart">
        <span class="rw-title">Edit Message — {{ oid.slice(0, 8) }}</span>
        <button class="rw-close" @click="emit('close')" title="Close">
          <svg width="14" height="14" viewBox="0 0 16 16">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
      </div>

      <div class="rw-body">
        <label class="rw-label">Commit message</label>
        <textarea
          v-model="text"
          class="rw-textarea"
          placeholder="Enter commit message…"
          autofocus
          @keydown="onTextareaKeydown"
        />
        <div class="rw-hint">Ctrl+Enter to confirm{{ !isHead ? ' · rebase -i will be used' : '' }}</div>
      </div>

      <div class="rw-footer">
        <button class="rw-btn-cancel" @click="emit('close')">Cancel</button>
        <button class="rw-btn-confirm" :disabled="busy || !text.trim()" @click="doConfirm">
          {{ busy ? 'Saving…' : 'Save' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 150;
  display: flex;
  align-items: center;
  justify-content: center;
}
.rw-dialog {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55);
  display: flex;
  flex-direction: column;
  width: 520px;
  min-width: 380px;
}
.rw-titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding: 0 8px 0 14px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  cursor: move;
  user-select: none;
  flex-shrink: 0;
}
.rw-title {
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  font-weight: 500;
}
.rw-close {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  background: transparent; border: none;
  color: var(--text-secondary);
  border-radius: var(--radius);
  cursor: pointer;
}
.rw-close:hover { background: var(--red); color: #fff; }
.rw-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.rw-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.rw-textarea {
  min-height: 140px;
  resize: vertical;
  padding: 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  font-family: var(--font-mono);
  line-height: 1.5;
}
.rw-textarea:focus {
  outline: none;
  border-color: var(--accent);
}
.rw-hint {
  font-size: 10px;
  color: var(--text-secondary);
  opacity: 0.7;
  text-align: right;
}
.rw-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
.rw-btn-cancel {
  padding: 5px 14px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
}
.rw-btn-cancel:hover { color: var(--text-primary); background: var(--bg-hover); }
.rw-btn-confirm {
  padding: 5px 16px;
  font-size: var(--font-size-sm);
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius);
  cursor: pointer;
  font-weight: 500;
}
.rw-btn-confirm:hover:not(:disabled) { opacity: 0.88; }
.rw-btn-confirm:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
