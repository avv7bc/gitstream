<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useBranches } from "@/composables/useBranches";
import { useDraggable } from "@/composables/useDraggable";
import type { BranchInfo } from "@/types";

const props = defineProps<{ target: BranchInfo; candidates?: BranchInfo[] }>();
const emit = defineEmits<{ close: []; confirm: [branch: string] }>();
const { branches } = useBranches();
const { dragStyle, onDragStart } = useDraggable();
const selected = ref(props.target.name);
const options = computed(() => props.candidates?.length ? props.candidates : branches.value.filter((b) => !b.is_current));
watch(() => props.target.name, (name) => { selected.value = name; }, { immediate: true });
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-dialog rebase-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart"><h3>Rebase current branch</h3><button class="close-btn" @click="emit('close')">×</button></div>
      <div class="dialog-body">
        <label class="form-label" for="rebase-branch">Upstream branch</label>
        <select id="rebase-branch" v-model="selected" class="form-select"><option v-for="branch in options" :key="branch.name" :value="branch.name">{{ branch.name }}</option></select>
        <p class="dialog-hint">Rebase preview is unavailable because it requires per-commit simulation.</p>
      </div>
      <div class="dialog-footer"><button class="btn btn-secondary" @click="emit('close')">Cancel</button><button class="btn btn-primary" :disabled="!selected" @click="emit('confirm', selected)">Rebase</button></div>
    </div>
  </div>
</template>

<style scoped>
.rebase-dialog { width: 430px; }
</style>
