import { ref } from "vue";
import { invoke, logError } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import { useLog } from "@/composables/useLog";
import type { BlameLine } from "@/types";

// Blame одного файла (git blame --porcelain). Модальная вью: перезагружается
// при каждом открытии, поэтому stale-данные между репозиториями не показываются.
const open = ref(false);
const path = ref("");
const lines = ref<BlameLine[]>([]);
const loading = ref(false);
let seq = 0;

export function useBlame() {
  const { repoPath } = useRepo();
  const { selectedCommit } = useLog();

  async function openFor(p: string) {
    path.value = p;
    open.value = true;
    lines.value = [];
    if (!repoPath.value) return;
    loading.value = true;
    const mySeq = ++seq;
    try {
      const data = await invoke<BlameLine[]>("get_blame", {
        repoPath: repoPath.value,
        path: p,
        rev: null,
      });
      if (mySeq !== seq) return;
      lines.value = data;
    } catch (e) {
      logError(`Blame failed: ${e}`);
    } finally {
      if (mySeq === seq) loading.value = false;
    }
  }

  // Выделяет коммит строки в основном графе и закрывает вью.
  function goToCommit(oid: string) {
    selectedCommit.value = oid;
    close();
  }

  function close() {
    open.value = false;
  }

  return { open, path, lines, loading, openFor, goToCommit, close };
}
