<script setup lang="ts">
import { ref } from "vue";
const props = defineProps<{ branches?: string[] }>();
const emit = defineEmits<{ close: []; submit: [path: string, branch: string, newBranch: string | null] }>();
const path = ref("");
const branch = ref("HEAD");
const newBranch = ref("");
</script>
<template>
  <div class="modal-overlay" @click.self="emit('close')"><div class="modal-dialog worktree-dialog">
    <div class="dialog-header"><h3>Add worktree</h3><button class="close-btn" @click="emit('close')">×</button></div>
    <div class="dialog-body"><label>Directory<input v-model="path" placeholder="../worktree" /></label><label>Branch<select v-model="branch"><option>HEAD</option><option v-for="name in props.branches || []" :key="name" :value="name">{{ name }}</option></select></label><label>New branch (optional)<input v-model="newBranch" placeholder="feature/name" /></label></div>
    <div class="dialog-footer"><button class="btn btn-secondary" @click="emit('close')">Cancel</button><button class="btn btn-primary" :disabled="!path" @click="emit('submit', path, branch, newBranch || null)">Add</button></div>
  </div></div>
</template>
<style scoped>.worktree-dialog{width:420px}.dialog-body{display:grid;gap:12px}.dialog-body label{display:grid;gap:4px}.dialog-body input,.dialog-body select{padding:7px}</style>
