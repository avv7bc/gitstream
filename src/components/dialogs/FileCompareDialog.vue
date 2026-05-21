<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@/composables/useProgress";
import { useFileCompare } from "@/composables/useFileCompare";
import { useRepo } from "@/composables/useRepo";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";
import type { FileDiff, DiffHunk } from "@/types";

const { target, close } = useFileCompare();
const { repoPath } = useRepo();
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

const diff = ref<FileDiff | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const bodyEl = ref<HTMLElement | null>(null);

const LINE_H = 20;

interface SideRow {
  no: number | null;
  content: string;
  kind: "context" | "removed" | "added";
}
interface ChangeGroup {
  leftStart: number;
  leftEnd: number;
  rightStart: number;
  rightEnd: number;
  index: number;
}
interface PreparedHunk {
  header: string;
  left: SideRow[];
  right: SideRow[];
  groups: ChangeGroup[];
  height: number;
}

function prepare(hunk: DiffHunk, baseIndex: number): { h: PreparedHunk; nextIndex: number } {
  const left: SideRow[] = [];
  const right: SideRow[] = [];
  const groups: ChangeGroup[] = [];
  let i = 0;
  let idx = baseIndex;
  while (i < hunk.lines.length) {
    const ln = hunk.lines[i];
    if (ln.kind === "context") {
      left.push({ no: ln.old_lineno, content: ln.content, kind: "context" });
      right.push({ no: ln.new_lineno, content: ln.content, kind: "context" });
      i++;
    } else {
      const leftStart = left.length;
      const rightStart = right.length;
      while (i < hunk.lines.length && hunk.lines[i].kind === "removed") {
        const l = hunk.lines[i];
        left.push({ no: l.old_lineno, content: l.content, kind: "removed" });
        i++;
      }
      while (i < hunk.lines.length && hunk.lines[i].kind === "added") {
        const a = hunk.lines[i];
        right.push({ no: a.new_lineno, content: a.content, kind: "added" });
        i++;
      }
      groups.push({
        leftStart,
        leftEnd: left.length,
        rightStart,
        rightEnd: right.length,
        index: idx++,
      });
    }
  }
  const height = Math.max(left.length, right.length) * LINE_H;
  return { h: { header: hunk.header, left, right, groups, height }, nextIndex: idx };
}

const hunks = computed<PreparedHunk[]>(() => {
  if (!diff.value) return [];
  const out: PreparedHunk[] = [];
  let idx = 0;
  for (const h of diff.value.hunks) {
    const r = prepare(h, idx);
    out.push(r.h);
    idx = r.nextIndex;
  }
  return out;
});

const totalChanges = computed(() => hunks.value.reduce((s, h) => s + h.groups.length, 0));
const currentChange = ref(0);

function scrollToCurrent() {
  if (!bodyEl.value) return;
  const el = bodyEl.value.querySelector<HTMLElement>(`[data-group="${currentChange.value}"]`);
  if (el) el.scrollIntoView({ block: "center", behavior: "smooth" });
}

function nextChange() {
  if (totalChanges.value === 0) return;
  currentChange.value = (currentChange.value + 1) % totalChanges.value;
  scrollToCurrent();
}
function prevChange() {
  if (totalChanges.value === 0) return;
  currentChange.value = (currentChange.value - 1 + totalChanges.value) % totalChanges.value;
  scrollToCurrent();
}

async function load() {
  if (!target.value || !repoPath.value) return;
  loading.value = true;
  error.value = null;
  diff.value = null;
  try {
    if (target.value.oid) {
      const diffs = await invoke<FileDiff[]>("get_diff_commit", {
        repoPath: repoPath.value,
        oid: target.value.oid,
      });
      diff.value = diffs.find((d) => d.path === target.value!.path) ?? null;
    } else {
      diff.value = await invoke<FileDiff>("get_diff_file", {
        repoPath: repoPath.value,
        file: target.value.path,
        staged: target.value.staged ?? false,
      });
    }
    currentChange.value = 0;
    await nextTick();
    scrollToCurrent();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

watch(target, load, { immediate: true });

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") close();
}
onMounted(() => window.addEventListener("keydown", onKey));
onUnmounted(() => window.removeEventListener("keydown", onKey));

// --- Minimal syntax highlight ---
const KEYWORDS = new Set([
  "const", "let", "var", "function", "async", "await", "return", "if", "else", "for", "while", "do",
  "import", "from", "export", "default", "class", "extends", "new", "this", "typeof", "instanceof",
  "true", "false", "null", "undefined", "try", "catch", "finally", "throw", "switch", "case", "break", "continue",
  "in", "of", "void", "delete", "interface", "type", "enum", "public", "private", "protected", "readonly",
  "as", "is", "namespace", "module", "declare",
]);

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[c]!));
}

function highlight(code: string): string {
  if (!code) return "";
  const re = /(\/\/[^\n]*)|("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)|(\b\d+(?:\.\d+)?\b)|(\b[A-Za-z_$][\w$]*\b)|(\s+)|([^\s])/g;
  let out = "";
  let m: RegExpExecArray | null;
  while ((m = re.exec(code)) !== null) {
    if (m[1]) out += `<span class="tk-comment">${escapeHtml(m[1])}</span>`;
    else if (m[2]) out += `<span class="tk-string">${escapeHtml(m[2])}</span>`;
    else if (m[3]) out += `<span class="tk-num">${escapeHtml(m[3])}</span>`;
    else if (m[4]) {
      const cls = KEYWORDS.has(m[4]) ? "tk-kw" : "tk-id";
      out += `<span class="${cls}">${escapeHtml(m[4])}</span>`;
    }
    else if (m[5]) out += m[5];
    else if (m[6]) out += `<span class="tk-punct">${escapeHtml(m[6])}</span>`;
  }
  return out;
}

function curvePath(g: ChangeGroup): string {
  const y1L = g.leftStart * LINE_H;
  const y2L = g.leftEnd * LINE_H;
  const y1R = g.rightStart * LINE_H;
  const y2R = g.rightEnd * LINE_H;
  const W = 100;
  const cx = W / 2;
  return `M 0 ${y1L} C ${cx} ${y1L}, ${cx} ${y1R}, ${W} ${y1R} L ${W} ${y2R} C ${cx} ${y2R}, ${cx} ${y2L}, 0 ${y2L} Z`;
}
</script>

<template>
  <div class="fc-overlay" @click.self="close">
    <div class="fc-window" :style="dragStyle">
      <!-- Title bar -->
      <div class="fc-titlebar" @mousedown="onDragStart">
        <span class="fc-title">[{{ target?.path }}] - File Compare</span>
        <div class="fc-window-buttons">
          <button class="fc-wb" tabindex="-1">
            <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 6h8" stroke="currentColor" stroke-width="1.2"/></svg>
          </button>
          <button class="fc-wb" tabindex="-1">
            <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2" y="2" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
          </button>
          <button class="fc-wb close" @click="close">
            <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.4"/></svg>
          </button>
        </div>
      </div>

      <!-- Toolbar -->
      <div class="fc-toolbar">
        <button class="fc-tb" @click="prevChange" :title="i18n.dialog.fileCompare.prevChange">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 19V5"/>
            <path d="M5 12l7-7 7 7"/>
          </svg>
          <span>{{ i18n.dialog.fileCompare.prevChange }}</span>
        </button>
        <button class="fc-tb active" @click="nextChange" :title="i18n.dialog.fileCompare.nextChange">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 5v14"/>
            <path d="M5 12l7 7 7-7"/>
          </svg>
          <span>{{ i18n.dialog.fileCompare.nextChange }}</span>
        </button>
        <button class="fc-tb" disabled :title="i18n.dialog.fileCompare.takeLeft">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="5" width="6" height="14" rx="1"/>
            <path d="M14 12h7"/>
            <path d="M17 9l-3 3 3 3"/>
          </svg>
          <span>{{ i18n.dialog.fileCompare.takeLeft }}</span>
        </button>
        <button class="fc-tb" disabled :title="i18n.dialog.fileCompare.takeRight">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <rect x="15" y="5" width="6" height="14" rx="1"/>
            <path d="M3 12h7"/>
            <path d="M7 9l3 3-3 3"/>
          </svg>
          <span>{{ i18n.dialog.fileCompare.takeRight }}</span>
        </button>
      </div>

      <!-- Path headers -->
      <div class="fc-paths">
        <div class="fc-path">{{ target?.path }} (Before)</div>
        <div class="fc-path">{{ target?.path }}</div>
      </div>

      <!-- Body -->
      <div ref="bodyEl" class="fc-body">
        <div v-if="loading" class="fc-status-msg">Loading…</div>
        <div v-else-if="error" class="fc-status-msg err">{{ error }}</div>
        <div v-else-if="hunks.length === 0" class="fc-status-msg">No changes</div>
        <template v-else>
          <div
            v-for="(h, hi) in hunks"
            :key="hi"
            class="fc-hunk"
            :style="{ height: h.height + 'px' }"
          >
            <div class="fc-side">
              <div
                v-for="(row, ri) in h.left"
                :key="ri"
                class="fc-row"
                :class="row.kind"
              >
                <span class="fc-lno">{{ row.no ?? '' }}</span>
                <span class="fc-content" v-html="highlight(row.content)" />
              </div>
            </div>

            <svg
              class="fc-curves"
              :viewBox="`0 0 100 ${h.height}`"
              :style="{ height: h.height + 'px' }"
              preserveAspectRatio="none"
            >
              <path
                v-for="g in h.groups"
                :key="g.index"
                :d="curvePath(g)"
                :class="['fc-curve', g.index === currentChange ? 'current' : '']"
                :data-group="g.index"
              />
            </svg>

            <div class="fc-side">
              <div
                v-for="(row, ri) in h.right"
                :key="ri"
                class="fc-row"
                :class="row.kind"
              >
                <span class="fc-lno">{{ row.no ?? '' }}</span>
                <span class="fc-content" v-html="highlight(row.content)" />
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- Status bar -->
      <div class="fc-statusbar">
        <span>Ready</span>
        <span class="spacer" />
        <span class="pos">3:1</span>
        <span class="pos">-1~2</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fc-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.fc-window {
  width: 96vw;
  height: 94vh;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-sans);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);
}

/* Title bar */
.fc-titlebar {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 8px 0 12px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  user-select: none;
  cursor: move;
}
.fc-title {
  flex: 1;
  text-align: center;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
}
.fc-window-buttons {
  display: flex;
  gap: 0;
}
.fc-wb {
  width: 36px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--text-secondary);
  border: none;
  cursor: pointer;
}
.fc-wb:hover { background: var(--bg-hover); color: var(--text-primary); }
.fc-wb.close:hover { background: var(--red); color: #fff; }

/* Toolbar */
.fc-toolbar {
  display: flex;
  align-items: stretch;
  gap: 0;
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
}
.fc-tb {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  padding: 4px 12px;
  min-width: 64px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: var(--font-size-xs);
  cursor: pointer;
}
.fc-tb:hover:not(:disabled) { background: var(--bg-hover); color: var(--text-primary); }
.fc-tb:disabled { opacity: 0.45; cursor: default; }
.fc-tb.active {
  color: var(--text-primary);
  font-weight: 600;
}
.fc-tb.active span { border-bottom: 1px solid var(--accent); }
.fc-tb { color: var(--accent); }
.fc-tb span { color: var(--text-secondary); }
.fc-tb.active span { color: var(--text-primary); }

/* Path bars */
.fc-paths {
  display: flex;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
}
.fc-path {
  flex: 1;
  padding: 4px 10px;
  font-size: var(--font-size-xs);
  color: var(--text-primary);
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  margin: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Body / diff */
.fc-body {
  flex: 1;
  overflow: auto;
  background: var(--bg-primary);
  font-family: var(--font-mono);
  font-size: var(--font-size-diff);
}
.fc-status-msg {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
}
.fc-status-msg.err { color: var(--red); }

.fc-hunk {
  position: relative;
  display: flex;
  align-items: flex-start;
}
.fc-side {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}
.fc-row {
  display: flex;
  align-items: center;
  height: 20px;
  line-height: 20px;
  white-space: pre;
  overflow: hidden;
}
.fc-row.removed { background: var(--diff-removed-bg); }
.fc-row.added { background: var(--diff-added-bg); }

.fc-lno {
  width: 36px;
  flex-shrink: 0;
  text-align: right;
  padding-right: 8px;
  color: var(--text-muted);
  user-select: none;
  font-size: var(--font-size-xs);
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-subtle);
}
.fc-content {
  flex: 1;
  padding-left: 4px;
  color: var(--text-primary);
}

/* Connecting curves */
.fc-curves {
  position: absolute;
  left: 50%;
  top: 0;
  width: 80px;
  margin-left: -40px;
  pointer-events: none;
  overflow: visible;
}
.fc-curve {
  fill: var(--diff-removed-bg);
  stroke: var(--red);
  stroke-width: 0.6;
  opacity: 0.7;
}
.fc-curve.current {
  fill: var(--bg-active);
  stroke: var(--accent);
  opacity: 0.85;
}

/* Syntax tokens — derived from theme accents */
.fc-content :deep(.tk-kw) { color: var(--purple); }
.fc-content :deep(.tk-string) { color: var(--orange); }
.fc-content :deep(.tk-num) { color: var(--orange); }
.fc-content :deep(.tk-comment) { color: var(--text-muted); font-style: italic; }
.fc-content :deep(.tk-id) { color: var(--text-primary); }
.fc-content :deep(.tk-punct) { color: var(--teal); }

/* Status bar */
.fc-statusbar {
  display: flex;
  align-items: center;
  height: 22px;
  padding: 0 12px;
  background: var(--bg-surface);
  border-top: 1px solid var(--border);
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  user-select: none;
}
.fc-statusbar .spacer { flex: 1; }
.fc-statusbar .pos {
  margin-left: 24px;
  color: var(--text-secondary);
}
</style>
