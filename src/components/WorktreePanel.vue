<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
interface Worktree { path: string; head: string; branch?: string | null; detached: boolean; locked: boolean; prunable: boolean }
const { repoPath } = useRepo(); const rows = ref<Worktree[]>([]); const busy = ref(false);
async function refresh() { if (repoPath.value) rows.value = await invoke<Worktree[]>("get_worktrees", { repoPath: repoPath.value }); }
async function prune() { if (!repoPath.value) return; busy.value = true; try { await invoke("do_worktree_prune", { repoPath: repoPath.value }); await refresh(); } finally { busy.value = false; } }
onMounted(refresh); watch(repoPath, refresh);
</script>
<template><section class="worktree-panel"><header><strong>Worktrees</strong><button :disabled="busy" @click="prune">Prune</button></header><ul><li v-for="row in rows" :key="row.path" :title="row.path">{{ row.branch || (row.detached ? 'detached' : row.head.slice(0, 8)) }}<small>{{ row.path }}</small></li></ul></section></template>
<style scoped>.worktree-panel{padding:8px;border-top:1px solid var(--border-color)}header,li{display:flex;gap:8px;justify-content:space-between}ul{padding-left:16px}small{opacity:.65;font-size:.8em}</style>
