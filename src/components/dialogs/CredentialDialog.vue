<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useDraggable } from "@/composables/useDraggable";
import { useI18n } from "@/composables/useI18n";
import { useAuth } from "@/composables/useAuth";

const { current, submitAskpass, cancelAskpass } = useAuth();
const { dragStyle, onDragStart } = useDraggable();
const { i18n } = useI18n();

const value = ref("");
const remember = ref(true);
const inputRef = ref<HTMLInputElement | null>(null);

const c = computed(() => i18n.value.dialog.credential);

const isConfirm = computed(() => current.value?.kind === "confirm");
const isMasked = computed(
  () => current.value?.kind === "password" || current.value?.kind === "passphrase",
);
const showRemember = computed(
  () => current.value?.kind === "password" || current.value?.kind === "username",
);

const title = computed(() => {
  switch (current.value?.kind) {
    case "passphrase":
      return c.value.titlePassphrase;
    case "confirm":
      return c.value.titleConfirm;
    default:
      return c.value.titleAuth;
  }
});

const fieldLabel = computed(() => {
  switch (current.value?.kind) {
    case "username":
      return c.value.usernameLabel;
    case "password":
      return c.value.passwordLabel;
    case "passphrase":
      return c.value.passphraseLabel;
    default:
      return c.value.valueLabel;
  }
});

// Сброс и фокус при появлении нового запроса.
watch(
  current,
  async (req) => {
    if (!req) return;
    value.value = "";
    await nextTick();
    inputRef.value?.focus();
  },
  { immediate: true },
);

function submit() {
  void submitAskpass(value.value, remember.value);
}

function confirmYes() {
  void submitAskpass("yes", false);
}

function confirmNo() {
  void submitAskpass("no", false);
}

function cancel() {
  void cancelAskpass();
}
</script>

<template>
  <div
    v-if="current"
    class="modal-overlay"
    @keydown.escape="cancel"
    tabindex="-1"
  >
    <div class="modal-dialog credential-dialog" :style="dragStyle">
      <div class="dialog-header" @mousedown="onDragStart">
        <h3>{{ title }}</h3>
        <button class="close-btn" @click="cancel">
          <svg width="14" height="14" viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      <div class="dialog-body">
        <div v-if="current.host" class="cred-meta">
          <span class="cred-meta-label">{{ c.hostLabel }}</span>
          <span class="cred-meta-value">{{ current.host }}</span>
        </div>
        <div v-if="current.key_path" class="cred-meta">
          <span class="cred-meta-label">{{ c.keyLabel }}</span>
          <span class="cred-meta-value">{{ current.key_path }}</span>
        </div>

        <!-- Подтверждение host-key (yes/no) -->
        <template v-if="isConfirm">
          <p class="cred-prompt">{{ current.prompt }}</p>
        </template>

        <!-- Ввод значения -->
        <template v-else>
          <div class="form-group">
            <label class="form-label" for="cred-value-input">{{ fieldLabel }}</label>
            <input
              id="cred-value-input"
              ref="inputRef"
              v-model="value"
              class="form-input"
              :type="isMasked ? 'password' : 'text'"
              autocomplete="off"
              @keydown.enter="submit"
              @keydown.escape="cancel"
            />
          </div>
          <label v-if="showRemember" class="cred-remember">
            <input type="checkbox" v-model="remember" />
            <span>{{ c.remember }}</span>
          </label>
        </template>
      </div>

      <div class="dialog-footer">
        <template v-if="isConfirm">
          <button class="btn btn-secondary" @click="confirmNo">{{ c.no }}</button>
          <button class="btn btn-primary" @click="confirmYes">{{ c.yes }}</button>
        </template>
        <template v-else>
          <button class="btn btn-secondary" @click="cancel">{{ c.cancel }}</button>
          <button class="btn btn-primary" @click="submit">{{ c.ok }}</button>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.credential-dialog {
  width: 440px;
}
.cred-meta {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 12px;
}
.cred-meta-label {
  color: var(--text-secondary, #888);
  min-width: 48px;
}
.cred-meta-value {
  word-break: break-all;
}
.cred-prompt {
  margin: 4px 0 0;
  white-space: pre-wrap;
}
.cred-remember {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  font-size: 13px;
  cursor: pointer;
}
</style>
