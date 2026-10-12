<script setup lang="ts">
const emit = defineEmits<{ open: []; clone: []; init: [] }>();
</script>

<template>
  <main class="welcome-screen">
    <section class="welcome-card" aria-labelledby="welcome-title">
      <div class="welcome-mark" aria-hidden="true">G</div>
      <h1 id="welcome-title">Welcome to GitStream</h1>
      <p>Open a repository, clone one by URL, or initialize a new project.</p>
      <div class="welcome-actions">
        <button class="welcome-action primary" type="button" @click="emit('open')"><span class="action-icon">⌂</span><span><strong>Open repository…</strong><small>Add a folder with .git to the list</small></span></button>
        <button class="welcome-action" type="button" @click="emit('clone')"><span class="action-icon">↓</span><span><strong>Clone repository…</strong><small>HTTPS or SSH URL</small></span></button>
        <button class="welcome-action" type="button" @click="emit('init')"><span class="action-icon">＋</span><span><strong>Initialize repository…</strong><small>Create a new Git repository</small></span></button>
      </div>
      <p class="welcome-hint">You can also drag a project folder here.</p>
    </section>
  </main>
</template>

<style scoped>
.welcome-screen { display: grid; flex: 1; min-height: 0; place-items: center; background: var(--bg-primary); }
.welcome-card { width: min(680px, calc(100vw - 48px)); padding: 42px 48px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg-secondary); box-shadow: 0 18px 55px rgba(0, 0, 0, .28); }
.welcome-mark { display: grid; width: 48px; height: 48px; place-items: center; border-radius: 10px; background: var(--accent-hover); color: #fff; font: 600 24px/1 var(--font-mono); }
.welcome-card h1 { margin-top: 16px; font-size: 24px; font-weight: 600; }
.welcome-card > p { margin-top: 5px; color: var(--text-secondary); }
.welcome-actions { display: grid; gap: 8px; margin-top: 24px; }
.welcome-action { display: flex; min-height: 58px; align-items: center; gap: 12px; padding: 0 14px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-surface); text-align: left; cursor: pointer; }
.welcome-action:hover { border-color: var(--accent); background: var(--bg-hover); }
.welcome-action.primary { border-color: color-mix(in srgb, var(--accent) 55%, var(--border)); background: color-mix(in srgb, var(--accent) 9%, var(--bg-surface)); }
.action-icon { display: grid; width: 30px; height: 30px; place-items: center; border-radius: 6px; background: var(--bg-hover); color: var(--accent); font: 18px/1 var(--font-mono); }
.primary .action-icon { background: var(--accent-hover); color: #fff; }
.welcome-action span:last-child { display: flex; flex-direction: column; gap: 2px; }
.welcome-action strong { font-weight: 500; }
.welcome-action small { color: var(--text-secondary); font-size: 11.5px; }
.welcome-hint { color: var(--text-muted) !important; font-size: 12px; }
</style>
