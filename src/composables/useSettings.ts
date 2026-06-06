import { ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";
import { useI18n, type Lang } from "@/composables/useI18n";

interface AppSettings {
  network_timeout_secs: number;
  workbench_font_family: string;
  workbench_font_size: number;
  editor_font_family: string;
  editor_font_size: number;
  language: string;
  files_tree_view: boolean;
}

const TIMEOUT_OPTIONS = [5, 10, 20, 30, 60];
const DEFAULT_TIMEOUT = 10;
const DEFAULT_WB_FONT_FAMILY = "Ubuntu";
const DEFAULT_WB_FONT_SIZE = 15;
const DEFAULT_ED_FONT_FAMILY = "Ubuntu Mono";
const DEFAULT_ED_FONT_SIZE = 13;

// Берём только первое имя из CSS font-family строки — для совпадения с именами в <select>
function primaryFont(fontFamily: string): string {
  return fontFamily.split(",")[0].trim().replace(/['"]/g, "") || fontFamily;
}

const networkTimeoutSecs = ref<number>(DEFAULT_TIMEOUT);
const workbenchFontFamily = ref<string>(DEFAULT_WB_FONT_FAMILY);
const workbenchFontSize = ref<number>(DEFAULT_WB_FONT_SIZE);
const editorFontFamily = ref<string>(DEFAULT_ED_FONT_FAMILY);
const editorFontSize = ref<number>(DEFAULT_ED_FONT_SIZE);
const filesTreeView = ref<boolean>(false);

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
  const wb = workbenchFontFamily.value || DEFAULT_WB_FONT_FAMILY;
  const ed = editorFontFamily.value || DEFAULT_ED_FONT_FAMILY;
  root.setProperty("--font-sans", `${wb}, sans-serif`);
  const n = workbenchFontSize.value;
  root.setProperty("--font-size", `${n}px`);
  root.setProperty("--font-size-sm", `${Math.max(n - 2, 11)}px`);
  root.setProperty("--font-size-xs", `${Math.max(n - 3, 10)}px`);
  root.setProperty("--font-mono", `${ed}, monospace`);
  root.setProperty("--font-size-diff", `${editorFontSize.value}px`);
}

function scheduleSave() {
  const { locale } = useI18n();
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    invoke("set_settings", {
      settings: {
        network_timeout_secs: networkTimeoutSecs.value,
        workbench_font_family: workbenchFontFamily.value,
        workbench_font_size: workbenchFontSize.value,
        editor_font_family: editorFontFamily.value,
        editor_font_size: editorFontSize.value,
        language: locale.value,
        files_tree_view: filesTreeView.value,
      },
    }).catch(() => {});
  }, 400);
}

async function loadSettings() {
  if (loaded) return;
  loaded = true;
  const { setLocale } = useI18n();
  try {
    const s = await invoke<AppSettings>("get_settings");
    networkTimeoutSecs.value = clampTimeout(s.network_timeout_secs);
    workbenchFontFamily.value = primaryFont(s.workbench_font_family || DEFAULT_WB_FONT_FAMILY);
    workbenchFontSize.value = clampFontSize(s.workbench_font_size, 11, 20, DEFAULT_WB_FONT_SIZE);
    editorFontFamily.value = primaryFont(s.editor_font_family || DEFAULT_ED_FONT_FAMILY);
    editorFontSize.value = clampFontSize(s.editor_font_size, 10, 24, DEFAULT_ED_FONT_SIZE);
    filesTreeView.value = !!s.files_tree_view;
    if (s.language === "ru" || s.language === "en") {
      setLocale(s.language as Lang);
    }
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

// Дерево папок — простой флаг, без clamp/css-эффектов: сохраняем при изменении.
watch(filesTreeView, () => scheduleSave());

void loadSettings();

export function useSettings() {
  return {
    networkTimeoutSecs,
    workbenchFontFamily,
    workbenchFontSize,
    editorFontFamily,
    editorFontSize,
    filesTreeView,
    scheduleSave,
  };
}
