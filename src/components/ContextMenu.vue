<script setup lang="ts">
import type { ContextMenuItem } from "@/commands/types";

withDefaults(defineProps<{
  x: number;
  y: number;
  items: ContextMenuItem[];
}>(), { });

const emit = defineEmits<{
  select: [id: string];
  close: [];
}>();

function select(item: ContextMenuItem) {
  if (!item.id || item.disabled || item.separator) return;
  emit("select", item.id);
}
</script>

<template>
  <Teleport to="body">
    <div class="context-menu-layer">
      <div
        class="ctx-menu"
        :style="{ left: `${x}px`, top: `${y}px` }"
        role="menu"
        @click.stop
      >
        <template v-for="(item, index) in items" :key="item.id ?? `separator-${index}`">
          <div v-if="item.separator" class="ctx-separator" role="separator" />
          <button
            v-else
            class="ctx-item"
            :class="{ 'ctx-danger': item.danger }"
            :disabled="item.disabled"
            role="menuitem"
            @click="select(item)"
          >
            <span>{{ item.label }}</span>
            <span v-if="item.shortcut" class="ctx-hotkey">{{ item.shortcut }}</span>
          </button>
        </template>
      </div>
      <div class="ctx-backdrop" @click="emit('close')" @contextmenu.prevent="emit('close')" />
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu-layer { position: fixed; inset: 0; z-index: 2200; pointer-events: none; }
.ctx-menu {
  position: fixed;
  z-index: 2201;
  min-width: 180px;
  padding: 4px 0;
  border: 1px solid var(--border-subtle);
  border-radius: 4px;
  background: var(--bg-secondary);
  box-shadow: 0 5px 18px rgb(0 0 0 / 35%);
  pointer-events: auto;
}
.ctx-item {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 6px 10px;
  border: 0;
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}
.ctx-item:hover:not(:disabled) { background: var(--bg-hover); }
.ctx-item:disabled { color: var(--text-muted); cursor: default; }
.ctx-danger { color: var(--red, #e06060); }
.ctx-hotkey { color: var(--text-muted); font-size: 11px; }
.ctx-separator { height: 1px; margin: 4px 0; background: var(--border-subtle); }
.ctx-backdrop { position: fixed; inset: 0; z-index: 2200; pointer-events: auto; }
</style>
