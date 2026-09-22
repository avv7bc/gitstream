import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import type { RepoInfo } from "@/types";

const LAST_REPO_KEY = "gitstream:last-repo-path";

const repoPath = ref<string | null>(null);
const repoInfo = ref<RepoInfo | null>(null);
const onOpenCallbacks: Array<(isCurrent: () => boolean) => Promise<void>> = [];
let openSeq = 0;
let infoSeq = 0;

export function useRepo() {
  // false: запрос устарел (другое открытие или закрытие). Вызывающий не
  // должен продолжать операции, рассчитанные на эту сессию репозитория.
  async function openRepo(path: string): Promise<boolean> {
    const seq = ++openSeq;
    const isCurrent = () => seq === openSeq;
    let info: RepoInfo;
    try {
      info = await invoke<RepoInfo>("get_repo_info", { repoPath: path });
    } catch (e) {
      if (!isCurrent()) return false;
      // Удалённый сохранённый путь не восстанавливаем при каждом запуске.
      // Ошибка загрузки панелей после успешного открытия сюда не попадает.
      if (localStorage.getItem(LAST_REPO_KEY) === path && repoPath.value !== path) {
        localStorage.removeItem(LAST_REPO_KEY);
      }
      throw e;
    }
    if (!isCurrent()) return false;
    // Публикуем путь и сведения только после успешной проверки. Поздние
    // refreshInfo предыдущей сессии больше не могут перезаписать сведения.
    ++infoSeq;
    repoPath.value = path;
    repoInfo.value = info;
    localStorage.setItem(LAST_REPO_KEY, path);
    for (const cb of onOpenCallbacks) {
      if (!isCurrent()) return false;
      await cb(isCurrent);
    }
    return isCurrent();
  }

  async function restoreLastRepo() {
    const last = localStorage.getItem(LAST_REPO_KEY);
    if (last) {
      try {
        await openRepo(last);
      } catch { /* repo may be gone */ }
    }
  }

  function closeRepo() {
    ++openSeq;
    ++infoSeq;
    repoPath.value = null;
    repoInfo.value = null;
    localStorage.removeItem(LAST_REPO_KEY);
  }

  async function refreshInfo() {
    const path = repoPath.value;
    if (!path) return;
    const seq = ++infoSeq;
    const info = await invoke<RepoInfo>("get_repo_info", { repoPath: path });
    if (seq !== infoSeq || repoPath.value !== path) return;
    repoInfo.value = info;
  }

  function onRepoOpened(cb: (isCurrent: () => boolean) => Promise<void>) {
    if (!onOpenCallbacks.includes(cb)) {
      onOpenCallbacks.push(cb);
    }
  }

  return { repoPath, repoInfo, openRepo, closeRepo, refreshInfo, onRepoOpened, restoreLastRepo };
}
