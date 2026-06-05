import { ref } from "vue";
import { invoke, logError } from "@/composables/useProgress";
import { useRepo } from "./useRepo";
import { useSettings } from "./useSettings";

const isBusy = ref(false);

export function useRemote() {
  const { repoPath } = useRepo();
  const { networkTimeoutSecs } = useSettings();

  // Ошибки сетевых операций уходят в Git output (красным, с авто-открытием
  // панели) — без модальных окон.
  async function wrapAsync(fn: () => Promise<unknown>) {
    isBusy.value = true;
    try {
      await fn();
    } catch (e) {
      logError(String(e));
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

  return { isBusy, fetchRemote, pull, push };
}
