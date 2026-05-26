<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{
  message: string;
  log: string[];
}>();

const emit = defineEmits<{ close: [] }>();

const { dragStyle, onDragStart } = useDraggable();
const copied = ref(false);

function copyLog() {
  const text = [props.message, "", ...props.log].join("\n");
  navigator.clipboard.writeText(text).then(() => {
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  });
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog network-error-dialog" :style="dragStyle">

      <div class="dialog-header" @mousedown="onDragStart">
        <div class="header-title">
          <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" class="error-icon">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm-.75 4a.75.75 0 0 1 1.5 0v3.5a.75.75 0 0 1-1.5 0V5zm.75 7a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"/>
          </svg>
          <span>Git error</span>
        </div>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="error-message">{{ message }}</div>

        <template v-if="log.length > 0">
          <div class="log-toolbar">
            <span class="log-label">Git output</span>
            <button class="copy-btn" @click="copyLog">
              <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
                <path fill-rule="evenodd" clip-rule="evenodd" d="M4 2h7a1 1 0 0 1 1 1v1h1a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-1H3a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1h1zm0 2H3v8h1V5a1 1 0 0 1 1-1zm1 0v9h7V5H5z"/>
              </svg>
              {{ copied ? "Скопировано" : "Копировать" }}
            </button>
          </div>
          <pre class="log-content">{{ log.join('\n') }}</pre>
        </template>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-primary" @click="$emit('close')">OK</button>
      </div>

    </div>
  </div>
</template>

<style scoped>
.network-error-dialog {
  width: 620px;
  max-width: 92vw;
}

.dialog-header {
  background: color-mix(in srgb, var(--color-error, #f38ba8) 12%, var(--titlebar-bg, #181825));
  border-bottom-color: color-mix(in srgb, var(--color-error, #f38ba8) 30%, transparent);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 7px;
  font-weight: 600;
  font-size: var(--font-size-sm);
}

.error-icon {
  color: var(--color-error, #f38ba8);
  flex-shrink: 0;
}

.error-message {
  font-size: var(--font-size-sm);
  color: var(--color-error, #f38ba8);
  line-height: 1.5;
  padding: 2px 0 12px;
  border-bottom: 1px solid var(--border-subtle);
  word-break: break-word;
}

.log-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 12px;
  margin-bottom: 6px;
}

.log-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.copy-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  cursor: pointer;
  transition: color 0.12s, border-color 0.12s;
}
.copy-btn:hover {
  color: var(--text);
  border-color: var(--border);
}

.log-content {
  margin: 0;
  padding: 10px 12px;
  max-height: 340px;
  overflow: auto;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  line-height: 1.65;
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  white-space: pre-wrap;
  word-break: break-all;
  user-select: text;
  cursor: text;
  scrollbar-width: thin;
}
</style>
