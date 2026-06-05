<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from "vue";
import { useLog } from "@/composables/useLog";
import { logError } from "@/composables/useProgress";
import { useBranches } from "@/composables/useBranches";
import RefIcon from "@/components/RefIcon.vue";
import { useFiles } from "@/composables/useFiles";
import { useRepo } from "@/composables/useRepo";
import type { RefLabel, GraphLine } from "@/types";
import { highlight } from "@/utils/highlight";

const emit = defineEmits<{
  commit: [];
  discard: [];
  createTag: [target: { oid: string; subject: string }];
  changed: [];
  squash: [payload: { oids: string[] }];
  reword: [payload: { oid: string; message: string; isHead: boolean }];
}>();

const { commits, selectedCommit, hasMore, isLoadingMore, loadMore, resetTo, revertCommit, cherryPick } = useLog();
const { branches } = useBranches();
const currentBranch = computed(() => branches.value.find((b) => b.is_current));
function isCurrentBranchRow(c: { refs: { kind: string }[] }): boolean {
  return c.refs.some((r) => r.kind === "current-branch");
}
const GRAPH_PALETTE = ["--blue", "--green", "--purple", "--teal", "--orange", "--yellow"];
const GRAPH_COL_W = 14;
const GRAPH_PAD = 8;
const GRAPH_ROW_H = 24;

function laneX(c: number): number {
  return GRAPH_PAD + c * GRAPH_COL_W;
}
function laneColor(colorIdx: number): string {
  return `var(${GRAPH_PALETTE[colorIdx % GRAPH_PALETTE.length]})`;
}
function linePath(l: GraphLine, topRow = false): string {
  const x1 = laneX(l.from_column);
  const x2 = laneX(l.to_column);
  const mid = GRAPH_ROW_H / 2;
  if (l.style === "straight") {
    // На самой верхней строке strand начинается от узла, а не от верха
    // ячейки — чтобы граф заканчивался на коммите (нет «палки» вверх к WT).
    return `M ${x1} ${topRow ? mid : 0} L ${x1} ${GRAPH_ROW_H}`;
  }
  if (l.style === "tip") {
    // Lane появляется на этом коммите впервые — над dot ничего нет.
    return `M ${x1} ${mid} L ${x1} ${GRAPH_ROW_H}`;
  }
  if (l.style === "fork") {
    const cy = (mid + GRAPH_ROW_H) / 2;
    return `M ${x1} ${mid} C ${x1} ${cy} ${x2} ${cy} ${x2} ${GRAPH_ROW_H}`;
  }
  if (topRow) return "";
  const cy = mid / 2;
  return `M ${x1} 0 C ${x1} ${cy} ${x2} ${cy} ${x2} ${mid}`;
}
const graphMaxCol = computed(() => {
  let m = 0;
  for (const c of commits.value) {
    if (c.column > m) m = c.column;
    for (const l of c.lines) {
      if (l.from_column > m) m = l.from_column;
      if (l.to_column > m) m = l.to_column;
    }
  }
  return m;
});
const graphColW = computed(() => Math.max(80, laneX(graphMaxCol.value) + 16));

// Индекс HEAD-коммита в отфильтрованном списке: ищем по ref-у текущей
// локальной ветки. С `git log --all` HEAD больше не обязательно сверху —
// WT-строка должна стоять прямо перед ним, а не всегда на самом верху.
const headIdx = computed(() => {
  return filteredCommits.value.findIndex((c) =>
    c.refs.some((r) => r.kind === "current-branch")
  );
});
const wtInline = computed(() => headIdx.value > 0);
const wtAtTop = computed(() => headIdx.value <= 0);
// Колонка для WT, когда он наверху: либо колонка HEAD (если headIdx === 0),
// либо колонка самого первого коммита (detached/нет HEAD в списке).
const wtCol = computed(() => {
  if (headIdx.value === 0) return filteredCommits.value[0]?.column ?? 0;
  return filteredCommits.value[0]?.column ?? 0;
});
// Lane-ы, рисуемые сквозь WT-строку между commits[headIdx-1] и commits[headIdx].
// Берём из `lines` HEAD-коммита: straight и merge-* идут из верхней половины
// ячейки → lane был активен над HEAD → рисуем full-vertical через WT.
// Колонка самого HEAD: если её lane не был активен сверху (tip-коммит),
// рисуем только нижнюю половину под dot, иначе будет «обрубок» вверх.
const wtInlineLines = computed<{ col: number; full: boolean }[]>(() => {
  if (!wtInline.value) return [];
  const next = filteredCommits.value[headIdx.value];
  if (!next) return [];
  const activeAbove = new Set<number>();
  for (const ln of next.lines) {
    if (ln.style === "straight" || ln.style === "merge-left" || ln.style === "merge-right") {
      activeAbove.add(ln.from_column);
    }
  }
  const out: { col: number; full: boolean }[] = [];
  for (const c of activeAbove) out.push({ col: c, full: true });
  if (!activeAbove.has(next.column)) {
    out.push({ col: next.column, full: false });
  }
  return out.sort((a, b) => a.col - b.col);
});
const { files } = useFiles();
const { repoPath } = useRepo();

const graphFilter = ref("");

const repoName = computed(() => {
  if (!repoPath.value) return "";
  const parts = repoPath.value.replace(/\/+$/, "").split("/");
  return parts[parts.length - 1];
});

// Range selection
const rangeAnchor = ref<string | null>(null);
const selectedOids = ref<string[]>([]);

const headOid = computed(() => filteredCommits.value[0]?.oid ?? null);
const rangeIncludesHead = computed(() =>
  !!headOid.value && selectedOids.value.includes(headOid.value)
);

function handleCommitClick(oid: string, e: MouseEvent) {
  if (
    e.shiftKey &&
    rangeAnchor.value &&
    rangeAnchor.value !== "__worktree__"
  ) {
    const anchorIdx = filteredCommits.value.findIndex(c => c.oid === rangeAnchor.value);
    const clickIdx  = filteredCommits.value.findIndex(c => c.oid === oid);
    if (anchorIdx !== -1 && clickIdx !== -1) {
      const lo = Math.min(anchorIdx, clickIdx);
      const hi = Math.max(anchorIdx, clickIdx);
      selectedOids.value = filteredCommits.value.slice(lo, hi + 1).map(c => c.oid);
      selectedCommit.value = oid;
      return;
    }
  }
  rangeAnchor.value = oid;
  selectedOids.value = [oid];
  selectedCommit.value = oid;
}

function handleWtClick() {
  rangeAnchor.value = "__worktree__";
  selectedOids.value = [];
  selectedCommit.value = "__worktree__";
}

const ctxMenu = ref<{ x: number; y: number } | null>(null);
const ctxCommitOid = ref<string | null>(null);

function onContextMenu(e: MouseEvent, oid: string) {
  e.preventDefault();
  ctxMenu.value = { x: e.clientX, y: e.clientY };
  ctxCommitOid.value = oid;
}

function closeCtxMenu() {
  ctxMenu.value = null;
  ctxCommitOid.value = null;
}

function ctxAction(action: "commit" | "discard") {
  closeCtxMenu();
  if (action === "commit") emit("commit");
  else emit("discard");
}

function emitAddTag(oid: string | null) {
  if (!oid || oid === "__worktree__") return;
  const c = commits.value.find((x) => x.oid === oid);
  emit("createTag", { oid, subject: (c?.message ?? "").split('\n')[0] });
}

function ctxCreateTag() {
  const oid = ctxCommitOid.value;
  closeCtxMenu();
  emitAddTag(oid);
}

async function ctxReset(mode: "soft" | "mixed" | "hard") {
  const oid = ctxCommitOid.value;
  closeCtxMenu();
  if (!oid || oid === "__worktree__") return;
  if (mode === "hard" && !window.confirm(
    "Reset --hard discards all uncommitted changes. Continue?"
  )) return;
  try {
    await resetTo(oid, mode);
    emit("changed");
  } catch (e) {
    logError(`Reset failed: ${e}`);
  }
}

async function runRevert(oid: string | null) {
  if (!oid || oid === "__worktree__") return;
  try {
    await revertCommit(oid, false);
  } catch (e) {
    logError(`Revert failed (resolve conflicts if any): ${e}`);
  } finally {
    emit("changed");
  }
}

async function ctxRevert() {
  const oid = ctxCommitOid.value;
  closeCtxMenu();
  await runRevert(oid);
}

async function runCherryPick(oid: string | null) {
  if (!oid || oid === "__worktree__") return;
  try {
    await cherryPick(oid);
  } catch (e) {
    logError(`Cherry-pick failed (resolve conflicts if any): ${e}`);
  } finally {
    emit("changed");
  }
}

async function ctxCherryPick() {
  const oid = ctxCommitOid.value;
  closeCtxMenu();
  await runCherryPick(oid);
}

function ctxSquash() {
  const oids = selectedOids.value.slice(); // newest→oldest (filteredCommits order)
  closeCtxMenu();
  emit("squash", { oids });
}

function ctxReword() {
  const oid = ctxCommitOid.value;
  closeCtxMenu();
  if (!oid || oid === "__worktree__") return;
  const c = commits.value.find((x) => x.oid === oid);
  if (!c) return;
  const isHead = commits.value[0]?.oid === oid;
  emit("reword", { oid, message: c.message, isHead });
}

function triggerReword(oid: string | null) {
  if (!oid || oid === "__worktree__") return;
  const c = commits.value.find((x) => x.oid === oid);
  if (!c) return;
  const isHead = commits.value[0]?.oid === oid;
  emit("reword", { oid, message: c.message, isHead });
}

const ctxIsWorkingTree = computed(() => ctxCommitOid.value === "__worktree__");
const canSquash = computed(() =>
  selectedOids.value.length >= 2 && rangeIncludesHead.value
);

const changedCount = computed(() => files.value.length);
const isWorkingTreeSelected = computed(() => selectedCommit.value === "__worktree__");

function selectWorkingTree() {
  selectedCommit.value = "__worktree__";
}

function navigateCommits(direction: 'up' | 'down'): void {
  const commits = filteredCommits.value;
  if (commits.length === 0) return;

  // WT-якорь: индекс коммита, перед которым стоит WT-строка
  // (либо headIdx при inline, либо 0 при wtAtTop).
  const wtAnchor = wtInline.value ? headIdx.value : 0;
  const wtIsBefore = wtAtTop.value || wtInline.value; // WT всегда есть

  const isWt = selectedCommit.value === "__worktree__";
  const curIdx = isWt ? -1 : commits.findIndex((c) => c.oid === selectedCommit.value);

  if (direction === 'up') {
    if (isWt) {
      // ArrowUp от WT: на коммит непосредственно над WT
      if (wtInline.value && wtAnchor > 0) {
        selectedCommit.value = commits[wtAnchor - 1].oid;
      }
      return;
    }
    if (curIdx < 0) return;
    // Если над текущим коммитом стоит WT — прыгаем на WT
    if (wtIsBefore && curIdx === wtAnchor) {
      selectWorkingTree();
      return;
    }
    if (curIdx > 0) selectedCommit.value = commits[curIdx - 1].oid;
    return;
  }

  // direction === 'down'
  if (isWt) {
    if (wtAnchor < commits.length) selectedCommit.value = commits[wtAnchor].oid;
    return;
  }
  if (curIdx < 0) return;
  // Если под текущим коммитом стоит WT — прыгаем на WT
  if (wtInline.value && curIdx + 1 === wtAnchor) {
    selectWorkingTree();
    return;
  }
  if (curIdx < commits.length - 1) selectedCommit.value = commits[curIdx + 1].oid;
}

function handleKeyDown(e: KeyboardEvent): void {
  if (e.key === 'F2') {
    e.preventDefault();
    triggerReword(selectedCommit.value);
    return;
  }
  if (e.shiftKey && e.key === 'F7') {
    e.preventDefault();
    emitAddTag(selectedCommit.value);
    return;
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    navigateCommits('up');
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    navigateCommits('down');
  }
}

// Глобальный обработчик горячих клавиш — работает независимо от фокуса,
// но не перехватывает ввод в полях.
function isEditableTarget(el: EventTarget | null): boolean {
  const t = el as HTMLElement | null;
  if (!t) return false;
  const tag = t.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || t.isContentEditable;
}

function onGlobalKeyDown(e: KeyboardEvent): void {
  if (isEditableTarget(e.target)) return;
  if (e.shiftKey && e.ctrlKey && (e.key === 'M' || e.key === 'm')) {
    e.preventDefault();
    runCherryPick(selectedCommit.value);
  } else if (e.shiftKey && e.altKey && (e.key === 'M' || e.key === 'm' || e.code === 'KeyM')) {
    e.preventDefault();
    runRevert(selectedCommit.value);
  }
}

const graphBodyRef = ref<HTMLElement | null>(null);
const loadMoreSentinelRef = ref<HTMLElement | null>(null);
let infiniteIO: IntersectionObserver | null = null;

// Догрузка истории при прокрутке: следим за sentinel-элементом в конце
// списка и подтягиваем следующую страницу, когда он попадает в поле
// зрения. Фильтр пропускаем — при пустом результате наблюдатель сорвался
// бы в цикл «грузим всё подряд».
function shouldAutoLoad(): boolean {
  return hasMore.value && !isLoadingMore.value && !graphFilter.value;
}

async function onSentinelIntersect() {
  if (!shouldAutoLoad()) return;
  await loadMore();
  // Если sentinel всё ещё в поле зрения (страница маленькая, окно
  // высокое), IntersectionObserver не сработает повторно сам — нужен
  // ре-observe.
  if (shouldAutoLoad() && infiniteIO && loadMoreSentinelRef.value) {
    infiniteIO.unobserve(loadMoreSentinelRef.value);
    infiniteIO.observe(loadMoreSentinelRef.value);
  }
}

function setupInfiniteScroll() {
  if (infiniteIO) {
    infiniteIO.disconnect();
    infiniteIO = null;
  }
  if (!graphBodyRef.value || !loadMoreSentinelRef.value) return;
  infiniteIO = new IntersectionObserver(
    (entries) => {
      if (entries.some((e) => e.isIntersecting)) onSentinelIntersect();
    },
    { root: graphBodyRef.value, rootMargin: "300px 0px" },
  );
  infiniteIO.observe(loadMoreSentinelRef.value);
}

// Sentinel может скрываться/появляться (v-if по hasMore/фильтру) — на
// каждое его пересоздание нужен новый observe, иначе старый DOM-узел
// уйдёт в утиль и догрузка больше не сработает.
watch(loadMoreSentinelRef, () => setupInfiniteScroll(), { flush: "post" });

onMounted(() => {
  window.addEventListener("keydown", onGlobalKeyDown);
  setupInfiniteScroll();
});
onUnmounted(() => {
  window.removeEventListener("keydown", onGlobalKeyDown);
  infiniteIO?.disconnect();
  infiniteIO = null;
});

defineExpose({ focus: () => graphBodyRef.value?.focus() });

watch(selectedCommit, (oid) => {
  if (!oid || oid === "__worktree__") return;
  const el = graphBodyRef.value?.querySelector<HTMLElement>(".graph-row.selected");
  el?.scrollIntoView({ block: "nearest" });
}, { flush: "post" });

// Коммиты выше HEAD — это история, существующая только на remote-ах
// (после fetch). Визуально приглушаем, чтобы взгляд цеплялся за «свои».
function isAboveHead(idx: number): boolean {
  const hi = headIdx.value;
  return hi > 0 && idx < hi;
}

// «Unpushed» — коммиты, которые есть локально на HEAD, но ещё не запушены
// в upstream. Берём `ahead` из текущей ветки и помечаем подряд идущие
// коммиты в lane HEAD, начиная с самого HEAD. С `git log --all` старая
// логика «всё до первого remote-ref-а» больше не работает — remote-ref
// теперь чаще всего наверху.
function isUnpushed(idx: number): boolean {
  const ahead = currentBranch.value?.ahead ?? 0;
  if (ahead <= 0) return false;
  const hi = headIdx.value;
  if (hi < 0) return false;
  if (idx < hi || idx >= hi + ahead) return false;
  const headCol = filteredCommits.value[hi]?.column;
  return headCol !== undefined && filteredCommits.value[idx]?.column === headCol;
}

const maxAuthorLen = computed(() => {
  let m = 0;
  for (const c of commits.value) {
    if (c.author.length > m) m = c.author.length;
  }
  return m;
});

const filteredCommits = computed(() => {
  const q = graphFilter.value.toLowerCase();
  if (!q) return commits.value;
  return commits.value.filter((c) =>
    c.message.toLowerCase().includes(q) ||
    c.author.toLowerCase().includes(q) ||
    c.author_email.toLowerCase().includes(q) ||
    c.oid.toLowerCase().includes(q) ||
    c.short_oid.toLowerCase().includes(q) ||
    c.date.toLowerCase().includes(q) ||
    c.refs.some((r) => r.name.toLowerCase().includes(q))
  );
});

function refClass(r: RefLabel): string {
  return `ref-label ref-${r.kind}`;
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    const hours = String(d.getHours()).padStart(2, "0");
    const mins = String(d.getMinutes()).padStart(2, "0");
    return `${year}-${month}-${day} ${hours}:${mins}`;
  } catch {
    return iso;
  }
}
</script>

<template>
  <div class="commit-graph" :style="{ '--author-col-w': (maxAuthorLen + 2) + 'ch', '--graph-col-w': graphColW + 'px' }" @contextmenu.prevent>
    <div class="panel-title-bar">
      <span class="panel-title">Graph<template v-if="repoName"> | {{ repoName }}</template></span>
      <div class="graph-toolbar">
        <input v-model="graphFilter" type="text" placeholder="Filter" class="graph-filter" />
      </div>
    </div>


    <div ref="graphBodyRef" class="graph-body" tabindex="0" role="listbox" @keydown="handleKeyDown">
      <!-- Working Tree / Index row (когда HEAD сверху списка или вовсе отсутствует) -->
      <div
        v-if="wtAtTop"
        class="graph-row wt-row"
        :class="{ selected: isWorkingTreeSelected }"
        @mousedown.shift.prevent
        @click="handleWtClick"
        @dblclick="emit('commit')"
        @contextmenu="onContextMenu($event, '__worktree__')"
      >
        <div class="graph-col">
          <svg :width="graphColW" height="24" class="graph-svg">
            <line
              v-if="filteredCommits.length"
              :x1="laneX(wtCol)"
              y1="12"
              :x2="laneX(wtCol)"
              y2="24"
              :stroke="laneColor(wtCol)"
              stroke-width="2"
              stroke-opacity="0.35"
            />
            <circle :cx="laneX(wtCol)" cy="12" r="5" fill="var(--green)" stroke="var(--bg-primary)" stroke-width="1.5" />
          </svg>
        </div>
        <div class="message-col">
          <span class="wt-label">Working Tree/Index ({{ changedCount > 0 ? changedCount + ' changed' : 'clean' }})</span>
        </div>
        <span class="author-col"></span>
        <span class="date-col"></span>
      </div>

      <template v-for="(commit, idx) in filteredCommits" :key="commit.oid">
        <!-- Inline WT-строка — прямо перед HEAD (после --all HEAD редко на самом верху) -->
        <div
          v-if="wtInline && idx === headIdx"
          class="graph-row wt-row"
          :class="{ selected: isWorkingTreeSelected }"
          @mousedown.shift.prevent
          @click="handleWtClick"
          @dblclick="emit('commit')"
          @contextmenu="onContextMenu($event, '__worktree__')"
        >
          <div class="graph-col">
            <svg :width="graphColW" height="24" class="graph-svg">
              <line
                v-for="ln in wtInlineLines"
                :key="ln.col"
                :x1="laneX(ln.col)"
                :y1="ln.full ? 0 : 12"
                :x2="laneX(ln.col)"
                y2="24"
                :stroke="laneColor(ln.col)"
                stroke-width="2"
                stroke-opacity="0.35"
              />
              <circle
                :cx="laneX(commit.column)"
                cy="12"
                r="5"
                fill="var(--green)"
                stroke="var(--bg-primary)"
                stroke-width="1.5"
              />
            </svg>
          </div>
          <div class="message-col">
            <span class="wt-label">Working Tree/Index ({{ changedCount > 0 ? changedCount + ' changed' : 'clean' }})</span>
          </div>
          <span class="author-col"></span>
          <span class="date-col"></span>
        </div>

        <div
          class="graph-row"
          :class="{ selected: selectedCommit === commit.oid, unpushed: isUnpushed(idx), 'above-head': isAboveHead(idx), 'in-range': selectedOids.includes(commit.oid) && selectedOids.length > 1 }"
          @mousedown.shift.prevent
          @click="handleCommitClick(commit.oid, $event)"
          @contextmenu="onContextMenu($event, commit.oid)"
        >
        <!-- Graph column with SVG lane lines -->
        <div class="graph-col">
          <svg :width="graphColW" height="24" class="graph-svg">
            <!-- Полу-линия от верха ячейки до dot нужна, только если над этим
                 коммитом стоит WT-строка (наверху над первым или inline над HEAD) —
                 иначе это "обрубок" в воздухе. Для tip-коммита замыкает разрыв,
                 для обычного перекрывается существующей straight без артефактов. -->
            <line
              v-if="(idx === 0 && wtAtTop) || (wtInline && idx === headIdx)"
              :x1="laneX(commit.column)"
              y1="0"
              :x2="laneX(commit.column)"
              y2="12"
              :stroke="laneColor(commit.column)"
              stroke-width="2"
              stroke-opacity="0.35"
            />
            <path
              v-for="(ln, li) in commit.lines"
              :key="li"
              :d="linePath(ln, idx === 0)"
              :stroke="laneColor(ln.color)"
              stroke-width="2"
              fill="none"
            />
            <!-- Незапушенный коммит — hollow кружок с контуром --unpushed
                 (отдельный per-theme токен янтаря). Размер и стиль как у
                 обычного, отличие только в цвете обводки. -->
            <circle
              :cx="laneX(commit.column)"
              cy="12"
              r="4"
              fill="var(--bg-primary)"
              :stroke="isUnpushed(idx) ? 'var(--unpushed)' : laneColor(commit.column)"
              stroke-width="2"
            />
          </svg>
        </div>

        <!-- Message + refs -->
        <div class="message-col">
          <template v-if="isCurrentBranchRow(commit)">
            <span
              v-if="currentBranch && currentBranch.ahead > 0"
              class="ref-label ref-ahead"
              :title="`${currentBranch.ahead} ahead`"
            >+{{ currentBranch.ahead }}</span>
            <span
              v-if="currentBranch && currentBranch.behind > 0"
              class="ref-label ref-behind"
              :title="`${currentBranch.behind} behind`"
            >&minus;{{ currentBranch.behind }}</span>
          </template>
          <span
            v-for="r in commit.refs"
            :key="r.name"
            :class="refClass(r)"
          >
            <RefIcon :kind="r.kind" />
            <span v-html="highlight(r.name, graphFilter)" />
          </span>
          <span class="commit-message" v-html="highlight(commit.message.split('\n')[0], graphFilter)" />
        </div>

        <span class="author-col" v-html="highlight(commit.author, graphFilter)" />
        <span class="date-col" v-html="highlight(formatDate(commit.date), graphFilter)" />
      </div>
      </template>

      <div
        v-if="hasMore && !graphFilter"
        ref="loadMoreSentinelRef"
        class="load-more-sentinel"
      >
        <span v-if="isLoadingMore">Loading more…</span>
      </div>
    </div>

    <!-- Context menu -->
    <Teleport to="body">
      <div
        v-if="ctxMenu"
        class="ctx-menu"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" :disabled="!ctxIsWorkingTree" @click="ctxAction('commit')">
          <span class="ctx-label">Commit</span>
        </button>
        <button class="ctx-item ctx-danger" :disabled="!ctxIsWorkingTree" @click="ctxAction('discard')">
          <span class="ctx-label">Discard</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxCreateTag">
          <span class="ctx-label">Add Tag</span>
          <span class="ctx-shortcut">Shift+F7</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxCherryPick">
          <span class="ctx-label">Cherry-pick</span>
          <span class="ctx-shortcut">Shift+Ctrl+M</span>
        </button>
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxRevert">
          <span class="ctx-label">Revert</span>
          <span class="ctx-shortcut">Shift+Alt+M</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxReset('soft')">
          <span class="ctx-label">Reset (soft)</span>
        </button>
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxReset('mixed')">
          <span class="ctx-label">Reset (mixed)</span>
        </button>
        <button class="ctx-item ctx-danger" :disabled="ctxIsWorkingTree" @click="ctxReset('hard')">
          <span class="ctx-label">Reset (hard)</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item" :disabled="ctxIsWorkingTree" @click="ctxReword">
          <span class="ctx-label">Edit Message…</span>
          <span class="ctx-shortcut">F2</span>
        </button>
        <button class="ctx-item" :disabled="!canSquash" @click="ctxSquash">
          <span class="ctx-label">Squash…</span>
        </button>
      </div>
      <div v-if="ctxMenu" class="ctx-backdrop" @click="closeCtxMenu" @contextmenu.prevent="closeCtxMenu" />
    </Teleport>
  </div>
</template>

<style scoped>
.commit-graph {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  container-type: inline-size;
}

.panel-title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
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
.graph-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
}
.graph-filter {
  padding: 1px 6px;
  font-size: var(--font-size-xs);
  width: 120px;
  height: 20px;
  border-color: var(--border);
}

.graph-header {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-subtle);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  user-select: none;
}

.graph-body {
  flex: 1;
  overflow-y: auto;
  user-select: none;
  -webkit-user-select: none;
}

.graph-row {
  display: flex;
  align-items: center;
  padding: 0 8px;
  cursor: pointer;
  font-size: var(--font-size-sm);
  height: 24px;
  user-select: none;
}
.graph-row:hover {
  background: var(--bg-hover);
}
.graph-row.selected {
  background: var(--bg-surface);
}
.graph-row.in-range {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}
/* Незапушенные коммиты помечаются жёлтым кружком в графе — текст не трогаем. */
/* Коммиты выше HEAD: история только с remote-ов, приглушаем. */
.graph-row.above-head .commit-message,
.graph-row.above-head .author-col,
.graph-row.above-head .date-col,
.graph-row.above-head .hash-col {
  color: var(--text-muted);
  opacity: 0.7;
}
.graph-row.above-head .ref-label {
  opacity: 0.75;
}

.graph-col {
  width: var(--graph-col-w, 80px);
  flex-shrink: 0;
  overflow: hidden;
}
.graph-svg {
  display: block;
}

.message-col {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 4px;
  overflow: hidden;
  margin-right: 12px;
}
.commit-message {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.author-col {
  display: inline-block;
  width: var(--author-col-w, 0);
  max-width: 240px;
  flex-shrink: 0;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-right: 8px;
  font-size: var(--font-size-xs);
}

.date-col {
  width: 130px;
  flex-shrink: 0;
  color: var(--text-muted);
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-xs);
}

@container (max-width: 500px) {
  .author-col { display: none; }
  .date-col { width: 110px; }
}
@container (max-width: 380px) {
  .date-col { display: none; }
}

.hash-col {
  width: 70px;
  flex-shrink: 0;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  text-align: right;
}

/* Догрузка истории при прокрутке */
.load-more-sentinel {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 24px;
  padding: 4px 8px;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
}

/* Working tree row */
.wt-label {
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
}

.graph-col-head {
  font-size: var(--font-size-xs);
}

/* Ref labels */
.ref-label {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 0 5px;
  border-radius: 3px;
  font-size: var(--font-size-xs);
  font-weight: 600;
  line-height: 16px;
  white-space: nowrap;
  flex-shrink: 0;
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
  font-weight: 800;
}
.ref-stash {
  background: rgba(203, 166, 247, 0.15);
  color: var(--purple);
}
.ref-current-branch {
  background: rgba(166, 227, 161, 0.35);
  color: var(--green);
  font-weight: 800;
}
.ref-ahead {
  background: rgba(166, 227, 161, 0.2);
  color: var(--green);
}
.ref-behind {
  background: rgba(243, 139, 168, 0.2);
  color: var(--red);
}
.ref-label .ref-icon {
  width: 11px;
  height: 11px;
}

/* Context menu */
.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
}
.ctx-menu {
  position: fixed;
  z-index: 1000;
  min-width: 140px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
}
.ctx-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  width: 100%;
  padding: 6px 12px;
  text-align: left;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  background: none;
  border: none;
  cursor: pointer;
}
.ctx-label {
  white-space: nowrap;
}
.ctx-shortcut {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  white-space: nowrap;
}
.ctx-item:hover:not(:disabled) {
  background: var(--bg-hover);
}
.ctx-item:disabled {
  color: var(--text-muted);
  opacity: 0.4;
  cursor: not-allowed;
}
.ctx-danger {
  color: var(--red);
}
.ctx-danger:hover {
  background: rgba(243, 139, 168, 0.1);
}
.ctx-separator {
  height: 0;
  border-top: 1px solid var(--border);
  margin: 0;
}
</style>
