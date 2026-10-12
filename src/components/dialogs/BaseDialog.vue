<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

withDefaults(defineProps<{ title: string; subtitle?: string; size?: "sm" | "md" | "lg" | "full"; busy?: boolean; dismissable?: boolean }>(), { subtitle: "", size: "md", busy: false, dismissable: true });
const emit = defineEmits<{ close: [] }>();
const { dragStyle, onDragStart } = useDraggable();
function onKeydown(event: KeyboardEvent) { if (event.key === "Escape") emit("close"); }
onMounted(() => document.addEventListener("keydown", onKeydown));
onUnmounted(() => document.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div class="base-dialog-overlay" @click.self="dismissable && emit('close')">
    <section class="base-dialog" :class="`size-${size}`" :style="dragStyle" role="dialog" aria-modal="true" :aria-label="title">
      <header class="base-dialog-header" @mousedown="onDragStart">
        <div><h2>{{ title }}</h2><p v-if="subtitle">{{ subtitle }}</p></div>
        <button v-if="dismissable" class="base-dialog-close" type="button" aria-label="Close" @click="emit('close')">×</button>
      </header>
      <div v-if="busy" class="base-dialog-busy" aria-live="polite">Working…</div>
      <div class="base-dialog-body"><slot /></div>
      <footer v-if="$slots.footer" class="base-dialog-footer"><slot name="footer" /></footer>
    </section>
  </div>
</template>

<style scoped>
.base-dialog-overlay { position: fixed; inset: 0; z-index: 3000; display: grid; place-items: center; padding: 24px; background: rgba(0, 0, 0, .45); }
.base-dialog { display: flex; max-height: min(760px, calc(100vh - 48px)); flex-direction: column; overflow: hidden; border: 1px solid var(--border); border-radius: 7px; background: var(--bg-secondary); box-shadow: 0 18px 60px rgba(0, 0, 0, .5); }
.size-sm { width: min(420px, 100%); } .size-md { width: min(640px, 100%); } .size-lg { width: min(900px, 100%); } .size-full { width: min(1180px, 100%); height: min(760px, 100%); }
.base-dialog-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; padding: 15px 18px; border-bottom: 1px solid var(--border-subtle); cursor: move; }
.base-dialog-header h2 { font-size: 15px; font-weight: 600; } .base-dialog-header p { margin-top: 2px; color: var(--text-secondary); font-size: 12px; }
.base-dialog-close { color: var(--text-muted); font-size: 22px; line-height: 1; cursor: pointer; } .base-dialog-close:hover { color: var(--text-primary); }
.base-dialog-body { min-height: 0; overflow: auto; padding: 18px; } .base-dialog-footer { display: flex; justify-content: flex-end; gap: 8px; padding: 12px 18px; border-top: 1px solid var(--border-subtle); }
.base-dialog-busy { padding: 6px 18px; border-bottom: 1px solid var(--border-subtle); color: var(--accent); font-size: 12px; }
</style>
