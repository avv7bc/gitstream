<script setup lang="ts">
import type { RefLabel } from "@/types";

defineProps<{ kind: RefLabel["kind"] }>();
</script>

<template>
  <svg
    v-if="kind === 'tag'"
    class="ref-icon ref-icon--tag"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <path d="M2 9V2h7l5 5-7 7-5-5z" fill="none" stroke="currentColor" stroke-width="1.2" />
    <circle cx="6" cy="6" r="1" fill="currentColor" />
  </svg>
  <svg
    v-else-if="kind === 'stash'"
    class="ref-icon ref-icon--stash"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <rect x="3" y="3" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2" />
    <rect x="3" y="8" width="10" height="3" rx="1" fill="none" stroke="currentColor" stroke-width="1.2" />
  </svg>
  <svg
    v-else-if="kind === 'remote-branch'"
    class="ref-icon ref-icon--remote"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <circle cx="8" cy="4" r="2" fill="none" stroke="currentColor" stroke-width="1.2" />
    <path d="M8 6v4M4 12h8M4 12v-2M12 12v-2" fill="none" stroke="currentColor" stroke-width="1.2" />
  </svg>
  <svg
    v-else
    class="ref-icon ref-icon--branch"
    width="14" height="14" viewBox="0 0 16 16"
  >
    <path
      d="M5 3a2 2 0 100 4 2 2 0 000-4zM5 9a2 2 0 100 4 2 2 0 000-4z"
      fill="none" stroke="currentColor" stroke-width="1.2"
    />
    <path d="M5 7v2" fill="none" stroke="currentColor" stroke-width="1.2" />
  </svg>
</template>

<style scoped>
.ref-icon {
  flex-shrink: 0;
  vertical-align: middle;
}
</style>
