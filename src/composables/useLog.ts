import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import type { CommitInfo } from "@/types";
import { useRepo } from "@/composables/useRepo";

const PAGE_SIZE = 500;

const commits = ref<CommitInfo[]>([]);
const selectedCommit = ref<string | null>(null);
const hasMore = ref(true);
const isLoadingMore = ref(false);

// Защита от гонки перекрывающихся refresh: применяем только самый свежий ответ.
// loadMore тоже использует этот счётчик, чтобы откатывающийся refresh не
// перезаписался устаревшим ответом догрузки.
let refreshSeq = 0;

export function useLog() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) {
      commits.value = [];
      selectedCommit.value = null;
      hasMore.value = true;
      return;
    }
    const seq = ++refreshSeq;
    // Сохраняем уже загруженный объём, чтобы прокрутка пользователя
    // не «обрезалась» после очередной мутации/refresh.
    const target = Math.max(commits.value.length, PAGE_SIZE);
    const data = await invoke<CommitInfo[]>("get_log", {
      repoPath: repoPath.value,
      limit: target,
    });
    // Более новый refresh уже стартовал — отбрасываем устаревший ответ.
    if (seq !== refreshSeq) return;
    commits.value = data;
    hasMore.value = data.length >= target;
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
    // При первом открытии / смене репозитория выбираем HEAD (первую строку).
    if (!selectedCommit.value && data.length > 0) {
      selectedCommit.value = data[0].oid;
    }
  }

  async function loadMore() {
    if (!repoPath.value || isLoadingMore.value || !hasMore.value) return;
    if (commits.value.length === 0) return;
    isLoadingMore.value = true;
    const seq = refreshSeq;
    const target = commits.value.length + PAGE_SIZE;
    try {
      const data = await invoke<CommitInfo[]>("get_log", {
        repoPath: repoPath.value,
        limit: target,
      });
      // Параллельный refresh уже применил свой результат — не затираем его.
      if (seq !== refreshSeq) return;
      // Бэк не вернул новых строк → история исчерпана.
      if (data.length <= commits.value.length) {
        hasMore.value = false;
        return;
      }
      commits.value = data;
      hasMore.value = data.length >= target;
    } finally {
      isLoadingMore.value = false;
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

  async function rewordCommit(oid: string, message: string): Promise<string> {
    if (!repoPath.value) return "";
    return invoke<string>("do_reword_commit", { repoPath: repoPath.value, oid, message });
  }

  function clear() {
    commits.value = [];
    selectedCommit.value = null;
    hasMore.value = true;
  }

  return {
    commits,
    selectedCommit,
    hasMore,
    isLoadingMore,
    refresh,
    loadMore,
    clear,
    resetTo,
    revertCommit,
    cherryPick,
    squashCommits,
    rewordCommit,
  };
}
