<script setup lang="ts">
import { computed } from "vue";
import { useLog } from "@/composables/useLog";

const { commits, selectedCommit } = useLog();

const commit = computed(() => {
  if (!selectedCommit.value || selectedCommit.value === "__worktree__") return null;
  return commits.value.find((c) => c.oid === selectedCommit.value) ?? null;
});

const dateFormatter = new Intl.DateTimeFormat("ru-RU", {
  day: "2-digit",
  month: "2-digit",
  year: "numeric",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

const formattedDate = computed(() => {
  if (!commit.value?.date) return "";
  const d = new Date(commit.value.date);
  if (isNaN(d.getTime())) return commit.value.date;
  return dateFormatter.format(d).replace(",", "");
});
</script>

<template>
  <div class="commit-details" @contextmenu.prevent>
    <div class="panel-title-bar">
      <span class="panel-title">Commit</span>
      <span v-if="commit" class="panel-title-date">{{ formattedDate }}</span>
    </div>

    <div class="details-body" v-if="commit">
      <div class="details-top">
        <div class="details-info">
          <div class="detail-row">
            <span class="detail-label">Commit</span>
            <span class="detail-value hash">{{ commit.short_oid }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">by</span>
            <span class="detail-value author">{{ commit.author }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">parent</span>
            <span class="detail-value hash link">{{ commit.parents[0].slice(0, 8) }}</span>
          </div>
        </div>

        <div v-if="commit.refs.length" class="commit-refs">
          <span
            v-for="ref in commit.refs"
            :key="ref.name"
            class="ref-badge"
            :class="'ref-' + ref.kind"
          >
            {{ ref.name }}
          </span>
        </div>
      </div>

      <p class="commit-msg">{{ commit.message }}</p>
    </div>
  </div>
</template>

<style scoped>
.commit-details {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.panel-title-bar {
  display: flex;
  align-items: center;
  height: 28px;
  padding: 0 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-subtle);
  user-select: none;
  flex-shrink: 0;
}
.panel-title {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.panel-title-date {
  margin-left: auto;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
}

.details-body {
  padding: 10px 12px;
  overflow-y: auto;
  font-size: var(--font-size-sm);
  flex: 1;
  display: flex;
  flex-direction: column;
}

.details-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.details-info {
  flex: 1;
  min-width: 0;
}

.detail-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;
  line-height: 1.6;
}

.detail-label {
  color: var(--text-muted);
  min-width: 48px;
  flex-shrink: 0;
}

.detail-value {
  color: var(--text-primary);
}
.detail-value.hash {
  font-family: var(--font-mono);
  color: var(--accent);
  font-size: var(--font-size-xs);
}
.detail-value.hash.link {
  cursor: pointer;
  text-decoration: underline;
  text-decoration-color: transparent;
}
.detail-value.hash.link:hover {
  text-decoration-color: var(--accent);
}
.detail-value.author {
  font-weight: 400;
}
.commit-msg {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
  white-space: pre-wrap;
  line-height: 1.4;
  margin-top: 8px;
  flex: 1;
}

.commit-refs {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  flex-shrink: 0;
}

.ref-badge {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: var(--font-size-xs);
  font-weight: 600;
  line-height: 16px;
}
.ref-local-branch {
  background: rgba(166, 227, 161, 0.2);
  color: var(--green);
}
.ref-remote-branch {
  background: rgba(137, 180, 250, 0.15);
  color: var(--blue);
}
.ref-tag {
  background: rgba(249, 226, 175, 0.15);
  color: var(--yellow);
}
.ref-head {
  background: rgba(243, 139, 168, 0.2);
  color: var(--red);
}
</style>
