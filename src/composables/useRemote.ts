import { ref } from "vue";
import { invoke, logError } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import { useSettings } from "@/composables/useSettings";
import { useSync } from "@/composables/useSync";

const isBusy = ref(false);

// Push отклонён, потому что на remote есть коммиты, которых нет локально.
export function isRejectedNeedsFetch(e: unknown): boolean {
  return /fetch first|non-fast-forward/i.test(String(e));
}

export function useRemote() {
  const { repoPath } = useRepo();
  const { networkTimeoutSecs } = useSettings();
  const { diagnose, dismiss } = useSync();

  // Ошибки сетевых операций уходят в Git output (красным, с авто-открытием
  // панели) — без модальных окон. Возвращает true при успехе.
  async function wrapAsync(fn: () => Promise<unknown>): Promise<boolean> {
    isBusy.value = true;
    try {
      await fn();
      return true;
    } catch (e) {
      logError(String(e));
      return false;
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

  async function fetchAll(prune = false) {
    if (!repoPath.value) return;
    await wrapAsync(() =>
      invoke("do_fetch_all", {
        repoPath: repoPath.value!,
        prune,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
  }

  async function pull(remote: string, rebase: boolean) {
    if (!repoPath.value) return;
    const ok = await wrapAsync(() =>
      invoke("do_pull", {
        repoPath: repoPath.value!,
        remote,
        rebase,
        timeoutSecs: networkTimeoutSecs.value,
      })
    );
    // Успешный pull устраняет расхождение — гасим ассистента, если он висел
    // (например после ручного Pull в обход кнопки решения).
    if (ok) dismiss();
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
      // Успешный push устраняет расхождение — гасим прежнюю ситуацию.
      dismiss();
    } catch (e) {
      logError(String(e));
      // Автоматический fetch: подтягиваем удалённые коммиты, чтобы причина
      // отказа была видна в графе и behind-счётчике, а remote-tracking ref
      // стал актуальным для диагностики.
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
        // Распознаём ситуацию и показываем Sync Assistant с решениями
        // вместо немой красной строки в Git output.
        await diagnose(remote);
      }
    } finally {
      isBusy.value = false;
    }
  }

  return { isBusy, fetchRemote, fetchAll, pull, push };
}
