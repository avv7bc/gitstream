<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";

const props = defineProps<{
  target: { oid: string; subject: string } | null;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [payload: { name: string; message: string | null; force: boolean }];
}>();

const name = ref("");
const message = ref("");
const force = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();

// git ref name rules (subset): non-empty, no whitespace or ~^:?*[\, no leading '-'
const INVALID = /[\s~^:?*[\\]/;
const canCreate = computed(() => {
  const t = name.value.trim();
  return t.length > 0 && !t.startsWith("-") && !INVALID.test(t);
});

const targetLabel = computed(() =>
  props.target
    ? `${props.target.oid.slice(0, 9)} ${props.target.subject}`
    : "HEAD",
);

function submit() {
  if (!canCreate.value) return;
  emit("confirm", {
    name: name.value.trim(),
    message: message.value.trim() ? message.value : null,
    force: force.value,
  });
}

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog add-tag-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>Add Tag</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-subhint">Tag will point to: <b>{{ targetLabel }}</b></p>

        <div class="form-group">
          <label class="form-label" for="add-tag-name">Tag Name:</label>
          <input
            id="add-tag-name"
            ref="inputRef"
            v-model="name"
            class="form-input"
            type="text"
            placeholder="v1.0.0"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>

        <div class="form-group">
          <label class="form-label" for="add-tag-msg">Message (empty = lightweight):</label>
          <textarea
            id="add-tag-msg"
            v-model="message"
            class="form-input add-tag-msg"
            rows="3"
            @keydown.escape="$emit('close')"
          />
        </div>

        <label class="add-tag-force">
          <input type="checkbox" v-model="force" />
          Force (overwrite if tag exists)
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">Cancel</button>
        <button class="btn btn-primary" :disabled="!canCreate" @click="submit">
          Create Tag
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-tag-dialog {
  width: 420px;
}
.dialog-subhint {
  font-size: var(--font-size-sm);
  color: var(--text-muted);
  margin-bottom: 12px;
}
.add-tag-msg {
  resize: vertical;
  font-family: inherit;
}
.add-tag-force {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  margin-top: 4px;
}
</style>
