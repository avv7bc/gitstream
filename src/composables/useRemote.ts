import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRepo } from "./useRepo";

const isBusy = ref(false);
const lastError = ref<string | null>(null);

export function useRemote() {
  const { repoPath } = useRepo();

  async function wrapAsync(fn: () => Promise<unknown>) {
    isBusy.value = true;
    lastError.value = null;
    try {
      await fn();
    } catch (e) {
      lastError.value = String(e);
    } finally {
      isBusy.value = false;
    }
  }

  async function fetchRemote(remote: string) {
    await wrapAsync(() => invoke("do_fetch", { repoPath: repoPath.value!, remote }));
  }

  async function pull(remote: string, rebase: boolean) {
    await wrapAsync(() => invoke("do_pull", { repoPath: repoPath.value!, remote, rebase }));
  }

  async function push(remote: string, force: boolean) {
    await wrapAsync(() => invoke("do_push", { repoPath: repoPath.value!, remote, force }));
  }

  async function cloneRepo(url: string, dest: string) {
    await wrapAsync(() => invoke("do_clone", { url, dest }));
  }

  return { isBusy, lastError, fetchRemote, pull, push, cloneRepo };
}
