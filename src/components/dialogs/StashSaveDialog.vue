<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

const emit = defineEmits<{
  close: [];
  confirm: [payload: { message: string | null; includeUntracked: boolean; scope: "all" | "selected" | "staged" }];
}>();

const props = withDefaults(defineProps<{ selectedPaths?: string[] }>(), { selectedPaths: () => [] });

const message = ref("");
const includeUntracked = ref(true);
const scope = ref<"all" | "selected" | "staged">("all");
const hasSelected = computed(() => props.selectedPaths.length > 0);
const inputRef = ref<HTMLInputElement | null>(null);
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

function submit() {
  emit("confirm", {
    message: message.value.trim() ? message.value.trim() : null,
    includeUntracked: includeUntracked.value,
    scope: scope.value,
  });
}

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog stash-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.stashSave.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label class="form-label" for="stash-msg">{{ i18n.dialog.stashSave.msgLabel }}</label>
          <input
            id="stash-msg"
            ref="inputRef"
            v-model="message"
            class="form-input"
            type="text"
            placeholder="WIP: ..."
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>

        <div class="stash-scope" role="radiogroup" aria-label="Stash scope">
          <label><input v-model="scope" type="radio" value="all" /> All changes</label>
          <label :class="{ disabled: !hasSelected }"><input v-model="scope" type="radio" value="selected" :disabled="!hasSelected" /> Selected files<span v-if="hasSelected"> ({{ selectedPaths.length }})</span></label>
          <label><input v-model="scope" type="radio" value="staged" /> Staged only</label>
        </div>

        <label class="stash-opt">
          <input type="checkbox" v-model="includeUntracked" :disabled="scope === 'staged'" />
          {{ i18n.dialog.stashSave.includeUntracked }}
        </label>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.stashSave.cancel }}</button>
        <button class="btn btn-primary" @click="submit">{{ i18n.dialog.stashSave.stash }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stash-dialog {
  width: 420px;
}
.stash-opt {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  margin-top: 4px;
}
.stash-scope {
  display: grid;
  gap: 6px;
  margin: 12px 0 4px;
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}
.stash-scope label { display: flex; align-items: center; gap: 6px; }
.stash-scope label.disabled { opacity: .55; }
</style>
