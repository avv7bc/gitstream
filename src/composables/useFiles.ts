import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileStatus } from "@/types";
import { useRepo } from "./useRepo";
import { useDiff } from "./useDiff";

const files = ref<FileStatus[]>([]);
const selectedFile = ref<string | null>(null);

export function useFiles() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) {
      files.value = [];
      selectedFile.value = null;
      return;
    }
    files.value = await invoke<FileStatus[]>("get_status", { repoPath: repoPath.value });
  }

  async function stageFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("stage_files", { repoPath: repoPath.value, files: paths });
    await refresh();
  }

  async function unstageFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("unstage_files", { repoPath: repoPath.value, files: paths });
    await refresh();
  }

  async function discardFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("discard_files", { repoPath: repoPath.value, files: paths });
    await refresh();
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

  return {
    files,
    selectedFile,
    refresh,
    stageFiles,
    unstageFiles,
    discardFiles,
    stageHunk,
    unstageHunk,
    discardHunk,
  };
}
