import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";

export function useCommit() {
  const { repoPath } = useRepo();

  async function commit(message: string, amend: boolean): Promise<string | undefined> {
    if (!repoPath.value) return undefined;
    return await invoke<string>("do_commit", { repoPath: repoPath.value, message, amend });
  }

  return { commit };
}
