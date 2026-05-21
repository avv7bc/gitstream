import { ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";

interface AppSettings {
  network_timeout_secs: number;
  workbench_font_family: string;
  workbench_font_size: number;
  editor_font_family: string;
  editor_font_size: number;
}

const TIMEOUT_OPTIONS = [5, 10, 30, 60];
const DEFAULT_TIMEOUT = 10;
const DEFAULT_WB_FONT_FAMILY = "Ubuntu, -apple-system, BlinkMacSystemFont, sans-serif";
const DEFAULT_WB_FONT_SIZE = 15;
const DEFAULT_ED_FONT_FAMILY = "Ubuntu Mono, Courier New, monospace";
const DEFAULT_ED_FONT_SIZE = 13;

const networkTimeoutSecs = ref<number>(DEFAULT_TIMEOUT);
const workbenchFontFamily = ref<string>(DEFAULT_WB_FONT_FAMILY);
const workbenchFontSize = ref<number>(DEFAULT_WB_FONT_SIZE);
const editorFontFamily = ref<string>(DEFAULT_ED_FONT_FAMILY);
const editorFontSize = ref<number>(DEFAULT_ED_FONT_SIZE);

let loaded = false;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

function clampTimeout(v: number): number {
  if (!Number.isFinite(v)) return DEFAULT_TIMEOUT;
  if (TIMEOUT_OPTIONS.includes(v)) return v;
  return TIMEOUT_OPTIONS.reduce((best, o) =>
    Math.abs(o - v) < Math.abs(best - v) ? o : best
  );
}

function clampFontSize(v: number, min: number, max: number, def: number): number {
  if (!Number.isFinite(v) || v < min || v > max) return def;
  return Math.round(v);
}

function applyCssFonts() {
  const root = document.documentElement.style;
  root.setProperty("--font-sans", workbenchFontFamily.value || DEFAULT_WB_FONT_FAMILY);
  const n = workbenchFontSize.value;
  root.setProperty("--font-size", `${n}px`);
  root.setProperty("--font-size-sm", `${n - 2}px`);
  root.setProperty("--font-size-xs", `${n - 3}px`);
  root.setProperty("--font-mono", editorFontFamily.value || DEFAULT_ED_FONT_FAMILY);
  root.setProperty("--font-size-diff", `${editorFontSize.value}px`);
}

function scheduleSave() {
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    invoke("set_settings", {
      settings: {
        network_timeout_secs: networkTimeoutSecs.value,
        workbench_font_family: workbenchFontFamily.value,
        workbench_font_size: workbenchFontSize.value,
        editor_font_family: editorFontFamily.value,
        editor_font_size: editorFontSize.value,
      },
    }).catch(() => {});
  }, 400);
}

async function loadSettings() {
  if (loaded) return;
  loaded = true;
  try {
    const s = await invoke<AppSettings>("get_settings");
    networkTimeoutSecs.value = clampTimeout(s.network_timeout_secs);
    workbenchFontFamily.value = s.workbench_font_family || DEFAULT_WB_FONT_FAMILY;
    workbenchFontSize.value = clampFontSize(s.workbench_font_size, 11, 20, DEFAULT_WB_FONT_SIZE);
    editorFontFamily.value = s.editor_font_family || DEFAULT_ED_FONT_FAMILY;
    editorFontSize.value = clampFontSize(s.editor_font_size, 10, 24, DEFAULT_ED_FONT_SIZE);
  } catch {
    // defaults already set
  }
  applyCssFonts();
}

watch(networkTimeoutSecs, (v) => {
  const safe = clampTimeout(v);
  if (safe !== v) { networkTimeoutSecs.value = safe; return; }
  scheduleSave();
});

watch(workbenchFontFamily, () => { applyCssFonts(); scheduleSave(); });
watch(workbenchFontSize, (v) => {
  const safe = clampFontSize(v, 11, 20, DEFAULT_WB_FONT_SIZE);
  if (safe !== v) { workbenchFontSize.value = safe; return; }
  applyCssFonts(); scheduleSave();
});
watch(editorFontFamily, () => { applyCssFonts(); scheduleSave(); });
watch(editorFontSize, (v) => {
  const safe = clampFontSize(v, 10, 24, DEFAULT_ED_FONT_SIZE);
  if (safe !== v) { editorFontSize.value = safe; return; }
  applyCssFonts(); scheduleSave();
});

void loadSettings();

export function useSettings() {
  return {
    networkTimeoutSecs,
    workbenchFontFamily,
    workbenchFontSize,
    editorFontFamily,
    editorFontSize,
  };
}
