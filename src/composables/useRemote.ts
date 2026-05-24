import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "./useRepo";
import { useSettings } from "./useSettings";

const isBusy = ref(false);
const lastError = ref<string | null>(null);

export function useRemote() {
  const { repoPath } = useRepo();
  const { networkTimeoutSecs } = useSettings();

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
    if (!repoPath.value) return;
    await wrapAsync(() =>
      invoke("do_fetch", {
        repoPath: repoPath.value!,
        remote,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  async function pull(remote: string, rebase: boolean) {
    if (!repoPath.value) return;
    await wrapAsync(() =>
      invoke("do_pull", {
        repoPath: repoPath.value!,
        remote,
        rebase,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  async function push(remote: string, force: boolean) {
    if (!repoPath.value) return;
    await wrapAsync(() =>
      invoke("do_push", {
        repoPath: repoPath.value!,
        remote,
        force,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  return { isBusy, lastError, fetchRemote, pull, push };
}
