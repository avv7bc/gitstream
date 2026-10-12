<script setup lang="ts">
withDefaults(defineProps<{ icon?: string; title: string; description?: string }>(), { icon: "·", description: "" });
const emit = defineEmits<{ action: [] }>();
</script>

<template>
  <div class="empty-state" role="status">
    <span class="empty-icon" aria-hidden="true">{{ icon }}</span>
    <strong>{{ title }}</strong>
    <p v-if="description">{{ description }}</p>
    <button v-if="$slots.default" class="btn btn-secondary" type="button" @click="emit('action')"><slot /></button>
  </div>
</template>

<style scoped>
.empty-state { display: flex; min-height: 160px; flex-direction: column; align-items: center; justify-content: center; gap: 7px; padding: 24px; color: var(--text-secondary); text-align: center; }
.empty-icon { display: inline-flex; width: 36px; height: 36px; align-items: center; justify-content: center; border-radius: 9px; background: var(--bg-hover); color: var(--accent); font: 600 22px/1 var(--font-mono); }
.empty-state strong { color: var(--text-primary); font-size: 14px; }
.empty-state p { max-width: 34ch; color: var(--text-muted); font-size: 12px; }
</style>
