<script setup lang="ts">
import { computed } from "vue";
import { useRepo } from "@/composables/useRepo";
import { useRemote } from "@/composables/useRemote";
import { useBranches } from "@/composables/useBranches";

const { repoInfo } = useRepo();
const { isBusy, lastError } = useRemote();
const { branches } = useBranches();

const currentBranchInfo = computed(() => branches.value.find((b) => b.is_current));
const ahead = computed(() => currentBranchInfo.value?.ahead ?? 0);
const behind = computed(() => currentBranchInfo.value?.behind ?? 0);
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
      <span class="status-message">{{ lastError ?? (isBusy ? 'Working...' : '') }}</span>
    </div>

    <div class="statusbar-right">
      <span class="version">0.1.25</span>
    </div>
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
  text-align: center;
}

.statusbar-right {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 8px;
}

.status-message {
  color: var(--statusbar-fg-muted);
}
.version {
  color: var(--statusbar-fg-muted);
}
</style>
