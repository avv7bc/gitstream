import { ref, watch } from "vue";
import { invoke, logError } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import { useSettings } from "@/composables/useSettings";
import type { Remedy, Situation } from "@/types";

// Текущая распознанная проблема синхронизации (или null). Module-level —
// одно состояние на приложение, как у других composable'ов.
const situation = ref<Situation | null>(null);
const busy = ref(false);
// Remote, к которому относится текущая ситуация — чтобы решения применялись
// к тому же remote, что вызвал отказ push (бар не обязан его знать).
const currentRemote = ref<string>("");

// Смена репозитория делает прежнюю ситуацию неактуальной — иначе бар с
// проблемой репо A висел бы над репо B, а его кнопки применились бы к B.
// repoPath — module-level ref, поэтому watch можно завести один раз здесь.
watch(useRepo().repoPath, () => {
  situation.value = null;
  currentRemote.value = "";
});

export function useSync() {
  const { repoPath } = useRepo();
  const { networkTimeoutSecs } = useSettings();

  // Спросить backend, есть ли расхождение с remote-tracking веткой.
  // Вызывается после отказа push (когда фронт уже сделал fetch).
  async function diagnose(remote: string) {
    if (!repoPath.value) return;
    currentRemote.value = remote;
    try {
      situation.value = await invoke<Situation | null>("do_diagnose_sync", {
        repoPath: repoPath.value,
        remote,
      });
    } catch (e) {
      logError(String(e));
      situation.value = null;
    }
  }

  // Выполнить выбранное пользователем решение. Деструктивные операции
  // (force-with-lease) запускаются только отсюда — по явному клику.
  async function applyRemedy(remedy: Remedy) {
    if (!repoPath.value || busy.value) return;
    const remote = currentRemote.value;
    busy.value = true;
    const ts = networkTimeoutSecs.value;
    try {
      switch (remedy.id) {
        case "push_force_lease":
          await invoke("do_push_force_lease", { repoPath: repoPath.value, remote, timeoutSecs: ts });
          break;
        case "pull_rebase":
          await invoke("do_pull", { repoPath: repoPath.value, remote, rebase: true, timeoutSecs: ts });
          break;
        case "pull_merge":
          await invoke("do_pull", { repoPath: repoPath.value, remote, rebase: false, timeoutSecs: ts });
          break;
        case "fetch":
          await invoke("do_fetch", { repoPath: repoPath.value, remote, prune: false, timeoutSecs: ts });
          break;
      }
      // Переспрашиваем диагноз: если решено — situation станет null и бар
      // исчезнет; если pull дал конфликт — им займётся ConflictBar.
      await diagnose(remote);
    } catch (e) {
      logError(String(e));
      // Расхождение могло измениться (например force-with-lease отклонён,
      // т.к. remote ушёл вперёд) — обновим диагноз, но не роняем бар молча.
      await diagnose(remote);
    } finally {
      busy.value = false;
    }
  }

  function dismiss() {
    situation.value = null;
  }

  return { situation, busy, diagnose, applyRemedy, dismiss };
}
