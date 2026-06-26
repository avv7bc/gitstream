<script setup lang="ts">
import { computed, ref } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";
import { useBlame } from "@/composables/useBlame";
import { useVirtualList } from "@/composables/useVirtualList";

const { path, lines, loading, goToCommit, close } = useBlame();
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

const t = computed(() => i18n.value.dialog.blame);

// Строки с признаком начала группы коммита: gutter (sha/автор/дата) рисуем
// только на первой строке подряд идущего одного коммита — как в GitHub/SmartGit.
const rows = computed(() =>
  lines.value.map((line, i) => ({
    line,
    groupStart: i === 0 || lines.value[i - 1].oid !== line.oid,
  })),
);

// Виртуализация: blame больших файлов (тысячи строк) рендерим окном видимых
// строк, иначе UI подвисает. Высота строки зафиксирована в CSS под LINE_HEIGHT.
const blScroll = ref<HTMLElement | null>(null);
const { visibleItems, paddingTop, paddingBottom } = useVirtualList(rows, blScroll);

function shortDate(epoch: number): string {
  return new Date(epoch * 1000).toISOString().slice(0, 10);
}
</script>

<template>
  <div class="modal-overlay" @keydown.escape="close" tabindex="-1">
    <div class="modal-dialog blame-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ t.title }} — <span class="bl-path">{{ path }}</span></h3>
        <button class="close-btn" @click="close">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="bl-body">
        <div v-if="!loading && lines.length === 0" class="bl-empty">{{ t.empty }}</div>
        <div v-else ref="blScroll" class="bl-scroll">
          <div :style="{ paddingTop: paddingTop + 'px', paddingBottom: paddingBottom + 'px' }">
            <div v-for="{ item: row } in visibleItems" :key="row.line.line_no" class="bl-row">
              <button
                class="bl-gutter"
                :class="{ 'group-start': row.groupStart }"
                :title="row.groupStart ? `${row.line.short_oid} · ${row.line.summary}` : ''"
                @click="goToCommit(row.line.oid)"
              >
                <template v-if="row.groupStart">
                  <span class="bl-sha">{{ row.line.short_oid }}</span>
                  <span class="bl-author">{{ row.line.author }}</span>
                  <span class="bl-date">{{ shortDate(row.line.author_time) }}</span>
                </template>
              </button>
              <span class="bl-lineno">{{ row.line.line_no }}</span>
              <span class="bl-content">{{ row.line.content }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <span class="bl-hint">{{ t.goHint }}</span>
        <button class="btn btn-secondary" @click="close">{{ t.close }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.blame-dialog {
  width: 940px;
  max-width: 95vw;
  height: 640px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
}
.bl-path {
  font-weight: 400;
  opacity: 0.8;
  font-family: var(--font-mono, monospace);
}
.bl-body {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--border-color, #333);
  border-radius: 4px;
  overflow: hidden;
}
.bl-empty {
  padding: 12px;
  opacity: 0.6;
}
.bl-scroll {
  height: 100%;
  overflow: auto;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
}
.bl-row {
  display: flex;
  align-items: stretch;
  white-space: pre;
  /* Высота строки зафиксирована под LINE_HEIGHT в useVirtualList (20px) —
     при изменении синхронизировать обе константы. */
  height: 20px;
  line-height: 20px;
}
.bl-row:hover {
  background: var(--hover-bg, rgba(255, 255, 255, 0.04));
}
.bl-gutter {
  flex: none;
  width: 260px;
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 0 8px;
  background: var(--panel-bg, rgba(255, 255, 255, 0.02));
  border: none;
  border-right: 1px solid var(--border-color, #333);
  color: var(--text-secondary, #88a);
  font-size: 11px;
  text-align: left;
  cursor: pointer;
  overflow: hidden;
}
.bl-gutter.group-start:hover {
  background: var(--selection-bg, rgba(120, 170, 255, 0.18));
}
.bl-sha {
  font-family: var(--font-mono, monospace);
  flex: none;
}
.bl-author {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}
.bl-date {
  flex: none;
  opacity: 0.7;
}
.bl-lineno {
  flex: none;
  width: 3.5em;
  text-align: right;
  padding: 0 8px;
  opacity: 0.5;
  user-select: none;
}
.bl-content {
  flex: 1;
}
.bl-hint {
  flex: 1;
  font-size: 12px;
  opacity: 0.6;
}
</style>
