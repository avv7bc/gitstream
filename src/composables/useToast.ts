import { ref } from "vue";

export type ToastKind = "info" | "success" | "warning" | "error" | "progress";

export interface ToastAction { label: string; run: () => void; primary?: boolean; }
export interface ToastProgress { current: number; total?: number; cancel?: () => void; }
export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  detail?: string;
  actions?: ToastAction[];
  timeout?: number | null;
  progress?: ToastProgress;
  repoPath?: string;
}

export const toasts = ref<Toast[]>([]);
let nextId = 1;
const timers = new Map<number, ReturnType<typeof setTimeout>>();

function defaultTimeout(kind: ToastKind): number | null {
  if (kind === "warning") return 8000;
  if (kind === "error" || kind === "progress") return null;
  return 4000;
}

export function toast(input: Omit<Toast, "id">): number;
export function toast(title: string, kind?: ToastKind, options?: { detail?: string; duration?: number }): number;
export function toast(input: Omit<Toast, "id"> | string, kind: ToastKind = "info", options: { detail?: string; duration?: number } = {}): number {
  const item: Omit<Toast, "id"> = typeof input === "string"
    ? { kind, title: input, detail: options.detail, timeout: options.duration ?? defaultTimeout(kind) }
    : { ...input, timeout: input.timeout === undefined ? defaultTimeout(input.kind) : input.timeout };
  const id = nextId++;
  toasts.value = [...toasts.value, { id, ...item }];
  if (item.timeout !== null && item.timeout !== undefined && item.timeout > 0) {
    timers.set(id, setTimeout(() => dismissToast(id), item.timeout));
  }
  return id;
}

export function updateToast(id: number, patch: Partial<Toast>): void {
  const index = toasts.value.findIndex((item) => item.id === id);
  if (index < 0) return;
  toasts.value = toasts.value.map((item, i) => i === index ? { ...item, ...patch } : item);
  if (patch.timeout !== undefined) {
    const timer = timers.get(id);
    if (timer) clearTimeout(timer);
    timers.delete(id);
    if (patch.timeout !== null && patch.timeout > 0) timers.set(id, setTimeout(() => dismissToast(id), patch.timeout));
  }
}

export function dismissToast(id: number): void {
  const timer = timers.get(id);
  if (timer) clearTimeout(timer);
  timers.delete(id);
  toasts.value = toasts.value.filter((item) => item.id !== id);
}

export function clearToasts(): void {
  for (const timer of timers.values()) clearTimeout(timer);
  timers.clear();
  toasts.value = [];
}

export function useToast() {
  return { toasts, toast, updateToast, dismissToast, clearToasts };
}
