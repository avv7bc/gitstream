<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, watch, computed } from "vue";
import { useStats, type StatsPeriod } from "@/composables/useStats";
import { useI18n } from "@/composables/useI18n";

const emit = defineEmits<{ close: [] }>();

const { loading, stats, error, period, progress, loadStats, abort } = useStats();
const { i18n } = useI18n();

// --- Persisted window geometry ---
const GEOM_KEY = "gitstream-stats-geom";

interface Geom { x: number; y: number; w: number; h: number; }

function loadGeom(): Geom {
  try {
    const raw = localStorage.getItem(GEOM_KEY);
    if (raw) {
      const g = JSON.parse(raw);
      if (typeof g.x === "number" && typeof g.y === "number" && typeof g.w === "number" && typeof g.h === "number") {
        return clampGeom(g);
      }
    }
  } catch { /* ignore */ }
  const w = Math.min(860, Math.round(window.innerWidth * 0.75));
  const h = Math.min(720, Math.round(window.innerHeight * 0.82));
  return { x: Math.round((window.innerWidth - w) / 2), y: Math.round((window.innerHeight - h) / 2), w, h };
}

function clampGeom(g: Geom): Geom {
  const w = Math.max(520, Math.min(g.w, window.innerWidth));
  const h = Math.max(360, Math.min(g.h, window.innerHeight));
  const x = Math.max(0, Math.min(g.x, window.innerWidth - w));
  const y = Math.max(0, Math.min(g.y, window.innerHeight - h));
  return { x, y, w, h };
}

import { ref } from "vue";
const geom = ref<Geom>(loadGeom());
function saveGeom() { localStorage.setItem(GEOM_KEY, JSON.stringify(geom.value)); }

// Drag
let dragStart: { x: number; y: number; gx: number; gy: number } | null = null;
function onTitleMouseDown(e: MouseEvent) {
  if ((e.target as HTMLElement).closest(".st-icon-btn")) return;
  dragStart = { x: e.clientX, y: e.clientY, gx: geom.value.x, gy: geom.value.y };
  document.addEventListener("mousemove", onDragMove);
  document.addEventListener("mouseup", onDragEnd);
  e.preventDefault();
}
function onDragMove(e: MouseEvent) {
  if (!dragStart) return;
  geom.value = clampGeom({ ...geom.value, x: dragStart.gx + (e.clientX - dragStart.x), y: dragStart.gy + (e.clientY - dragStart.y) });
}
function onDragEnd() {
  dragStart = null;
  document.removeEventListener("mousemove", onDragMove);
  document.removeEventListener("mouseup", onDragEnd);
  saveGeom();
}

// Resize
type ResizeDir = "e" | "s" | "se" | "n" | "w" | "ne" | "nw" | "sw";
let resizeStart: { x: number; y: number; g: Geom; dir: ResizeDir } | null = null;
function onResizeStart(e: MouseEvent, dir: ResizeDir) {
  resizeStart = { x: e.clientX, y: e.clientY, g: { ...geom.value }, dir };
  document.addEventListener("mousemove", onResizeMove);
  document.addEventListener("mouseup", onResizeEnd);
  e.preventDefault();
  e.stopPropagation();
}
function onResizeMove(e: MouseEvent) {
  if (!resizeStart) return;
  const dx = e.clientX - resizeStart.x;
  const dy = e.clientY - resizeStart.y;
  let { x, y, w, h } = resizeStart.g;
  if (resizeStart.dir.includes("e")) w += dx;
  if (resizeStart.dir.includes("s")) h += dy;
  if (resizeStart.dir.includes("w")) { w -= dx; x += dx; }
  if (resizeStart.dir.includes("n")) { h -= dy; y += dy; }
  geom.value = clampGeom({ x, y, w, h });
}
function onResizeEnd() {
  resizeStart = null;
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
  saveGeom();
}

function onWindowResize() { geom.value = clampGeom(geom.value); }
onMounted(() => {
  window.addEventListener("resize", onWindowResize);
  nextTick(() => loadStats());
});
onUnmounted(() => {
  abort();
  window.removeEventListener("resize", onWindowResize);
  document.removeEventListener("mousemove", onDragMove);
  document.removeEventListener("mouseup", onDragEnd);
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
});

watch(period, () => loadStats());

// --- Helpers ---

// --- Contributors sort ---
type SortKey = "name" | "commits" | "insertions" | "deletions" | "active_days" | "first_date" | "last_date";
const sortKey = ref<SortKey>("commits");
const sortAsc = ref(false);

function setSort(key: SortKey) {
  if (sortKey.value === key) sortAsc.value = !sortAsc.value;
  else { sortKey.value = key; sortAsc.value = key === "name" || key === "first_date" || key === "last_date"; }
}

const sortedAuthors = computed(() => {
  if (!stats.value) return [];
  return [...stats.value.authors].sort((a, b) => {
    const av = a[sortKey.value];
    const bv = b[sortKey.value];
    const cmp = typeof av === "string" ? av.localeCompare(bv as string) : (av as number) - (bv as number);
    return sortAsc.value ? cmp : -cmp;
  });
});

// --- Helpers ---
// Локализованные дни недели для барчарта by_weekday и labels heatmap.
const WEEKDAYS = computed(() => i18n.value.stats.weekdays);

function maxVal(arr: number[]): number {
  return arr.reduce((m, v) => Math.max(m, v), 0) || 1;
}

function pct(v: number, total: number): string {
  return total > 0 ? ((v / total) * 100).toFixed(1) : "0.0";
}

function fmtNum(n: number): string {
  return n.toLocaleString();
}

// --- Heatmap ---
// Месяцы тянем из i18n, чтобы лейблы heatmap'а переключались с языком.
const HM_MONTHS = computed(() => i18n.value.stats.months);

interface HmCell { date: string; count: number; future: boolean; }
interface HmData {
  weeks: HmCell[][];
  monthLabels: { label: string; wi: number }[];
  maxCount: number;
  total: number;
}

const heatmap = computed((): HmData | null => {
  if (!stats.value || stats.value.by_day.length === 0) return null;

  const dayMap = new Map<string, number>();
  for (const d of stats.value.by_day) dayMap.set(d.date, d.count);

  const todayStr = new Date().toISOString().split('T')[0];
  const firstDate = stats.value.by_day[0].date;
  const lastDate = stats.value.by_day[stats.value.by_day.length - 1].date;
  const endStr = lastDate > todayStr ? todayStr : lastDate;

  const start = new Date(firstDate + 'T00:00:00');
  const startDow = (start.getDay() + 6) % 7;
  start.setDate(start.getDate() - startDow);

  const end = new Date(endStr + 'T00:00:00');
  const endDow = (end.getDay() + 6) % 7;
  end.setDate(end.getDate() + (6 - endDow));

  const weeks: HmCell[][] = [];
  const monthLabels: { label: string; wi: number }[] = [];
  let lastMonth = -1;
  const cur = new Date(start);

  while (cur <= end) {
    const m = cur.getMonth();
    if (m !== lastMonth) { monthLabels.push({ label: HM_MONTHS.value[m], wi: weeks.length }); lastMonth = m; }
    const week: HmCell[] = [];
    for (let i = 0; i < 7; i++) {
      const d = cur.toISOString().split('T')[0];
      week.push({ date: d, count: dayMap.get(d) ?? 0, future: d > todayStr });
      cur.setDate(cur.getDate() + 1);
    }
    weeks.push(week);
  }

  const maxCount = Math.max(...stats.value.by_day.map(d => d.count), 1);
  const total = stats.value.by_day.reduce((s, d) => s + d.count, 0);
  return { weeks, monthLabels, maxCount, total };
});

function cellLevel(count: number, maxCount: number): number {
  if (count === 0) return 0;
  const r = count / maxCount;
  if (r <= 0.25) return 1;
  if (r <= 0.5)  return 2;
  if (r <= 0.75) return 3;
  return 4;
}

const periods = computed<{ value: StatsPeriod; label: string }[]>(() => [
  { value: "7d",  label: i18n.value.stats.periodWeek },
  { value: "30d", label: i18n.value.stats.periodMonth },
  { value: "1y",  label: i18n.value.stats.periodYear },
  { value: "all", label: i18n.value.stats.periodAll },
]);
</script>

<template>
  <div class="modal-overlay" @mousedown.self="emit('close')">
    <div
      class="st-dialog"
      :style="{ left: geom.x + 'px', top: geom.y + 'px', width: geom.w + 'px', height: geom.h + 'px' }"
      @mousedown.stop
    >
      <!-- Resize handles -->
      <div class="rh rh-n" @mousedown="onResizeStart($event, 'n')" />
      <div class="rh rh-s" @mousedown="onResizeStart($event, 's')" />
      <div class="rh rh-e" @mousedown="onResizeStart($event, 'e')" />
      <div class="rh rh-w" @mousedown="onResizeStart($event, 'w')" />
      <div class="rh rh-ne" @mousedown="onResizeStart($event, 'ne')" />
      <div class="rh rh-nw" @mousedown="onResizeStart($event, 'nw')" />
      <div class="rh rh-se" @mousedown="onResizeStart($event, 'se')" />
      <div class="rh rh-sw" @mousedown="onResizeStart($event, 'sw')" />

      <!-- Title bar -->
      <div class="st-titlebar" @mousedown="onTitleMouseDown">
        <span class="st-title">{{ i18n.stats.title }}</span>
        <div class="st-period-tabs" @mousedown.stop>
          <button
            v-for="p in periods"
            :key="p.value"
            class="st-period-btn"
            :class="{ active: period === p.value }"
            @click="period = p.value"
          >{{ p.label }}</button>
        </div>
        <button class="st-icon-btn close" @click="abort(); emit('close')" title="Close">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <!-- Body -->
      <div class="st-body">
        <!-- Loading overlay (поверх контента при пересчёте) -->
        <div v-if="loading" class="st-loading">
          <div class="st-loading-inner">
            <div class="st-spinner" />
            <div class="st-progress-track">
              <div class="st-progress-fill" :style="{ width: progress + '%' }" />
            </div>
            <span class="st-progress-label">{{ progress }}%</span>
          </div>
        </div>

        <!-- Error -->
        <div v-if="error && !loading" class="st-error">{{ error }}</div>

        <!-- Content -->
        <template v-if="stats">

          <!-- ── Activity Heatmap ──────────────────── -->
          <section v-if="heatmap" class="st-section">
            <div class="st-section-title-row">
              <h2 class="st-section-title">{{ i18n.stats.activity }} &mdash; {{ fmtNum(heatmap.total) }} {{ i18n.stats.commits }}</h2>
              <div class="st-hm-legend">
                <span>{{ i18n.stats.less }}</span>
                <div v-for="l in [0,1,2,3,4]" :key="l" class="st-hm-cell" :class="`lv-${l}`" />
                <span>{{ i18n.stats.more }}</span>
              </div>
            </div>
            <div class="st-heatmap-scroll">
              <!-- Month labels -->
              <div class="st-hm-top">
                <div class="st-hm-day-spacer" />
                <div class="st-hm-month-row" :style="{ gridTemplateColumns: `repeat(${heatmap.weeks.length}, 13px)` }">
                  <div v-for="(_, wi) in heatmap.weeks" :key="wi" class="st-hm-month-cell">
                    <span v-if="heatmap.monthLabels.some(m => m.wi === wi)">
                      {{ heatmap.monthLabels.find(m => m.wi === wi)!.label }}
                    </span>
                  </div>
                </div>
              </div>
              <!-- Day labels + cells -->
              <div class="st-hm-body">
                <div class="st-hm-day-labels">
                  <div
                    v-for="(label, di) in [i18n.stats.weekdays[0], '', i18n.stats.weekdays[2], '', i18n.stats.weekdays[4], '', i18n.stats.weekdays[6]]"
                    :key="di"
                    class="st-hm-day-label"
                  >{{ label }}</div>
                </div>
                <div class="st-hm-cells">
                  <template v-for="(week, wi) in heatmap.weeks" :key="wi">
                    <div
                      v-for="(cell, di) in week"
                      :key="di"
                      class="st-hm-cell"
                      :class="[`lv-${cellLevel(cell.count, heatmap.maxCount)}`, { future: cell.future }]"
                      :title="cell.count > 0 ? `${cell.date}: ${cell.count} ${i18n.stats.commits}` : cell.date"
                    />
                  </template>
                </div>
              </div>
            </div>
          </section>

          <!-- ── Contributors ───────────────────────── -->
          <section class="st-section">
            <h2 class="st-section-title">{{ i18n.stats.contributors }}</h2>
            <div class="st-table-wrapper">
              <table class="st-table">
                <thead>
                  <tr>
                    <th class="st-th-idx">#</th>
                    <th class="st-th-name st-th-sort" @click="setSort('name')">
                      Author<span class="st-sort-icon">{{ sortKey === 'name' ? (sortAsc ? ' ↑' : ' ↓') : '' }}</span>
                    </th>
                    <th class="st-th-num st-th-sort" @click="setSort('commits')">
                      Commits<span class="st-sort-icon">{{ sortKey === 'commits' ? (sortAsc ? ' ↑' : ' ↓') : '' }}</span>
                    </th>
                    <th class="st-th-num st-th-sort" @click="setSort('insertions')">
                      +Lines<span class="st-sort-icon">{{ sortKey === 'insertions' ? (sortAsc ? ' ↑' : ' ↓') : '' }}</span>
                    </th>
                    <th class="st-th-num st-th-sort" @click="setSort('deletions')">
                      −Lines<span class="st-sort-icon">{{ sortKey === 'deletions' ? (sortAsc ? ' ↑' : ' ↓') : '' }}</span>
                    </th>
                    <th class="st-th-num st-th-sort" @click="setSort('active_days')">
                      Days<span class="st-sort-icon">{{ sortKey === 'active_days' ? (sortAsc ? ' ↑' : ' ↓') : '' }}</span>
                    </th>
                    <th class="st-th-date st-th-sort" @click="setSort('first_date')">
                      First<span class="st-sort-icon">{{ sortKey === 'first_date' ? (sortAsc ? ' ↑' : ' ↓') : '' }}</span>
                    </th>
                    <th class="st-th-date st-th-sort" @click="setSort('last_date')">
                      Last<span class="st-sort-icon">{{ sortKey === 'last_date' ? (sortAsc ? ' ↑' : ' ↓') : '' }}</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(a, i) in sortedAuthors" :key="a.email" class="st-tr">
                    <td class="st-td-idx">{{ i + 1 }}</td>
                    <td class="st-td-name">
                      <span class="st-author-name">{{ a.name }}</span>
                      <span class="st-author-email">{{ a.email }}</span>
                    </td>
                    <td class="st-td-num">
                      {{ fmtNum(a.commits) }}
                      <span class="st-pct">({{ pct(a.commits, stats.total_commits) }}%)</span>
                    </td>
                    <td class="st-td-num st-green">+{{ fmtNum(a.insertions) }}</td>
                    <td class="st-td-num st-red">−{{ fmtNum(a.deletions) }}</td>
                    <td class="st-td-num">{{ a.active_days }}</td>
                    <td class="st-td-date">{{ a.first_date }}</td>
                    <td class="st-td-date">{{ a.last_date }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>

          <!-- ── By Weekday ─────────────────────────── -->
          <section class="st-section">
            <h2 class="st-section-title">{{ i18n.stats.commitsByDow }}</h2>
            <div class="st-bars">
              <div
                v-for="(count, i) in stats.by_weekday"
                :key="i"
                class="st-bar-row"
              >
                <span class="st-bar-label">{{ WEEKDAYS[i] }}</span>
                <div class="st-bar-track">
                  <div
                    class="st-bar-fill"
                    :style="{ width: (count / maxVal(stats.by_weekday) * 100) + '%' }"
                  />
                </div>
                <span class="st-bar-count">{{ fmtNum(count) }}</span>
                <span class="st-bar-pct">({{ pct(count, stats.total_commits) }}%)</span>
              </div>
            </div>
          </section>

          <!-- ── By Hour ────────────────────────────── -->
          <section class="st-section">
            <h2 class="st-section-title">{{ i18n.stats.commitsByHour }}</h2>
            <div class="st-bars st-bars-hour">
              <template v-for="(count, h) in stats.by_hour" :key="h">
                <div v-if="count > 0" class="st-bar-row">
                  <span class="st-bar-label st-bar-label-hour">{{ String(h).padStart(2, '0') }}</span>
                  <div class="st-bar-track">
                    <div
                      class="st-bar-fill"
                      :style="{ width: (count / maxVal(stats.by_hour) * 100) + '%' }"
                    />
                  </div>
                  <span class="st-bar-count">{{ fmtNum(count) }}</span>
                </div>
              </template>
            </div>
          </section>

          <!-- ── By Month ───────────────────────────── -->
          <section class="st-section">
            <h2 class="st-section-title">{{ i18n.stats.commitsByMonth }}</h2>
            <div v-if="stats.by_month.length === 0" class="st-empty">{{ i18n.stats.noData }}</div>
            <div v-else class="st-bars">
              <div
                v-for="m in stats.by_month"
                :key="m.month"
                class="st-bar-row"
              >
                <span class="st-bar-label st-bar-label-month">{{ m.month }}</span>
                <div class="st-bar-track">
                  <div
                    class="st-bar-fill"
                    :style="{ width: (m.commits / maxVal(stats.by_month.map(x => x.commits)) * 100) + '%' }"
                  />
                </div>
                <span class="st-bar-count">{{ fmtNum(m.commits) }}</span>
              </div>
            </div>
          </section>

          <!-- ── Overview ─────────────────────────────── -->
          <section class="st-section">
            <h2 class="st-section-title">{{ i18n.stats.overview }}</h2>
            <div class="st-overview-grid">
              <div class="st-kv">
                <span class="st-kv-label">{{ i18n.stats.totalCommits }}</span>
                <span class="st-kv-value">{{ fmtNum(stats.total_commits) }}</span>
              </div>
              <div class="st-kv">
                <span class="st-kv-label">{{ i18n.stats.authors }}</span>
                <span class="st-kv-value">{{ stats.total_authors }}</span>
              </div>
              <div class="st-kv">
                <span class="st-kv-label">{{ i18n.stats.linesAdded }}</span>
                <span class="st-kv-value st-green">+{{ fmtNum(stats.total_insertions) }}</span>
              </div>
              <div class="st-kv">
                <span class="st-kv-label">{{ i18n.stats.linesRemoved }}</span>
                <span class="st-kv-value st-red">−{{ fmtNum(stats.total_deletions) }}</span>
              </div>
              <div class="st-kv">
                <span class="st-kv-label">{{ i18n.stats.firstCommit }}</span>
                <span class="st-kv-value">{{ stats.first_commit_date || '—' }}</span>
              </div>
              <div class="st-kv">
                <span class="st-kv-label">{{ i18n.stats.lastCommit }}</span>
                <span class="st-kv-value">{{ stats.last_commit_date || '—' }}</span>
              </div>
            </div>
          </section>

        </template>

        <div v-if="!stats && !loading && !error" class="st-empty-center">{{ i18n.stats.noRepoSelected }}</div>
      </div>
    </div>
  </div>
</template>

<style>
@keyframes st-spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}
</style>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 150;
}

.st-dialog {
  position: absolute;
  background: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55);
  font-family: var(--font-sans);
  min-width: 520px;
  min-height: 360px;
}

/* Resize handles */
.rh { position: absolute; z-index: 10; }
.rh-n  { top: 0; left: 6px; right: 6px; height: 4px; cursor: ns-resize; }
.rh-s  { bottom: 0; left: 6px; right: 6px; height: 4px; cursor: ns-resize; }
.rh-e  { top: 6px; bottom: 6px; right: 0; width: 4px; cursor: ew-resize; }
.rh-w  { top: 6px; bottom: 6px; left: 0; width: 4px; cursor: ew-resize; }
.rh-ne { top: 0; right: 0; width: 8px; height: 8px; cursor: nesw-resize; }
.rh-nw { top: 0; left: 0; width: 8px; height: 8px; cursor: nwse-resize; }
.rh-se { bottom: 0; right: 0; width: 10px; height: 10px; cursor: nwse-resize; }
.rh-sw { bottom: 0; left: 0; width: 8px; height: 8px; cursor: nesw-resize; }

/* Title bar */
.st-titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 8px 0 14px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
  cursor: move;
  flex-shrink: 0;
  position: relative;
  z-index: 20;
}
.st-title {
  flex: 1;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.st-period-tabs {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.st-period-btn {
  padding: 5px 16px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  border: none;
  border-right: 1px solid var(--border);
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.12s, color 0.12s;
}
.st-period-btn:last-child { border-right: none; }
.st-period-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
.st-period-btn.active {
  background: var(--accent, #4e86d4);
  color: #fff;
}
.st-icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  border-radius: var(--radius);
  cursor: pointer;
  flex-shrink: 0;
}
.st-icon-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
.st-icon-btn.close:hover { background: var(--red); color: #fff; }

/* Body */
.st-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px 32px;
  position: relative;
}

/* Loading overlay — полупрозрачный поверх контента */
.st-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--bg-primary) 70%, transparent);
  z-index: 10;
}
.st-loading-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}
.st-spinner {
  width: 28px;
  height: 28px;
  border: 2px solid var(--border);
  border-top-color: var(--accent, #6b9ce8);
  border-radius: 50%;
  animation: st-spin 0.75s linear infinite;
}
.st-progress-track {
  width: 160px;
  height: 4px;
  background: var(--border);
  border-radius: 2px;
  overflow: hidden;
}
.st-progress-fill {
  height: 100%;
  background: var(--accent, #6b9ce8);
  border-radius: 2px;
  transition: width 0.15s ease;
}
.st-progress-label {
  font-size: 11px;
  color: var(--text-secondary);
}
.st-error {
  color: var(--red);
  font-size: var(--font-size-sm);
  padding: 16px 0;
}
.st-empty-center {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  font-size: var(--font-size-sm);
}

/* Sections */
.st-section {
  margin-bottom: 36px;
}
.st-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-secondary);
  margin-bottom: 14px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-subtle);
}

/* Overview grid */
.st-overview-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  grid-template-rows: repeat(2, auto);
  grid-auto-flow: column;
}
.st-kv {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 7px 20px 7px 0;
  border-bottom: 1px solid var(--border-subtle);
}
.st-kv:nth-child(2),
.st-kv:nth-child(4),
.st-kv:nth-child(6) {
  border-bottom: none;
}
.st-kv-label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  white-space: nowrap;
}
.st-kv-value {
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  font-weight: 600;
  white-space: nowrap;
}
.st-green { color: var(--green); }
.st-red   { color: var(--red); }

/* Table */
.st-table-wrapper {
  overflow-x: auto;
}
.st-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--font-size-sm);
}
.st-table th {
  text-align: left;
  color: var(--text-secondary);
  font-weight: 600;
  padding: 4px 10px 8px 0;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.st-th-num, .st-th-date { text-align: left; }
.st-th-sort {
  cursor: pointer;
  user-select: none;
}
.st-th-sort:hover { color: var(--text-primary); }
.st-sort-icon { color: var(--accent); font-size: 11px; }
.st-tr:hover { background: var(--bg-hover); }
.st-tr td { padding: 5px 10px 5px 0; border-bottom: 1px solid var(--border-subtle); vertical-align: top; }
.st-th-idx { width: 28px; color: var(--text-secondary); font-weight: 600; padding: 4px 8px 8px 0; border-bottom: 1px solid var(--border); }
.st-td-idx { width: 28px; color: var(--text-secondary); font-size: 11px; padding: 5px 8px 5px 0; border-bottom: 1px solid var(--border-subtle); vertical-align: top; }
.st-td-name { min-width: 160px; }
.st-author-name { display: block; color: var(--text-primary); }
.st-author-email { display: block; color: var(--text-secondary); font-size: 11px; }
.st-td-num { text-align: left; white-space: nowrap; color: var(--text-primary); }
.st-td-date { text-align: left; white-space: nowrap; color: var(--text-secondary); font-size: 11px; }
.st-pct { color: var(--text-secondary); font-size: 11px; margin-left: 3px; }

/* Bars */
.st-bars { display: flex; flex-direction: column; gap: 5px; }
.st-bar-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: var(--font-size-sm);
}
.st-bar-label {
  width: 36px;
  text-align: right;
  color: var(--text-secondary);
  flex-shrink: 0;
  font-size: 12px;
}
.st-bar-label-hour { width: 22px; }
.st-bar-label-month { width: 56px; font-size: 11px; }
.st-bar-track {
  flex: 1;
  height: 10px;
  background: var(--border);
  border-radius: 3px;
  overflow: hidden;
  min-width: 80px;
}
.st-bar-fill {
  height: 100%;
  background: color-mix(in srgb, var(--accent) 50%, transparent);
  border-radius: 3px;
  transition: width 0.3s ease;
  min-width: 2px;
}
.st-bar-count {
  width: 50px;
  text-align: right;
  color: var(--text-primary);
  flex-shrink: 0;
  font-size: 12px;
}
.st-bar-pct {
  width: 46px;
  color: var(--text-secondary);
  flex-shrink: 0;
  font-size: 11px;
}
.st-empty {
  color: var(--text-muted);
  font-size: var(--font-size-sm);
}

.st-bars-hour .st-bar-row { gap: 7px; }

/* Heatmap title row */
.st-section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-subtle);
}
.st-section-title-row .st-section-title {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}
.st-section-title-row .st-hm-legend {
  margin-top: 0;
}

/* Heatmap */
.st-heatmap-scroll {
  overflow-x: auto;
  padding-bottom: 4px;
}
.st-hm-top {
  display: flex;
  align-items: flex-end;
  margin-bottom: 3px;
}
.st-hm-day-spacer {
  width: 26px;
  flex-shrink: 0;
}
.st-hm-month-row {
  display: grid;
  gap: 2px;
}
.st-hm-month-cell {
  width: 13px;
  font-size: 10px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: visible;
  line-height: 1;
}
.st-hm-body {
  display: flex;
  gap: 4px;
}
.st-hm-day-labels {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 22px;
  flex-shrink: 0;
}
.st-hm-day-label {
  height: 13px;
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 13px;
  text-align: right;
}
.st-hm-cells {
  display: grid;
  grid-template-rows: repeat(7, 13px);
  grid-auto-flow: column;
  grid-auto-columns: 13px;
  gap: 2px;
}
.st-hm-cell {
  width: 13px;
  height: 13px;
  border-radius: 2px;
}
.st-hm-cell.lv-0 { background: var(--border); }
.st-hm-cell.lv-1 { background: color-mix(in srgb, var(--accent) 30%, var(--border)); }
.st-hm-cell.lv-2 { background: color-mix(in srgb, var(--accent) 55%, transparent); }
.st-hm-cell.lv-3 { background: color-mix(in srgb, var(--accent) 78%, transparent); }
.st-hm-cell.lv-4 { background: var(--accent); }
.st-hm-cell.future { opacity: 0.2; }
.st-hm-legend {
  display: flex;
  align-items: center;
  gap: 3px;
  margin-top: 6px;
  justify-content: flex-end;
  font-size: 11px;
  color: var(--text-secondary);
}
.st-hm-legend .st-hm-cell { width: 11px; height: 11px; }
</style>
