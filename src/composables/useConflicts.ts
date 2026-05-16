import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRepo } from "./useRepo";

export type RepoState =
  | "clean"
  | "merging"
  | "rebasing"
  | "cherry-picking"
  | "reverting";

const repoState = ref<RepoState>("clean");

export function useConflicts() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) {
      repoState.value = "clean";
      return;
    }
    repoState.value = await invoke<RepoState>("get_repo_state", {
      repoPath: repoPath.value,
    });
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
