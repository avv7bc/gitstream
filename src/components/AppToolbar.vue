<script setup lang="ts">
import { ref, computed } from "vue";
import { useFiles } from "@/composables/useFiles";
import { useI18n } from "@/composables/useI18n";
const { i18n } = useI18n();

defineEmits<{
  commit: [];
  push: [];
  pull: [];
  checkout: [];
  settings: [];
  stats: [];
  addRepository: [];
  addGroup: [];
  discard: [];
  merge: [];
  rebase: [];
  stash: [];
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
const showLocalMenu = ref(false);

function toggleRepoMenu() {
  showRepoMenu.value = !showRepoMenu.value;
}

function closeMenu() {
  showRepoMenu.value = false;
}

function toggleLocalMenu() {
  showLocalMenu.value = !showLocalMenu.value;
}

function closeLocalMenu() {
  showLocalMenu.value = false;
}

const showBranchMenu = ref(false);

function toggleBranchMenu() {
  showBranchMenu.value = !showBranchMenu.value;
}

function closeBranchMenu() {
  showBranchMenu.value = false;
}

</script>

<template>
  <div class="toolbar">
    <div class="toolbar-group">
      <div class="menu-button-wrapper">
        <button class="toolbar-btn" @click="toggleRepoMenu" :title="i18n.toolbar.menu">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M3 5h10l-1.5-1H3V5zm0 3h10V7H3v1zm0 3h10v-1H3v1z" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span>{{ i18n.toolbar.menu }}</span>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor" style="opacity:0.5">
            <path d="M2 3.5l3 3 3-3"/>
          </svg>
        </button>

        <div v-if="showRepoMenu" class="dropdown-menu" @click.stop>
          <button class="dropdown-item" @click="$emit('addRepository'); closeMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M8 3v10M3 8h10"/>
            </svg>
            {{ i18n.toolbar.addRepository }}
          </button>
          <button class="dropdown-item" @click="$emit('addGroup'); closeMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M8 3v10M3 8h10"/>
            </svg>
            {{ i18n.toolbar.addGroup }}
          </button>
        </div>

        <div v-if="showRepoMenu" class="menu-backdrop" @click="closeMenu" @contextmenu.prevent="closeMenu" />
      </div>
    </div>

    <div class="toolbar-group">
      <div class="menu-button-wrapper">
        <button class="toolbar-btn" @click="toggleLocalMenu" :title="i18n.toolbar.local">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="4" cy="4" r="1.5" fill="currentColor" stroke="none"/>
            <circle cx="4" cy="12" r="1.5" fill="currentColor" stroke="none"/>
            <circle cx="12" cy="8" r="1.5" fill="currentColor" stroke="none"/>
            <path d="M4 5.5v5M5.5 4h4.5a2 2 0 0 1 2 2v1M5.5 12h4.5a2 2 0 0 0 2-2v-1"/>
          </svg>
          <span>{{ i18n.toolbar.local }}</span>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor" style="opacity:0.5">
            <path d="M2 3.5l3 3 3-3"/>
          </svg>
        </button>

        <div v-if="showLocalMenu" class="dropdown-menu" @click.stop>
          <button class="dropdown-item" @click="$emit('commit'); closeLocalMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="8" cy="8" r="3"/>
              <path d="M8 1v4M8 11v4M1 8h4M11 8h4"/>
            </svg>
            {{ i18n.toolbar.commit }}
            <span class="dropdown-shortcut">Ctrl+K</span>
          </button>
          <button class="dropdown-item" @click="$emit('checkout'); closeLocalMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 4v8M4 8h5l3-3M12 5v3h-3"/>
            </svg>
            {{ i18n.toolbar.checkout }}
          </button>
          <div class="dropdown-separator" />
          <button class="dropdown-item" :disabled="!canStage" @click="doStage(); closeLocalMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M3 8l3 3 7-7"/>
            </svg>
            {{ i18n.toolbar.stage }}
            <span class="dropdown-shortcut">Ctrl+T</span>
          </button>
          <button class="dropdown-item" :disabled="!canUnstage" @click="doUnstage(); closeLocalMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 4l8 8M12 4l-8 8"/>
            </svg>
            {{ i18n.toolbar.unstage }}
            <span class="dropdown-shortcut">⇧Ctrl+T</span>
          </button>
          <button class="dropdown-item" :disabled="!canDiscard" @click="$emit('discard'); closeLocalMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M3 5h10l-1 9H4L3 5zM6 2h4M2 5h12"/>
            </svg>
            {{ i18n.toolbar.discard }}
          </button>
        </div>

        <div v-if="showLocalMenu" class="menu-backdrop" @click="closeLocalMenu" @contextmenu.prevent="closeLocalMenu" />
      </div>
    </div>

    <div class="toolbar-group">
      <div class="menu-button-wrapper">
        <button class="toolbar-btn" @click="toggleBranchMenu" :title="i18n.toolbar.branch">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="4" cy="3" r="1.5" fill="currentColor" stroke="none"/>
            <circle cx="4" cy="13" r="1.5" fill="currentColor" stroke="none"/>
            <circle cx="12" cy="3" r="1.5" fill="currentColor" stroke="none"/>
            <path d="M4 4.5v7M4 4.5C4 7 12 7 12 4.5"/>
          </svg>
          <span>{{ i18n.toolbar.branch }}</span>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor" style="opacity:0.5">
            <path d="M2 3.5l3 3 3-3"/>
          </svg>
        </button>

        <div v-if="showBranchMenu" class="dropdown-menu" @click.stop>
          <button class="dropdown-item" disabled @click="$emit('merge'); closeBranchMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="4" cy="4" r="1.5"/>
              <circle cx="12" cy="4" r="1.5"/>
              <circle cx="8" cy="12" r="1.5"/>
              <path d="M4 5.5v2c0 2 4 3 4 3M12 5.5v2c0 2-4 3-4 3"/>
            </svg>
            {{ i18n.toolbar.merge }}
            <span class="dropdown-shortcut">Ctrl+M</span>
          </button>
          <button class="dropdown-item" disabled @click="$emit('rebase'); closeBranchMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="4" cy="3" r="1.5" fill="currentColor" stroke="none"/>
              <circle cx="4" cy="8" r="1.5" fill="currentColor" stroke="none"/>
              <circle cx="4" cy="13" r="1.5" fill="currentColor" stroke="none"/>
              <path d="M6 3h4M6 8h4M6 13h4" stroke="currentColor"/>
              <path d="M10 3l2 2-2 2"/>
            </svg>
            {{ i18n.toolbar.rebase }}
            <span class="dropdown-shortcut">Ctrl+D</span>
          </button>
          <div class="dropdown-separator" />
          <button class="dropdown-item" @click="$emit('stash'); closeBranchMenu()">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2">
              <rect x="3" y="2" width="10" height="3" rx="1"/>
              <rect x="3" y="6.5" width="10" height="3" rx="1"/>
              <rect x="3" y="11" width="10" height="3" rx="1"/>
            </svg>
            {{ i18n.toolbar.stash }}
          </button>
        </div>

        <div v-if="showBranchMenu" class="menu-backdrop" @click="closeBranchMenu" @contextmenu.prevent="closeBranchMenu" />
      </div>
    </div>

    <div class="toolbar-spacer" />

    <div class="toolbar-group-center">
      <button class="toolbar-btn-action" @click="$emit('pull')" data-shortcut="Alt+PageDown">
        <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 12l-4-4h2.5V3h3v5H12L8 12z"/>
        </svg>
        <span>{{ i18n.toolbar.pull }}</span>
      </button>
      <button class="toolbar-btn-action" @click="$emit('push')" data-shortcut="Alt+PageUp">
        <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 3l4 4H9.5v5h-3V7H4L8 3z"/>
        </svg>
        <span>{{ i18n.toolbar.push }}</span>
      </button>
    </div>

    <div class="toolbar-spacer" />

    <div class="toolbar-group">
      <button class="toolbar-btn icon-only" @click="$emit('stats')" title="Repository Statistics">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <rect x="2" y="10" width="3" height="4" rx="0.5"/>
          <rect x="6.5" y="6" width="3" height="8" rx="0.5"/>
          <rect x="11" y="2" width="3" height="12" rx="0.5"/>
        </svg>
      </button>
      <button class="toolbar-btn icon-only" @click="$emit('settings')" title="Settings">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <line x1="2" y1="4" x2="7.5" y2="4"/>
          <line x1="12.5" y1="4" x2="14" y2="4"/>
          <circle cx="10" cy="4" r="2.5" fill="currentColor" stroke="none"/>
          <line x1="2" y1="8" x2="3.5" y2="8"/>
          <line x1="8.5" y1="8" x2="14" y2="8"/>
          <circle cx="6" cy="8" r="2.5" fill="currentColor" stroke="none"/>
          <line x1="2" y1="12" x2="7.5" y2="12"/>
          <line x1="12.5" y1="12" x2="14" y2="12"/>
          <circle cx="10" cy="12" r="2.5" fill="currentColor" stroke="none"/>
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
  position: relative;
}

.toolbar-group-center {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-btn-action[data-shortcut] {
  position: relative;
}
.toolbar-btn-action[data-shortcut]::after {
  content: attr(data-shortcut);
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-tooltip, #1e1e2e);
  color: var(--text-muted, var(--text-secondary));
  font-size: 10px;
  white-space: nowrap;
  padding: 2px 6px;
  border-radius: var(--radius);
  border: 1px solid var(--border-subtle);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s;
  z-index: 200;
}
.toolbar-btn-action[data-shortcut]:hover::after {
  opacity: 1;
}

.toolbar-btn-action {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg-primary, var(--bg-secondary));
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}
.toolbar-btn-action:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--border-hover, var(--border));
}
.toolbar-btn-action:active:not(:disabled) {
  background: var(--bg-active);
}
.toolbar-btn-action:disabled {
  opacity: 0.35;
  cursor: not-allowed;
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
  color: var(--text-primary);
  opacity: 0.7;
}
.toolbar-btn.icon-only:hover:not(:disabled) {
  opacity: 1;
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

.dropdown-shortcut {
  margin-left: auto;
  padding-left: 16px;
  font-size: 11px;
  color: var(--text-muted, var(--text-secondary));
  opacity: 0.6;
  white-space: nowrap;
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
