import { ref } from "vue";
import { invoke, logError } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import { useLog } from "@/composables/useLog";
import type { CommitInfo, FileDiff } from "@/types";

// История одного файла (git log --follow). Модальный диалог: состояние
// перезагружается при каждом открытии, поэтому stale-данные между репозиториями
// не показываются.
const open = ref(false);
const path = ref("");
const commits = ref<CommitInfo[]>([]);
const selectedOid = ref<string | null>(null);
const fileDiff = ref<FileDiff | null>(null);
const loading = ref(false);
let logSeq = 0;

export function useFileHistory() {
  const { repoPath } = useRepo();
  const { selectedCommit } = useLog();

  async function openFor(p: string) {
    path.value = p;
    open.value = true;
    commits.value = [];
    selectedOid.value = null;
    fileDiff.value = null;
    if (!repoPath.value) return;
    loading.value = true;
    const mySeq = ++logSeq;
    try {
      const data = await invoke<CommitInfo[]>("get_file_log", {
        repoPath: repoPath.value,
        path: p,
        limit: 500,
      });
      if (mySeq !== logSeq) return;
      commits.value = data;
      if (data.length) await selectCommit(data[0].oid);
    } catch (e) {
      logError(`File history failed: ${e}`);
    } finally {
      if (mySeq === logSeq) loading.value = false;
    }
  }

  async function selectCommit(oid: string) {
    selectedOid.value = oid;
    fileDiff.value = null;
    if (!repoPath.value) return;
    try {
      const diff = await invoke<FileDiff>("get_diff_commit_file", {
        repoPath: repoPath.value,
        oid,
        file: path.value,
      });
      // Защита от гонки: пока грузили, мог быть выбран другой коммит.
      if (selectedOid.value !== oid) return;
      fileDiff.value = diff;
    } catch (e) {
      logError(`File diff failed: ${e}`);
    }
  }

  // Выделяет коммит в основном графе (CommitGraph скроллит к нему сам) и
  // закрывает диалог.
  function goToCommit(oid: string) {
    selectedCommit.value = oid;
    close();
  }

  function close() {
    open.value = false;
  }

  return {
    open,
    path,
    commits,
    selectedOid,
    fileDiff,
    loading,
    openFor,
    selectCommit,
    goToCommit,
    close,
  };
}
