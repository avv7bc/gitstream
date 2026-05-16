import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileDiff } from "@/types";
import { useRepo } from "./useRepo";

const currentDiff = ref<FileDiff | null>(null);
// Контекст текущего диффа — определяет, какие операции с хунками доступны.
const diffContext = ref<"unstaged" | "staged" | "commit" | null>(null);

// Параметры последнего вызова, чтобы перезагрузить дифф после stage/unstage хунка.
let lastFile: { path: string; staged: boolean } | null = null;
let lastCommit: { oid: string; filePath?: string } | null = null;

export function useDiff() {
  const { repoPath } = useRepo();

  async function diffFile(path: string, staged: boolean) {
    if (!repoPath.value) return;
    lastFile = { path, staged };
    lastCommit = null;
    diffContext.value = staged ? "staged" : "unstaged";
    currentDiff.value = await invoke<FileDiff>("get_diff_file", {
      repoPath: repoPath.value,
      file: path,
      staged,
    });
  }

  async function diffCommit(oid: string, filePath?: string) {
    if (!repoPath.value) return;
    lastCommit = { oid, filePath };
    lastFile = null;
    diffContext.value = "commit";
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

  // Повторяет последний дифф (после изменения индекса/рабочего дерева).
  async function reloadDiff() {
    if (lastFile) {
      await diffFile(lastFile.path, lastFile.staged);
    } else if (lastCommit) {
      await diffCommit(lastCommit.oid, lastCommit.filePath);
    }
  }

  function clearDiff() {
    currentDiff.value = null;
    diffContext.value = null;
    lastFile = null;
    lastCommit = null;
  }

  return { currentDiff, diffContext, diffFile, diffCommit, reloadDiff, clearDiff };
}
