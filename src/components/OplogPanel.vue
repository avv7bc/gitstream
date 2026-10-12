<script setup lang="ts">
import { onMounted } from "vue";
import { useOplog } from "@/composables/useOplog";
const emit = defineEmits<{ close: [] }>();
const { entries, loading, refresh, undo, redo, canRedo } = useOplog();
onMounted(() => { void refresh(); });
async function restore(id: number) { try { await undo(id); } catch { /* command errors are shown by the global output surface */ } }
</script>
<template>
  <section class="oplog-panel" aria-label="Operation log">
    <header><strong>Operations</strong><button type="button" @click="emit('close')">×</button></header>
    <p v-if="loading">Loading…</p><p v-else-if="!entries.length">No operations yet.</p>
    <ol v-else>
      <li v-for="entry in entries" :key="entry.id" :class="{ undone: entry.undone }">
        <span>{{ entry.label }}</span><time>{{ new Date(entry.at * 1000).toLocaleString() }}</time>
        <button v-if="entry.kind !== 'undo' && !entry.undone && entry.eligible !== false" type="button" @click="restore(entry.id)">Undo</button>
      </li>
    </ol>
    <button v-if="canRedo" type="button" @click="redo()">Redo</button>
  </section>
</template>
<style scoped>
.oplog-panel { padding: 10px 14px; background: var(--bg-surface); color: var(--text-primary); border-top: 1px solid var(--border-color); max-height: 180px; overflow: auto; }
header, li { display: flex; align-items: center; gap: 10px; } header { justify-content: space-between; }
ol { margin: 8px 0; padding-left: 24px; } li { margin: 4px 0; } li time { opacity: .65; font-size: .8em; } li button { margin-left: auto; } .undone { opacity: .55; }
</style>
