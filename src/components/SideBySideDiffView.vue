<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useDiff } from "@/composables/useDiff";
import { useFiles } from "@/composables/useFiles";
import { useSyncScroll } from "@/composables/useSyncScroll";
import { useSideBySideDiff, type DiffHunkWithWordDiff, type DiffLineWithWordDiff } from "@/composables/useSideBySideDiff";
import { useVirtualList } from "@/composables/useVirtualList";
import { useI18n } from "@/composables/useI18n";

const LINE_HEIGHT = 20;

type FlatItem =
  | { type: "hunk-header"; hunkIdx: number; hunk: DiffHunkWithWordDiff }
  | { type: "line"; hunkIdx: number; lineIdx: number; line: DiffLineWithWordDiff };

const { currentDiff, diffContext } = useDiff();
const { selectedFile, stageHunk, unstageHunk, discardHunk, applyLines } = useFiles();
const { leftPanelRef, rightPanelRef } = useSyncScroll();
const { enrichAllHunks } = useSideBySideDiff();

const busyHunk = ref<number | null>(null);

// id строки = "<hunkIdx>:<lineIdxInHunkLines>"
const selectedLines = ref<Set<string>>(new Set());
const selAnchor = ref<string | null>(null);

const hasSelection = computed(() => selectedLines.value.size > 0);

function isSelectable(kind: string) {
  return kind === "added" || kind === "removed";
}

// Плоский список выделяемых строк в порядке отображения (для Shift-диапазона).
const selectableFlat = computed(() => {
  const arr: string[] = [];
  enrichedHunks.value.forEach((h, hi) => {
    h.lines.forEach((ln, li) => {
      if (isSelectable(ln.kind)) arr.push(`${hi}:${li}`);
    });
  });
  return arr;
});

function clearSelection() {
  selectedLines.value = new Set();
  selAnchor.value = null;
}

function onLineClick(hi: number, li: number, kind: string, ev: MouseEvent) {
  if (!isSelectable(kind)) return;
  const id = `${hi}:${li}`;
  if (ev.shiftKey && selAnchor.value) {
    const flat = selectableFlat.value;
    const a = flat.indexOf(selAnchor.value);
    const b = flat.indexOf(id);
    if (a !== -1 && b !== -1) {
      const [lo, hiIdx] = a <= b ? [a, b] : [b, a];
      selectedLines.value = new Set(flat.slice(lo, hiIdx + 1));
    }
  } else if (ev.ctrlKey || ev.metaKey) {
    const next = new Set(selectedLines.value);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedLines.value = next;
    selAnchor.value = id;
  } else {
    selectedLines.value = new Set([id]);
    selAnchor.value = id;
  }
}

// Группирует выделение по хункам, считая ordinal среди +/- строк хунка.
function buildSelectionPayload(): { raw: string; selected: number[] }[] {
  const result: { raw: string; selected: number[] }[] = [];
  enrichedHunks.value.forEach((h, hi) => {
    let ord = -1;
    const selected: number[] = [];
    h.lines.forEach((ln, li) => {
      if (isSelectable(ln.kind)) {
        ord++;
        if (selectedLines.value.has(`${hi}:${li}`)) selected.push(ord);
      }
    });
    if (selected.length > 0 && currentDiff.value) {
      result.push({ raw: currentDiff.value.hunks[hi].raw, selected });
    }
  });
  return result;
}

// Сброс выделения при смене файла/контекста.
watch([currentDiff, diffContext], clearSelection);

async function onSelectionAction(action: "stage" | "unstage" | "discard") {
  if (!hasSelection.value || busyHunk.value !== null) return;
  const payload = buildSelectionPayload();
  if (payload.length === 0) return;
  if (action === "discard") {
    const n = selectedLines.value.size;
    if (!window.confirm(`Discard changes in the selected lines (${n})? This action cannot be undone.`)) {
      return;
    }
  }
  busyHunk.value = -1; // блокирует все кнопки на время операции
  try {
    await applyLines(action, currentDiff.value?.header ?? "", payload);
    clearSelection();
  } catch (e) {
    console.error("Selection action failed:", e);
  } finally {
    busyHunk.value = null;
  }
}

function onStageBtn(hi: number) {
  if (hasSelection.value) onSelectionAction("stage");
  else onHunkAction("stage", hi);
}
function onUnstageBtn(hi: number) {
  if (hasSelection.value) onSelectionAction("unstage");
  else onHunkAction("unstage", hi);
}
function onDiscardBtn(hi: number) {
  if (hasSelection.value) onSelectionAction("discard");
  else onHunkAction("discard", hi);
}

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

const flatItems = computed<FlatItem[]>(() => {
  const result: FlatItem[] = [];
  enrichedHunks.value.forEach((hunk, hunkIdx) => {
    result.push({ type: "hunk-header", hunkIdx, hunk });
    hunk.lines.forEach((line, lineIdx) => {
      result.push({ type: "line", hunkIdx, lineIdx, line });
    });
  });
  return result;
});

const { visibleItems, paddingTop, paddingBottom } = useVirtualList(flatItems, leftPanelRef);

const { i18n } = useI18n();

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
  const idx = flatItems.value.findIndex(
    (item) => item.type === "hunk-header" && item.hunkIdx === currentHunkIndex.value,
  );
  if (idx !== -1 && leftPanelRef.value) {
    leftPanelRef.value.scrollTop = idx * LINE_HEIGHT;
  }
}
</script>

<template>
  <div class="diff-view">
    <div class="panel-title-bar">
      <span class="panel-title">Changes of {{ diffFileName }}</span>
      <div class="diff-actions">
        <span class="sel-counter" v-if="hasSelection">
          {{ selectedLines.size }} selected
        </span>
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
      <!-- LEFT: old version — placeholder for added lines -->
      <div ref="leftPanelRef" class="diff-side old">
        <div class="side-label">{{ i18n.diff.oldVersion }}</div>
        <div :style="{ height: paddingTop + 'px' }" />
        <template v-for="{ item, index } in visibleItems" :key="index">
          <div v-if="item.type === 'hunk-header'" class="hunk-header">
            {{ item.hunk.header }}
          </div>
          <template v-else-if="item.type === 'line'">
            <div v-if="item.line.kind === 'added'" class="diff-line placeholder-line" />
            <div
              v-else
              class="diff-line"
              :class="[
                item.line.kind,
                {
                  'line-selected': selectedLines.has(`${item.hunkIdx}:${item.lineIdx}`),
                  selectable: isSelectable(item.line.kind),
                },
              ]"
              @click="onLineClick(item.hunkIdx, item.lineIdx, item.line.kind, $event)"
            >
              <span class="line-no">{{ item.line.old_lineno ?? "" }}</span>
              <span class="line-prefix">{{ item.line.kind === "removed" ? "-" : " " }}</span>
              <span class="line-content">
                <template v-if="item.line.wordDiffs">
                  <template v-for="span in item.line.wordDiffs" :key="`${span.text}-${span.kind}`">
                    <span :class="['word-diff', span.kind]">{{ span.text }}</span>
                  </template>
                </template>
                <template v-else>{{ item.line.content }}</template>
              </span>
            </div>
          </template>
        </template>
        <div :style="{ height: paddingBottom + 'px' }" />
      </div>

      <div class="diff-divider" />

      <!-- RIGHT: new version — placeholder for removed lines -->
      <div ref="rightPanelRef" class="diff-side new">
        <div class="side-label">{{ i18n.diff.newVersion }}</div>
        <div :style="{ height: paddingTop + 'px' }" />
        <template v-for="{ item, index } in visibleItems" :key="index">
          <div v-if="item.type === 'hunk-header'" class="hunk-header hunk-header-actions">
            <span class="hunk-header-text">{{ item.hunk.header }}</span>
            <span v-if="diffContext === 'unstaged'" class="hunk-btns">
              <button class="hunk-btn" :disabled="busyHunk !== null" @click="onStageBtn(item.hunkIdx)" :title="i18n.diff.stageHunkTitle">{{ i18n.diff.stageHunk }}</button>
              <button class="hunk-btn danger" :disabled="busyHunk !== null" @click="onDiscardBtn(item.hunkIdx)" :title="i18n.diff.discardHunkTitle">{{ i18n.diff.discardHunk }}</button>
            </span>
            <span v-else-if="diffContext === 'staged'" class="hunk-btns">
              <button class="hunk-btn" :disabled="busyHunk !== null" @click="onUnstageBtn(item.hunkIdx)" :title="i18n.diff.unstageHunkTitle">{{ i18n.diff.unstageHunk }}</button>
            </span>
          </div>
          <template v-else-if="item.type === 'line'">
            <div v-if="item.line.kind === 'removed'" class="diff-line placeholder-line" />
            <div
              v-else
              class="diff-line"
              :class="[
                item.line.kind,
                {
                  'line-selected': selectedLines.has(`${item.hunkIdx}:${item.lineIdx}`),
                  selectable: isSelectable(item.line.kind),
                },
              ]"
              @click="onLineClick(item.hunkIdx, item.lineIdx, item.line.kind, $event)"
            >
              <span class="line-no">{{ item.line.new_lineno ?? "" }}</span>
              <span class="line-prefix">{{ item.line.kind === "added" ? "+" : " " }}</span>
              <span class="line-content">
                <template v-if="item.line.wordDiffs">
                  <template v-for="span in item.line.wordDiffs" :key="`${span.text}-${span.kind}`">
                    <span :class="['word-diff', span.kind]">{{ span.text }}</span>
                  </template>
                </template>
                <template v-else>{{ item.line.content }}</template>
              </span>
            </div>
          </template>
        </template>
        <div :style="{ height: paddingBottom + 'px' }" />
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
  font-size: var(--font-size-diff);
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

.diff-line.placeholder-line {
  min-height: 20px;
  background: rgba(69, 71, 90, 0.15);
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

.diff-line.selectable {
  cursor: pointer;
}

.diff-line.line-selected {
  background: var(--accent-soft, rgba(137, 180, 250, 0.25));
  box-shadow: inset 2px 0 0 var(--blue, #89b4fa);
}

.sel-counter {
  font-size: var(--font-size-xs);
  color: var(--blue, #89b4fa);
  user-select: none;
}
</style>
