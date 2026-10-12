<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import type { CommitInfo } from "@/types";
import ConfirmDialog from "./ConfirmDialog.vue";
const props = defineProps<{ commits: CommitInfo[]; base?: string | null }>();
const emit = defineEmits<{ close: []; done: [] }>();
const { repoPath, repoInfo } = useRepo();
const actions = ["pick", "reword", "edit", "squash", "fixup", "drop"] as const;
const rangeError = ref("");
function rangeCommits() {
  const result: CommitInfo[] = [];
  let oid: string | undefined = repoInfo.value?.head_oid;
  while (oid && oid !== props.base) {
    const commit = props.commits.find((item) => item.oid === oid);
    if (!commit) { rangeError.value = "The complete rebase range is not loaded. Load more history before rebasing."; break; }
    if (commit.parents.length > 1) { rangeError.value = "Interactive rebase requires a linear history range."; break; }
    result.push(commit);
    oid = commit.parents[0];
  }
  if (props.base && oid !== props.base && !rangeError.value) rangeError.value = "The base is outside the current branch history.";
  return result.reverse();
}
const plan = ref(rangeCommits().map((commit) => ({ sha: commit.oid, action: "pick" as typeof actions[number], message: null as string | null })));
const busy = ref(false);
const autostash = ref(true);
const updateRefs = ref(true);
const messageFor = ref<string | null>(null);
const editingItem = computed(() => plan.value.find((item) => item.sha === messageFor.value));
const pushedCommits = computed(() => plan.value.filter((item) => props.commits.find((commit) => commit.oid === item.sha)?.unpushed === false));
const confirmPushed = ref(false);
const allowPushed = ref(false);
async function submit() {
  if (!repoPath.value) return;
  if (pushedCommits.value.length && !allowPushed.value) { confirmPushed.value = true; return; }
  busy.value = true;
  try {
    // The backend currently enables both safety flags for every plan. Keep the
    // controls visible so the operation's policy is explicit to the user.
    await invoke("do_rebase_plan", { repoPath: repoPath.value, base: props.base ?? null, plan: plan.value, allowPushed: allowPushed.value }); emit("done");
  } finally { busy.value = false; }
}
</script>
<template>
  <div class="modal-overlay" @click.self="emit('close')"><div class="modal-dialog" style="width: 640px">
    <div class="dialog-header"><h3>Interactive rebase</h3><button class="close-btn" @click="emit('close')">×</button></div>
    <div class="dialog-body"><p>Arrange commits and choose an action.</p><p v-if="props.base" class="hint">Base: <code>{{ props.base.slice(0, 12) }}</code> · only commits after this base are included.</p>
      <div class="options"><label><input v-model="autostash" type="checkbox" disabled /> Autostash (enabled)</label><label><input v-model="updateRefs" type="checkbox" disabled /> Update refs (enabled)</label><span class="hint">These safety policies are enforced by the backend.</span></div>
      <p v-if="rangeError" class="hint">{{ rangeError }}</p><ol><li v-for="(item, index) in plan" :key="item.sha"><select v-model="item.action"><option v-for="action in actions" :key="action" :value="action">{{ action }}</option></select><code>{{ item.sha.slice(0, 8) }}</code><span>{{ props.commits.find(c => c.oid === item.sha)?.message }}</span><button v-if="item.action === 'reword'" class="edit" @click="messageFor = item.sha">Edit message</button><button v-if="index" @click="[plan[index - 1], plan[index]] = [plan[index], plan[index - 1]]">↑</button><button v-if="index + 1 < plan.length" @click="[plan[index], plan[index + 1]] = [plan[index + 1], plan[index]]">↓</button></li></ol>
      <div v-if="editingItem" class="message-editor"><label>New commit message<textarea v-model="editingItem.message" rows="3" /></label><button @click="messageFor = null">Done</button></div>
    </div>
    <div class="dialog-footer"><button class="btn btn-secondary" @click="emit('close')">Cancel</button><button class="btn btn-primary" :disabled="busy || !!rangeError || !plan.length" @click="submit">{{ busy ? 'Starting…' : 'Start rebase' }}</button></div>
    <ConfirmDialog v-if="confirmPushed" message="This plan rewrites commits already reachable from a remote-tracking branch. Continue and allow a force push later?" confirm-label="Allow rewrite" danger @close="confirmPushed = false" @confirm="allowPushed = true; confirmPushed = false; submit()" />
  </div></div>
</template>
