import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RepoInfo } from "@/types";

const LAST_REPO_KEY = "gitstream:last-repo-path";

const repoPath = ref<string | null>(null);
const repoInfo = ref<RepoInfo | null>(null);
const onOpenCallbacks: Array<() => Promise<void>> = [];

export function useRepo() {
  async function openRepo(path: string) {
    repoPath.value = path;
    repoInfo.value = await invoke<RepoInfo>("get_repo_info", { repoPath: path });
    localStorage.setItem(LAST_REPO_KEY, path);
    for (const cb of onOpenCallbacks) {
      await cb();
    }
  }

  async function restoreLastRepo() {
    const last = localStorage.getItem(LAST_REPO_KEY);
    if (last) {
      try {
        await openRepo(last);
      } catch { /* repo may be gone */ }
    }
  }

  function closeRepo() {
    repoPath.value = null;
    repoInfo.value = null;
    localStorage.removeItem(LAST_REPO_KEY);
  }

  async function refreshInfo() {
    if (!repoPath.value) return;
    repoInfo.value = await invoke<RepoInfo>("get_repo_info", { repoPath: repoPath.value });
  }

  function onRepoOpened(cb: () => Promise<void>) {
    if (!onOpenCallbacks.includes(cb)) {
      onOpenCallbacks.push(cb);
    }
  }

  return { repoPath, repoInfo, openRepo, closeRepo, refreshInfo, onRepoOpened, restoreLastRepo };
}
