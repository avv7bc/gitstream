<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
interface Entry { sha: string; selector: string; action: string; timestamp: number; subject: string }
const { repoPath } = useRepo(); const refname = ref("HEAD"); const entries = ref<Entry[]>([]); const emit = defineEmits<{ close: [] }>();
async function refresh() { if (repoPath.value) entries.value = await invoke<Entry[]>("get_reflog", { repoPath: repoPath.value, refname: refname.value, limit: 100 }); }
onMounted(refresh); watch(refname, refresh); watch(repoPath, refresh);
</script>
<template><section class="reflog-panel"><header><strong>Reflog</strong><select v-model="refname"><option>HEAD</option></select><button @click="emit('close')">×</button></header><ol><li v-for="entry in entries" :key="entry.selector"><code>{{ entry.sha.slice(0, 8) }}</code><span>{{ entry.action }}</span><small>{{ entry.subject }}</small></li></ol></section></template>
<style scoped>.reflog-panel{padding:8px 12px;background:var(--bg-surface);border-top:1px solid var(--border-color);max-height:220px;overflow:auto}header,li{display:flex;gap:8px;align-items:center}header{justify-content:space-between}ol{padding-left:20px}li{margin:4px 0}small{flex:1;opacity:.7}</style>
