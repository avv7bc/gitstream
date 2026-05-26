<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useBranches } from "@/composables/useBranches";
import { useRepo } from "@/composables/useRepo";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";
import type { RemoteInfo } from "@/types";

const emit = defineEmits<{
  close: [];
  pull: [remote: string, rebase: boolean];
  fetch: [remote: string];
}>();

const { remotes, branches } = useBranches();
const { repoPath, repoInfo } = useRepo();

const remoteInfos = ref<RemoteInfo[]>([]);
const selectedRemote = ref("origin");
const pullMode = ref<"merge" | "rebase">("merge");
const showMoreOptions = ref(false);

const currentBranch = computed(() => repoInfo.value?.current_branch ?? "");
const currentBranchInfo = computed(() => branches.value.find((b) => b.is_current));
const behindCount = computed(() => currentBranchInfo.value?.behind ?? 0);

const selectedRemoteUrl = computed(
  () => remoteInfos.value.find((r) => r.name === selectedRemote.value)?.url ?? "",
);

const dialogRef = ref<HTMLElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

async function loadRemoteUrls() {
  if (!repoPath.value) return;
  try {
    remoteInfos.value = await invoke<RemoteInfo[]>("get_remote_urls", {
      repoPath: repoPath.value,
    });
  } catch {
    remoteInfos.value = remotes.value.map((name) => ({ name, url: "" }));
  }
}

function handlePull() {
  emit("pull", selectedRemote.value, pullMode.value === "rebase");
}

function handleFetch() {
  emit("fetch", selectedRemote.value);
}

onMounted(() => {
  dialogRef.value?.focus();
  loadRemoteUrls();
});

watch(repoPath, () => loadRemoteUrls());
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog pull-dialog" :style="dragStyle" @keydown.enter.prevent="handlePull" @keydown.escape="$emit('close')" tabindex="-1" ref="dialogRef">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.pull.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="hero">
          <div class="hero-text">
            <h4 class="hero-heading">{{ i18n.dialog.pull.heading }}</h4>
            <p class="hero-description">{{ i18n.dialog.pull.description }}</p>
          </div>
          <div class="hero-icon" aria-hidden="true">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 3v10" />
              <path d="M7 9l5 5 5-5" />
              <path d="M4 15v4a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-4" />
            </svg>
          </div>
        </div>

        <div class="fetch-from-row">
          <label class="fetch-from-label" for="pull-remote-select">{{ i18n.dialog.pull.fetchFrom }}</label>
          <select id="pull-remote-select" v-model="selectedRemote" class="form-select fetch-from-select">
            <option v-for="r in (remoteInfos.length ? remoteInfos.map(x => x.name) : remotes)" :key="r" :value="r">
              {{ r }}{{ remoteInfos.find(x => x.name === r)?.url ? ` (${remoteInfos.find(x => x.name === r)!.url})` : "" }}
            </option>
          </select>
        </div>

        <label class="more-options-toggle">
          <input type="checkbox" v-model="showMoreOptions" />
          <span>{{ i18n.dialog.pull.moreOptions }}</span>
        </label>

        <div v-if="showMoreOptions" class="more-options-panel">
          <div class="form-group">
            <label class="form-label">{{ i18n.dialog.pull.mode }}</label>
            <div class="radio-group">
              <label class="radio-label">
                <input type="radio" v-model="pullMode" value="merge" />
                <span>{{ i18n.dialog.pull.merge }}</span>
              </label>
              <label class="radio-label">
                <input type="radio" v-model="pullMode" value="rebase" />
                <span>{{ i18n.dialog.pull.rebase }}</span>
              </label>
            </div>
          </div>
        </div>

        <p v-if="selectedRemoteUrl || behindCount > 0" class="behind-info">
          {{ behindCount }} commits behind
          <strong>{{ selectedRemote }}/{{ currentBranch }}</strong>
        </p>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.pull.cancel }}</button>
        <button class="btn btn-primary" @click="handlePull">{{ i18n.dialog.pull.pull }}</button>
        <button class="btn btn-secondary" @click="handleFetch">{{ i18n.dialog.pull.fetch }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pull-dialog {
  width: 520px;
}

.hero {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding-bottom: 14px;
  margin-bottom: 14px;
  border-bottom: 1px solid var(--border-subtle);
}
.hero-text {
  flex: 1;
  min-width: 0;
}
.hero-heading {
  font-size: 17px;
  font-weight: 600;
  margin: 0 0 6px;
  color: var(--text-primary);
  line-height: 1.25;
}
.hero-description {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin: 0;
  line-height: 1.4;
}
.hero-icon {
  flex-shrink: 0;
  color: var(--green);
  display: flex;
  align-items: flex-end;
  padding-top: 4px;
}

.fetch-from-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 14px;
}
.fetch-from-label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  flex-shrink: 0;
}
.fetch-from-select {
  flex: 1;
  min-width: 0;
}

.more-options-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
}
.more-options-toggle input {
  accent-color: var(--accent);
}

.more-options-panel {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-subtle);
}

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.radio-label input {
  accent-color: var(--accent);
}

.behind-info {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin: 12px 0 0;
}
.behind-info strong {
  color: var(--text-secondary);
}
</style>
