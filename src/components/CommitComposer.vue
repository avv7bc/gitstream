<script setup lang="ts">
import { computed, ref } from "vue";
import { useCommit } from "@/composables/useCommit";
import { useFiles } from "@/composables/useFiles";
import { logError } from "@/composables/useProgress";
import { toast } from "@/composables/useToast";

const emit = defineEmits<{ committed: [oid: string | undefined] }>();
const message = ref("");
const amend = ref(false);
const busy = ref(false);
const { commit } = useCommit();
const { files, refresh } = useFiles();
const canCommit = computed(() => !busy.value && !!message.value.trim() && files.value.some((file) => file.staged === "staged" || file.staged === "partial"));

async function submit() {
  if (!canCommit.value) return;
  busy.value = true;
  try {
    const result = await commit(message.value.trim(), amend.value);
    message.value = "";
    await refresh();
    toast({ kind: "success", title: "Commit created" });
    emit("committed", typeof result === "string" ? result : undefined);
  } catch (error) {
    logError(`Commit failed: ${error}`);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <section class="commit-composer" aria-label="Commit composer">
    <textarea v-model="message" rows="2" placeholder="Commit message" :disabled="busy" @keydown.ctrl.enter.prevent="submit" />
    <div class="composer-footer">
      <label><input v-model="amend" type="checkbox" :disabled="busy" /> Amend</label>
      <button class="btn btn-primary" :disabled="!canCommit" @click="submit">{{ busy ? "Committing…" : "Commit" }}</button>
    </div>
  </section>
</template>

<style scoped>
.commit-composer { padding: 8px 10px; border-bottom: 1px solid var(--border-subtle); background: var(--bg-secondary); }
.commit-composer textarea { width: 100%; box-sizing: border-box; resize: vertical; min-height: 42px; border: 1px solid var(--border-subtle); border-radius: var(--radius); background: var(--bg-primary); color: var(--text-primary); padding: 7px; font: inherit; }
.composer-footer { display: flex; justify-content: space-between; align-items: center; margin-top: 6px; color: var(--text-muted); font-size: var(--font-size-xs); }
.composer-footer label { display: flex; gap: 5px; align-items: center; }
</style>
