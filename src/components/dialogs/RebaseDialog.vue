<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useRepo } from "@/composables/useRepo";
import { useBranches } from "@/composables/useBranches";
import { useDraggable } from "@/composables/useDraggable";
import type { BranchInfo, MergePreview } from "@/types";

const props = defineProps<{ target: BranchInfo; candidates?: BranchInfo[] }>();
const emit = defineEmits<{ close: []; confirm: [branch: string] }>();
const { repoPath } = useRepo();
const { branches } = useBranches();
const { dragStyle, onDragStart } = useDraggable();
const selected = ref(props.target.name);
const preview = ref<MergePreview | null>(null);
const previewSupported = ref(true);
const loading = ref(false);
const options = computed(() => props.candidates?.length ? props.candidates : branches.value.filter((b) => !b.is_current));
async function loadPreview() {
  preview.value = null;
  if (!repoPath.value || !selected.value) return;
  loading.value = true;
  try {
    const result = await invoke<MergePreview | null>("preview_merge", { repoPath: repoPath.value, branch: selected.value });
    preview.value = result;
    previewSupported.value = result !== null;
  } catch { previewSupported.value = false; }
  finally { loading.value = false; }
}
watch(() => props.target.name, (name) => { selected.value = name; void loadPreview(); }, { immediate: true });
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-dialog rebase-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart"><h3>Rebase current branch</h3><button class="close-btn" @click="emit('close')">×</button></div>
      <div class="dialog-body">
        <label class="form-label" for="rebase-branch">Upstream branch</label>
        <select id="rebase-branch" v-model="selected" class="form-select" @change="loadPreview"><option v-for="branch in options" :key="branch.name" :value="branch.name">{{ branch.name }}</option></select>
        <p v-if="loading" class="dialog-hint">Checking rebase preview…</p>
        <div v-else-if="previewSupported && preview" class="preview" :class="{ conflict: !preview.clean }"><strong>{{ preview.clean ? 'No merge conflicts expected' : 'Conflicts may need resolving' }}</strong><span v-if="preview.conflicted_files.length">{{ preview.conflicted_files.join(', ') }}</span></div>
        <p v-else class="dialog-hint">Preview requires Git 2.38 or newer.</p>
      </div>
      <div class="dialog-footer"><button class="btn btn-secondary" @click="emit('close')">Cancel</button><button class="btn btn-primary" :disabled="!selected" @click="emit('confirm', selected)">Rebase</button></div>
    </div>
  </div>
</template>

<style scoped>
.rebase-dialog { width: 430px; }
.preview { display: grid; gap: 4px; margin-top: 14px; padding: 9px; border: 1px solid var(--border-subtle); color: var(--green); font-size: var(--font-size-sm); }
.preview.conflict { color: var(--red); }
.preview span { color: var(--text-muted); overflow-wrap: anywhere; }
</style>
