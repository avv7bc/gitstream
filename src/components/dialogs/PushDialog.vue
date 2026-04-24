<script setup lang="ts">
import { ref, computed } from "vue";
import { useBranches } from "@/composables/useBranches";
import { useRepo } from "@/composables/useRepo";
import { useDraggable } from "@/composables/useDraggable";

const emit = defineEmits<{
  close: [];
  push: [remote: string, force: boolean];
}>();
const { remotes } = useBranches();
const { repoInfo } = useRepo();
const { dragStyle, onDragStart } = useDraggable();

const selectedRemote = ref("origin");
const forcePush = ref(false);

const currentBranch = computed(() => repoInfo.value?.current_branch ?? "");

function handlePush() {
  emit("push", selectedRemote.value, forcePush.value);
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog push-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Push</h3>
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

        <p class="push-info">
          Push branch <strong>{{ currentBranch }}</strong> to
          <strong>{{ selectedRemote }}/{{ currentBranch }}</strong>
        </p>

        <label class="checkbox-label">
          <input type="checkbox" v-model="forcePush" />
          <span>Force push</span>
        </label>
        <p v-if="forcePush" class="force-warning">
          Force push will overwrite remote history. Use with caution!
        </p>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button class="btn btn-primary" :class="{ danger: forcePush }" @click="handlePush">
          {{ forcePush ? "Force Push" : "Push" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.push-dialog {
  width: 400px;
}

.push-info {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  margin: 8px 0;
}
.push-info strong {
  color: var(--text-primary);
}

.force-warning {
  font-size: var(--font-size-xs);
  color: var(--red);
  margin-top: 4px;
  padding: 4px 8px;
  background: rgba(243, 139, 168, 0.1);
  border-radius: var(--radius);
}

.btn.danger {
  background: var(--red);
  color: var(--bg-primary);
}
</style>
