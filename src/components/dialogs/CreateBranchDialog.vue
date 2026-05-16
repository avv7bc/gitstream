<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{
  defaultStart: string | null;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [payload: { name: string; startPoint: string | null; checkout: boolean }];
}>();

const name = ref("");
const startPoint = ref(props.defaultStart ?? "");
const checkout = ref(true);
const inputRef = ref<HTMLInputElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();

const INVALID = /[\s~^:?*[\\]/;
const canCreate = computed(() => {
  const t = name.value.trim();
  return t.length > 0 && !t.startsWith("-") && !INVALID.test(t);
});

function submit() {
  if (!canCreate.value) return;
  emit("confirm", {
    name: name.value.trim(),
    startPoint: startPoint.value.trim() ? startPoint.value.trim() : null,
    checkout: checkout.value,
  });
}

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog create-branch-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Create Branch</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label class="form-label" for="cb-name">Branch Name:</label>
          <input
            id="cb-name"
            ref="inputRef"
            v-model="name"
            class="form-input"
            type="text"
            placeholder="feature/my-branch"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>

        <div class="form-group">
          <label class="form-label" for="cb-start">Start Point (empty = HEAD):</label>
          <input
            id="cb-start"
            v-model="startPoint"
            class="form-input"
            type="text"
            placeholder="HEAD"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>

        <label class="cb-opt">
          <input type="checkbox" v-model="checkout" />
          Check out new branch
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button class="btn btn-primary" :disabled="!canCreate" @click="submit">
          Create
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.create-branch-dialog {
  width: 420px;
}
.cb-opt {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  margin-top: 4px;
}
</style>
