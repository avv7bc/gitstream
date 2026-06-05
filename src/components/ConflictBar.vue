<script setup lang="ts">
import { computed, ref } from "vue";
import { useConflicts } from "@/composables/useConflicts";
import { logError } from "@/composables/useProgress";
import { useFiles } from "@/composables/useFiles";
import { useI18n } from "@/composables/useI18n";

const emit = defineEmits<{ changed: [] }>();

const { repoState, acceptOurs, acceptTheirs, abort, cont } = useConflicts();
const { files } = useFiles();
const { i18n } = useI18n();

const busy = ref(false);

const conflicted = computed(() =>
  files.value.filter((f) => f.state === "conflicted").map((f) => f.path),
);

const label = computed(() => {
  switch (repoState.value) {
    case "merging": return "Merge in progress";
    case "rebasing": return "Rebase in progress";
    case "cherry-picking": return "Cherry-pick in progress";
    case "reverting": return "Revert in progress";
    default: return "";
  }
});

async function run(fn: () => Promise<void>) {
  if (busy.value) return;
  busy.value = true;
  try {
    await fn();
    emit("changed");
  } catch (e) {
    logError(`Operation failed: ${e}`);
  } finally {
    busy.value = false;
  }
}

const resolveOurs = (p: string) => run(() => acceptOurs([p]));
const resolveTheirs = (p: string) => run(() => acceptTheirs([p]));
const onAbort = () => run(abort);
const onContinue = () => run(cont);
</script>

<template>
  <div v-if="repoState !== 'clean'" class="conflict-bar">
    <div class="conflict-head">
      <span class="conflict-state">{{ label }}</span>
      <span v-if="conflicted.length" class="conflict-count">
        {{ conflicted.length }} conflicted file(s)
      </span>
      <span v-else class="conflict-ok">{{ i18n.conflict.noConflicts }}</span>
      <div class="conflict-actions">
        <button class="cb-btn" :disabled="busy" @click="onAbort">{{ i18n.conflict.abort }}</button>
        <button
          class="cb-btn primary"
          :disabled="busy || conflicted.length > 0"
          @click="onContinue"
        >{{ i18n.conflict.continue }}</button>
      </div>
    </div>
    <div v-if="conflicted.length" class="conflict-files">
      <div v-for="path in conflicted" :key="path" class="conflict-file">
        <span class="cf-path" :title="path">{{ path }}</span>
        <span class="cf-btns">
          <button class="cb-btn" :disabled="busy" @click="resolveOurs(path)">{{ i18n.conflict.useOurs }}</button>
          <button class="cb-btn" :disabled="busy" @click="resolveTheirs(path)">{{ i18n.conflict.useTheirs }}</button>
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conflict-bar {
  background: var(--diff-removed-bg, rgba(243, 139, 168, 0.12));
  border-bottom: 1px solid var(--border-subtle);
  font-size: var(--font-size-sm);
  flex-shrink: 0;
}
.conflict-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 10px;
}
.conflict-state {
  font-weight: 600;
  color: var(--red, #f38ba8);
}
.conflict-count {
  color: var(--text-muted);
}
.conflict-ok {
  color: var(--green, #a6e3a1);
}
.conflict-actions {
  margin-left: auto;
  display: flex;
  gap: 6px;
}
.conflict-files {
  padding: 0 10px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 160px;
  overflow: auto;
}
.conflict-file {
  display: flex;
  align-items: center;
  gap: 8px;
}
.cf-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
}
.cf-btns {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}
.cb-btn {
  padding: 2px 10px;
  border-radius: var(--radius);
  border: 1px solid var(--border-subtle);
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: var(--font-size-xs);
  cursor: pointer;
}
.cb-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}
.cb-btn.primary {
  background: var(--blue, #89b4fa);
  color: var(--bg-primary, #1e1e2e);
  border-color: transparent;
}
.cb-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
