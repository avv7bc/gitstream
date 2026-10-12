<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { allCommands } from "@/commands/registry";
import type { Command, CommandContext } from "@/commands/types";
import { logError } from "@/composables/useProgress";

const props = defineProps<{ context: CommandContext }>();
const emit = defineEmits<{ close: [] }>();
const query = ref("");
const input = ref<HTMLInputElement | null>(null);

function fuzzyMatch(value: string, queryValue: string): boolean {
  let cursor = 0;
  for (const char of queryValue) {
    cursor = value.indexOf(char, cursor);
    if (cursor === -1) return false;
    cursor += 1;
  }
  return true;
}

const candidates = computed(() => {
  const q = query.value.trim().toLowerCase();
  return allCommands().filter((command) => {
    if (command.visible && !command.visible(props.context)) return false;
    if (command.enabled && !command.enabled(props.context)) return false;
    if (!q) return true;
    return fuzzyMatch(`${command.label} ${command.description ?? ""} ${command.id}`.toLowerCase(), q);
  });
});

async function execute(command: Command) {
  try {
    await command.execute(props.context);
    emit("close");
  } catch (error) {
    logError(String(error));
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    emit("close");
  } else if (event.key === "Enter" && candidates.value[0]) {
    event.preventDefault();
    void execute(candidates.value[0]);
  }
}

void nextTick(() => input.value?.focus());
</script>

<template>
  <div class="modal-overlay palette-overlay" @click.self="emit('close')">
    <div class="command-palette" role="dialog" aria-modal="true" aria-label="Command Palette">
      <input ref="input" v-model="query" class="palette-input" placeholder="Search commands…" @keydown="onKeydown" />
      <div class="palette-list">
        <button v-for="command in candidates" :key="command.id" class="palette-item" @click="execute(command)">
          <span>{{ command.label }}</span>
          <kbd v-if="Array.isArray(command.shortcut) ? command.shortcut[0] : command.shortcut">{{ Array.isArray(command.shortcut) ? command.shortcut[0] : command.shortcut }}</kbd>
        </button>
        <p v-if="!candidates.length" class="palette-empty">No matching commands</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.palette-overlay { align-items: flex-start; padding-top: 12vh; }
.command-palette { width: min(560px, calc(100vw - 32px)); overflow: hidden; border: 1px solid var(--border-subtle); border-radius: var(--radius-lg, 8px); background: var(--bg-secondary); box-shadow: 0 16px 45px rgba(0, 0, 0, .35); }
.palette-input { width: 100%; box-sizing: border-box; border: 0; border-bottom: 1px solid var(--border-subtle); outline: none; padding: 14px 16px; background: var(--bg-primary); color: var(--text-primary); font: inherit; }
.palette-list { max-height: 55vh; overflow: auto; padding: 6px; }
.palette-item { display: flex; width: 100%; justify-content: space-between; border: 0; border-radius: var(--radius); padding: 9px 10px; background: transparent; color: var(--text-primary); text-align: left; cursor: pointer; }
.palette-item:hover { background: var(--bg-hover); }
.palette-item kbd { color: var(--text-muted); font-size: var(--font-size-xs); }
.palette-empty { margin: 12px 10px; color: var(--text-muted); font-size: var(--font-size-sm); }
</style>
