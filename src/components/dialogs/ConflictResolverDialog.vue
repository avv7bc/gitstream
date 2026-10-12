<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import { useSyncScroll } from "@/composables/useSyncScroll";

const props = defineProps<{ path: string }>();
const emit = defineEmits<{ close: []; saved: [] }>();
const { repoPath } = useRepo();
interface Side { content: string | null; binary: boolean; size: number; oid?: string | null; mode?: string | null }
const sides = ref<{ base: Side; ours: Side; theirs: Side; merged_draft: string | null; chunks?: { ours: string; base?: string | null; theirs: string }[] } | null>(null);
const result = ref("");
const mark = ref(true);
const busy = ref(false);
const binary = computed(() => !!sides.value && (sides.value.ours.binary || sides.value.theirs.binary));
const conflictCount = computed(() => sides.value?.chunks?.length ?? 0);
const resultRef = ref<HTMLTextAreaElement | null>(null);
const { leftPanelRef, rightPanelRef, setupSync } = useSyncScroll();
function nextConflict() {
  const textarea = resultRef.value;
  const text = textarea?.value;
  if (!textarea || !text) return;
  const marker = text.indexOf("<<<<<<<", textarea.selectionEnd || 0);
  const start = marker >= 0 ? marker : text.indexOf("<<<<<<<");
  if (start < 0) return;
  textarea.focus();
  textarea.setSelectionRange(start, Math.min(text.length, start + 7));
}

onMounted(async () => {
  if (!repoPath.value) return;
  sides.value = await invoke("get_conflict_sides", { repoPath: repoPath.value, path: props.path });
  result.value = sides.value?.merged_draft ?? sides.value?.ours.content ?? "";
  requestAnimationFrame(() => { setupSync(); });
});

function take(side: "ours" | "theirs" | "base") {
  const content = sides.value?.[side].content;
  if (content != null) result.value = content;
}

async function takeBinary(side: "ours" | "theirs") {
  if (!repoPath.value || !binary.value) return;
  busy.value = true;
  try {
    await invoke(side === "ours" ? "do_accept_ours" : "do_accept_theirs", { repoPath: repoPath.value, files: [props.path] });
    emit("saved");
  } finally { busy.value = false; }
}

async function save() {
  if (!repoPath.value || binary.value) return;
  busy.value = true;
  try {
    await invoke("save_resolved", { repoPath: repoPath.value, path: props.path, content: result.value, markResolved: mark.value });
    emit("saved");
  } finally { busy.value = false; }
}

async function openMergetool() {
  if (!repoPath.value || binary.value) return;
  busy.value = true;
  try {
    await invoke("open_mergetool", { repoPath: repoPath.value, path: props.path });
    emit("saved");
  } finally { busy.value = false; }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-dialog resolver-dialog">
      <div class="dialog-header"><h3>Resolve {{ path }}</h3><button class="close-btn" @click="emit('close')">×</button></div>
      <div v-if="sides" class="dialog-body">
        <div v-if="binary" class="binary-choice">
          <p>Binary conflict: choose the complete file to keep.</p>
          <div class="binary-actions">
            <button class="btn btn-secondary" :disabled="busy || !sides.ours.oid" @click="takeBinary('ours')">Use ours ({{ sides.ours.size }} bytes)</button>
            <button class="btn btn-secondary" :disabled="busy || !sides.theirs.oid" @click="takeBinary('theirs')">Use theirs ({{ sides.theirs.size }} bytes)</button>
          </div>
        </div>
        <div v-else class="resolver-grid">
          <div class="conflict-nav"><strong v-if="conflictCount">{{ conflictCount }} conflict block<span v-if="conflictCount !== 1">s</span></strong><span><button v-if="conflictCount" class="btn btn-secondary" @click="nextConflict">Next conflict</button><button class="btn btn-secondary" :disabled="busy" @click="openMergetool">Open external mergetool</button></span></div>
          <section><h4>Base</h4><pre>{{ sides.base.content ?? "(none)" }}</pre><button class="btn btn-secondary" @click="take('base')">Use base</button></section>
          <section><h4>Ours</h4><pre ref="leftPanelRef">{{ sides.ours.content ?? "(none)" }}</pre><button class="btn btn-secondary" @click="take('ours')">Use ours</button></section>
          <section><h4>Theirs</h4><pre ref="rightPanelRef">{{ sides.theirs.content ?? "(none)" }}</pre><button class="btn btn-secondary" @click="take('theirs')">Use theirs</button></section>
          <section class="result-column"><h4>Result</h4><textarea ref="resultRef" v-model="result" rows="16" aria-label="Resolved result" /></section>
        </div>
        <label v-if="!binary"><input v-model="mark" type="checkbox" /> Mark resolved (git add)</label>
      </div>
      <div class="dialog-footer"><button class="btn btn-secondary" @click="emit('close')">Cancel</button><button v-if="!binary" class="btn btn-primary" :disabled="busy || !sides" @click="save">Save</button></div>
    </div>
  </div>
</template>

<style scoped>
.resolver-dialog { width: min(1100px, 96vw); }
.resolver-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
.result-column { grid-column: 1 / -1; }
.conflict-nav { grid-column: 1 / -1; display: flex; align-items: center; justify-content: space-between; color: var(--warn); }
.resolver-grid pre, .resolver-grid textarea { width: 100%; min-height: 180px; box-sizing: border-box; background: var(--bg-tertiary); color: var(--text-primary); font: var(--font-mono); white-space: pre-wrap; overflow: auto; border: 1px solid var(--border-subtle); }
.resolver-grid textarea { min-height: 240px; padding: 8px; }
h4 { margin: 0 0 4px; }
.binary-choice { padding: 18px 0; }
.binary-actions { display: flex; gap: 8px; margin-top: 14px; }
</style>
