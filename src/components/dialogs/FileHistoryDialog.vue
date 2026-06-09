<script setup lang="ts">
import { computed, ref } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";
import { useFileHistory } from "@/composables/useFileHistory";
import { filterCommits } from "@/utils/commitFilter";
import { highlight } from "@/utils/highlight";

const { path, commits, selectedOid, fileDiff, loading, selectCommit, goToCommit, close } =
  useFileHistory();
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

const filter = ref("");
const t = computed(() => i18n.value.dialog.fileHistory);

const shown = computed(() => filterCommits(commits.value, filter.value));

function shortDate(iso: string): string {
  // %aI отдаёт ISO-дату; показываем YYYY-MM-DD без времени.
  return iso.slice(0, 10);
}

function subject(message: string): string {
  return message.split("\n")[0];
}
</script>

<template>
  <div class="modal-overlay" @keydown.escape="close" tabindex="-1">
    <div class="modal-dialog file-history-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ t.title }} — <span class="fh-path">{{ path }}</span></h3>
        <button class="close-btn" @click="close">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="fh-body">
        <!-- Список коммитов файла -->
        <div class="fh-list">
          <input
            v-model="filter"
            class="form-input fh-filter"
            type="text"
            :placeholder="t.filter"
          />
          <div class="fh-commits">
            <div v-if="!loading && commits.length === 0" class="fh-empty">{{ t.empty }}</div>
            <button
              v-for="c in shown"
              :key="c.oid"
              class="fh-commit"
              :class="{ selected: c.oid === selectedOid }"
              @click="selectCommit(c.oid)"
              @dblclick="goToCommit(c.oid)"
            >
              <div class="fh-commit-top">
                <span class="fh-sha">{{ c.short_oid }}</span>
                <span class="fh-date">{{ shortDate(c.date) }}</span>
              </div>
              <div class="fh-subject" v-html="highlight(subject(c.message), filter)" />
              <div class="fh-author" v-html="highlight(c.author, filter)" />
            </button>
          </div>
        </div>

        <!-- Diff файла на выбранном коммите -->
        <div class="fh-diff">
          <template v-if="fileDiff">
            <div v-if="fileDiff.binary" class="fh-note">{{ t.binary }}</div>
            <div v-else-if="fileDiff.hunks.length === 0" class="fh-note">{{ t.noChanges }}</div>
            <div v-else class="fh-diff-scroll">
              <div v-for="(hunk, hi) in fileDiff.hunks" :key="hi" class="fh-hunk">
                <div class="fh-hunk-header">{{ hunk.header }}</div>
                <div
                  v-for="(line, li) in hunk.lines"
                  :key="li"
                  class="fh-line"
                  :class="line.kind"
                >
                  <span class="fh-prefix">{{ line.kind === "removed" ? "-" : line.kind === "added" ? "+" : " " }}</span>
                  <span class="fh-content">{{ line.content }}</span>
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="close">{{ t.close }}</button>
        <button class="btn btn-primary" :disabled="!selectedOid" @click="selectedOid && goToCommit(selectedOid)">
          {{ t.goToCommit }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-history-dialog {
  width: 900px;
  max-width: 95vw;
  height: 620px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
}
.fh-path {
  font-weight: 400;
  opacity: 0.8;
  font-family: var(--font-mono, monospace);
}
.fh-body {
  display: flex;
  flex: 1;
  min-height: 0;
  gap: 8px;
}
.fh-list {
  width: 320px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.fh-filter {
  margin-bottom: 6px;
}
.fh-commits {
  flex: 1;
  overflow-y: auto;
  border: 1px solid var(--border-color, #333);
  border-radius: 4px;
}
.fh-empty {
  padding: 12px;
  opacity: 0.6;
  font-size: 13px;
}
.fh-commit {
  display: block;
  width: 100%;
  text-align: left;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--border-color, #2a2a2a);
  padding: 6px 8px;
  cursor: pointer;
  color: inherit;
}
.fh-commit:hover {
  background: var(--hover-bg, rgba(255, 255, 255, 0.04));
}
.fh-commit.selected {
  background: var(--selection-bg, rgba(120, 170, 255, 0.18));
}
.fh-commit-top {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  opacity: 0.7;
}
.fh-sha {
  font-family: var(--font-mono, monospace);
}
.fh-subject {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.fh-author {
  font-size: 11px;
  opacity: 0.6;
}
.fh-diff {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--border-color, #333);
  border-radius: 4px;
  overflow: hidden;
}
.fh-diff-scroll {
  height: 100%;
  overflow: auto;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
}
.fh-note {
  padding: 12px;
  opacity: 0.6;
}
.fh-hunk-header {
  background: var(--diff-hunk-bg, rgba(120, 170, 255, 0.1));
  color: var(--text-secondary, #88a);
  padding: 2px 8px;
  white-space: pre;
}
.fh-line {
  display: flex;
  white-space: pre;
}
.fh-line.added {
  background: var(--diff-added-bg, rgba(80, 200, 120, 0.12));
}
.fh-line.removed {
  background: var(--diff-removed-bg, rgba(230, 100, 100, 0.12));
}
.fh-prefix {
  width: 1.4em;
  text-align: center;
  flex: none;
  opacity: 0.7;
}
.fh-content {
  flex: 1;
}
</style>
