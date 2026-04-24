<script setup lang="ts">
import { ref, computed, onMounted, nextTick, useTemplateRef } from "vue";
import { useBranches } from "@/composables/useBranches";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{ remoteBranch: string }>();
const emit = defineEmits<{ close: [] }>();

const { checkoutRemote } = useBranches();
const { dragStyle, onDragStart } = useDraggable();

// Default local name = part after the first "/" (strip "origin/")
const defaultLocal = computed(() => {
  const idx = props.remoteBranch.indexOf("/");
  return idx >= 0 ? props.remoteBranch.slice(idx + 1) : props.remoteBranch;
});

const mode = ref<"create" | "detach">("create");
const localName = ref(defaultLocal.value);
const busy = ref(false);
const error = ref<string | null>(null);

const inputRef = useTemplateRef<HTMLInputElement>("inputRef");

onMounted(async () => {
  await nextTick();
  inputRef.value?.focus();
  inputRef.value?.select();
});

const canCheckout = computed(() => {
  if (mode.value === "detach") return true;
  return localName.value.trim().length > 0;
});

async function handleCheckout() {
  if (!canCheckout.value || busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await checkoutRemote(
      props.remoteBranch,
      mode.value === "create" ? localName.value.trim() : null
    );
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-dialog checkout-remote-dialog" :style="dragStyle" @keydown.enter="handleCheckout">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Check Out</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="title">Check out remote branch</div>
        <div class="subtitle">Create and checkout a new local branch for the selected commit</div>

        <div class="remote-info">
          <span class="label">Remote:</span>
          <span class="remote-name">{{ remoteBranch }}</span>
        </div>

        <label class="option-row">
          <input type="radio" value="create" v-model="mode" />
          <span class="option-label">Create local branch:</span>
          <input
            ref="inputRef"
            v-model="localName"
            type="text"
            class="form-input local-input"
            :disabled="mode !== 'create'"
            @focus="mode = 'create'"
          />
        </label>

        <div class="suboption">
          <label class="track-row">
            <input type="checkbox" checked disabled />
            <span>Track remote branch</span>
          </label>
        </div>

        <label class="option-row">
          <input type="radio" value="detach" v-model="mode" />
          <span class="option-label">Don't create local branch (just work read-only)</span>
        </label>

        <div v-if="error" class="error">{{ error }}</div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button
          class="btn btn-primary"
          :disabled="!canCheckout || busy"
          @click="handleCheckout"
        >
          Checkout
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.checkout-remote-dialog {
  width: 480px;
}

.title {
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}
.subtitle {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin-bottom: 14px;
}

.remote-info {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 14px;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius);
  font-size: var(--font-size-sm);
}
.remote-info .label {
  color: var(--text-muted);
}
.remote-info .remote-name {
  color: var(--text-primary);
  font-family: var(--font-mono, monospace);
}

.option-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  cursor: pointer;
  font-size: var(--font-size-sm);
}

.option-label {
  color: var(--text-primary);
  flex-shrink: 0;
}

.local-input {
  flex: 1;
  min-width: 0;
}

.suboption {
  padding-left: 24px;
  margin-top: 6px;
}

.track-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  cursor: default;
}

.error {
  margin-top: 10px;
  color: var(--red);
  font-size: var(--font-size-sm);
  white-space: pre-wrap;
}
</style>
