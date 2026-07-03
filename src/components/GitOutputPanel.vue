<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import { useProgress, logOpen, clearLog } from "@/composables/useProgress";

// Тело вкладки «Git output» в нижней панели: строка управления + строки лога
// git-команд (текст ошибок — красным). Автоскролл к последней строке, когда
// вкладка активна.
const { networkProgressLog } = useProgress();

const contentRef = ref<HTMLElement | null>(null);

function scrollToBottom() {
  if (contentRef.value) contentRef.value.scrollTop = contentRef.value.scrollHeight;
}

function copyLog() {
  const text = networkProgressLog.value.map((l) => l.text).join("\n");
  if (text) navigator.clipboard?.writeText(text);
}

watch(networkProgressLog, () => {
  if (logOpen.value) nextTick(scrollToBottom);
});

watch(logOpen, (open) => {
  if (open) nextTick(scrollToBottom);
});

onMounted(() => {
  if (logOpen.value) nextTick(scrollToBottom);
});
</script>

<template>
  <div class="git-output">
    <div class="git-output-toolbar">
      <button
        class="go-btn"
        :disabled="networkProgressLog.length === 0"
        title="Прокрутить вниз"
        @click="scrollToBottom"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 11.5L3.5 7l.7-.7L8 10.1l3.8-3.8.7.7z"/>
          <path d="M3.5 12.5h9v1h-9z"/>
        </svg>
      </button>
      <button
        class="go-btn"
        :disabled="networkProgressLog.length === 0"
        title="Копировать вывод"
        @click="copyLog"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M4 2h7v2h1V1.5A.5.5 0 0 0 11.5 1h-8A.5.5 0 0 0 3 1.5V11h1V2z"/>
          <path fill-rule="evenodd" clip-rule="evenodd" d="M5 4.5A.5.5 0 0 1 5.5 4h8a.5.5 0 0 1 .5.5v10a.5.5 0 0 1-.5.5h-8a.5.5 0 0 1-.5-.5v-10zM6 5v9h7V5H6z"/>
        </svg>
      </button>
      <button
        class="go-btn"
        :disabled="networkProgressLog.length === 0"
        title="Очистить"
        @click="clearLog"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M10 3h3v1h-1v9.5a.5.5 0 0 1-.5.5h-7a.5.5 0 0 1-.5-.5V4H3V3h3V1.5a.5.5 0 0 1 .5-.5h3a.5.5 0 0 1 .5.5V3zM7 2v1h2V2H7zM5 4v9h6V4H5zm2 2h1v5H7V6zm2 0h1v5H9V6z"/>
        </svg>
      </button>
    </div>
    <div ref="contentRef" class="git-output-content">
      <div
        v-for="(line, i) in networkProgressLog"
        :key="i"
        class="log-line"
        :class="{ 'log-error': line.isError }"
      >{{ line.text }}</div>
      <div v-if="networkProgressLog.length === 0" class="git-output-empty">
        Нет вывода git
      </div>
    </div>
  </div>
</template>

<style scoped>
.git-output {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.git-output-toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 2px;
  height: 26px;
  padding: 0 6px;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.go-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3px;
  border: none;
  border-radius: var(--radius);
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}
.go-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.go-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.git-output-content {
  flex: 1;
  min-height: 0;
  margin: 0;
  padding: 8px 10px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-primary, #cdd6f4);
  overflow: auto;
  user-select: text;
  cursor: text;
}

.log-line {
  white-space: pre-wrap;
  /* Переносим по словам; очень длинные токены (SHA, URL) рвём лишь при нужде,
     а не по каждой букве (как делал break-all). */
  overflow-wrap: anywhere;
  word-break: normal;
}
.log-line.log-error {
  color: var(--red);
}

.git-output-empty {
  color: var(--text-muted);
  font-style: italic;
}
</style>
