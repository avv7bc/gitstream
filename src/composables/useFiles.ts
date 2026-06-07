import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import type { FileStatus, LineOp, LineHunkSelection } from "@/types";
import { useRepo } from "@/composables/useRepo";
import { useDiff } from "@/composables/useDiff";
import { useLog } from "@/composables/useLog";
import { useSettings } from "@/composables/useSettings";

const files = ref<FileStatus[]>([]);
// Все отслеживаемые файлы рабочей копии — заполняется только при "Show all files".
const allPaths = ref<string[]>([]);
const selectedFile = ref<string | null>(null);
// Множественное выделение в FileList (Shift/Ctrl). Общий стейт, чтобы
// диалоги Commit/Discard могли показывать именно выбранные файлы.
const selectedPaths = ref<string[]>([]);

// Защита от гонки перекрывающихся refresh: применяем только самый свежий ответ.
let refreshSeq = 0;

export function useFiles() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) {
      files.value = [];
      allPaths.value = [];
      selectedFile.value = null;
      selectedPaths.value = [];
      return;
    }
    const { filesShowAll } = useSettings();
    const seq = ++refreshSeq;
    const data = await invoke<FileStatus[]>("get_status", { repoPath: repoPath.value });
    // Более новый refresh уже стартовал — отбрасываем устаревший ответ.
    if (seq !== refreshSeq) return;
    files.value = data;
    // Полный список файлов нужен только для "Show all files" (вся структура
    // рабочей копии). Без него дерево показывает только папки с изменениями.
    if (filesShowAll.value) {
      const all = await invoke<string[]>("list_all_files", { repoPath: repoPath.value });
      if (seq !== refreshSeq) return;
      allPaths.value = all;
    } else {
      allPaths.value = [];
    }
    // В режиме рабочего дерева: если выбранный файл больше не имеет изменений
    // (например, после переключения ветки), сбрасываем устаревший дифф.
    const { selectedCommit } = useLog();
    const isWorktree = !selectedCommit.value || selectedCommit.value === "__worktree__";
    if (isWorktree && selectedFile.value && !data.some((f) => f.path === selectedFile.value)) {
      const { clearDiff } = useDiff();
      selectedFile.value = null;
      clearDiff();
    }
  }

  // После изменения индекса/рабочего дерева: обновить список файлов и
  // синхронизировать дифф выбранного файла с его новым staged-состоянием.
  async function refreshAfterMutation() {
    const { diffFile, clearDiff } = useDiff();
    await refresh();
    const path = selectedFile.value;
    if (!path) return;
    const f = files.value.find((x) => x.path === path);
    if (!f) {
      // Файл больше не имеет изменений — дифф пуст.
      clearDiff();
      return;
    }
    await diffFile(path, f.staged === "staged");
  }

  async function stageFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("stage_files", { repoPath: repoPath.value, files: paths });
    await refreshAfterMutation();
  }

  async function unstageFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("unstage_files", { repoPath: repoPath.value, files: paths });
    await refreshAfterMutation();
  }

  async function discardFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("discard_files", { repoPath: repoPath.value, files: paths });
    await refreshAfterMutation();
  }

  async function removeFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("remove_files", { repoPath: repoPath.value, files: paths });
    await refreshAfterMutation();
  }

  async function deleteFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("delete_files", { repoPath: repoPath.value, files: paths });
    await refreshAfterMutation();
  }

  async function applyHunk(
    command: "stage_hunk" | "unstage_hunk" | "discard_hunk",
    patch: string,
  ) {
    if (!repoPath.value) return;
    const { reloadDiff } = useDiff();
    await invoke(command, { repoPath: repoPath.value, patch });
    await refresh();
    await reloadDiff();
  }

  const stageHunk = (patch: string) => applyHunk("stage_hunk", patch);
  const unstageHunk = (patch: string) => applyHunk("unstage_hunk", patch);
  const discardHunk = (patch: string) => applyHunk("discard_hunk", patch);

  async function applyLines(
    op: LineOp,
    fileHeader: string,
    hunks: LineHunkSelection[],
  ) {
    if (!repoPath.value) return;
    if (hunks.length === 0) return;
    const { reloadDiff } = useDiff();
    await invoke("apply_lines", {
      repoPath: repoPath.value,
      fileHeader,
      hunks,
      op,
    });
    await refresh();
    await reloadDiff();
  }

  return {
    files,
    allPaths,
    selectedFile,
    selectedPaths,
    refresh,
    stageFiles,
    unstageFiles,
    discardFiles,
    removeFiles,
    deleteFiles,
    stageHunk,
    unstageHunk,
    discardHunk,
    applyLines,
  };
}
