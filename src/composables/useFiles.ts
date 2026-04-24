import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileStatus } from "@/types";
import { useRepo } from "./useRepo";

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

  return { files, selectedFile, refresh, stageFiles, unstageFiles, discardFiles };
}
