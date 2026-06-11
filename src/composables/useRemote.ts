import { ref } from "vue";
import { invoke, logError } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import { useSettings } from "@/composables/useSettings";

const isBusy = ref(false);

// Push отклонён, потому что на remote есть коммиты, которых нет локально.
export function isRejectedNeedsFetch(e: unknown): boolean {
  return /fetch first|non-fast-forward/i.test(String(e));
}

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

  async function fetchRemote(remote: string, prune = false) {
    if (!repoPath.value) return;
    await wrapAsync(() =>
      invoke("do_fetch", {
        repoPath: repoPath.value!,
        remote,
        prune,
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
    isBusy.value = true;
    try {
      await invoke("do_push", {
        repoPath: repoPath.value!,
        remote,
        force,
        timeoutSecs: networkTimeoutSecs.value,
      });
    } catch (e) {
      logError(String(e));
      // Автоматический fetch: подтягиваем удалённые коммиты, чтобы причина
      // отказа была видна в графе и behind-счётчике.
      if (isRejectedNeedsFetch(e)) {
        try {
          await invoke("do_fetch", {
            repoPath: repoPath.value!,
            remote,
            prune: false,
            timeoutSecs: networkTimeoutSecs.value,
          });
        } catch (fe) {
          logError(String(fe));
        }
      }
    } finally {
      isBusy.value = false;
    }
  }

  return { isBusy, fetchRemote, pull, push };
}
