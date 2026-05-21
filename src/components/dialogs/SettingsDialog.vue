<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useTheme, type ThemeMode } from "@/composables/useTheme";
import { useSettings } from "@/composables/useSettings";
import { invoke } from "@/composables/useProgress";

const emit = defineEmits<{ close: [] }>();

// --- Persisted window geometry ---
const GEOM_KEY = "gitstream-settings-geom";

interface Geom {
  x: number;
  y: number;
  w: number;
  h: number;
}

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
  const w = Math.min(1200, Math.round(window.innerWidth * 0.85));
  const h = Math.min(800, Math.round(window.innerHeight * 0.85));
  return {
    x: Math.round((window.innerWidth - w) / 2),
    y: Math.round((window.innerHeight - h) / 2),
    w,
    h,
  };
}

function clampGeom(g: Geom): Geom {
  const w = Math.max(480, Math.min(g.w, window.innerWidth));
  const h = Math.max(320, Math.min(g.h, window.innerHeight));
  const x = Math.max(0, Math.min(g.x, window.innerWidth - w));
  const y = Math.max(0, Math.min(g.y, window.innerHeight - h));
  return { x, y, w, h };
}

const geom = ref<Geom>(loadGeom());

function saveGeom() {
  localStorage.setItem(GEOM_KEY, JSON.stringify(geom.value));
}

// Drag
let dragStart: { x: number; y: number; gx: number; gy: number } | null = null;
function onTitleMouseDown(e: MouseEvent) {
  if ((e.target as HTMLElement).closest(".vs-icon-btn")) return;
  dragStart = { x: e.clientX, y: e.clientY, gx: geom.value.x, gy: geom.value.y };
  document.addEventListener("mousemove", onDragMove);
  document.addEventListener("mouseup", onDragEnd);
  e.preventDefault();
}
function onDragMove(e: MouseEvent) {
  if (!dragStart) return;
  const nx = dragStart.gx + (e.clientX - dragStart.x);
  const ny = dragStart.gy + (e.clientY - dragStart.y);
  geom.value = clampGeom({ ...geom.value, x: nx, y: ny });
}
function onDragEnd() {
  dragStart = null;
  document.removeEventListener("mousemove", onDragMove);
  document.removeEventListener("mouseup", onDragEnd);
  saveGeom();
}

// Resize (multiple edges)
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
  if (resizeStart.dir.includes("e")) w = resizeStart.g.w + dx;
  if (resizeStart.dir.includes("s")) h = resizeStart.g.h + dy;
  if (resizeStart.dir.includes("w")) {
    w = resizeStart.g.w - dx;
    x = resizeStart.g.x + dx;
  }
  if (resizeStart.dir.includes("n")) {
    h = resizeStart.g.h - dy;
    y = resizeStart.g.y + dy;
  }
  geom.value = clampGeom({ x, y, w, h });
}
function onResizeEnd() {
  resizeStart = null;
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
  saveGeom();
}

function onWindowResize() {
  geom.value = clampGeom(geom.value);
}
onMounted(() => {
  window.addEventListener("resize", onWindowResize);
  invoke<string[]>("list_system_fonts").then((fonts) => { systemFonts.value = fonts; }).catch(() => {});
});
onUnmounted(() => {
  window.removeEventListener("resize", onWindowResize);
  document.removeEventListener("mousemove", onDragMove);
  document.removeEventListener("mouseup", onDragEnd);
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
});

const { mode } = useTheme();
const { networkTimeoutSecs, workbenchFontFamily, workbenchFontSize, editorFontFamily, editorFontSize } = useSettings();

const systemFonts = ref<string[]>([]);

const timeoutOptions = [5, 10, 30, 60];
const wbFontSizes = Array.from({ length: 10 }, (_, i) => i + 11); // 11..20
const edFontSizes = Array.from({ length: 15 }, (_, i) => i + 10); // 10..24

const themes: { value: ThemeMode; label: string }[] = [
  { value: "system", label: "System (как в ОС)" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark+ (VSCode)" },
  { value: "material", label: "Material" },
  { value: "smartgit", label: "SmartGit" },
  { value: "catppuccin", label: "Catppuccin Mocha" },
  { value: "dracula", label: "Dracula" },
  { value: "sublime", label: "Sublime Text 4 (Monokai Pro)" },
  { value: "github", label: "GitHub Dark" },
];

const activeCategory = ref("appearance");
const search = ref("");

const categories = [
  { id: "appearance", label: "Внешний вид" },
  { id: "network", label: "Сеть" },
];

interface SettingItem {
  id: string;
  category: string;
  label: string;
  description: string;
}

const settings: SettingItem[] = [
  {
    id: "color-theme",
    category: "appearance",
    label: "Workbench: Color Theme",
    description: "Задаёт цветовую тему интерфейса. Тема «System» подстраивается под настройки операционной системы.",
  },
  {
    id: "workbench-font-family",
    category: "appearance",
    label: "Workbench: Font Family",
    description: "Шрифт интерфейса: панели, тулбар, диалоги. Задаётся как CSS font-family. Пример: «Segoe UI, sans-serif».",
  },
  {
    id: "workbench-font-size",
    category: "appearance",
    label: "Workbench: Font Size",
    description: "Базовый размер шрифта интерфейса в пикселях (11–20). Пропорционально масштабирует все уровни шрифта UI.",
  },
  {
    id: "editor-font-family",
    category: "appearance",
    label: "Editor: Font Family",
    description: "Шрифт diff-вьюера. Рекомендуется моноширинный. Пример: «Fira Code, monospace».",
  },
  {
    id: "editor-font-size",
    category: "appearance",
    label: "Editor: Font Size",
    description: "Размер шрифта diff-вьюера в пикселях (10–24).",
  },
  {
    id: "network-timeout",
    category: "network",
    label: "Network: Timeout (сек)",
    description:
      "Максимальное время сетевой git-операции (fetch, pull, push, clone). По истечении операция прерывается, а зависший процесс git принудительно завершается.",
  },
];

const filteredSettings = computed(() => {
  const q = search.value.trim().toLowerCase();
  return settings.filter((s) => {
    if (s.category !== activeCategory.value && q === "") return s.category === activeCategory.value;
    if (q) {
      const hay = `${s.label} ${s.description}`.toLowerCase();
      return hay.includes(q);
    }
    return s.category === activeCategory.value;
  });
});

const activeCategoryLabel = computed(
  () => categories.find((c) => c.id === activeCategory.value)?.label ?? ""
);
</script>

<template>
  <div class="modal-overlay" @mousedown.self="emit('close')">
    <div
      class="vs-settings"
      :style="{
        left: geom.x + 'px',
        top: geom.y + 'px',
        width: geom.w + 'px',
        height: geom.h + 'px',
      }"
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
      <div class="vs-titlebar" @mousedown="onTitleMouseDown">
        <div class="vs-title">
          <span>Параметры</span>
        </div>
        <div class="vs-title-actions">
          <button class="vs-icon-btn close" @click="emit('close')" title="Close">
            <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
          </button>
        </div>
      </div>

      <!-- Search -->
      <div class="vs-search-row">
        <div class="vs-search">
          <input
            v-model="search"
            type="text"
            placeholder="Параметры поиска"
          />
          <div class="vs-search-actions">
            <button class="vs-icon-btn sm" tabindex="-1" title="Clear filter">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M3 3l10 10M13 3L3 13"/></svg>
            </button>
            <button class="vs-icon-btn sm" tabindex="-1" title="Filter">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M2 3h12l-5 6v5l-2-1V9z"/></svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Body -->
      <div class="vs-body">
        <!-- Sidebar categories -->
        <aside class="vs-sidebar">
          <div
            v-for="cat in categories"
            :key="cat.id"
            class="vs-cat"
            :class="{ active: activeCategory === cat.id }"
            @click="activeCategory = cat.id"
          >
            <svg class="chev" width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M5 3l6 5-6 5z"/></svg>
            <span>{{ cat.label }}</span>
          </div>
        </aside>

        <!-- Settings content -->
        <main class="vs-content">
          <h2 class="vs-section-title">{{ activeCategoryLabel }}</h2>

          <div v-if="filteredSettings.length === 0" class="vs-empty">
            Параметры в этой категории отсутствуют.
          </div>

          <div
            v-for="s in filteredSettings"
            :key="s.id"
            class="vs-setting"
          >
            <div class="vs-setting-header">
              <span class="vs-setting-label">{{ s.label }}</span>
            </div>
            <div class="vs-setting-desc">{{ s.description }}</div>

            <div v-if="s.id === 'color-theme'" class="vs-setting-control">
              <select v-model="mode" class="vs-select">
                <option v-for="t in themes" :key="t.value" :value="t.value">{{ t.label }}</option>
              </select>
            </div>
            <div v-if="s.id === 'workbench-font-family'" class="vs-setting-control">
              <select v-model="workbenchFontFamily" class="vs-select">
                <option v-for="f in systemFonts" :key="f" :value="f">{{ f }}</option>
              </select>
            </div>
            <div v-if="s.id === 'workbench-font-size'" class="vs-setting-control">
              <select v-model.number="workbenchFontSize" class="vs-select vs-select-narrow">
                <option v-for="n in wbFontSizes" :key="n" :value="n">{{ n }} px</option>
              </select>
            </div>
            <div v-if="s.id === 'editor-font-family'" class="vs-setting-control">
              <select v-model="editorFontFamily" class="vs-select">
                <option v-for="f in systemFonts" :key="f" :value="f">{{ f }}</option>
              </select>
            </div>
            <div v-if="s.id === 'editor-font-size'" class="vs-setting-control">
              <select v-model.number="editorFontSize" class="vs-select vs-select-narrow">
                <option v-for="n in edFontSizes" :key="n" :value="n">{{ n }} px</option>
              </select>
            </div>
            <div v-if="s.id === 'network-timeout'" class="vs-setting-control">
              <select v-model.number="networkTimeoutSecs" class="vs-select">
                <option v-for="t in timeoutOptions" :key="t" :value="t">{{ t }} сек</option>
              </select>
            </div>
          </div>
        </main>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 150;
}

.vs-settings {
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
  min-width: 480px;
  min-height: 320px;
}

/* Resize handles */
.rh {
  position: absolute;
  z-index: 10;
}
.rh-n { top: 0; left: 6px; right: 6px; height: 4px; cursor: ns-resize; }
.rh-s { bottom: 0; left: 6px; right: 6px; height: 4px; cursor: ns-resize; }
.rh-e { top: 6px; bottom: 6px; right: 0; width: 4px; cursor: ew-resize; }
.rh-w { top: 6px; bottom: 6px; left: 0; width: 4px; cursor: ew-resize; }
.rh-ne { top: 0; right: 0; width: 8px; height: 8px; cursor: nesw-resize; }
.rh-nw { top: 0; left: 0; width: 8px; height: 8px; cursor: nwse-resize; }
.rh-se { bottom: 0; right: 0; width: 10px; height: 10px; cursor: nwse-resize; }
.rh-sw { bottom: 0; left: 0; width: 8px; height: 8px; cursor: nesw-resize; }

/* Title bar */
.vs-titlebar {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 6px 0 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
  cursor: move;
}
.vs-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  flex: 1;
}
.vs-title-actions {
  display: flex;
  gap: 0;
}
.vs-icon-btn {
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
}
.vs-icon-btn.sm {
  width: 22px;
  height: 22px;
}
.vs-icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.vs-icon-btn.close:hover {
  background: var(--red);
  color: #fff;
}

/* Search row */
.vs-search-row {
  padding: 12px 16px 8px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-subtle);
}
.vs-search {
  position: relative;
  display: flex;
  align-items: center;
  background: var(--bg-tertiary);
  border: 1px solid color-mix(in srgb, var(--border-subtle) 45%, transparent);
  border-radius: var(--radius);
  padding: 0 4px 0 0;
}
.vs-search input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-primary);
  padding: 6px 10px;
  font-size: var(--font-size-sm);
  outline: none;
}
.vs-search-actions {
  display: flex;
  gap: 2px;
  padding-right: 4px;
}

/* Body */
.vs-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

/* Sidebar */
.vs-sidebar {
  width: 240px;
  flex-shrink: 0;
  background: var(--bg-primary);
  border-right: 1px solid var(--border-subtle);
  overflow-y: auto;
  padding: 8px 0;
}
.vs-cat {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px 5px 14px;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  cursor: pointer;
  user-select: none;
}
.vs-cat .chev {
  color: var(--text-muted);
  flex-shrink: 0;
}
.vs-cat:hover {
  background: var(--bg-hover);
}
.vs-cat.active {
  background: var(--bg-active);
  color: var(--text-primary);
  font-weight: 600;
}

/* Content */
.vs-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 40px;
}
.vs-section-title {
  font-size: 22px;
  font-weight: 400;
  color: var(--text-primary);
  margin-bottom: 24px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-subtle);
}
.vs-empty {
  color: var(--text-muted);
  font-size: var(--font-size-sm);
  padding: 12px 0;
}

.vs-setting {
  margin-bottom: 28px;
  max-width: 760px;
}
.vs-setting-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}
.vs-setting-label {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
}
.vs-setting-desc {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  margin-bottom: 10px;
  line-height: 1.5;
}
.vs-setting-control {
  margin-top: 4px;
}
.vs-select {
  width: 320px;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  padding: 6px 28px 6px 10px;
  font-size: var(--font-size-sm);
  border-radius: var(--radius);
  outline: none;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'><path d='M0 0l5 6 5-6z' fill='%23a8a8a8'/></svg>");
  background-repeat: no-repeat;
  background-position: right 10px center;
  cursor: pointer;
}
:root[data-theme="smartgit"] .vs-select {
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'><path d='M0 0l5 6 5-6z' fill='%23e0e0e0'/></svg>");
}
.vs-select:focus { border-color: var(--accent); }
.vs-select option {
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
}

.vs-select-narrow {
  width: 120px;
}
</style>
