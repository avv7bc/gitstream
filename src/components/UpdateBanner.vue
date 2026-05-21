<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "@/types";
import { useI18n } from "@/composables/useI18n";

const props = defineProps<{ info: UpdateInfo }>();
const emit = defineEmits<{ dismiss: [] }>();
const { i18n } = useI18n();

async function openUrl(url: string) {
  try {
    await invoke("open_url", { url });
  } catch {
    // fallback: window.open может не работать в Tauri, но как запасной вариант
    window.open(url, "_blank");
  }
}

async function download() {
  await openUrl(props.info.release_url);
  emit("dismiss");
}
</script>

<template>
  <div class="update-banner">
    <div class="update-icon">↑</div>
    <div class="update-content">
      <div class="update-title">
        GitStream update available (v{{ info.version }})
      </div>
      <button class="update-changelog" @click="openUrl(info.release_url)">
        Changelog ›
      </button>
    </div>
    <div class="update-actions">
      <button class="btn btn-primary" @click="download">{{ i18n.update.download }}</button>
      <button class="btn btn-secondary" @click="emit('dismiss')">
        Dismiss
      </button>
    </div>
  </div>
</template>

<style scoped>
.update-banner {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 90;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  padding: 12px 16px;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  min-width: 280px;
  max-width: 340px;
}

.update-icon {
  font-size: 18px;
  color: var(--accent);
  flex-shrink: 0;
  padding-top: 2px;
}

.update-content {
  flex: 1;
  min-width: 0;
}

.update-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
  line-height: 1.3;
}

.update-changelog {
  font-size: var(--font-size-xs);
  color: var(--accent);
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}

.update-changelog:hover {
  text-decoration: underline;
}

.update-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
}

.update-actions .btn {
  font-size: var(--font-size-xs);
  padding: 4px 10px;
  white-space: nowrap;
}
</style>
