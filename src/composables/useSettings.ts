import { ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";

interface AppSettings {
  network_timeout_secs: number;
}

const TIMEOUT_OPTIONS = [5, 10, 30, 60];
const DEFAULT_TIMEOUT = 10;

const networkTimeoutSecs = ref<number>(DEFAULT_TIMEOUT);
let loaded = false;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

// Значение всегда приводится к одному из допустимых вариантов combo.
function clamp(v: number): number {
  if (!Number.isFinite(v)) return DEFAULT_TIMEOUT;
  if (TIMEOUT_OPTIONS.includes(v)) return v;
  return TIMEOUT_OPTIONS.reduce((best, o) =>
    Math.abs(o - v) < Math.abs(best - v) ? o : best
  );
}

async function loadSettings() {
  if (loaded) return;
  loaded = true;
  try {
    const s = await invoke<AppSettings>("get_settings");
    networkTimeoutSecs.value = clamp(s.network_timeout_secs);
  } catch {
    networkTimeoutSecs.value = DEFAULT_TIMEOUT;
  }
}

// Дебаунс-запись, чтобы не писать файл на каждый ввод цифры.
watch(networkTimeoutSecs, (v) => {
  const safe = clamp(v);
  if (safe !== v) {
    networkTimeoutSecs.value = safe;
    return; // повторный watch с уже валидным значением запишет файл
  }
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    invoke("set_settings", { settings: { network_timeout_secs: safe } }).catch(() => {});
  }, 400);
});

void loadSettings();

export function useSettings() {
  return { networkTimeoutSecs };
}
