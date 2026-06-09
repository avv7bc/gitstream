<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

const props = defineProps<{
  branch: string;
  current: string | null;
  remoteBranches: string[];
}>();

const emit = defineEmits<{
  close: [];
  confirm: [upstream: string | null];
}>();

// "" — снять трекинг
const selected = ref(props.current ?? "");
const selectRef = ref<HTMLSelectElement | null>(null);

const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

function submit() {
  emit("confirm", selected.value === "" ? null : selected.value);
}

onMounted(() => selectRef.value?.focus());
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog upstream-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ i18n.dialog.setUpstream.title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <p class="dialog-subhint">{{ i18n.dialog.setUpstream.subhint }} <b>{{ branch }}</b></p>
        <div class="form-group">
          <label class="form-label" for="upstream-select">{{ i18n.dialog.setUpstream.label }}</label>
          <select id="upstream-select" ref="selectRef" v-model="selected" class="form-input">
            <option value="">{{ i18n.dialog.setUpstream.none }}</option>
            <option v-for="rb in remoteBranches" :key="rb" :value="rb">{{ rb }}</option>
          </select>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.setUpstream.cancel }}</button>
        <button class="btn btn-primary" @click="submit">{{ i18n.dialog.setUpstream.ok }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.upstream-dialog {
  width: 440px;
}
select.form-input {
  cursor: pointer;
}
</style>
