# Side-by-Side Diff View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace unified diff view with a professional two-panel side-by-side diff featuring synchronized scroll, word-level highlighting, and hunk navigation.

**Architecture:** Replace `DiffView.vue` with `SideBySideDiffView.vue` that uses two new panel components (`DiffPanel.vue`, `DiffLinesPair.vue`). Use `diff-match-patch` library for word-level diffing. Two new composables (`useSyncScroll.ts`, `useSideBySideDiff.ts`) handle scroll synchronization and word-level diff computation.

**Tech Stack:** Vue 3 (Composition API), TypeScript, `diff-match-patch` library, existing Tauri IPC.

---

## File Structure

**New Files:**
- `src/components/SideBySideDiffView.vue` - Main diff view component (replaces DiffView.vue)
- `src/components/DiffPanel.vue` - Single panel (left or right)
- `src/components/DiffLinesPair.vue` - Line pair renderer
- `src/composables/useSyncScroll.ts` - Synchronized scroll logic
- `src/composables/useSideBySideDiff.ts` - Word-level diff parsing

**Modified Files:**
- `package.json` - Add `diff-match-patch` dependency
- `src/styles/main.css` - Add `--diff-word-added-bg`, `--diff-word-removed-bg` CSS vars
- `src/components/CommitDetails.vue` - Remove DiffView import, use SideBySideDiffView
- `src/App.vue` - If DiffView is imported directly, update to SideBySideDiffView

**Deleted Files:**
- `src/components/DiffView.vue` - Old unified/side-by-side component

---

## Task 1: Add Dependencies

**Files:**
- Modify: `package.json`

- [ ] **Step 1: Add diff-match-patch to dependencies**

Edit `package.json` and add to `dependencies`:
```json
"diff-match-patch": "^20240101"
```

- [ ] **Step 2: Run npm install**

```bash
npm install
```

Expected: `diff-match-patch` appears in `node_modules/` and `package-lock.json`

- [ ] **Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore: add diff-match-patch dependency"
```

---

## Task 2: Create useSyncScroll Composable

**Files:**
- Create: `src/composables/useSyncScroll.ts`

- [ ] **Step 1: Write useSyncScroll composable**

```typescript
import { ref, onMounted, onBeforeUnmount } from "vue";

export function useSyncScroll() {
  const leftPanelRef = ref<HTMLElement | null>(null);
  const rightPanelRef = ref<HTMLElement | null>(null);
  const isSyncing = ref(false);

  function setupSync() {
    if (!leftPanelRef.value || !rightPanelRef.value) return;

    const leftPanel = leftPanelRef.value;
    const rightPanel = rightPanelRef.value;

    const handleLeftScroll = () => {
      if (isSyncing.value) return;
      isSyncing.value = true;
      rightPanel.scrollTop = leftPanel.scrollTop;
      isSyncing.value = false;
    };

    const handleRightScroll = () => {
      if (isSyncing.value) return;
      isSyncing.value = true;
      leftPanel.scrollTop = rightPanel.scrollTop;
      isSyncing.value = false;
    };

    leftPanel.addEventListener("scroll", handleLeftScroll);
    rightPanel.addEventListener("scroll", handleRightScroll);

    return () => {
      leftPanel.removeEventListener("scroll", handleLeftScroll);
      rightPanel.removeEventListener("scroll", handleRightScroll);
    };
  }

  let cleanup: (() => void) | null = null;

  onMounted(() => {
    cleanup = setupSync() || null;
  });

  onBeforeUnmount(() => {
    if (cleanup) cleanup();
  });

  return {
    leftPanelRef,
    rightPanelRef,
    setupSync,
  };
}
```

- [ ] **Step 2: Commit**

```bash
git add src/composables/useSyncScroll.ts
git commit -m "feat: add useSyncScroll composable for synchronized scrolling"
```

---

## Task 3: Create useSideBySideDiff Composable

**Files:**
- Create: `src/composables/useSideBySideDiff.ts`

- [ ] **Step 1: Write word-level diff logic**

```typescript
import { DiffMatchPatch } from "diff-match-patch";
import type { FileDiff, DiffHunk, DiffLine } from "@/types";

export interface WordDiffSpan {
  text: string;
  kind: "added" | "removed" | "context";
}

export interface DiffLineWithWordDiff extends DiffLine {
  wordDiffs?: WordDiffSpan[];
}

export interface DiffHunkWithWordDiff extends DiffHunk {
  lines: DiffLineWithWordDiff[];
}

export function useSideBySideDiff() {
  const dmp = new DiffMatchPatch();

  function matchRelatedLines(hunk: DiffHunk): Map<number, number> {
    // Map of removed line index -> added line index
    const mapping = new Map<number, number>();
    const removedLines: Array<{ idx: number; content: string }> = [];
    const addedLines: Array<{ idx: number; content: string }> = [];

    for (let i = 0; i < hunk.lines.length; i++) {
      const line = hunk.lines[i];
      if (line.kind === "removed") {
        removedLines.push({ idx: i, content: line.content });
      } else if (line.kind === "added") {
        addedLines.push({ idx: i, content: line.content });
      }
    }

    // Simple matching: for each removed line, find the most similar added line
    for (const removed of removedLines) {
      let bestMatch = -1;
      let bestScore = 0;

      for (let j = 0; j < addedLines.length; j++) {
        const added = addedLines[j];
        const diffs = dmp.diff_main(removed.content, added.content);
        const similarity = computeSimilarity(diffs);
        if (similarity > bestScore && !mapping.values().toArray().includes(j)) {
          bestScore = similarity;
          bestMatch = j;
        }
      }

      if (bestMatch >= 0) {
        mapping.set(removed.idx, addedLines[bestMatch].idx);
      }
    }

    return mapping;
  }

  function computeSimilarity(diffs: Array<[number, string]>): number {
    let sameCount = 0;
    let totalCount = 0;
    for (const [op] of diffs) {
      if (op === 0) sameCount++;
      totalCount++;
    }
    return totalCount > 0 ? sameCount / totalCount : 0;
  }

  function computeWordDiffs(oldText: string, newText: string): WordDiffSpan[] {
    const diffs = dmp.diff_main(oldText, newText);
    dmp.diff_cleanupSemantic(diffs);

    const spans: WordDiffSpan[] = [];
    for (const [op, text] of diffs) {
      if (op === 0) {
        spans.push({ text, kind: "context" });
      } else if (op === 1) {
        spans.push({ text, kind: "added" });
      } else if (op === -1) {
        spans.push({ text, kind: "removed" });
      }
    }
    return spans;
  }

  function enrichHunkWithWordDiff(
    hunk: DiffHunk
  ): DiffHunkWithWordDiff {
    const mapping = matchRelatedLines(hunk);
    const enrichedLines: DiffLineWithWordDiff[] = [];

    for (let i = 0; i < hunk.lines.length; i++) {
      const line = hunk.lines[i];
      const enrichedLine: DiffLineWithWordDiff = { ...line };

      if (line.kind === "removed" && mapping.has(i)) {
        const relatedIdx = mapping.get(i)!;
        const relatedLine = hunk.lines[relatedIdx];
        enrichedLine.wordDiffs = computeWordDiffs(
          line.content,
          relatedLine.content
        );
      } else if (line.kind === "added" && !Array.from(mapping.values()).includes(i)) {
        // Orphaned added line - highlight entire content as added
        enrichedLine.wordDiffs = [{ text: line.content, kind: "added" }];
      } else if (line.kind === "removed" && !mapping.has(i)) {
        // Orphaned removed line - highlight entire content as removed
        enrichedLine.wordDiffs = [{ text: line.content, kind: "removed" }];
      }

      enrichedLines.push(enrichedLine);
    }

    return { ...hunk, lines: enrichedLines };
  }

  function enrichAllHunks(hunks: DiffHunk[]): DiffHunkWithWordDiff[] {
    return hunks.map(enrichHunkWithWordDiff);
  }

  return {
    enrichHunkWithWordDiff,
    enrichAllHunks,
  };
}
```

- [ ] **Step 2: Commit**

```bash
git add src/composables/useSideBySideDiff.ts
git commit -m "feat: add useSideBySideDiff for word-level diff computation"
```

---

## Task 4: Create DiffLinesPair Component

**Files:**
- Create: `src/components/DiffLinesPair.vue`

- [ ] **Step 1: Write DiffLinesPair component**

```vue
<script setup lang="ts">
import type { DiffLineWithWordDiff, WordDiffSpan } from "@/composables/useSideBySideDiff";

interface Props {
  oldLine?: DiffLineWithWordDiff;
  newLine?: DiffLineWithWordDiff;
  showPlaceholder?: boolean;
}

withDefaults(defineProps<Props>(), {
  showPlaceholder: false,
});

function renderWordDiffs(spans: WordDiffSpan[] | undefined): string {
  if (!spans) return "";
  return spans.map(s => s.text).join("");
}

function getLineClass(line?: DiffLineWithWordDiff): string {
  if (!line) return "placeholder";
  return line.kind;
}
</script>

<template>
  <div class="diff-lines-pair">
    <div class="diff-line-container old-line" :class="getLineClass(oldLine)">
      <span class="line-no">{{ oldLine?.old_lineno ?? "" }}</span>
      <span class="line-prefix">{{ oldLine ? (oldLine.kind === "removed" ? "-" : " ") : "" }}</span>
      <span class="line-content">
        <template v-if="oldLine?.wordDiffs">
          <template v-for="(span, idx) in oldLine.wordDiffs" :key="idx">
            <span :class="['word-diff', span.kind]">{{ span.text }}</span>
          </template>
        </template>
        <template v-else>
          {{ oldLine?.content ?? "" }}
        </template>
      </span>
    </div>

    <div class="diff-line-container new-line" :class="getLineClass(newLine)">
      <span class="line-no">{{ newLine?.new_lineno ?? "" }}</span>
      <span class="line-prefix">{{ newLine ? (newLine.kind === "added" ? "+" : " ") : "" }}</span>
      <span class="line-content">
        <template v-if="newLine?.wordDiffs">
          <template v-for="(span, idx) in newLine.wordDiffs" :key="idx">
            <span :class="['word-diff', span.kind]">{{ span.text }}</span>
          </template>
        </template>
        <template v-else>
          {{ newLine?.content ?? "" }}
        </template>
      </span>
    </div>
  </div>
</template>

<style scoped>
.diff-lines-pair {
  display: flex;
  width: 100%;
}

.diff-line-container {
  flex: 1;
  display: flex;
  white-space: pre;
  min-height: 20px;
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  line-height: 1.5;
}

.diff-line-container.removed {
  background: var(--diff-removed-bg);
}

.diff-line-container.added {
  background: var(--diff-added-bg);
}

.diff-line-container.placeholder {
  background: rgba(69, 71, 90, 0.3);
}

.old-line {
  border-right: 1px solid var(--border-subtle);
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
  overflow-x: auto;
  white-space: pre;
}

.word-diff {
  transition: background-color 0.2s ease;
}

.word-diff.added {
  background: var(--diff-word-added-bg);
  color: inherit;
}

.word-diff.removed {
  background: var(--diff-word-removed-bg);
  color: inherit;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/DiffLinesPair.vue
git commit -m "feat: add DiffLinesPair component for side-by-side line rendering"
```

---

## Task 5: Create DiffPanel Component

**Files:**
- Create: `src/components/DiffPanel.vue`

- [ ] **Step 1: Write DiffPanel component**

```vue
<script setup lang="ts">
import { ref } from "vue";
import DiffLinesPair from "./DiffLinesPair.vue";
import type { DiffHunkWithWordDiff } from "@/composables/useSideBySideDiff";

interface Props {
  hunks: DiffHunkWithWordDiff[];
  isOldVersion: boolean;
}

withDefaults(defineProps<Props>());

const panelRef = ref<HTMLDivElement | null>(null);
</script>

<template>
  <div ref="panelRef" class="diff-panel">
    <div v-for="(hunk, hi) in hunks" :key="hi" class="diff-hunk">
      <div class="hunk-header">{{ hunk.header }}</div>
      <div
        v-for="(line, li) in hunk.lines"
        :key="li"
        class="line-wrapper"
        :class="{
          'show-old': isOldVersion && (line.kind === 'removed' || line.kind === 'context'),
          'show-new': !isOldVersion && (line.kind === 'added' || line.kind === 'context'),
        }"
      >
        <div class="diff-line" :class="line.kind">
          <span class="line-no">{{ isOldVersion ? line.old_lineno : line.new_lineno }}</span>
          <span class="line-prefix">
            {{ line.kind === "removed" ? "-" : line.kind === "added" ? "+" : " " }}
          </span>
          <span class="line-content">
            <template v-if="line.wordDiffs">
              <template v-for="(span, idx) in line.wordDiffs" :key="idx">
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
</template>

<style scoped>
.diff-panel {
  flex: 1;
  overflow: auto;
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
}

.diff-hunk {
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

.line-wrapper {
  display: none;
}

.line-wrapper.show-old,
.line-wrapper.show-new {
  display: block;
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
  overflow-x: auto;
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
```

- [ ] **Step 2: Commit**

```bash
git add src/components/DiffPanel.vue
git commit -m "feat: add DiffPanel component for single diff panel"
```

---

## Task 6: Create SideBySideDiffView Component (Replace DiffView)

**Files:**
- Create: `src/components/SideBySideDiffView.vue`

- [ ] **Step 1: Write SideBySideDiffView component**

```vue
<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useDiff } from "@/composables/useDiff";
import { useFiles } from "@/composables/useFiles";
import { useSyncScroll } from "@/composables/useSyncScroll";
import { useSideBySideDiff } from "@/composables/useSideBySideDiff";
import DiffPanel from "./DiffPanel.vue";
import type { DiffHunk } from "@/types";

const { currentDiff } = useDiff();
const { selectedFile } = useFiles();
const { leftPanelRef, rightPanelRef } = useSyncScroll();
const { enrichAllHunks } = useSideBySideDiff();

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
    </div>

    <div class="diff-header">
      <div class="diff-file-info">
        <span class="diff-compare-mode">Working Tree vs Index</span>
      </div>
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
        <div v-for="(hunk, hi) in enrichedHunks" :key="hi" :data-hunk-idx="hi" class="hunk-section">
          <div class="hunk-header">{{ hunk.header }}</div>
          <div
            v-for="(line, li) in hunk.lines"
            :key="li"
            class="diff-line"
            :class="[line.kind, { hidden: line.kind === 'added' }]"
          >
            <span class="line-no">{{ line.old_lineno ?? "" }}</span>
            <span class="line-prefix">{{ line.kind === "removed" ? "-" : " " }}</span>
            <span class="line-content">
              <template v-if="line.wordDiffs">
                <template v-for="(span, idx) in line.wordDiffs" :key="idx">
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
        <div v-for="(hunk, hi) in enrichedHunks" :key="hi" :data-hunk-idx="hi" class="hunk-section">
          <div class="hunk-header">{{ hunk.header }}</div>
          <div
            v-for="(line, li) in hunk.lines"
            :key="li"
            class="diff-line"
            :class="[line.kind, { hidden: line.kind === 'removed' }]"
          >
            <span class="line-no">{{ line.new_lineno ?? "" }}</span>
            <span class="line-prefix">{{ line.kind === "added" ? "+" : " " }}</span>
            <span class="line-content">
              <template v-if="line.wordDiffs">
                <template v-for="(span, idx) in line.wordDiffs" :key="idx">
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
  flex-shrink: 0;
}

.diff-file-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.diff-compare-mode {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
}

.diff-actions {
  display: flex;
  align-items: center;
  gap: 8px;
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
```

- [ ] **Step 2: Commit**

```bash
git add src/components/SideBySideDiffView.vue
git commit -m "feat: add SideBySideDiffView component with synchronized scroll and word-level highlighting"
```

---

## Task 7: Add CSS Variables for Word-Level Highlighting

**Files:**
- Modify: `src/styles/main.css`

- [ ] **Step 1: Add CSS variables**

In `src/styles/main.css`, find the `:root` selector and add:

```css
--diff-word-added-bg: rgba(34, 197, 94, 0.35);
--diff-word-removed-bg: rgba(239, 68, 68, 0.35);
```

Example location (after existing `--diff-added-bg` and `--diff-removed-bg`):

```css
:root {
  /* ... existing vars ... */
  --diff-added-bg: rgba(34, 197, 94, 0.15);
  --diff-removed-bg: rgba(239, 68, 68, 0.15);
  --diff-word-added-bg: rgba(34, 197, 94, 0.35);
  --diff-word-removed-bg: rgba(239, 68, 68, 0.35);
  /* ... */
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles/main.css
git commit -m "style: add CSS vars for word-level diff highlighting"
```

---

## Task 8: Update Imports in Components

**Files:**
- Modify: `src/App.vue` or wherever `DiffView.vue` is imported

- [ ] **Step 1: Find and update DiffView imports**

Search your codebase for `DiffView` imports:

```bash
grep -r "DiffView" src/
```

Expected output should show files that import DiffView.

- [ ] **Step 2: Update imports**

For each file importing `DiffView`, replace:

```typescript
import DiffView from "@/components/DiffView.vue";
```

with:

```typescript
import SideBySideDiffView from "@/components/SideBySideDiffView.vue";
```

And in templates, replace `<DiffView />` with `<SideBySideDiffView />`.

- [ ] **Step 3: Delete old DiffView.vue**

```bash
rm src/components/DiffView.vue
```

- [ ] **Step 4: Commit**

```bash
git add src/
git rm src/components/DiffView.vue
git commit -m "refactor: replace DiffView with SideBySideDiffView"
```

---

## Task 9: Test the Implementation

**Files:**
- Test: Manual testing in dev environment

- [ ] **Step 1: Start dev server**

```bash
npm run dev
```

- [ ] **Step 2: Open a repository with uncommitted changes**

Navigate to a repository in the app and make some changes to a file (e.g., edit `main.css` or any file).

- [ ] **Step 3: Click the file in the Files panel**

Select a modified file. The diff should now show in a two-panel side-by-side view.

- [ ] **Step 4: Verify synchronized scroll**

Scroll one panel vertically. The other panel should scroll in sync.

- [ ] **Step 5: Verify word-level highlighting**

Look for changed words/characters within lines — they should be highlighted with a more intense color than the line background.

- [ ] **Step 6: Test hunk navigation**

Use the Previous/Next hunk buttons to navigate between changed blocks. The hunk counter should update.

- [ ] **Step 7: Test horizontal scroll**

Find a long line and scroll horizontally. Both panels should scroll independently.

- [ ] **Step 8: Test with a commit**

Select a commit from the log. The diff should display the commit's changes in the same two-panel view.

- [ ] **Step 9: Commit test results**

```bash
git add -A
git commit -m "test: verify side-by-side diff implementation"
```

---

## Task 10: Cleanup and Final Verification

**Files:**
- Verify: All components are working, no console errors

- [ ] **Step 1: Check console for errors**

Open browser DevTools (F12). Navigate through files and commits. Verify there are no errors in the console.

- [ ] **Step 2: Verify styling is consistent**

Check that colors, spacing, and alignment match the design. The two panels should be equal width with a divider between them.

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete side-by-side diff view implementation"
```

---

## Summary

✅ **Dependencies added:** `diff-match-patch`
✅ **New composables:** `useSyncScroll`, `useSideBySideDiff`
✅ **New components:** `SideBySideDiffView`, `DiffPanel`, `DiffLinesPair`
✅ **Styling updated:** CSS vars for word-level highlighting
✅ **Old component removed:** `DiffView.vue`
✅ **Testing:** Manual verification of all features
