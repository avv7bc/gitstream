<script setup lang="ts">
import type { DiffLineWithWordDiff } from "@/composables/useSideBySideDiff";

interface Props {
  oldLine?: DiffLineWithWordDiff;
  newLine?: DiffLineWithWordDiff;
  showPlaceholder?: boolean;
}

withDefaults(defineProps<Props>(), {
  showPlaceholder: false,
});


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
          <template v-for="span in oldLine.wordDiffs" :key="`${span.text}-${span.kind}`">
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
          <template v-for="span in newLine.wordDiffs" :key="`${span.text}-${span.kind}`">
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
