<script setup lang="ts">
import { ref, computed } from "vue";
import { useFiles } from "@/composables/useFiles";

defineEmits<{
  commit: [];
  clone: [];
  push: [];
  pull: [];
  checkout: [];
  settings: [];
  stats: [];
  addRepository: [];
  addGroup: [];
  discard: [];
}>();

const { files, selectedFile, stageFiles, unstageFiles } = useFiles();

const selectedStatus = computed(() =>
  files.value.find((f) => f.path === selectedFile.value) ?? null,
);

const canStage = computed(
  () => !!selectedStatus.value && selectedStatus.value.staged !== "staged",
);
const canUnstage = computed(
  () =>
    !!selectedStatus.value &&
    (selectedStatus.value.staged === "staged" ||
      selectedStatus.value.staged === "partial"),
);
const canDiscard = computed(() => !!selectedStatus.value);

function doStage() {
  if (canStage.value && selectedFile.value) stageFiles([selectedFile.value]);
}
function doUnstage() {
  if (canUnstage.value && selectedFile.value) unstageFiles([selectedFile.value]);
}

const showRepoMenu = ref(false);

function toggleRepoMenu() {
  showRepoMenu.value = !showRepoMenu.value;
}

function closeMenu() {
  showRepoMenu.value = false;
}

</script>

<template>
  <div class="toolbar">
    <div class="toolbar-group">
      <div class="menu-button-wrapper">
        <button class="toolbar-btn" @click="toggleRepoMenu" title="Repository menu">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M3 5h10l-1.5-1H3V5zm0 3h10V7H3v1zm0 3h10v-1H3v1z" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span>Menu</span>
        </button>

        <div v-if="showRepoMenu" class="dropdown-menu" @click.stop>
          <button class="dropdown-item" @click="$emit('addRepository'); closeMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M8 3v10M3 8h10"/>
            </svg>
            Add Repository
          </button>
          <button class="dropdown-item" @click="$emit('addGroup'); closeMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M8 3v10M3 8h10"/>
            </svg>
            Add Group
          </button>
        </div>

        <div v-if="showRepoMenu" class="menu-backdrop" @click="closeMenu" @contextmenu.prevent="closeMenu" />
      </div>

      <button class="toolbar-btn" @click="$emit('pull')" title="Pull">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 12l-4-4h2.5V3h3v5H12L8 12z"/>
        </svg>
        <span>Pull</span>
      </button>
      <button class="toolbar-btn" @click="$emit('push')" title="Push">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 3l4 4H9.5v5h-3V7H4L8 3z"/>
        </svg>
        <span>Push</span>
      </button>
      <button class="toolbar-btn" title="Fetch" disabled>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 13V5M5 9l3 4 3-4M3 3h10"/>
        </svg>
        <span>Fetch</span>
      </button>
    </div>

    <div class="toolbar-separator" />

    <div class="toolbar-group">
      <button class="toolbar-btn" @click="$emit('commit')" title="Commit">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="8" cy="8" r="3" fill="none" stroke="currentColor" stroke-width="1.5"/>
          <path d="M8 1v4M8 11v4M1 8h4M11 8h4" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <span>Commit</span>
      </button>
      <button class="toolbar-btn" @click="$emit('checkout')" title="Checkout">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M4 4v8M4 8h5l3-3M12 5v3h-3" fill="none" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <span>Checkout</span>
      </button>
    </div>

    <div class="toolbar-separator" />

    <div class="toolbar-group">
      <button class="toolbar-btn" title="Stage" :disabled="!canStage" @click="doStage">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M3 8l3 3 7-7" fill="none" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <span>Stage</span>
      </button>
      <button class="toolbar-btn" title="Unstage" :disabled="!canUnstage" @click="doUnstage">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M4 4l8 8M12 4l-8 8" fill="none" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <span>Unstage</span>
      </button>
      <button class="toolbar-btn" title="Discard" :disabled="!canDiscard" @click="$emit('discard')">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M3 5h10l-1 9H4L3 5zM6 2h4M2 5h12" fill="none" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <span>Discard</span>
      </button>
    </div>

    <div class="toolbar-separator" />

    <div class="toolbar-group">
      <button class="toolbar-btn" title="Merge" disabled>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="4" cy="4" r="2" fill="none" stroke="currentColor" stroke-width="1.5"/>
          <circle cx="12" cy="4" r="2" fill="none" stroke="currentColor" stroke-width="1.5"/>
          <circle cx="8" cy="12" r="2" fill="none" stroke="currentColor" stroke-width="1.5"/>
          <path d="M4 6v2c0 2 4 4 4 4M12 6v2c0 2-4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <span>Merge</span>
      </button>
      <button class="toolbar-btn" title="Rebase" disabled>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="4" cy="3" r="1.5" fill="currentColor"/>
          <circle cx="4" cy="8" r="1.5" fill="currentColor"/>
          <circle cx="4" cy="13" r="1.5" fill="currentColor"/>
          <path d="M6 3h4M6 8h4M6 13h4" stroke="currentColor" stroke-width="1.5"/>
          <path d="M10 3l2 2-2 2" fill="none" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <span>Rebase</span>
      </button>
      <button class="toolbar-btn" title="Stash" disabled>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <rect x="3" y="2" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
          <rect x="3" y="6.5" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
          <rect x="3" y="11" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
        </svg>
        <span>Stash</span>
      </button>
      <button class="toolbar-btn" @click="$emit('clone')" title="Clone Repository">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M10 2H4a1 1 0 00-1 1v10a1 1 0 001 1h8a1 1 0 001-1V5l-3-3z" fill="none" stroke="currentColor" stroke-width="1.2"/>
          <path d="M10 2v3h3" fill="none" stroke="currentColor" stroke-width="1.2"/>
        </svg>
        <span>Clone</span>
      </button>
    </div>

    <div class="toolbar-spacer" />

    <div class="toolbar-group">
      <button class="toolbar-btn icon-only" @click="$emit('stats')" title="Repository Statistics">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="10" width="3" height="4" rx="0.5"/>
          <rect x="6.5" y="6" width="3" height="8" rx="0.5"/>
          <rect x="11" y="2" width="3" height="12" rx="0.5"/>
        </svg>
      </button>
      <button class="toolbar-btn icon-only" @click="$emit('settings')" title="Settings">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <line x1="4" y1="6" x2="20" y2="6"/>
          <line x1="4" y1="12" x2="20" y2="12"/>
          <line x1="4" y1="18" x2="20" y2="18"/>
          <circle cx="15" cy="6" r="2" fill="var(--bg-secondary)"/>
          <circle cx="9" cy="12" r="2" fill="var(--bg-secondary)"/>
          <circle cx="16" cy="18" r="2" fill="var(--bg-secondary)"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  height: var(--toolbar-height);
  padding: 0 8px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  gap: 2px;
  user-select: none;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: var(--radius);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  transition: background 0.15s, color 0.15s;
}
.toolbar-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.toolbar-btn:active:not(:disabled) {
  background: var(--bg-active);
}
.toolbar-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.toolbar-btn.icon-only {
  padding: 6px;
}

.toolbar-separator {
  width: 1px;
  height: 20px;
  background: var(--border-subtle);
  margin: 0 4px;
}

.toolbar-spacer {
  flex: 1;
}

/* Menu button wrapper and dropdown */
.menu-button-wrapper {
  position: relative;
}

.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 160px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
  z-index: 100;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  text-align: left;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  background: none;
  border: none;
  cursor: pointer;
  transition: background 0.15s;
}

.dropdown-item:hover:not(:disabled) {
  background: var(--bg-hover);
}

.dropdown-item:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.dropdown-separator {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}
</style>
