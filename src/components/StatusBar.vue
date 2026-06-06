<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useRepo } from "@/composables/useRepo";
import { useBranches } from "@/composables/useBranches";
import { useProgress, logOpen, toggleLog, closeLog } from "@/composables/useProgress";

const { repoInfo } = useRepo();
const { isWorking, progressLabel, networkProgressLog } = useProgress();
const { branches } = useBranches();

const currentBranchInfo = computed(() => branches.value.find((b) => b.is_current));
const ahead = computed(() => currentBranchInfo.value?.ahead ?? 0);
const behind = computed(() => currentBranchInfo.value?.behind ?? 0);

function onEscKey(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    closeLog();
  }
}

const logContentRef = ref<HTMLElement | null>(null);

watch(logOpen, (open) => {
  if (open) {
    window.addEventListener("keydown", onEscKey, true);
    nextTick(scrollToBottom);
  } else {
    window.removeEventListener("keydown", onEscKey, true);
  }
});

watch(networkProgressLog, () => {
  if (logOpen.value) nextTick(scrollToBottom);
});

function scrollToBottom() {
  if (logContentRef.value) logContentRef.value.scrollTop = logContentRef.value.scrollHeight;
}
</script>

<template>
  <div class="statusbar" @contextmenu.prevent>
    <div class="statusbar-left">
      <template v-if="repoInfo">
        <span class="branch-indicator" :title="repoInfo.current_branch">
          <svg class="codicon" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.492 2.492 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Zm-6 0a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Zm8.25-.75a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5ZM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Z"/>
          </svg>
          <span class="branch-name">{{ repoInfo.current_branch }}</span>
        </span>
        <span class="sync-indicator" :title="`Behind ${behind}, Ahead ${ahead}`">
          <span v-if="behind > 0" class="behind">{{ behind }}</span>
          <svg class="codicon" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M2.006 8.267L.78 9.5 0 8.73l2.09-2.07.76.01 2.09 2.12-.76.76-1.167-1.18a5 5 0 0 0 9.4 1.983l.813.597a6 6 0 0 1-11.22-2.683zm10.99-.466L11.76 6.55l-.76.77 2.09 2.11.76.01 2.09-2.07-.75-.76-1.194 1.18a6 6 0 0 0-11.11-2.92l.81.594a5 5 0 0 1 9.3 2.346z"/>
          </svg>
          <span v-if="ahead > 0" class="ahead">{{ ahead }}</span>
        </span>
      </template>
    </div>

    <div class="statusbar-center">
      <span
        v-if="isWorking || networkProgressLog.length > 0"
        class="status-message progress"
        :class="{ clickable: networkProgressLog.length > 0 }"
        :title="networkProgressLog.length > 0 ? 'Нажмите для просмотра лога' : undefined"
        @click="toggleLog"
      >
        <svg v-if="isWorking" class="codicon spin" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M8 1.5a6.5 6.5 0 1 0 6.5 6.5h-1.5a5 5 0 1 1-5-5V1.5z"/>
        </svg>
        <span>{{ progressLabel }}</span>
        <svg v-if="networkProgressLog.length > 0" class="log-chevron" :class="{ open: logOpen }" width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M7.976 10.072l-4.357-4.357.62-.618L7.976 8.837l3.737-3.74.62.618z"/>
        </svg>
      </span>
    </div>

    <div class="statusbar-right">
      <span class="git-output-btn" :class="{ active: logOpen }" title="Git output (Alt+O)" @click="toggleLog">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M2 2.5A.5.5 0 0 1 2.5 2h11a.5.5 0 0 1 .5.5v9a.5.5 0 0 1-.5.5H9.207l-2.854 2.854A.5.5 0 0 1 5.5 14.5V12H2.5a.5.5 0 0 1-.5-.5v-9zm1 .5v8h3.5a.5.5 0 0 1 .5.5v2.293L9.293 11.5a.5.5 0 0 1 .354-.146H13V3H3zm1 2h7v1H4V5zm0 2h7v1H4V7zm0 2h4v1H4V9z"/>
        </svg>
      </span>
    </div>

    <Teleport to="body">
      <div v-if="logOpen" class="progress-log-popup">
        <div class="progress-log-header">
          <span>Git output</span>
          <button class="progress-log-close" @click="closeLog">✕</button>
        </div>
        <div ref="logContentRef" class="progress-log-content">
          <div
            v-for="(line, i) in networkProgressLog"
            :key="i"
            class="log-line"
            :class="{ 'log-error': line.isError }"
          >{{ line.text }}</div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  height: var(--statusbar-height);
  padding: 0;
  background: var(--statusbar-bg);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  color: var(--statusbar-fg);
  user-select: none;
}

.statusbar-left {
  display: flex;
  align-items: center;
  height: 100%;
}

.branch-indicator {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 100%;
  padding: 0 6px;
  color: var(--statusbar-fg);
  cursor: default;
}
.branch-indicator:hover {
  background: var(--statusbar-hover);
}
.branch-name {
  font-weight: 400;
}
.codicon {
  flex-shrink: 0;
}

.sync-indicator {
  display: flex;
  align-items: center;
  gap: 3px;
  height: 100%;
  padding: 0 6px;
  color: var(--statusbar-fg);
  cursor: default;
}
.sync-indicator:hover {
  background: var(--statusbar-hover);
}

.statusbar-center {
  flex: 1;
  min-width: 0;
  text-align: center;
  overflow: hidden;
}

.statusbar-right {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 4px;
}

.git-output-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 0 6px;
  cursor: pointer;
  color: var(--statusbar-fg-muted, #7f849c);
  opacity: 0.7;
}
.git-output-btn:hover,
.git-output-btn.active {
  color: var(--statusbar-fg, #cdd6f4);
  opacity: 1;
}

.status-message {
  display: block;
  color: var(--statusbar-fg-muted);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.status-message.progress {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 100%;
  overflow: hidden;
}

.status-message.progress span {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.status-message.clickable {
  cursor: pointer;
}
.status-message.clickable:hover {
  color: var(--statusbar-fg);
}

.codicon.spin {
  animation: gs-spin 0.8s linear infinite;
}

@keyframes gs-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.log-chevron {
  flex-shrink: 0;
  transition: transform 0.15s ease;
}
.log-chevron.open {
  transform: rotate(180deg);
}

.progress-log-popup {
  position: fixed;
  bottom: calc(var(--statusbar-height) + 4px);
  left: 50%;
  transform: translateX(-50%);
  z-index: 50;
  width: min(680px, 90vw);
  height: 320px;
  background: var(--panel-bg, #1e1e2e);
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.progress-log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted, #a6adc8);
  background: var(--titlebar-bg, #181825);
  border-bottom: 1px solid var(--border, #45475a);
  user-select: none;
  flex-shrink: 0;
}

.progress-log-close {
  background: none;
  border: none;
  color: var(--text-muted, #a6adc8);
  cursor: pointer;
  padding: 0 2px;
  font-size: 12px;
  line-height: 1;
}
.progress-log-close:hover {
  color: var(--text, #cdd6f4);
}

.progress-log-content {
  flex: 1;
  min-height: 0;
  margin: 0;
  padding: 8px 10px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  line-height: 1.6;
  color: var(--text, #cdd6f4);
  overflow: auto;
  user-select: text;
  cursor: text;
}

.log-line {
  white-space: pre-wrap;
  word-break: break-all;
}
.log-line.log-error {
  color: var(--red);
}
</style>
