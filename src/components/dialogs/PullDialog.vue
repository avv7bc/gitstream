<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useBranches } from "@/composables/useBranches";
import { useRepo } from "@/composables/useRepo";
import { useDraggable } from "@/composables/useDraggable";

const emit = defineEmits<{
  close: [];
  pull: [remote: string, rebase: boolean];
}>();

const { remotes, branches } = useBranches();
const { repoInfo } = useRepo();

const selectedRemote = ref("origin");
const pullMode = ref<"merge" | "rebase">("merge");

const currentBranch = computed(() => repoInfo.value?.current_branch ?? "");
const currentBranchInfo = computed(() => branches.value.find((b) => b.is_current));
const behindCount = computed(() => currentBranchInfo.value?.behind ?? 0);

const dialogRef = ref<HTMLElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();

function handlePull() {
  emit("pull", selectedRemote.value, pullMode.value === "rebase");
}

onMounted(() => {
  dialogRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog pull-dialog" :style="dragStyle" @keydown.enter.prevent="handlePull" @keydown.escape="$emit('close')" tabindex="-1" ref="dialogRef">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Pull</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label class="form-label">Remote</label>
          <select v-model="selectedRemote" class="form-select">
            <option v-for="r in remotes" :key="r" :value="r">{{ r }}</option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">Mode</label>
          <div class="radio-group">
            <label class="radio-label">
              <input type="radio" v-model="pullMode" value="merge" />
              <span>Merge</span>
            </label>
            <label class="radio-label">
              <input type="radio" v-model="pullMode" value="rebase" />
              <span>Rebase</span>
            </label>
          </div>
        </div>

        <p class="behind-info">
          {{ behindCount }} commits behind
          <strong>{{ selectedRemote }}/{{ currentBranch }}</strong>
        </p>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button class="btn btn-primary" @click="handlePull">Pull</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pull-dialog {
  width: 400px;
}

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  cursor: pointer;
}
.radio-label input {
  accent-color: var(--accent);
}

.behind-info {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin-top: 8px;
}
.behind-info strong {
  color: var(--text-secondary);
}
</style>
