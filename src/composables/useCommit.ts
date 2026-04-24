import { invoke } from "@tauri-apps/api/core";
import { useRepo } from "./useRepo";

export function useCommit() {
  const { repoPath } = useRepo();

  async function commit(message: string, amend: boolean) {
    if (!repoPath.value) return;
    await invoke("do_commit", { repoPath: repoPath.value, message, amend });
  }

  return { commit };
}
