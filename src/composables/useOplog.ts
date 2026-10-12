import { computed, ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";

export interface OplogEntry {
  id: number;
  at: number;
  kind: string;
  label: string;
  undone: boolean;
  target_id?: number | null;
  before?: { head: string; head_sha: string; refs: [string, string][]; files?: unknown[] };
  after?: { head: string; head_sha: string; refs: [string, string][]; files?: unknown[] } | null;
}
const entries = ref<OplogEntry[]>([]);
const loading = ref(false);
let activeRepo: string | null = null;

export function useOplog() {
  const { repoPath, onRepoOpened } = useRepo();
  async function refresh() {
    const path = repoPath.value;
    if (!path) { entries.value = []; activeRepo = null; return; }
    activeRepo = path; loading.value = true;
    try { entries.value = await invoke<OplogEntry[]>("get_oplog", { repoPath: path }); }
    finally { loading.value = false; }
  }
  async function undo(id: number, confirmation?: string) {
    if (!repoPath.value) return;
    await invoke("do_undo", { repoPath: repoPath.value, id, confirmation: confirmation ?? null });
    await refresh();
  }
  async function redo(confirmation?: string) {
    if (!repoPath.value) return;
    await invoke("do_redo", { repoPath: repoPath.value, confirmation: confirmation ?? null });
    await refresh();
  }
  async function previewUndo(id: number) {
    if (!repoPath.value) return null;
    return invoke<{ token: string; changed_files: string[]; needs_confirmation: boolean }>("preview_undo", { repoPath: repoPath.value, id });
  }
  onRepoOpened(async () => { entries.value = []; activeRepo = null; await refresh(); });
  watch(repoPath, (path) => { if (path !== activeRepo) { entries.value = []; activeRepo = path; } });
  const canUndo = computed(() => entries.value.some((entry) => !entry.undone && entry.kind !== "undo"));
  const canRedo = computed(() => entries.value[0]?.kind === "undo" && !entries.value[0].undone);
  return { entries, loading, canUndo, canRedo, refresh, undo, redo, previewUndo };
}
