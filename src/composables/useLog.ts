import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CommitInfo } from "@/types";
import { useRepo } from "./useRepo";

const commits = ref<CommitInfo[]>([]);
const selectedCommit = ref<string | null>(null);

export function useLog() {
  const { repoPath } = useRepo();

  async function refresh(limit?: number) {
    if (!repoPath.value) {
      commits.value = [];
      selectedCommit.value = null;
      return;
    }
    commits.value = await invoke<CommitInfo[]>("get_log", {
      repoPath: repoPath.value,
      limit: limit ?? 500,
    });
  }

  function clear() {
    commits.value = [];
    selectedCommit.value = null;
  }

  return { commits, selectedCommit, refresh, clear };
}
