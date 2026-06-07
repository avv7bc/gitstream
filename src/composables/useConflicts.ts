import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";

export type RepoState =
  | "clean"
  | "merging"
  | "rebasing"
  | "cherry-picking"
  | "reverting";

const repoState = ref<RepoState>("clean");

// Защита от гонки перекрывающихся refresh: применяем только самый свежий ответ.
let refreshSeq = 0;

export function useConflicts() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) {
      refreshSeq++;
      repoState.value = "clean";
      return;
    }
    const seq = ++refreshSeq;
    const state = await invoke<RepoState>("get_repo_state", {
      repoPath: repoPath.value,
    });
    if (seq !== refreshSeq) return;
    repoState.value = state;
  }

  async function acceptOurs(files: string[]) {
    if (!repoPath.value) return;
    await invoke("do_accept_ours", { repoPath: repoPath.value, files });
  }

  async function acceptTheirs(files: string[]) {
    if (!repoPath.value) return;
    await invoke("do_accept_theirs", { repoPath: repoPath.value, files });
  }

  async function abort() {
    if (!repoPath.value) return;
    await invoke("do_op_abort", {
      repoPath: repoPath.value,
      state: repoState.value,
    });
  }

  async function cont() {
    if (!repoPath.value) return;
    await invoke("do_op_continue", {
      repoPath: repoPath.value,
      state: repoState.value,
    });
  }

  return { repoState, refresh, acceptOurs, acceptTheirs, abort, cont };
}
