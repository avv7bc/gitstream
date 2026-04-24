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
  <div class="statusbar">
    <div class="statusbar-left">
      <template v-if="repoInfo">
        <span class="branch-indicator">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <path d="M5 3a2 2 0 100 4 2 2 0 000-4zM5 9a2 2 0 100 4 2 2 0 000-4zM11 3a2 2 0 100 4 2 2 0 000-4z"
                  fill="none" stroke="currentColor" stroke-width="1.2"/>
            <path d="M5 7v2M11 7v-2c0 3-6 4-6 6" fill="none" stroke="currentColor" stroke-width="1.2"/>
          </svg>
          <span class="branch-name">{{ repoInfo.current_branch }}</span>
        </span>
        <span v-if="ahead > 0 || behind > 0" class="ahead-behind">
          <span v-if="ahead > 0" class="ahead" title="Ahead">{{ ahead }}</span>
          <span v-if="behind > 0" class="behind" title="Behind">{{ behind }}</span>
        </span>
      </template>
    </div>

    <div class="statusbar-center">
      <span class="status-message">{{ lastError ?? (isBusy ? 'Working...' : '') }}</span>
    </div>

    <div class="statusbar-right">
      <span class="version">0.1.21</span>
    </div>
  </div>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  height: var(--statusbar-height);
  padding: 0 8px;
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border-subtle);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  user-select: none;
}

.statusbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.branch-indicator {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--accent);
}
.branch-name {
  font-weight: 600;
}

.ahead-behind {
  display: flex;
  gap: 6px;
}
.ahead::before { content: "\2191"; margin-right: 2px; color: var(--green); }
.behind::before { content: "\2193"; margin-right: 2px; color: var(--red); }

.statusbar-center {
  flex: 1;
  text-align: center;
}

.statusbar-right {
  display: flex;
  gap: 12px;
}

.status-message {
  color: var(--text-muted);
}
</style>
