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
  do_fetch_all: () => "Fetch all remotes…",
  do_pull: (a) => withRemote("Pull from", a),
  do_push: (a) => withRemote("Push to", a),
  do_push_branch: (a) => withRemote("Push to", a),
  do_push_tag: (a) => withRemote("Push tag to", a),
  do_push_force_lease: (a) => withRemote("Force push to", a),
  do_delete_remote_branch: (a) => withRemote("Push to", a),
  get_status: () => "File status…",
  get_log: () => "Loading log…",
  stage_files: () => "Stage…",
  unstage_files: () => "Unstage…",
  discard_files: () => "Discard…",
};

const FALLBACK_LABEL = "Working…";

// Сетевые команды с направлением: push — исходящая (стрелка вверх),
// pull/fetch — входящие (стрелка вниз). Такие операции привязываются к
// репозиторию и показываются крутилкой на его узле в панели Repositories.
export type NetworkOpKind = "push" | "pull" | "fetch";

const NETWORK_OP_KIND: Record<string, NetworkOpKind> = {
  do_push: "push",
  do_push_branch: "push",
  do_push_tag: "push",
  do_push_force_lease: "push",
  do_delete_remote_branch: "push",
  do_pull: "pull",
  do_fetch: "fetch",
  do_fetch_all: "fetch",
};

interface ActiveOp {
  cmd: string;
  label: string;
  startedAt: number;
  timeoutSecs?: number;
  kind?: NetworkOpKind;
  repoPath?: string;
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
// Строка лога Git output: текст + признак ошибки (для красной подсветки).
export interface LogLine {
  text: string;
  isError?: boolean;
}
export const networkProgressLog = ref<LogLine[]>([]);
export const logOpen = ref(false);
let networkProgressTimer: ReturnType<typeof setTimeout> | null = null;

function timestamp(): string {
  const d = new Date();
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  return `[${hh}:${mm}:${ss}]`;
}

function pushLines(items: LogLine[]) {
  let log = networkProgressLog.value;
  for (const item of items) {
    log = [...log.slice(-(MAX_LOG_LINES - 1)), item];
  }
  networkProgressLog.value = log;
}

function appendLog(...lines: string[]) {
  pushLines(lines.map((text) => ({ text })));
}

// Записать ошибку в Git output (красным) и открыть панель — единый канал
// для всех ошибок git-операций вместо модальных окон.
export function logError(message: string, detail?: string[]) {
  const items: LogLine[] = [{ text: `${timestamp()} ${message}`, isError: true }];
  if (detail) for (const d of detail) if (d) items.push({ text: d, isError: true });
  pushLines(items);
  logOpen.value = true;
}

export function toggleLog() {
  logOpen.value = !logOpen.value;
}

// Показать вкладку Git output в нижней панели.
export function showOutput() {
  logOpen.value = true;
}

// Вернуться на вкладку Changes (Git output скрыт, данные лога сохраняются).
export function closeLog() {
  logOpen.value = false;
}

// Очистить лог Git output.
export function clearLog() {
  networkProgressLog.value = [];
}

listen<{ op: string; line: string }>("network_progress", (event) => {
  const line = event.payload.line;
  networkProgressLine.value = line;
  appendLog(`${timestamp()} ${line}`);
  if (networkProgressTimer) clearTimeout(networkProgressTimer);
  networkProgressTimer = setTimeout(() => { networkProgressLine.value = ""; }, 3000);
});

listen<{ cmd: string; output: string; success: boolean }>("git_command", (event) => {
  const { cmd, output, success } = event.payload;
  const isError = success === false;
  const items: LogLine[] = [{ text: `${timestamp()} ${cmd}`, isError }];
  if (output) items.push({ text: output, isError });
  pushLines(items);
});

// Операции без привязки к репозиторию — показываются в статус-баре.
// Сетевые операции с repoPath выводятся крутилкой на узле репозитория.
const statusBarOps = computed(() =>
  [...active.value.values()].filter((op) => !(op.kind && op.repoPath)),
);

export const isWorking = computed(() => statusBarOps.value.length > 0);

function formatCountdown(op: ActiveOp): string {
  if (!op.timeoutSecs) return op.label;
  void tick.value;
  const elapsed = Math.min(op.timeoutSecs, Math.floor((Date.now() - op.startedAt) / 1000));
  return `${op.label} ${elapsed}/${op.timeoutSecs}`;
}

export const progressLabel = computed(() => {
  if (networkProgressLine.value) return networkProgressLine.value;
  const ops = statusBarOps.value;
  if (ops.length === 0) return "";
  if (ops.length > 1) return `Operations: ${ops.length}`;
  return formatCountdown(ops[0]);
});

export interface RepoNetworkOp {
  kind: NetworkOpKind;
  label: string;
}

// Активная сетевая операция по пути репозитория: kind — направление стрелки,
// label — текст с обратным отсчётом для тултипа.
export const repoNetworkOps = computed(() => {
  const map = new Map<string, RepoNetworkOp>();
  for (const op of active.value.values()) {
    if (op.kind && op.repoPath && !map.has(op.repoPath)) {
      map.set(op.repoPath, { kind: op.kind, label: formatCountdown(op) });
    }
  }
  return map;
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
  const kind = NETWORK_OP_KIND[cmd];
  const pathRaw = args?.repoPath;
  const repoPath = typeof pathRaw === "string" && pathRaw ? pathRaw : undefined;

  const timer = setTimeout(() => {
    const next = new Map(active.value);
    next.set(id, { cmd, label, startedAt: Date.now(), timeoutSecs, kind, repoPath });
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
