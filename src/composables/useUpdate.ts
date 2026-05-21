import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "@/types";

const updateInfo = ref<UpdateInfo | null>(null);

export function useUpdate() {
  async function checkForUpdate() {
    try {
      const info = await invoke<UpdateInfo | null>("check_for_update");
      updateInfo.value = info;
    } catch {
      // silent fail — no network or API error
    }
  }

  return { updateInfo, checkForUpdate };
}
