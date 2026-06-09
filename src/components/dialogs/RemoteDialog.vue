<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";

const props = defineProps<{
  mode: "add" | "editUrl" | "rename";
  name?: string;
  url?: string;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [name: string, url: string];
}>();

const name = ref(props.name ?? "");
const url = ref(props.url ?? "");
const nameRef = ref<HTMLInputElement | null>(null);
const urlRef = ref<HTMLInputElement | null>(null);

const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

const showName = computed(() => props.mode !== "editUrl");
const showUrl = computed(() => props.mode !== "rename");
const nameReadonly = computed(() => props.mode === "editUrl");

const title = computed(() => {
  const d = i18n.value.dialog.remote;
  if (props.mode === "add") return d.titleAdd;
  if (props.mode === "rename") return d.titleRename;
  return d.titleEditUrl;
});

// remote name: непусто, без пробелов/'/' и без ведущего '-'
const nameValid = computed(() => {
  const n = name.value.trim();
  return n.length > 0 && !n.startsWith("-") && !/[\s/]/.test(n);
});
const urlValid = computed(() => {
  const u = url.value.trim();
  return u.length > 0 && !u.startsWith("-");
});

const canConfirm = computed(() => {
  if (props.mode === "rename") return nameValid.value && name.value.trim() !== props.name;
  if (props.mode === "editUrl") return urlValid.value && url.value.trim() !== props.url;
  return nameValid.value && urlValid.value;
});

function submit() {
  if (!canConfirm.value) return;
  emit("confirm", name.value.trim(), url.value.trim());
}

onMounted(() => {
  if (props.mode === "editUrl") {
    urlRef.value?.focus();
    urlRef.value?.select();
  } else {
    nameRef.value?.focus();
    nameRef.value?.select();
  }
});
</script>

<template>
  <div class="modal-overlay" @click.self="$emit('close')" @keydown.escape="$emit('close')" tabindex="-1">
    <div class="modal-dialog remote-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ title }}</h3>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div v-if="showName" class="form-group">
          <label class="form-label" for="remote-name-input">{{ i18n.dialog.remote.nameLabel }}</label>
          <input
            id="remote-name-input"
            ref="nameRef"
            v-model="name"
            class="form-input"
            type="text"
            placeholder="origin"
            :readonly="nameReadonly"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>
        <div v-if="showUrl" class="form-group">
          <label class="form-label" for="remote-url-input">{{ i18n.dialog.remote.urlLabel }}</label>
          <input
            id="remote-url-input"
            ref="urlRef"
            v-model="url"
            class="form-input"
            type="text"
            placeholder="https://github.com/user/repo.git"
            @keydown.enter="submit"
            @keydown.escape="$emit('close')"
          />
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn btn-secondary" @click="$emit('close')">{{ i18n.dialog.remote.cancel }}</button>
        <button class="btn btn-primary" :disabled="!canConfirm" @click="submit">{{ i18n.dialog.remote.ok }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.remote-dialog {
  width: 460px;
}
.form-input[readonly] {
  opacity: 0.6;
  cursor: default;
}
</style>
