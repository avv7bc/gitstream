import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import type { CommitInfo } from "@/types";
import { useRepo } from "./useRepo";

const commits = ref<CommitInfo[]>([]);
const selectedCommit = ref<string | null>(null);

// Защита от гонки перекрывающихся refresh: применяем только самый свежий ответ.
let refreshSeq = 0;

export function useLog() {
  const { repoPath } = useRepo();

  async function refresh(limit?: number) {
    if (!repoPath.value) {
      commits.value = [];
      selectedCommit.value = null;
      return;
    }
    const seq = ++refreshSeq;
    const data = await invoke<CommitInfo[]>("get_log", {
      repoPath: repoPath.value,
      limit: limit ?? 500,
    });
    // Более новый refresh уже стартовал — отбрасываем устаревший ответ.
    if (seq !== refreshSeq) return;
    commits.value = data;
    // Выделение указывает на коммит, которого больше нет в свежем логе
    // (смена репозитория, reset/rebase/удаление ветки) — сбрасываем его,
    // иначе панель Files показывает файлы чужого/исчезнувшего коммита.
    if (
      selectedCommit.value &&
      selectedCommit.value !== "__worktree__" &&
      !data.some((c) => c.oid === selectedCommit.value)
    ) {
      selectedCommit.value = null;
    }
  }

  async function resetTo(oid: string, mode: "soft" | "mixed" | "hard") {
    if (!repoPath.value) return;
    await invoke("do_reset", { repoPath: repoPath.value, oid, mode });
  }

  async function revertCommit(oid: string, noCommit: boolean) {
    if (!repoPath.value) return;
    await invoke("do_revert", { repoPath: repoPath.value, oid, noCommit });
  }

  async function cherryPick(oid: string) {
    if (!repoPath.value) return;
    await invoke("do_cherry_pick", { repoPath: repoPath.value, oid });
  }

  async function squashCommits(oids: string[], message: string) {
    if (!repoPath.value) return;
    await invoke("do_squash", { repoPath: repoPath.value, oids, message });
  }

  async function rewordCommit(oid: string, message: string) {
    if (!repoPath.value) return;
    await invoke("do_reword_commit", { repoPath: repoPath.value, oid, message });
  }

  function clear() {
    commits.value = [];
    selectedCommit.value = null;
  }

  return {
    commits,
    selectedCommit,
    refresh,
    clear,
    resetTo,
    revertCommit,
    cherryPick,
    squashCommits,
    rewordCommit,
  };
}
