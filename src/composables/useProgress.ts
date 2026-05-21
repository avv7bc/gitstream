import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, ref } from "vue";

const THRESHOLD_MS = 100;

const COMMAND_LABELS: Record<string, string> = {
  do_fetch: "Fetch…",
  do_pull: "Pull…",
  do_push: "Push…",
  do_clone: "Клонирование…",
  get_status: "Статус файлов…",
  get_log: "Загрузка лога…",
  stage_files: "Stage…",
  unstage_files: "Unstage…",
  discard_files: "Discard…",
};

const FALLBACK_LABEL = "Работаем…";

interface ActiveOp {
  cmd: string;
  label: string;
}

const active = ref(new Map<number, ActiveOp>());
let seq = 0;

const networkProgressLine = ref("");
let networkProgressTimer: ReturnType<typeof setTimeout> | null = null;

listen<{ op: string; line: string }>("network_progress", (event) => {
  networkProgressLine.value = event.payload.line;
  if (networkProgressTimer) clearTimeout(networkProgressTimer);
  networkProgressTimer = setTimeout(() => { networkProgressLine.value = ""; }, 3000);
});

export const isWorking = computed(() => active.value.size > 0);

export const progressLabel = computed(() => {
  if (networkProgressLine.value) return networkProgressLine.value;
  const size = active.value.size;
  if (size === 0) return "";
  if (size > 1) return `Операций: ${size}`;
  const first = active.value.values().next().value as ActiveOp | undefined;
  return first?.label ?? FALLBACK_LABEL;
});

export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const id = ++seq;
  const label = COMMAND_LABELS[cmd] ?? FALLBACK_LABEL;

  const timer = setTimeout(() => {
    const next = new Map(active.value);
    next.set(id, { cmd, label });
    active.value = next;
  }, THRESHOLD_MS);

  try {
    return await tauriInvoke<T>(cmd, args);
  } finally {
    clearTimeout(timer);
    if (active.value.has(id)) {
      const next = new Map(active.value);
      next.delete(id);
      active.value = next;
    }
  }
}

export function useProgress() {
  return { isWorking, progressLabel };
}
