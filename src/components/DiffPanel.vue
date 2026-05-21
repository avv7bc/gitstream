<script setup lang="ts">
import { ref } from "vue";
import type { DiffHunkWithWordDiff } from "@/composables/useSideBySideDiff";

interface Props {
  hunks: DiffHunkWithWordDiff[];
  isOldVersion: boolean;
}

withDefaults(defineProps<Props>(), {});

const panelRef = ref<HTMLDivElement | null>(null);
</script>

<template>
  <div ref="panelRef" class="diff-panel">
    <div v-for="(hunk, hunkIdx) in hunks" :key="hunkIdx" class="diff-hunk">
      <div class="hunk-header">{{ hunk.header }}</div>
      <div
        v-for="(line, lineIdx) in hunk.lines"
        :key="`${hunkIdx}-${lineIdx}`"
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
