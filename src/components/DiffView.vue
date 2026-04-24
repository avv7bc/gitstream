<script setup lang="ts">
import { ref, computed } from "vue";
import { useDiff } from "@/composables/useDiff";
import { useFiles } from "@/composables/useFiles";
import type { DiffMode } from "@/types";

const { currentDiff } = useDiff();
const { selectedFile } = useFiles();

const mode = ref<DiffMode>("unified");
const compareMode = "Working Tree vs Index";

const hunks = computed(() => currentDiff.value?.hunks ?? []);
const diffFileName = computed(() => currentDiff.value?.path ?? selectedFile.value ?? "");
</script>

<template>
  <div class="diff-view" @contextmenu.prevent>
    <div class="panel-title-bar">
      <span class="panel-title">Changes of {{ diffFileName }}</span>
    </div>
    <div class="diff-header">
      <div class="diff-file-info">
        <span class="diff-compare-mode">{{ compareMode }}</span>
      </div>
      <div class="diff-actions">
        <button
          class="diff-mode-btn"
          :class="{ active: mode === 'unified' }"
          @click="mode = 'unified'"
        >
          Unified
        </button>
        <button
          class="diff-mode-btn"
          :class="{ active: mode === 'side-by-side' }"
          @click="mode = 'side-by-side'"
        >
          Side-by-side
        </button>
        <div class="diff-separator" />
        <button class="diff-action-btn" title="Stage Hunk">Stage Hunk</button>
        <button class="diff-action-btn" title="Discard Hunk">Discard Hunk</button>
        <div class="diff-separator" />
        <button class="diff-nav-btn" title="Previous Change">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M8 3l-5 5h10z" fill="currentColor"/></svg>
        </button>
        <button class="diff-nav-btn" title="Next Change">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M8 13l5-5H3z" fill="currentColor"/></svg>
        </button>
      </div>
    </div>

    <!-- Unified mode -->
    <div v-if="mode === 'unified'" class="diff-body unified">
      <div v-for="(hunk, hi) in hunks" :key="hi" class="diff-hunk">
        <div class="hunk-header">{{ hunk.header }}</div>
        <div
          v-for="(line, li) in hunk.lines"
          :key="li"
          class="diff-line"
          :class="line.kind"
        >
          <span class="line-no old">{{ line.old_lineno ?? "" }}</span>
          <span class="line-no new">{{ line.new_lineno ?? "" }}</span>
          <span class="line-prefix">{{ line.kind === "added" ? "+" : line.kind === "removed" ? "-" : " " }}</span>
          <span class="line-content">{{ line.content }}</span>
        </div>
      </div>
    </div>

    <!-- Side-by-side mode -->
    <div v-else class="diff-body side-by-side">
      <div v-for="(hunk, hi) in hunks" :key="hi" class="diff-hunk">
        <div class="hunk-header-sbs">{{ hunk.header }}</div>
        <div class="sbs-container">
          <div class="sbs-left">
            <template v-for="(line, li) in hunk.lines" :key="'l' + li">
              <div
                v-if="line.kind !== 'added'"
                class="diff-line"
                :class="line.kind"
              >
                <span class="line-no">{{ line.old_lineno ?? "" }}</span>
                <span class="line-content">{{ line.content }}</span>
              </div>
              <div v-else class="diff-line placeholder">
                <span class="line-no"></span>
                <span class="line-content"></span>
              </div>
            </template>
          </div>
          <div class="sbs-right">
            <template v-for="(line, li) in hunk.lines" :key="'r' + li">
              <div
                v-if="line.kind !== 'removed'"
                class="diff-line"
                :class="line.kind"
              >
                <span class="line-no">{{ line.new_lineno ?? "" }}</span>
                <span class="line-content">{{ line.content }}</span>
              </div>
              <div v-else class="diff-line placeholder">
                <span class="line-no"></span>
                <span class="line-content"></span>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.diff-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
}

.panel-title-bar {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
  flex-shrink: 0;
}
.panel-title {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.diff-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
}

.diff-file-info {
  display: flex;
  align-items: center;
  gap: 8px;
}
.diff-filename {
  font-weight: 600;
  font-size: var(--font-size-sm);
}
.diff-compare-mode {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
}

.diff-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.diff-mode-btn {
  padding: 2px 8px;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  border-radius: var(--radius);
}
.diff-mode-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}
.diff-mode-btn.active {
  background: var(--bg-surface);
  color: var(--text-primary);
}

.diff-action-btn {
  padding: 2px 8px;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  border-radius: var(--radius);
}
.diff-action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.diff-separator {
  width: 1px;
  height: 14px;
  background: var(--border-subtle);
  margin: 0 2px;
}

.diff-nav-btn {
  padding: 2px;
  border-radius: var(--radius);
  color: var(--text-muted);
}
.diff-nav-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.diff-body {
  flex: 1;
  overflow: auto;
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  line-height: 1.5;
}

.diff-hunk {
  margin-bottom: 4px;
}

.hunk-header, .hunk-header-sbs {
  padding: 4px 8px;
  background: rgba(137, 180, 250, 0.08);
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  border-top: 1px solid var(--border-subtle);
  border-bottom: 1px solid var(--border-subtle);
}

.diff-line {
  display: flex;
  white-space: pre;
  min-height: 20px;
}
.diff-line.added {
  background: var(--diff-added-bg);
}
.diff-line.removed {
  background: var(--diff-removed-bg);
}
.diff-line.placeholder {
  background: rgba(69, 71, 90, 0.3);
}

.line-no {
  display: inline-block;
  width: 42px;
  padding: 0 8px;
  text-align: right;
  color: var(--text-muted);
  user-select: none;
  flex-shrink: 0;
}
.line-no.old, .line-no.new {
  width: 42px;
}

.line-prefix {
  display: inline-block;
  width: 16px;
  text-align: center;
  user-select: none;
  flex-shrink: 0;
  color: var(--text-muted);
}
.added .line-prefix { color: var(--green); }
.removed .line-prefix { color: var(--red); }

.line-content {
  flex: 1;
  padding-right: 8px;
}

/* Side-by-side */
.sbs-container {
  display: flex;
}
.sbs-left, .sbs-right {
  flex: 1;
  overflow: hidden;
}
.sbs-left {
  border-right: 1px solid var(--border-subtle);
}
</style>
