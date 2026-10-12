<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import WorktreeDialog from "./dialogs/WorktreeDialog.vue";
import ConfirmDialog from "./dialogs/ConfirmDialog.vue";
import { useBranches } from "@/composables/useBranches";
const { branches } = useBranches();
const removePath = ref<string | null>(null);
interface Worktree { path: string; head: string; branch?: string | null; common_dir?: string | null; detached: boolean; locked: boolean; prunable: boolean }
const { repoPath } = useRepo(); const rows = ref<Worktree[]>([]); const busy = ref(false); const showAdd = ref(false);
async function refresh() { if (repoPath.value) rows.value = await invoke<Worktree[]>("get_worktrees", { repoPath: repoPath.value }); }
async function prune() { if (!repoPath.value) return; busy.value = true; try { await invoke("do_worktree_prune", { repoPath: repoPath.value }); await refresh(); } finally { busy.value = false; } }
async function add(path: string, branch: string, newBranch: string | null) { if (!repoPath.value) return; busy.value = true; try { await invoke("do_worktree_add", { repoPath: repoPath.value, path, branch, newBranch }); showAdd.value = false; await refresh(); } finally { busy.value = false; } }
async function remove(path: string, force = false) { if (!repoPath.value) return; busy.value = true; try { await invoke("do_worktree_remove", { repoPath: repoPath.value, path, force }); removePath.value = null; await refresh(); } finally { busy.value = false; } }
onMounted(refresh); watch(repoPath, refresh);
</script>
<template><section class="worktree-panel"><header><strong>Worktrees</strong><span><button :disabled="busy" @click="showAdd = true">Add</button><button :disabled="busy" @click="prune">Prune</button></span></header><ul><li v-for="row in rows" :key="row.path" :title="row.path"><span>{{ row.branch || (row.detached ? 'detached' : row.head.slice(0, 8)) }}<b v-if="row.branch" class="occupied"> (occupied)</b><small>{{ row.path }}{{ row.locked ? ' · locked' : '' }}{{ row.prunable ? ' · prunable' : '' }}</small></span><button v-if="row.path !== repoPath" :disabled="busy" @click="removePath = row.path">Remove</button></li></ul><ConfirmDialog v-if="removePath" :message="`Remove worktree ${removePath}?`" confirm-label="Remove" danger checkbox-label="Force removal (discard local changes)" @close="removePath = null" @confirm="(force) => remove(removePath!, force)" /><WorktreeDialog v-if="showAdd" :branches="branches.filter(b => !b.is_remote && !rows.some(w => w.branch === b.name)).map(b => b.name)" @close="showAdd = false" @submit="add" /></section></template>
<style scoped>.worktree-panel{padding:8px;border-top:1px solid var(--border-color)}header,li{display:flex;gap:8px;justify-content:space-between}ul{padding-left:16px}small{opacity:.65;font-size:.8em}</style>
