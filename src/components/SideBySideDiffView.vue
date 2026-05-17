<script setup lang="ts">
import { ref, computed } from "vue";
import { useDiff } from "@/composables/useDiff";
import { useFiles } from "@/composables/useFiles";
import { useSyncScroll } from "@/composables/useSyncScroll";
import { useSideBySideDiff } from "@/composables/useSideBySideDiff";

const { currentDiff, diffContext } = useDiff();
const { selectedFile, stageHunk, unstageHunk, discardHunk } = useFiles();
const { leftPanelRef, rightPanelRef } = useSyncScroll();
const { enrichAllHunks } = useSideBySideDiff();

const busyHunk = ref<number | null>(null);

function hunkPatch(rawHunk: string): string {
  return (currentDiff.value?.header ?? "") + rawHunk;
}

async function onHunkAction(
  action: "stage" | "unstage" | "discard",
  hunkIdx: number,
) {
  const hunk = currentDiff.value?.hunks[hunkIdx];
  if (!hunk || busyHunk.value !== null) return;
  busyHunk.value = hunkIdx;
  try {
    const patch = hunkPatch(hunk.raw);
    if (action === "stage") await stageHunk(patch);
    else if (action === "unstage") await unstageHunk(patch);
    else await discardHunk(patch);
  } catch (e) {
    console.error("Hunk action failed:", e);
  } finally {
    busyHunk.value = null;
  }
}

const enrichedHunks = computed(() => {
  if (!currentDiff.value?.hunks) return [];
  return enrichAllHunks(currentDiff.value.hunks);
});

const currentHunkIndex = ref(0);

const diffFileName = computed(() => currentDiff.value?.path ?? selectedFile.value ?? "");

const hasHunks = computed(() => enrichedHunks.value.length > 0);

function goToPreviousHunk() {
  if (currentHunkIndex.value > 0) {
    currentHunkIndex.value--;
    scrollToHunk();
  }
}

function goToNextHunk() {
  if (currentHunkIndex.value < enrichedHunks.value.length - 1) {
    currentHunkIndex.value++;
    scrollToHunk();
  }
}

function scrollToHunk() {
  setTimeout(() => {
    const hunkElement = document.querySelector(`[data-hunk-idx="${currentHunkIndex.value}"]`);
    if (hunkElement && leftPanelRef.value) {
      hunkElement.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  }, 0);
}
</script>

<template>
  <div class="diff-view">
    <div class="panel-title-bar">
      <span class="panel-title">Changes of {{ diffFileName }}</span>
      <div class="diff-actions">
        <button
          class="diff-nav-btn"
          :disabled="!hasHunks || currentHunkIndex === 0"
          @click="goToPreviousHunk"
          title="Previous Hunk"
        >
          <svg width="14" height="14" viewBox="0 0 16 16">
            <path d="M8 3l-5 5h10z" fill="currentColor" />
          </svg>
        </button>
        <span class="hunk-counter" v-if="hasHunks">
          {{ currentHunkIndex + 1 }} / {{ enrichedHunks.length }}
        </span>
        <button
          class="diff-nav-btn"
          :disabled="!hasHunks || currentHunkIndex === enrichedHunks.length - 1"
          @click="goToNextHunk"
          title="Next Hunk"
        >
          <svg width="14" height="14" viewBox="0 0 16 16">
            <path d="M8 13l5-5H3z" fill="currentColor" />
          </svg>
        </button>
      </div>
    </div>

    <div class="diff-container">
      <div ref="leftPanelRef" class="diff-side old">
        <div class="side-label">Old Version</div>
        <div v-for="(hunk, hi) in enrichedHunks" :key="`${hunk.header}`" :data-hunk-idx="hi" class="hunk-section">
          <div class="hunk-header">{{ hunk.header }}</div>
          <div
            v-for="line in hunk.lines"
            :key="`${line.content}-${line.kind}-old`"
            class="diff-line"
            :class="[line.kind, { hidden: line.kind === 'added' }]"
          >
            <span class="line-no">{{ line.old_lineno ?? "" }}</span>
            <span class="line-prefix">{{ line.kind === "removed" ? "-" : " " }}</span>
            <span class="line-content">
              <template v-if="line.wordDiffs">
                <template v-for="span in line.wordDiffs" :key="`${span.text}-${span.kind}`">
                  <span :class="['word-diff', span.kind]">{{ span.text }}</span>
                </template>
              </template>
              <template v-else>
                {{ line.content }}
              </template>
            </span>
          </div>
        </div>
      </div>

      <div class="diff-divider" />

      <div ref="rightPanelRef" class="diff-side new">
        <div class="side-label">New Version</div>
        <div v-for="(hunk, hi) in enrichedHunks" :key="`${hunk.header}`" :data-hunk-idx="hi" class="hunk-section">
          <div class="hunk-header hunk-header-actions">
            <span class="hunk-header-text">{{ hunk.header }}</span>
            <span v-if="diffContext === 'unstaged'" class="hunk-btns">
              <button
                class="hunk-btn"
                :disabled="busyHunk !== null"
                @click="onHunkAction('stage', hi)"
                title="Добавить хунк в индекс"
              >
                Stage
              </button>
              <button
                class="hunk-btn danger"
                :disabled="busyHunk !== null"
                @click="onHunkAction('discard', hi)"
                title="Отменить изменения хунка"
              >
                Discard
              </button>
            </span>
            <span v-else-if="diffContext === 'staged'" class="hunk-btns">
              <button
                class="hunk-btn"
                :disabled="busyHunk !== null"
                @click="onHunkAction('unstage', hi)"
                title="Убрать хунк из индекса"
              >
                Unstage
              </button>
            </span>
          </div>
          <div
            v-for="line in hunk.lines"
            :key="`${line.content}-${line.kind}-new`"
            class="diff-line"
            :class="[line.kind, { hidden: line.kind === 'removed' }]"
          >
            <span class="line-no">{{ line.new_lineno ?? "" }}</span>
            <span class="line-prefix">{{ line.kind === "added" ? "+" : " " }}</span>
            <span class="line-content">
              <template v-if="line.wordDiffs">
                <template v-for="span in line.wordDiffs" :key="`${span.text}-${span.kind}`">
                  <span :class="['word-diff', span.kind]">{{ span.text }}</span>
                </template>
              </template>
              <template v-else>
                {{ line.content }}
              </template>
            </span>
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
  gap: 8px;
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

.diff-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.diff-nav-btn {
  padding: 2px;
  border-radius: var(--radius);
  color: var(--text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
}

.diff-nav-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.diff-nav-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hunk-counter {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  min-width: 40px;
  text-align: center;
}

.diff-container {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.diff-side {
  flex: 1;
  overflow-y: auto;
  overflow-x: auto;
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
}

.side-label {
  position: sticky;
  top: 0;
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  font-weight: 600;
  z-index: 10;
  user-select: none;
}

.diff-divider {
  width: 1px;
  background: var(--border-subtle);
  flex-shrink: 0;
}

.hunk-section {
  margin-bottom: 4px;
}

.hunk-header {
  padding: 4px 8px;
  background: rgba(137, 180, 250, 0.08);
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  border-top: 1px solid var(--border-subtle);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
}

.hunk-header-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.hunk-header-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hunk-btns {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.hunk-btn {
  padding: 1px 8px;
  border-radius: var(--radius);
  border: 1px solid var(--border-subtle);
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: var(--font-size-xs);
  cursor: pointer;
}

.hunk-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}

.hunk-btn.danger:hover:not(:disabled) {
  background: var(--diff-removed-bg);
  color: var(--red);
}

.hunk-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.diff-line {
  display: flex;
  white-space: pre;
  min-height: 20px;
  line-height: 1.5;
}

.diff-line.added {
  background: var(--diff-added-bg);
}

.diff-line.removed {
  background: var(--diff-removed-bg);
}

.diff-line.hidden {
  display: none;
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

.line-prefix {
  display: inline-block;
  width: 16px;
  text-align: center;
  user-select: none;
  flex-shrink: 0;
  color: var(--text-muted);
}

.removed .line-prefix {
  color: var(--red);
}

.added .line-prefix {
  color: var(--green);
}

.line-content {
  flex: 1;
  padding-right: 8px;
  white-space: pre;
}

.word-diff {
  transition: background-color 0.2s ease;
}

.word-diff.added {
  background: var(--diff-word-added-bg);
}

.word-diff.removed {
  background: var(--diff-word-removed-bg);
}
</style>
