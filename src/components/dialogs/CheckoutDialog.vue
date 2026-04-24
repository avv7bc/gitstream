<script setup lang="ts">
import { ref, computed } from "vue";
import { useBranches } from "@/composables/useBranches";
import { useDraggable } from "@/composables/useDraggable";
import { highlight } from "@/utils/highlight";

const emit = defineEmits<{ close: [] }>();

const { branches, checkout } = useBranches();
const { dragStyle, onDragStart } = useDraggable();

const search = ref("");
const selectedBranch = ref<string | null>(null);

const filteredBranches = computed(() => {
  const q = search.value.toLowerCase();
  if (!q) return branches.value;
  return branches.value.filter((b) =>
    b.name.toLowerCase().includes(q) ||
    (b.upstream && b.upstream.toLowerCase().includes(q))
  );
});

async function handleCheckout() {
  if (!selectedBranch.value) return;
  await checkout(selectedBranch.value);
  emit("close");
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog checkout-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Checkout Branch</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <input
          v-model="search"
          type="text"
          placeholder="Search branches..."
          class="form-input search-input"
        />

        <div class="branch-list">
          <div
            v-for="branch in filteredBranches"
            :key="branch.name"
            class="branch-option"
            :class="{
              selected: selectedBranch === branch.name,
              current: branch.is_current,
            }"
            @click="selectedBranch = branch.name"
          >
            <span class="branch-option-name" v-html="highlight(branch.name, search)" />
            <span v-if="branch.is_current" class="current-badge">(current)</span>
            <span v-if="branch.is_remote" class="remote-hint">will create local branch</span>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button
          class="btn btn-primary"
          :disabled="!selectedBranch || branches.find(b => b.name === selectedBranch)?.is_current"
          @click="handleCheckout"
        >
          Checkout
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.checkout-dialog {
  width: 400px;
}

.search-input {
  width: 100%;
  margin-bottom: 8px;
}

.branch-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-tertiary);
}

.branch-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  cursor: pointer;
  font-size: var(--font-size-sm);
}
.branch-option:hover {
  background: var(--bg-hover);
}
.branch-option.selected {
  background: var(--bg-surface);
}
.branch-option.current {
  color: var(--accent);
}

.branch-option-name {
  flex: 1;
}

.current-badge {
  font-size: var(--font-size-xs);
  color: var(--accent);
}

.remote-hint {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  font-style: italic;
}
</style>
