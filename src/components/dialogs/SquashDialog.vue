<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import type { CommitInfo } from "@/types";

const props = defineProps<{
  oids: string[];
  commits: CommitInfo[];
}>();

const emit = defineEmits<{
  confirm: [message: string];
  close: [];
}>();

const { dragStyle, onDragStart } = useDraggable();

const message = ref("");
const busy = ref(false);

onMounted(() => {
  // Pre-fill with all commit messages, newest first, separated by blank lines
  message.value = props.commits.map(c => c.message.trim()).join("\n\n");
});

const canConfirm = computed(() => !busy.value && message.value.trim().length > 0);

async function doSquash() {
  if (!canConfirm.value) return;
  busy.value = true;
  try {
    emit("confirm", message.value.trim());
  } finally {
    busy.value = false;
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
  if ((e.ctrlKey || e.metaKey) && e.key === "Enter") doSquash();
}
</script>

<template>
  <div class="modal-overlay" @mousedown.self="emit('close')" @keydown="onKeydown">
    <div class="sq-dialog" :style="dragStyle" @mousedown.stop>
      <div class="sq-titlebar" @mousedown="onDragStart">
        <span class="sq-title">Squash {{ oids.length }} commits</span>
        <button class="sq-close" @click="emit('close')" title="Close">
          <svg width="14" height="14" viewBox="0 0 16 16">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
      </div>

      <div class="sq-body">
        <div class="sq-hint">Commits being squashed (newest → oldest):</div>
        <div class="sq-commit-list">
          <div v-for="c in commits" :key="c.oid" class="sq-commit-item">
            <span class="sq-hash">{{ c.short_oid }}</span>
            <span class="sq-msg">{{ c.message.split('\n')[0] }}</span>
          </div>
        </div>

        <label class="sq-label">Combined commit message</label>
        <textarea
          v-model="message"
          class="sq-textarea"
          placeholder="Enter commit message…"
          autofocus
          @keydown.stop
        />
        <div class="sq-hint-small">Ctrl+Enter to confirm</div>
      </div>

      <div class="sq-footer">
        <button class="sq-btn-cancel" @click="emit('close')">Cancel</button>
        <button class="sq-btn-confirm" :disabled="!canConfirm" @click="doSquash">
          {{ busy ? 'Squashing…' : 'Squash' }}
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
.sq-dialog {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55);
  display: flex;
  flex-direction: column;
  width: 520px;
  max-height: 80vh;
  min-width: 380px;
}
.sq-titlebar {
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
.sq-title {
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  font-weight: 500;
}
.sq-close {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  background: transparent; border: none;
  color: var(--text-secondary);
  border-radius: var(--radius);
  cursor: pointer;
}
.sq-close:hover { background: var(--red); color: #fff; }

.sq-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sq-hint {
  font-size: 11px;
  color: var(--text-secondary);
}
.sq-hint-small {
  font-size: 10px;
  color: var(--text-secondary);
  opacity: 0.7;
  text-align: right;
}
.sq-commit-list {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  overflow: hidden;
  max-height: 140px;
  overflow-y: auto;
}
.sq-commit-item {
  display: flex;
  gap: 8px;
  padding: 4px 8px;
  font-size: 12px;
  border-bottom: 1px solid var(--border-subtle);
}
.sq-commit-item:last-child { border-bottom: none; }
.sq-hash {
  color: var(--accent);
  font-family: var(--font-mono);
  flex-shrink: 0;
}
.sq-msg {
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sq-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.sq-textarea {
  flex: 1;
  min-height: 120px;
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
.sq-textarea:focus {
  outline: none;
  border-color: var(--accent);
}
.sq-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
.sq-btn-cancel {
  padding: 5px 14px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
}
.sq-btn-cancel:hover { color: var(--text-primary); background: var(--bg-hover); }
.sq-btn-confirm {
  padding: 5px 16px;
  font-size: var(--font-size-sm);
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius);
  cursor: pointer;
  font-weight: 500;
}
.sq-btn-confirm:hover:not(:disabled) { opacity: 0.88; }
.sq-btn-confirm:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
