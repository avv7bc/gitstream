import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateCheck, UpdateInfo } from "@/types";

const updateInfo = ref<UpdateInfo | null>(null);
const nextMajorInfo = ref<UpdateInfo | null>(null);
const DISMISSED_MAJOR_KEY = "gitstream.dismissed-major";

function dismissedMajor(): string | null {
  if (typeof localStorage === "undefined") return null;
  try {
    return localStorage.getItem(DISMISSED_MAJOR_KEY);
  } catch {
    return null;
  }
}

export function useUpdate() {
  async function checkForUpdate() {
    try {
      const result = await invoke<UpdateCheck>("check_for_update");
      updateInfo.value = result.update;
      const dismissed = dismissedMajor();
      nextMajorInfo.value = result.next_major?.version === dismissed ? null : result.next_major;
    } catch {
      // silent fail — no network or API error
    }
  }

  function dismissNextMajor() {
    const info = nextMajorInfo.value;
    if (!info) return;
    try {
      localStorage.setItem(DISMISSED_MAJOR_KEY, info.version);
    } catch {
      // Ignore unavailable storage; the current banner is still dismissed.
    }
    nextMajorInfo.value = null;
  }

  return { updateInfo, nextMajorInfo, checkForUpdate, dismissNextMajor };
}
