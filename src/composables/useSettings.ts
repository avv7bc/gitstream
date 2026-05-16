import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface AppSettings {
  network_timeout_secs: number;
}

const DEFAULT_TIMEOUT = 10;
const MIN_TIMEOUT = 1;
const MAX_TIMEOUT = 600;

const networkTimeoutSecs = ref<number>(DEFAULT_TIMEOUT);
let loaded = false;
let persistTimer: ReturnType<typeof setTimeout> | null = null;

function clamp(v: number): number {
  if (!Number.isFinite(v)) return DEFAULT_TIMEOUT;
  const n = Math.round(v);
  if (n < MIN_TIMEOUT) return MIN_TIMEOUT;
  if (n > MAX_TIMEOUT) return MAX_TIMEOUT;
  return n;
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
