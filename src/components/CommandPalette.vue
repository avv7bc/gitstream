<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { allCommands } from "@/commands/registry";
import { paletteCommandState } from "@/commands/palette";
import type { Command, CommandContext } from "@/commands/types";
import { logError } from "@/composables/useProgress";
import { fuzzyScore, rememberRecent } from "@/composables/useFuzzy";

const props = defineProps<{ context: CommandContext }>();
const emit = defineEmits<{ close: []; select: [entry: PaletteEntry] }>();
const query = ref("");
const input = ref<HTMLInputElement | null>(null);
const selectedIndex = ref(0);

type PaletteEntry = { kind: "command" | "ref" | "file" | "commit"; id: string; label: string; detail?: string; command?: Command; enabled: boolean };
watch(query, () => { selectedIndex.value = 0; });
function recentIds(): string[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const value: unknown = JSON.parse(localStorage.getItem("gitstream.palette.recent") ?? "[]");
    return Array.isArray(value) ? value.filter((id): id is string => typeof id === "string") : [];
  } catch { return []; }
}

function disabledReason(command: Command): string {
  if (!props.context.repoPath && command.id !== "settings.open") return "Open a repository first";
  if (command.id.startsWith("file.")) return "Select a changed file first";
  if (command.id.startsWith("branch.") || command.id === "remote.branch.delete") return "Select a branch first";
  if (command.id.startsWith("remote.")) return "Select a remote first";
  return "Unavailable in current context";
}

const candidates = computed<PaletteEntry[]>(() => {
  const raw = query.value.trim();
  const commandMode = raw.startsWith(">");
  const q = (commandMode ? raw.slice(1) : raw).trim();
  const commands: PaletteEntry[] = allCommands()
    .filter((command) => paletteCommandState(command, props.context, commandMode).listed)
    .map((command): PaletteEntry => ({ kind: "command", id: command.id, label: command.label, detail: command.description, command, enabled: paletteCommandState(command, props.context, commandMode).enabled }))
    .filter((entry) => !q || fuzzyScore(`${entry.label} ${entry.detail ?? ""} ${entry.id}`, q) >= 0)
    .sort((a, b) => fuzzyScore(`${b.label} ${b.id}`, q) - fuzzyScore(`${a.label} ${a.id}`, q));
  if (commandMode) return commands;
  const contextItems = (key: string, kind: PaletteEntry["kind"]) => (Array.isArray(props.context[key]) ? props.context[key] as unknown[] : [])
    .filter((value): value is string | { id?: string; label?: string; oid?: string; message?: string } => typeof value === "string" || (typeof value === "object" && value != null))
    .map((value) => typeof value === "string" ? { label: value, id: value } : { label: value.label ?? value.message ?? value.oid ?? "", id: value.id ?? value.oid ?? value.label ?? value.message ?? "" })
    .map((value) => ({ kind, id: value.id, label: value.label, enabled: true }));
  const entries = [...commands, ...contextItems("refs", "ref"), ...contextItems("files", "file"), ...contextItems("commits", "commit")];
  const recent = recentIds();
  return entries
    .filter((entry) => !q || fuzzyScore(`${entry.label} ${entry.id}`, q) >= 0)
    .sort((a, b) => { const ai = recent.indexOf(a.id); const bi = recent.indexOf(b.id); return (ai < 0 ? 999 : ai) - (bi < 0 ? 999 : bi); })
    .slice(0, 80);
});

async function execute(command: Command) {
  try {
    await command.execute(props.context);
    emit("close");
  } catch (error) {
    logError(String(error));
  }
}

function selectEntry(entry: PaletteEntry) {
  if (!entry.enabled) return;
  rememberRecent("gitstream.palette.recent", entry.id);
  if (entry.command) void execute(entry.command);
  else { emit("select", entry); emit("close"); }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    emit("close");
  } else if (event.key === "ArrowDown") {
    event.preventDefault(); selectedIndex.value = Math.min(selectedIndex.value + 1, candidates.value.length - 1);
  } else if (event.key === "ArrowUp") {
    event.preventDefault(); selectedIndex.value = Math.max(0, selectedIndex.value - 1);
  } else if (event.key === "Enter" && candidates.value[selectedIndex.value]) {
    event.preventDefault();
    selectEntry(candidates.value[selectedIndex.value]);
  }
}

void nextTick(() => input.value?.focus());
</script>

<template>
  <div class="modal-overlay palette-overlay" @click.self="emit('close')">
    <div class="command-palette" role="dialog" aria-modal="true" aria-label="Command Palette">
      <input ref="input" v-model="query" class="palette-input" placeholder="> commands · refs · files · commits" @keydown="onKeydown" />
      <div class="palette-list">
        <button v-for="(entry, index) in candidates" :key="`${entry.kind}:${entry.id}`" class="palette-item" :class="{ active: index === selectedIndex }" :disabled="!entry.enabled" @click="selectEntry(entry)">
          <span><strong v-if="entry.kind !== 'command'" class="palette-kind">{{ entry.kind }}</strong>{{ entry.label }} <small v-if="!entry.enabled">— {{ disabledReason(entry.command!) }}</small></span>
          <kbd v-if="entry.command && (Array.isArray(entry.command.shortcut) ? entry.command.shortcut[0] : entry.command.shortcut)">{{ Array.isArray(entry.command.shortcut) ? entry.command.shortcut[0] : entry.command.shortcut }}</kbd>
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
.palette-item.active { background: var(--bg-hover); }
.palette-item:disabled { color: var(--text-muted); cursor: not-allowed; opacity: .7; }
.palette-kind { margin-right: 8px; color: var(--accent); font-size: 10px; text-transform: uppercase; }
.palette-item kbd { color: var(--text-muted); font-size: var(--font-size-xs); }
.palette-empty { margin: 12px 10px; color: var(--text-muted); font-size: var(--font-size-sm); }
</style>
