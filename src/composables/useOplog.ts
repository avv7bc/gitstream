import { computed, ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";

export interface OplogEntry {
  id: number;
  at: number;
  kind: string;
  label: string;
  undone: boolean;
  eligible?: boolean;
  target_id?: number | null;
  before?: { head: string; head_sha: string; refs: [string, string][]; files?: unknown[] };
  after?: { head: string; head_sha: string; refs: [string, string][]; files?: unknown[] } | null;
}
const entries = ref<OplogEntry[]>([]);
const loading = ref(false);
let activeRepo: string | null = null;
let refreshGeneration = 0;

export function useOplog() {
  const { repoPath, onRepoOpened } = useRepo();
  async function refresh() {
    const path = repoPath.value;
    const generation = ++refreshGeneration;
    if (!path) { entries.value = []; activeRepo = null; loading.value = false; return; }
    activeRepo = path; loading.value = true;
    try {
      const next = await invoke<OplogEntry[]>("get_oplog", { repoPath: path });
      // A slow response from an old repository must never overwrite the newly
      // selected repository's history.
      if (generation === refreshGeneration && repoPath.value === path && activeRepo === path) entries.value = next;
    } finally {
      if (generation === refreshGeneration) loading.value = false;
    }
  }
  async function undo(id: number, confirmation?: string) {
    if (!repoPath.value) return;
    const path = repoPath.value;
    await invoke("do_undo", { repoPath: path, id, confirmation: confirmation ?? null });
    if (repoPath.value === path) await refresh();
  }
  async function redo(confirmation?: string) {
    if (!repoPath.value) return;
    const path = repoPath.value;
    await invoke("do_redo", { repoPath: path, confirmation: confirmation ?? null });
    if (repoPath.value === path) await refresh();
  }
  async function previewUndo(id: number) {
    if (!repoPath.value) return null;
    return invoke<{ token: string; changed_files: string[]; needs_confirmation: boolean }>("preview_undo", { repoPath: repoPath.value, id });
  }
  onRepoOpened(async () => { entries.value = []; activeRepo = null; await refresh(); });
  watch(repoPath, (path) => { if (path !== activeRepo) { entries.value = []; activeRepo = path; } });
  const canUndo = computed(() => entries.value.some((entry) => entry.eligible !== false && !entry.undone && entry.kind !== "undo"));
  const canRedo = computed(() => entries.value.some((entry) => entry.eligible !== false && entry.kind === "undo" && !entry.undone));
  return { entries, loading, canUndo, canRedo, refresh, undo, redo, previewUndo };
}
