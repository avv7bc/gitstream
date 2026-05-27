import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, ref } from "vue";

const THRESHOLD_MS = 100;
const MAX_LOG_LINES = 200;

type Args = Record<string, unknown> | undefined;

function remoteOf(args: Args): string {
  const r = args?.remote;
  return typeof r === "string" && r ? r : "";
}

function withRemote(prefix: string, args: Args): string {
  const r = remoteOf(args);
  return r ? `${prefix} ${r}…` : `${prefix}…`;
}

const COMMAND_LABEL_BUILDERS: Record<string, (args: Args) => string> = {
  do_fetch: (a) => withRemote("Fetch from", a),
  do_pull: (a) => withRemote("Pull from", a),
  do_push: (a) => withRemote("Push to", a),
  do_push_branch: (a) => withRemote("Push to", a),
  do_push_tag: (a) => withRemote("Push tag to", a),
  get_status: () => "File status…",
  get_log: () => "Loading log…",
  stage_files: () => "Stage…",
  unstage_files: () => "Unstage…",
  discard_files: () => "Discard…",
};

const FALLBACK_LABEL = "Working…";

interface ActiveOp {
  cmd: string;
  label: string;
  startedAt: number;
  timeoutSecs?: number;
}

const active = ref(new Map<number, ActiveOp>());
let seq = 0;

const tick = ref(0);
let tickTimer: ReturnType<typeof setInterval> | null = null;

function ensureTicker() {
  if (tickTimer) return;
  tickTimer = setInterval(() => {
    if (active.value.size === 0) {
      if (tickTimer) { clearInterval(tickTimer); tickTimer = null; }
      return;
    }
    tick.value = (tick.value + 1) | 0;
  }, 1000);
}

const networkProgressLine = ref("");
export const networkProgressLog = ref<string[]>([]);
export const logOpen = ref(false);
let networkProgressTimer: ReturnType<typeof setTimeout> | null = null;

function timestamp(): string {
  const d = new Date();
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  return `[${hh}:${mm}:${ss}]`;
}

function appendLog(...lines: string[]) {
  let log = networkProgressLog.value;
  for (const line of lines) {
    log = [...log.slice(-(MAX_LOG_LINES - 1)), line];
  }
  networkProgressLog.value = log;
}

export function toggleLog() {
  logOpen.value = !logOpen.value;
}

export function closeLog() {
  logOpen.value = false;
}

listen<{ op: string; line: string }>("network_progress", (event) => {
  const line = event.payload.line;
  networkProgressLine.value = line;
  appendLog(`${timestamp()} ${line}`);
  if (networkProgressTimer) clearTimeout(networkProgressTimer);
  networkProgressTimer = setTimeout(() => { networkProgressLine.value = ""; }, 3000);
});

listen<{ cmd: string; output: string; success: boolean }>("git_command", (event) => {
  const { cmd, output } = event.payload;
  if (output) {
    appendLog(`${timestamp()} ${cmd}`, output);
  } else {
    appendLog(`${timestamp()} ${cmd}`);
  }
});

export const isWorking = computed(() => active.value.size > 0);

function formatCountdown(op: ActiveOp): string {
  if (!op.timeoutSecs) return op.label;
  void tick.value;
  const elapsed = Math.min(op.timeoutSecs, Math.floor((Date.now() - op.startedAt) / 1000));
  return `${op.label} ${elapsed}/${op.timeoutSecs}`;
}

export const progressLabel = computed(() => {
  if (networkProgressLine.value) return networkProgressLine.value;
  const size = active.value.size;
  if (size === 0) return "";
  if (size > 1) return `Operations: ${size}`;
  const first = active.value.values().next().value as ActiveOp | undefined;
  if (!first) return FALLBACK_LABEL;
  return formatCountdown(first);
});

export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const id = ++seq;
  const builder = COMMAND_LABEL_BUILDERS[cmd];
  const label = builder ? builder(args) : FALLBACK_LABEL;
  const timeoutRaw = args?.timeoutSecs;
  const timeoutSecs = typeof timeoutRaw === "number" && timeoutRaw > 0 ? timeoutRaw : undefined;

  const timer = setTimeout(() => {
    const next = new Map(active.value);
    next.set(id, { cmd, label, startedAt: Date.now(), timeoutSecs });
    active.value = next;
    if (timeoutSecs) ensureTicker();
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
  return { isWorking, progressLabel, networkProgressLog };
}
