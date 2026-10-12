<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import type { CommitInfo } from "@/types";
const props = defineProps<{ commits: CommitInfo[]; base?: string | null }>();
const emit = defineEmits<{ close: []; done: [] }>();
const { repoPath } = useRepo();
const actions = ["pick", "reword", "edit", "squash", "fixup", "drop"] as const;
const plan = ref(props.commits.map((commit) => ({ sha: commit.oid, action: "pick" as typeof actions[number], message: null as string | null })));
const busy = ref(false);
async function submit() {
  if (!repoPath.value) return;
  busy.value = true;
  try { await invoke("do_rebase_plan", { repoPath: repoPath.value, base: props.base ?? null, plan: plan.value }); emit("done"); } finally { busy.value = false; }
}
</script>
<template>
  <div class="modal-overlay" @click.self="emit('close')"><div class="modal-dialog" style="width: 640px">
    <div class="dialog-header"><h3>Interactive rebase</h3><button class="close-btn" @click="emit('close')">×</button></div>
    <div class="dialog-body"><p>Arrange commits and choose an action.</p><ol><li v-for="(item, index) in plan" :key="item.sha"><select v-model="item.action"><option v-for="action in actions" :key="action" :value="action">{{ action }}</option></select><code>{{ item.sha.slice(0, 8) }}</code><span>{{ props.commits.find(c => c.oid === item.sha)?.message }}</span><button v-if="index" @click="[plan[index - 1], plan[index]] = [plan[index], plan[index - 1]]">↑</button><button v-if="index + 1 < plan.length" @click="[plan[index], plan[index + 1]] = [plan[index + 1], plan[index]]">↓</button></li></ol></div>
    <div class="dialog-footer"><button class="btn btn-secondary" @click="emit('close')">Cancel</button><button class="btn btn-primary" :disabled="busy" @click="submit">{{ busy ? 'Starting…' : 'Start rebase' }}</button></div>
  </div></div>
</template>
