<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke, logError } from "@/composables/useProgress";
import { useFiles } from "@/composables/useFiles";
import { useI18n } from "@/composables/useI18n";
import { useLog } from "@/composables/useLog";
import { useRepo } from "@/composables/useRepo";
import { useDiff } from "@/composables/useDiff";
import { useFileCompare } from "@/composables/useFileCompare";
import { useSettings } from "@/composables/useSettings";
import type { FileDiff, FileStatus } from "@/types";
import { highlight } from "@/utils/highlight";

const emit = defineEmits<{ commit: [] }>();

const { i18n } = useI18n();
const { files, allPaths, selectedFile, selectedPaths, refresh, stageFiles, unstageFiles, discardFiles, removeFiles, deleteFiles } = useFiles();
const { selectedCommit } = useLog();
const { repoPath } = useRepo();
const { diffFile, diffCommit, clearDiff } = useDiff();
const { open: openCompare } = useFileCompare();
const { filesTreeView, filesShowAll } = useSettings();

const activeFilter = ref<string>("all");
const fileFilter = ref("");
const commitFiles = ref<FileDiff[]>([]);

const anchorPath = ref<string | null>(null);

const listBodyRef = ref<HTMLElement | null>(null);

// --- Дерево папок ---
// Выбранная папка (""=корень/все). Свёрнутые узлы (по умолчанию все раскрыты).
const selectedDir = ref<string>("");
const collapsedDirs = ref<Set<string>>(new Set());

// Ширина панели дерева — персист в localStorage рядом с layout приложения.
const TREE_WIDTH_KEY = "gitstream.filesTreeWidth";
const treeWidth = ref<number>(
  Math.min(500, Math.max(120, Number(localStorage.getItem(TREE_WIDTH_KEY)) || 200)),
);
let resizing = false;
let resizeStartX = 0;
let resizeStartW = 0;
function onResizeMove(e: MouseEvent) {
  if (!resizing) return;
  treeWidth.value = Math.max(120, Math.min(500, resizeStartW + (e.clientX - resizeStartX)));
}
function onResizeEnd() {
  resizing = false;
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  localStorage.setItem(TREE_WIDTH_KEY, String(treeWidth.value));
}
function onResizeStart(e: MouseEvent) {
  resizing = true;
  resizeStartX = e.clientX;
  resizeStartW = treeWidth.value;
  document.addEventListener("mousemove", onResizeMove);
  document.addEventListener("mouseup", onResizeEnd);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
}

const ctxMenu = ref<{ x: number; y: number } | null>(null);

const ctxFiles = computed(() =>
  files.value.filter((f) => selectedPaths.value.includes(f.path)),
);
const canStage = computed(() =>
  ctxFiles.value.some((f) => f.staged === "unstaged" || f.staged === "partial" || f.state === "untracked"),
);
const canUnstage = computed(() =>
  ctxFiles.value.some((f) => f.staged === "staged" || f.staged === "partial"),
);

function onFileContextMenu(e: MouseEvent, path: string) {
  e.preventDefault();
  if (!selectedPaths.value.includes(path)) {
    selectedPaths.value = [path];
    anchorPath.value = path;
    selectedFile.value = path;
  }
  ctxMenu.value = { x: e.clientX, y: e.clientY };
}

function closeCtxMenu() {
  ctxMenu.value = null;
}

function ctxLabel(verb: string): string {
  const n = selectedPaths.value.length;
  return n > 1 ? `${verb} (${n} files)` : verb;
}

async function ctxRun(action: "stage" | "unstage" | "commit" | "discard" | "remove" | "delete") {
  const paths = [...selectedPaths.value];
  closeCtxMenu();
  if (paths.length === 0) return;
  const n = paths.length;
  const what = n > 1 ? `${n} files` : paths[0];
  try {
    if (action === "stage") await stageFiles(paths);
    else if (action === "unstage") await unstageFiles(paths);
    else if (action === "commit") emit("commit");
    else if (action === "discard") {
      if (window.confirm(`Discard changes in ${what}?`)) await discardFiles(paths);
    } else if (action === "remove") {
      if (window.confirm(`Remove ${what} (git rm)?`)) await removeFiles(paths);
    } else if (action === "delete") {
      if (window.confirm(`Delete ${what} from disk?`)) await deleteFiles(paths);
    }
  } catch (err) {
    logError(`Action failed: ${err}`);
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && ctxMenu.value) closeCtxMenu();
}

// Локальный keydown на body списка: сработает только когда панель файлов
// в фокусе (mousedown → .focus()). Скоупит Ctrl+A в пределах компонента,
// не задевая другие панели и инпуты.
function onListKeydown(e: KeyboardEvent) {
  // e.code вместо e.key: физическая клавиша A, независимо от раскладки
  // (в русской раскладке e.key === "ф", и проверка по букве не сработает).
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "KeyA") {
    const all = isWorkingTree.value
      ? dirFilteredFiles.value.map((f) => f.path)
      : dirFilteredCommitFiles.value.map((f) => f.path);
    if (all.length === 0) return;
    e.preventDefault();
    selectedPaths.value = all;
    const last = all[all.length - 1];
    anchorPath.value = last;
    selectedFile.value = last;
  }
}

function onBodyMousedown(e: MouseEvent) {
  if (e.detail > 1) e.preventDefault();
  // Программный фокус: клик может прийти на дочерний .file-item без tabindex,
  // поэтому форсим focus на сам контейнер, чтобы keydown ловился именно здесь.
  listBodyRef.value?.focus();
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
});

// Выключение дерева возвращает список к полному (корень).
watch(filesTreeView, (on) => { if (!on) selectedDir.value = ""; });

// "Show all files" и дерево требуют полный список файлов рабочей копии —
// подгружаем/сбрасываем его при переключении любого из режимов.
watch([filesShowAll, filesTreeView], () => { void refresh(); });

const isWorkingTree = computed(() => !selectedCommit.value || selectedCommit.value === "__worktree__");

// Load commit files when a commit is selected.
// Защита от гонки: при быстрой навигации по коммитам (стрелки ↑/↓)
// применяем только самый свежий ответ, иначе показываются файлы чужого коммита.
let commitFilesSeq = 0;
// immediate: true — компонент может смонтироваться, когда selectedCommit
// уже выставлен (HMR, переоткрытие FileList). Без immediate watch ждёт
// СМЕНЫ значения, и при стабильно выбранном коммите панель файлов остаётся
// пустой до следующего клика по графу.
watch(selectedCommit, async (oid) => {
  selectedPaths.value = [];
  anchorPath.value = null;
  selectedDir.value = "";
  // Любая смена выбора в графе (коммит→коммит, коммит→worktree, сброс)
  // делает показанный дифф неактуальным — панель Changes пуста, пока
  // пользователь не кликнет файл нового выбора.
  selectedFile.value = null;
  clearDiff();
  if (!oid || oid === "__worktree__" || !repoPath.value) {
    commitFilesSeq++;
    commitFiles.value = [];
    return;
  }
  const seq = ++commitFilesSeq;
  try {
    const data = await invoke<FileDiff[]>("get_diff_commit", { repoPath: repoPath.value, oid });
    if (seq !== commitFilesSeq) return;
    commitFiles.value = data;
    // Автоматически выбираем первый файл, чтобы дифф отображался сразу —
    // без ручного клика после переключения ветки или выбора коммита.
    if (data.length > 0) {
      const first = data[0].path;
      selectedPaths.value = [first];
      anchorPath.value = first;
      selectedFile.value = first;
      if (seq === commitFilesSeq) await diffCommit(oid, first);
    }
  } catch {
    if (seq !== commitFilesSeq) return;
    commitFiles.value = [];
  }
}, { immediate: true });

watch([activeFilter, fileFilter], () => {
  selectedPaths.value = [];
  anchorPath.value = null;
  selectedDir.value = "";
});

// "Show all files": к изменённым файлам добавляем неизменённые tracked-файлы
// (из allPaths, которых нет в status) с состоянием "unchanged".
const workingFiles = computed<FileStatus[]>(() => {
  if (!filesShowAll.value || allPaths.value.length === 0) return files.value;
  const changed = new Set(files.value.map((f) => f.path));
  const extra: FileStatus[] = allPaths.value
    .filter((p) => !changed.has(p))
    .map((p) => ({ path: p, state: "unchanged", staged: "unchanged" }));
  // Сортируем объединённый список по пути — иначе неизменённые свалятся в конец.
  return [...files.value, ...extra].sort((a, b) => a.path.localeCompare(b.path));
});

const filteredFiles = computed(() => {
  if (!isWorkingTree.value) return [];
  let result = workingFiles.value;
  if (activeFilter.value === "modified") result = result.filter((f) => f.state === "modified");
  else if (activeFilter.value === "staged") result = result.filter((f) => f.staged === "staged" || f.staged === "partial");
  else if (activeFilter.value === "untracked") result = result.filter((f) => f.state === "untracked");
  else if (activeFilter.value === "conflicted") result = result.filter((f) => f.state === "conflicted");
  const q = fileFilter.value.trim().toLowerCase();
  if (q) {
    result = result.filter((f) => {
      const hay = `${f.path} ${f.state} ${f.staged}`.toLowerCase();
      return hay.includes(q);
    });
  }
  return result;
});

const filteredCommitFiles = computed(() => {
  const q = fileFilter.value.trim().toLowerCase();
  if (!q) return commitFiles.value;
  return commitFiles.value.filter((f) => {
    const hay = `${f.path} ${f.insertions} ${f.deletions}`.toLowerCase();
    return hay.includes(q);
  });
});

// --- Фильтрация по выбранной папке (поверх текст-фильтра и табов) ---
function inSelectedDir(path: string): boolean {
  if (!filesTreeView.value || !selectedDir.value) return true;
  return path.startsWith(selectedDir.value + "/");
}
const dirFilteredFiles = computed(() => filteredFiles.value.filter((f) => inSelectedDir(f.path)));
const dirFilteredCommitFiles = computed(() => filteredCommitFiles.value.filter((f) => inSelectedDir(f.path)));

const displayCount = computed(() => isWorkingTree.value ? dirFilteredFiles.value.length : dirFilteredCommitFiles.value.length);

// --- Построение дерева папок из текущего (отфильтрованного) списка ---
interface TreeNode {
  name: string;
  path: string;
  children: Map<string, TreeNode>;
  fileCount: number;
}
interface TreeRow {
  name: string;
  path: string;
  depth: number;
  hasChildren: boolean;
  fileCount: number;
}

// Полный набор путей рабочей копии: tracked (allPaths) ∪ всё из status
// (untracked и т.п.). Дерево строится из него, поэтому показывает всю
// структуру папок репозитория, как браузер Files в SmartGit, а не только
// папки с изменениями.
const fullWorkingPaths = computed(() => {
  const set = new Set<string>(allPaths.value);
  for (const f of files.value) set.add(f.path);
  return [...set];
});

// Working tree → полная структура рабочей копии; коммит → файлы коммита.
const treePaths = computed(() =>
  isWorkingTree.value ? fullWorkingPaths.value : filteredCommitFiles.value.map((f) => f.path),
);

const folderRoot = computed<TreeNode>(() => {
  const root: TreeNode = { name: "", path: "", children: new Map(), fileCount: 0 };
  for (const p of treePaths.value) {
    const parts = p.split("/");
    parts.pop(); // имя файла отбрасываем — нужны только директории
    root.fileCount++;
    let node = root;
    let acc = "";
    for (const seg of parts) {
      acc = acc ? `${acc}/${seg}` : seg;
      let child = node.children.get(seg);
      if (!child) {
        child = { name: seg, path: acc, children: new Map(), fileCount: 0 };
        node.children.set(seg, child);
      }
      child.fileCount++; // считаем все файлы поддерева (incl. вложенные)
      node = child;
    }
  }
  return root;
});

function isDirExpanded(path: string): boolean {
  return !collapsedDirs.value.has(path);
}

// Плоский список видимых строк дерева (с учётом свёрнутых узлов) — рекурсивный
// шаблон не нужен, рендерим flat с отступом по depth.
const visibleTreeRows = computed<TreeRow[]>(() => {
  const out: TreeRow[] = [];
  const walk = (node: TreeNode, depth: number) => {
    const kids = [...node.children.values()].sort((a, b) => a.name.localeCompare(b.name));
    for (const k of kids) {
      out.push({ name: k.name, path: k.path, depth, hasChildren: k.children.size > 0, fileCount: k.fileCount });
      if (isDirExpanded(k.path)) walk(k, depth + 1);
    }
  };
  walk(folderRoot.value, 1);
  return out;
});

function toggleDir(path: string) {
  const s = new Set(collapsedDirs.value);
  if (s.has(path)) s.delete(path);
  else s.add(path);
  collapsedDirs.value = s;
}

// Все папки с детьми (для collapse all).
function allDirPaths(): string[] {
  const out: string[] = [];
  const walk = (node: TreeNode) => {
    for (const c of node.children.values()) {
      if (c.children.size > 0) out.push(c.path);
      walk(c);
    }
  };
  walk(folderRoot.value);
  return out;
}
function expandAll() {
  collapsedDirs.value = new Set();
}
function collapseAll() {
  collapsedDirs.value = new Set(allDirPaths());
}

function selectDir(path: string) {
  selectedDir.value = path;
  // Смена папки делает прежнее выделение/дифф неактуальными для нового списка.
  selectedPaths.value = [];
  anchorPath.value = null;
}

const filters = [
  { key: "all", label: "All" },
  { key: "modified", label: "Modified" },
  { key: "staged", label: "Staged" },
  { key: "untracked", label: "Untracked" },
  { key: "conflicted", label: "Conflicted" },
];

const stateIcons: Record<string, { color: string; letter: string }> = {
  modified: { color: "var(--blue)", letter: "M" },
  added: { color: "var(--green)", letter: "A" },
  deleted: { color: "var(--red)", letter: "D" },
  renamed: { color: "var(--purple)", letter: "R" },
  conflicted: { color: "var(--red)", letter: "C" },
  untracked: { color: "var(--text-muted)", letter: "?" },
  unchanged: { color: "var(--text-muted)", letter: "·" },
};

function fileName(path: string): string {
  return path.split("/").pop() || path;
}

function fileDir(path: string): string {
  const parts = path.split("/");
  parts.pop();
  return parts.join("/");
}

async function selectFile(path: string, e?: MouseEvent) {
  const list = dirFilteredFiles.value.map((f) => f.path);
  if (e?.shiftKey && anchorPath.value !== null) {
    const a = list.indexOf(anchorPath.value);
    const b = list.indexOf(path);
    if (a !== -1 && b !== -1) {
      const [lo, hi] = a < b ? [a, b] : [b, a];
      selectedPaths.value = list.slice(lo, hi + 1);
    }
  } else if (e?.ctrlKey || e?.metaKey) {
    const i = selectedPaths.value.indexOf(path);
    if (i === -1) selectedPaths.value = [...selectedPaths.value, path];
    else selectedPaths.value = selectedPaths.value.filter((p) => p !== path);
    anchorPath.value = path;
  } else {
    selectedPaths.value = [path];
    anchorPath.value = path;
  }
  // selectedFile всегда = последний кликнутый: diff остаётся виден даже после
  // снятия выделения (Ctrl+клик) — поведение как в эталонном клиенте.
  selectedFile.value = path;
  if (isWorkingTree.value) {
    const f = files.value.find((x) => x.path === path);
    const staged = f?.staged === "staged";
    await diffFile(path, staged);
  }
}

async function selectCommitFile(path: string, e?: MouseEvent) {
  const list = dirFilteredCommitFiles.value.map((f) => f.path);
  if (e?.shiftKey && anchorPath.value !== null) {
    const a = list.indexOf(anchorPath.value);
    const b = list.indexOf(path);
    if (a !== -1 && b !== -1) {
      const [lo, hi] = a < b ? [a, b] : [b, a];
      selectedPaths.value = list.slice(lo, hi + 1);
    }
  } else if (e?.ctrlKey || e?.metaKey) {
    const i = selectedPaths.value.indexOf(path);
    if (i === -1) selectedPaths.value = [...selectedPaths.value, path];
    else selectedPaths.value = selectedPaths.value.filter((p) => p !== path);
    anchorPath.value = path;
  } else {
    selectedPaths.value = [path];
    anchorPath.value = path;
  }
  selectedFile.value = path;
  if (selectedCommit.value && selectedCommit.value !== "__worktree__") {
    await diffCommit(selectedCommit.value, path);
  }
}

function compareWorkingTreeFile(path: string) {
  const f = files.value.find((x) => x.path === path);
  openCompare({ path, staged: f?.staged === "staged" });
}
function compareCommitFile(path: string) {
  if (selectedCommit.value && selectedCommit.value !== "__worktree__") {
    openCompare({ path, oid: selectedCommit.value });
  }
}
</script>

<template>
  <div class="file-list">
    <div class="file-list-header">
      <div class="header-left">
        <span class="panel-title">{{ i18n.files.title }}</span>
        <span class="files-hidden">{{ displayCount }} files</span>
      </div>
      <div class="header-right">
        <input
          v-model="fileFilter"
          type="text"
          :placeholder="i18n.files.filter"
          class="file-filter-input"
        />
        <button
          class="tree-toggle"
          :class="{ active: filesShowAll }"
          :title="i18n.files.showAllFiles"
          @click="filesShowAll = !filesShowAll"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3">
            <path d="M1 8s2.5-4.5 7-4.5S15 8 15 8s-2.5 4.5-7 4.5S1 8 1 8z" stroke-linejoin="round" />
            <circle cx="8" cy="8" r="1.8" />
          </svg>
        </button>
        <button
          class="tree-toggle"
          :class="{ active: filesTreeView }"
          :title="i18n.files.treeView"
          @click="filesTreeView = !filesTreeView"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4">
            <path d="M2 3h4l1.2 1.5H14V13H2V3z" stroke-linejoin="round" />
            <path d="M5.5 7.5h6M5.5 10h4" stroke-linecap="round" />
          </svg>
        </button>
        <div class="filter-tabs">
          <button
            v-for="f in filters"
            :key="f.key"
            class="filter-tab"
            :class="{ active: activeFilter === f.key }"
            @click="activeFilter = f.key"
            :title="f.label"
          >
            {{ f.label }}
          </button>
        </div>
      </div>
    </div>

    <div class="file-list-main" :class="{ split: filesTreeView }">
      <!-- Дерево папок (слева) -->
      <template v-if="filesTreeView">
        <div class="folder-tree" :style="{ width: treeWidth + 'px' }">
          <div class="tree-toolbar">
            <button class="tree-tool-btn" :title="i18n.files.collapseAll" @click="collapseAll">
              <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M4 8h8" stroke-linecap="round" />
              </svg>
            </button>
            <button class="tree-tool-btn" :title="i18n.files.expandAll" @click="expandAll">
              <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M8 4v8M4 8h8" stroke-linecap="round" />
              </svg>
            </button>
          </div>
          <div
            class="tree-row"
            :class="{ selected: selectedDir === '' }"
            @click="selectDir('')"
          >
            <span class="tree-chevron-spacer" />
            <span class="tree-name">{{ i18n.files.allFiles }}</span>
          </div>
          <div
            v-for="row in visibleTreeRows"
            :key="row.path"
            class="tree-row"
            :class="{ selected: selectedDir === row.path }"
            :style="{ paddingLeft: row.depth * 14 + 6 + 'px' }"
            @click="selectDir(row.path)"
          >
            <span
              v-if="row.hasChildren"
              class="tree-chevron"
              @click.stop="toggleDir(row.path)"
            >{{ isDirExpanded(row.path) ? '▾' : '▸' }}</span>
            <span v-else class="tree-chevron-spacer" />
            <span class="tree-name" :title="row.path">{{ row.name }}</span>
          </div>
        </div>
        <div class="tree-resizer" @mousedown="onResizeStart" />
      </template>

      <!-- @contextmenu.prevent на контейнере гасит нативное меню браузера на пустой области;
           меню файла открывается обработчиком на самом .file-item (событие всплывает). -->
      <div
        ref="listBodyRef"
        class="file-list-body"
        tabindex="-1"
        @mousedown="onBodyMousedown"
        @keydown="onListKeydown"
        @contextmenu.prevent
      >
      <!-- Working tree files -->
      <template v-if="isWorkingTree">
        <div
          v-for="file in dirFilteredFiles"
          :key="file.path"
          class="file-item"
          :class="{ selected: selectedPaths.includes(file.path) }"
          @click="selectFile(file.path, $event)"
          @dblclick="compareWorkingTreeFile(file.path)"
          @contextmenu="onFileContextMenu($event, file.path)"
        >
          <span
            class="state-badge"
            :style="{ color: stateIcons[file.state]?.color }"
            :title="file.state"
          >
            {{ stateIcons[file.state]?.letter }}
          </span>

          <span
            v-if="file.staged === 'staged'"
            class="staged-dot"
            :title="i18n.files.staged"
          />
          <span
            v-else-if="file.staged === 'partial'"
            class="staged-dot partial"
            :title="i18n.files.partiallyStaged"
          />

          <span class="file-name" v-html="highlight(fileName(file.path), fileFilter)" />
          <span class="file-dir" v-html="highlight(fileDir(file.path), fileFilter)" />
        </div>
      </template>

      <!-- Commit files -->
      <template v-else>
        <div
          v-for="cf in dirFilteredCommitFiles"
          :key="cf.path"
          class="file-item"
          :class="{ selected: selectedPaths.includes(cf.path) }"
          @click="selectCommitFile(cf.path, $event)"
          @dblclick="compareCommitFile(cf.path)"
        >
          <span class="state-badge" :style="{ color: 'var(--blue)' }">M</span>
          <span class="file-name" v-html="highlight(fileName(cf.path), fileFilter)" />
          <span class="file-dir" v-html="highlight(fileDir(cf.path), fileFilter)" />
          <span class="file-stats">
            <span class="stat-add">+{{ cf.insertions }}</span>
            <span class="stat-del">-{{ cf.deletions }}</span>
          </span>
        </div>
      </template>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="ctxMenu"
        class="ctx-menu"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" :disabled="!canStage" @click="ctxRun('stage')">
          <span class="ctx-label">{{ ctxLabel('Stage') }}</span>
        </button>
        <button class="ctx-item" :disabled="!canUnstage" @click="ctxRun('unstage')">
          <span class="ctx-label">{{ ctxLabel('Unstage') }}</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item" @click="ctxRun('commit')">
          <span class="ctx-label">{{ i18n.files.commitCtx }}</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item ctx-danger" @click="ctxRun('discard')">
          <span class="ctx-label">{{ ctxLabel('Discard') }}</span>
        </button>
        <button class="ctx-item ctx-danger" @click="ctxRun('remove')">
          <span class="ctx-label">{{ ctxLabel('Remove') }}</span>
        </button>
        <button class="ctx-item ctx-danger" @click="ctxRun('delete')">
          <span class="ctx-label">{{ ctxLabel('Delete') }}</span>
        </button>
      </div>
      <div v-if="ctxMenu" class="ctx-backdrop" @click="closeCtxMenu" @contextmenu.prevent="closeCtxMenu" />
    </Teleport>
  </div>
</template>

<style scoped>
.file-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  user-select: none;
  -webkit-user-select: none;
}

.file-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-tertiary);
  gap: 8px;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.files-hidden {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
}
.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  justify-content: flex-end;
}
.file-filter-input {
  width: 140px;
  padding: 2px 6px;
  font-size: var(--font-size-xs);
  border-color: var(--border);
}

.panel-title {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-secondary);
}

.filter-tabs {
  display: flex;
  gap: 2px;
}

.filter-tab {
  padding: 2px 6px;
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  border-radius: var(--radius);
}
.filter-tab:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}
.filter-tab.active {
  background: var(--bg-surface);
  color: var(--text-primary);
}

.file-list-main {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

.file-list-body {
  flex: 1;
  overflow-y: auto;
  outline: none;
}

/* Folder tree */
.tree-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3px 5px;
  border-radius: var(--radius);
  color: var(--text-muted);
  flex-shrink: 0;
}
.tree-toggle:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}
.tree-toggle.active {
  background: var(--bg-surface);
  color: var(--text-primary);
}

.folder-tree {
  flex-shrink: 0;
  overflow-y: auto;
  overflow-x: hidden;
  border-right: 1px solid var(--border-subtle);
}

.tree-toolbar {
  position: sticky;
  top: 0;
  z-index: 1;
  display: flex;
  gap: 2px;
  padding: 2px 4px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-subtle);
}
.tree-tool-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2px 4px;
  border-radius: var(--radius);
  color: var(--text-muted);
}
.tree-tool-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.tree-resizer {
  width: 4px;
  flex-shrink: 0;
  cursor: col-resize;
  background: transparent;
}
.tree-resizer:hover {
  background: var(--bg-hover);
}

.tree-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 6px;
  cursor: pointer;
  font-size: var(--font-size-sm);
  white-space: nowrap;
}
.tree-row:hover {
  background: var(--bg-hover);
}
.tree-row.selected {
  background: var(--bg-surface);
}

.tree-chevron,
.tree-chevron-spacer {
  width: 12px;
  flex-shrink: 0;
  text-align: center;
  color: var(--text-muted);
  font-size: var(--font-size-xs);
}

.tree-name {
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
}

.tree-count {
  margin-left: auto;
  padding-left: 8px;
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  flex-shrink: 0;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px;
  cursor: pointer;
  font-size: var(--font-size-sm);
  user-select: none;
  -webkit-user-select: none;
}
.file-item:hover {
  background: var(--bg-hover);
}
.file-item.selected {
  background: var(--bg-surface);
}

.state-badge {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  font-weight: 700;
  width: 14px;
  text-align: center;
  flex-shrink: 0;
}

.staged-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--green);
  flex-shrink: 0;
}
.staged-dot.partial {
  background: var(--yellow);
}

.file-name {
  color: var(--text-primary);
  white-space: nowrap;
}

.file-dir {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-stats {
  margin-left: auto;
  display: flex;
  gap: 6px;
  font-size: var(--font-size-xs);
  font-family: var(--font-mono);
  flex-shrink: 0;
}
.stat-add { color: var(--green); }
.stat-del { color: var(--red); }

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
