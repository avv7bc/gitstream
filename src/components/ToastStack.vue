<script setup lang="ts">
import { useToast } from "@/composables/useToast";

const { toasts, dismissToast } = useToast();
</script>

<template>
  <div v-if="toasts.length" class="toast-stack" role="status" aria-live="polite">
    <div
      v-for="item in toasts"
      :key="item.id"
      class="toast"
      :class="`toast-${item.kind}`"
      role="alert"
    >
      <span class="toast-message">{{ item.title }}</span>
      <span v-if="item.detail" class="toast-detail">{{ item.detail }}</span>
      <button class="toast-close" type="button" aria-label="Dismiss" @click="dismissToast(item.id)">×</button>
    </div>
  </div>
</template>

<style scoped>
.toast-stack {
  position: fixed;
  z-index: 3000;
  right: 16px;
  bottom: 38px;
  width: min(420px, calc(100vw - 32px));
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}
.toast {
  pointer-events: auto;
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 3px 10px;
  padding: 10px 12px;
  border: 1px solid var(--border-subtle);
  border-left: 3px solid var(--text-muted);
  border-radius: 4px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  box-shadow: 0 4px 18px rgb(0 0 0 / 28%);
  font-size: 12px;
}
.toast-error { border-left-color: var(--red, #e06060); }
.toast-success { border-left-color: var(--green, #5dbb7b); }
.toast-warning { border-left-color: var(--yellow, #d3a64d); }
.toast-message { line-height: 1.35; }
.toast-detail { grid-column: 1 / 2; color: var(--text-muted); white-space: pre-wrap; }
.toast-close {
  grid-column: 2;
  grid-row: 1 / span 2;
  align-self: start;
  border: 0;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
}
.toast-close:hover { color: var(--text-primary); }
</style>
