import { ref, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RepoStats } from "@/types";
import { useRepo } from "@/composables/useRepo";

export type StatsPeriod = "7d" | "30d" | "1y" | "all";

const periodDays: Record<StatsPeriod, number | null> = {
  "7d": 7,
  "30d": 30,
  "1y": 365,
  all: null,
};

export function useStats() {
  const { repoPath } = useRepo();
  const loading = ref(true);
  const stats = ref<RepoStats | null>(null);
  const error = ref<string | null>(null);
  const period = ref<StatsPeriod>("30d");
  const progress = ref(0);

  let progressTimer: ReturnType<typeof setInterval> | null = null;
  let loadToken = 0;
  let aborted = false;

  function clearTimer() {
    if (progressTimer) { clearInterval(progressTimer); progressTimer = null; }
  }

  function startProgress() {
    clearTimer();
    let current = 5;
    progress.value = current;
    progressTimer = setInterval(() => {
      current += (90 - current) * 0.2;
      progress.value = Math.round(current);
    }, 1000);
  }

  function stopProgress(success: boolean) {
    clearTimer();
    progress.value = success ? 100 : 0;
  }

  function abort() {
    aborted = true;
    clearTimer();
    loading.value = false;
  }

  async function loadStats() {
    if (!repoPath.value || aborted) return;
    const token = ++loadToken;
    loading.value = true;
    error.value = null;
    startProgress();
    await nextTick();
    await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
    if (aborted || token !== loadToken) return;
    try {
      const result = await invoke<RepoStats>("get_repo_stats", {
        repoPath: repoPath.value,
        sinceDays: periodDays[period.value],
      });
      if (aborted || token !== loadToken) return;
      stats.value = result;
      stopProgress(true);
    } catch (e) {
      if (aborted || token !== loadToken) return;
      error.value = String(e);
      stopProgress(false);
    } finally {
      if (!aborted && token === loadToken) loading.value = false;
    }
  }

  return { loading, stats, error, period, progress, loadStats, abort };
}
