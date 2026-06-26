<script setup lang="ts">
import { computed } from "vue";
import { useSync } from "@/composables/useSync";
import { useConflicts } from "@/composables/useConflicts";
import { useI18n } from "@/composables/useI18n";
import type { Remedy } from "@/types";

const emit = defineEmits<{ changed: [] }>();

const { situation, busy, applyRemedy, dismiss } = useSync();
const { repoState } = useConflicts();
const { i18n } = useI18n();

// Во время merge/rebase управление берёт ConflictBar — sync-бар прячем,
// чтобы не показывать две панели разом (например после pull с конфликтом).
const visible = computed(() => situation.value !== null && repoState.value === "clean");

// Текст ситуации/решений живёт во фронтовом i18n, индексируется по id из backend.
const sync = computed(() => i18n.value.sync);

const title = computed(() => {
  const s = situation.value;
  if (!s) return "";
  const t = sync.value.situations[s.id];
  return t ? t.title : s.id;
});

const detail = computed(() => {
  const s = situation.value;
  if (!s) return "";
  const t = sync.value.situations[s.id];
  if (!t) return "";
  // Подставляем число коммитов remote (behind) в шаблон вида "...{n}...".
  return t.detail.replace("{ahead}", String(s.ahead)).replace("{behind}", String(s.behind));
});

function remedyText(r: Remedy) {
  return sync.value.remedies[r.id] || { label: r.id, detail: "" };
}

async function onRemedy(r: Remedy) {
  await applyRemedy(r);
  emit("changed");
}
</script>

<template>
  <div v-if="visible && situation" class="sync-bar" :class="situation.severity">
    <div class="sync-head">
      <span class="sync-icon">⟳</span>
      <div class="sync-text">
        <span class="sync-title">{{ title }}</span>
        <span class="sync-detail">{{ detail }}</span>
      </div>
      <button class="sync-dismiss" :title="i18n.sync.dismiss" @click="dismiss">✕</button>
    </div>
    <div class="sync-remedies">
      <button
        v-for="r in situation.remedies"
        :key="r.id"
        class="sync-btn"
        :class="[r.danger, { recommended: r.recommended }]"
        :disabled="busy"
        :title="remedyText(r).detail"
        @click="onRemedy(r)"
      >
        {{ remedyText(r).label }}
        <span v-if="r.recommended" class="sync-rec">✓</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.sync-bar {
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-subtle);
  border-left: 3px solid var(--yellow, #f9e2af);
  font-size: var(--font-size-sm);
  flex-shrink: 0;
}
.sync-bar.danger {
  border-left-color: var(--red, #f38ba8);
}
.sync-head {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 7px 10px 4px;
}
.sync-icon {
  color: var(--yellow, #f9e2af);
  font-size: 15px;
  line-height: 1.2;
  flex-shrink: 0;
}
.sync-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}
.sync-title {
  font-weight: 600;
  color: var(--text-primary);
}
.sync-detail {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
}
.sync-dismiss {
  margin-left: auto;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  font-size: var(--font-size-xs);
  flex-shrink: 0;
}
.sync-dismiss:hover {
  color: var(--text-primary);
}
.sync-remedies {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 2px 10px 8px 34px;
}
.sync-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 11px;
  border-radius: var(--radius);
  border: 1px solid var(--border-subtle);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: var(--font-size-xs);
  cursor: pointer;
}
.sync-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}
/* Рекомендуемое — акцент; caution/danger подкрашивают рамку под уровень риска. */
.sync-btn.recommended {
  border-color: var(--accent, var(--blue));
  font-weight: 600;
}
.sync-btn.caution {
  border-color: var(--yellow, #f9e2af);
}
.sync-btn.danger {
  border-color: var(--red, #f38ba8);
  color: var(--red, #f38ba8);
}
.sync-rec {
  color: var(--green, #a6e3a1);
}
.sync-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
