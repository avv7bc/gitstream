import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileDiff } from "@/types";
import { useRepo } from "./useRepo";

const currentDiff = ref<FileDiff | null>(null);

export function useDiff() {
  const { repoPath } = useRepo();

  async function diffFile(path: string, staged: boolean) {
    if (!repoPath.value) return;
    currentDiff.value = await invoke<FileDiff>("get_diff_file", {
      repoPath: repoPath.value,
      file: path,
      staged,
    });
  }

  async function diffCommit(oid: string, filePath?: string) {
    if (!repoPath.value) return;
    const diffs = await invoke<FileDiff[]>("get_diff_commit", {
      repoPath: repoPath.value,
      oid,
    });
    if (filePath) {
      currentDiff.value = diffs.find((d) => d.path === filePath) ?? null;
    } else {
      currentDiff.value = diffs[0] ?? null;
    }
  }

  function clearDiff() {
    currentDiff.value = null;
  }

  return { currentDiff, diffFile, diffCommit, clearDiff };
}
