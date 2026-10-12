<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useDraggable } from "@/composables/useDraggable";

defineProps<{ currentBranch: string }>();
const emit = defineEmits<{ close: []; confirm: [name: string] }>();
const name = ref("");
const input = ref<HTMLInputElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();
onMounted(() => input.value?.focus());
function submit() {
  const value = name.value.trim();
  if (value) emit("confirm", value);
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')" @keydown.escape="emit('close')" tabindex="-1">
    <div class="modal-dialog spin-off-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart"><h3>Spin off unpublished commits</h3><button class="close-btn" @click="emit('close')">×</button></div>
      <div class="dialog-body">
        <p>Create a new branch from <strong>{{ currentBranch }}</strong> and reset the original branch to its upstream.</p>
        <label class="form-label" for="spin-off-name">New branch name</label>
        <input id="spin-off-name" ref="input" v-model="name" class="form-input" @keydown.enter="submit" />
      </div>
      <div class="dialog-footer"><button class="btn btn-secondary" @click="emit('close')">Cancel</button><button class="btn btn-primary" :disabled="!name.trim()" @click="submit">Create branch</button></div>
    </div>
  </div>
</template>

<style scoped>
.spin-off-dialog { width: 440px; }
.dialog-body p { color: var(--text-secondary); line-height: 1.45; }
</style>
