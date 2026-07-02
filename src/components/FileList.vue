<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke, logError, closeLog } from "@/composables/useProgress";
import { useFiles } from "@/composables/useFiles";
import { useI18n } from "@/composables/useI18n";
import { useLog } from "@/composables/useLog";
import { useRepo } from "@/composables/useRepo";
import { useDiff } from "@/composables/useDiff";
import { useFileCompare } from "@/composables/useFileCompare";
import { useFileHistory } from "@/composables/useFileHistory";
import { useBlame } from "@/composables/useBlame";
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

// Фильтры по состоянию файла — независимые тогглы (как в SmartGit).
// По умолчанию все включены (= показать всё). Состояние персистится в localStorage.
type StateFilterKey = "modified" | "staged" | "untracked" | "conflicted";
const STATE_FILTERS_KEY = "gitstream.fileStateFilters";
function loadStateFilters(): Record<StateFilterKey, boolean> {
  const def = { modified: true, staged: true, untracked: true, conflicted: true };
  try {
    const raw = localStorage.getItem(STATE_FILTERS_KEY);
    if (raw) return { ...def, ...JSON.parse(raw) };
  } catch { /* битый JSON → дефолты */ }
  return def;
}
const stateFilters = ref<Record<StateFilterKey, boolean>>(loadStateFilters());
watch(stateFilters, (v) => {
  localStorage.setItem(STATE_FILTERS_KEY, JSON.stringify(v));
}, { deep: true });
function toggleStateFilter(key: StateFilterKey) {
  stateFilters.value = { ...stateFilters.value, [key]: !stateFilters.value[key] };
}

const fileFilter = ref("");
const commitFiles = ref<FileDiff[]>([]);
// Все файлы дерева выбранного коммита — заполняется только при "Show all files".
const commitAllPaths = ref<string[]>([]);

const anchorPath = ref<string | null>(null);

const listBodyRef = ref<HTMLElement | null>(null);

// --- Дерево папок ---
// Выбранная папка (""=корень/все).
const selectedDir = ref<string>("");

// Раскрытые узлы дерева. Модель "expanded" (а не "collapsed"): пустое множество
// = все папки свёрнуты по умолчанию. Состояние персистится по репозиторию.
const TREE_EXPANDED_KEY = "gitstream.treeExpanded";
function loadExpandedDirs(repo: string | null): Set<string> {
  if (!repo) return new Set();
  try {
    const raw = localStorage.getItem(TREE_EXPANDED_KEY);
    const map = raw ? JSON.parse(raw) : {};
    return new Set<string>(map[repo] || []);
  } catch {
    return new Set();
  }
}
const expandedDirs = ref<Set<string>>(loadExpandedDirs(repoPath.value));
function saveExpandedDirs() {
  if (!repoPath.value) return;
  try {
    const raw = localStorage.getItem(TREE_EXPANDED_KEY);
    const map = raw ? JSON.parse(raw) : {};
    map[repoPath.value] = [...expandedDirs.value];
    localStorage.setItem(TREE_EXPANDED_KEY, JSON.stringify(map));
  } catch { /* localStorage недоступен — не критично */ }
}
// Смена репозитория → загрузить сохранённое состояние дерева этого репозитория.
watch(repoPath, (r) => { expandedDirs.value = loadExpandedDirs(r); });

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

const { openFor: openFileHistory } = useFileHistory();
function ctxHistory() {
  const path = selectedPaths.value[0];
  closeCtxMenu();
  if (path) openFileHistory(path);
}

const { openFor: openBlame } = useBlame();
function ctxBlame() {
  const path = selectedPaths.value[0];
  closeCtxMenu();
  if (path) openBlame(path);
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
watch([filesShowAll, filesTreeView], () => {
  void refresh();
  // Для выбранного коммита тоггл тоже переключает полное/изменённое дерево.
  const oid = selectedCommit.value;
  if (oid && oid !== "__worktree__") {
    if (filesShowAll.value) void loadCommitTree(oid, commitFilesSeq);
    else commitAllPaths.value = [];
  }
});

// Загрузка полного дерева файлов коммита (для "Show all files"). seq —
// текущий commitFilesSeq на момент запроса: применяем результат, только если
// выбор коммита с тех пор не сменился.
async function loadCommitTree(oid: string, seq: number) {
  if (!filesShowAll.value || !repoPath.value) return;
  try {
    const all = await invoke<string[]>("list_files_at", { repoPath: repoPath.value, oid });
    if (seq !== commitFilesSeq) return;
    commitAllPaths.value = all;
  } catch {
    if (seq !== commitFilesSeq) return;
    commitAllPaths.value = [];
  }
}

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
    commitAllPaths.value = [];
    return;
  }
  const seq = ++commitFilesSeq;
  // Полное дерево коммита нужно только для "Show all files". НЕ сбрасываем
  // commitAllPaths в [] до прихода нового списка: иначе дерево на миг
  // схлопывается до изменённых файлов и «дёргается». Между соседними
  // коммитами список файлов почти не меняется — старое дерево держится
  // стабильным, пока loadCommitTree (с защитой по seq) не заменит его.
  void loadCommitTree(oid, seq);
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

watch([stateFilters, fileFilter], () => {
  selectedPaths.value = [];
  anchorPath.value = null;
  selectedDir.value = "";
}, { deep: true });

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

// Принадлежность файла категории тоггла. modified∪staged покрывают любые
// tracked-изменения (M/A/D/R), partial попадает в обе категории.
function fileMatchesCategory(f: FileStatus, key: StateFilterKey): boolean {
  if (f.state === "unchanged") return false;
  const isStaged = f.staged === "staged" || f.staged === "partial";
  const isUnstaged = f.staged === "unstaged" || f.staged === "partial";
  switch (key) {
    case "staged": return isStaged;
    case "modified": return isUnstaged && f.state !== "untracked" && f.state !== "conflicted";
    case "untracked": return f.state === "untracked";
    case "conflicted": return f.state === "conflicted";
  }
}

function matchesStateFilters(f: FileStatus): boolean {
  // unchanged-файлы управляются кнопкой "Show all files", а не этими фильтрами.
  if (f.state === "unchanged") return true;
  const sf = stateFilters.value;
  return (sf.staged && fileMatchesCategory(f, "staged"))
    || (sf.modified && fileMatchesCategory(f, "modified"))
    || (sf.untracked && fileMatchesCategory(f, "untracked"))
    || (sf.conflicted && fileMatchesCategory(f, "conflicted"));
}

// Сколько файлов в каждой категории (по реальным изменениям рабочего дерева) —
// для disable тогглов, под которые нет файлов.
const stateCounts = computed<Record<StateFilterKey, number>>(() => {
  const c: Record<StateFilterKey, number> = { modified: 0, staged: 0, untracked: 0, conflicted: 0 };
  for (const f of files.value) {
    if (fileMatchesCategory(f, "staged")) c.staged++;
    if (fileMatchesCategory(f, "modified")) c.modified++;
    if (fileMatchesCategory(f, "untracked")) c.untracked++;
    if (fileMatchesCategory(f, "conflicted")) c.conflicted++;
  }
  return c;
});

const filteredFiles = computed(() => {
  if (!isWorkingTree.value) return [];
  let result = workingFiles.value.filter(matchesStateFilters);
  const q = fileFilter.value.trim().toLowerCase();
  if (q) {
    result = result.filter((f) => {
      const hay = `${f.path} ${f.state} ${f.staged}`.toLowerCase();
      return hay.includes(q);
    });
  }
  return result;
});

// Пути файлов, реально изменённых в коммите — для отличия от неизменённых
// (добавляемых тогглом "Show all files") при рендере бейджа и статистики.
const commitChangedPaths = computed(() => new Set(commitFiles.value.map((f) => f.path)));

// "Show all files" для коммита: к изменённым файлам добавляем все остальные
// файлы дерева коммита как неизменённые (пустой дифф). Симметрично workingFiles.
const commitFilesDisplay = computed<FileDiff[]>(() => {
  if (!filesShowAll.value || commitAllPaths.value.length === 0) return commitFiles.value;
  const changed = commitChangedPaths.value;
  const extra: FileDiff[] = commitAllPaths.value
    .filter((p) => !changed.has(p))
    .map((p) => ({
      path: p, hunks: [], insertions: 0, deletions: 0, header: "",
      binary: false, old_image: null, new_image: null, byte_size: null,
    }));
  return [...commitFiles.value, ...extra].sort((a, b) => a.path.localeCompare(b.path));
});

const filteredCommitFiles = computed(() => {
  const q = fileFilter.value.trim().toLowerCase();
  if (!q) return commitFilesDisplay.value;
  return commitFilesDisplay.value.filter((f) => {
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
  hasChanges: boolean;
}

// Полный набор путей рабочей копии: tracked (allPaths) ∪ всё из status
// (untracked и т.п.) — вся структура папок репозитория, как браузер Files
// в SmartGit. Используется, когда включён тоггл "Show all files".
const fullWorkingPaths = computed(() => {
  const set = new Set<string>(allPaths.value);
  for (const f of files.value) set.add(f.path);
  return [...set];
});

// Пути только изменённых файлов (с учётом тогглов состояния) — когда
// "Show all files" выключен, дерево показывает только папки с изменениями.
const changedTreePaths = computed(() =>
  files.value.filter(matchesStateFilters).map((f) => f.path),
);

// Working tree: "Show all files" вкл → вся структура (структурные папки
// приглушены), выкл → только папки с изменениями (все подсвечены).
// Коммит → файлы коммита.
const treePaths = computed(() => {
  if (!isWorkingTree.value) return filteredCommitFiles.value.map((f) => f.path);
  return filesShowAll.value ? fullWorkingPaths.value : changedTreePaths.value;
});

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

// Папки, содержащие изменённые файлы (с учётом включённых тогглов состояния) —
// для подсветки в дереве, как в SmartGit. files.value = реальные изменения
// рабочего дерева, поэтому добавляем все папки-предки каждого такого файла.
const changedDirs = computed(() => {
  const set = new Set<string>();
  for (const f of files.value) {
    if (!matchesStateFilters(f)) continue;
    const parts = f.path.split("/");
    parts.pop();
    let acc = "";
    for (const seg of parts) {
      acc = acc ? `${acc}/${seg}` : seg;
      set.add(acc);
    }
  }
  return set;
});
const hasAnyChanges = computed(() => files.value.some(matchesStateFilters));

function isDirExpanded(path: string): boolean {
  return expandedDirs.value.has(path);
}

// Плоский список видимых строк дерева (с учётом свёрнутых узлов) — рекурсивный
// шаблон не нужен, рендерим flat с отступом по depth.
const visibleTreeRows = computed<TreeRow[]>(() => {
  const out: TreeRow[] = [];
  const walk = (node: TreeNode, depth: number) => {
    const kids = [...node.children.values()].sort((a, b) => a.name.localeCompare(b.name));
    for (const k of kids) {
      out.push({ name: k.name, path: k.path, depth, hasChildren: k.children.size > 0, fileCount: k.fileCount, hasChanges: changedDirs.value.has(k.path) });
      if (isDirExpanded(k.path)) walk(k, depth + 1);
    }
  };
  walk(folderRoot.value, 1);
  return out;
});

function toggleDir(path: string) {
  const s = new Set(expandedDirs.value);
  if (s.has(path)) s.delete(path);
  else s.add(path);
  expandedDirs.value = s;
  saveExpandedDirs();
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
  expandedDirs.value = new Set(allDirPaths());
  saveExpandedDirs();
}
function collapseAll() {
  expandedDirs.value = new Set();
  saveExpandedDirs();
}

function selectDir(path: string) {
  selectedDir.value = path;
  // Смена папки делает прежнее выделение/дифф неактуальными для нового списка.
  selectedPaths.value = [];
  anchorPath.value = null;
}

// Кнопки-тогглы фильтра по состоянию: буквенный бейдж + цвет, как в строках файлов.
const stateFilterDefs: { key: StateFilterKey; letter: string; color: string; label: string }[] = [
  { key: "modified", letter: "M", color: "var(--blue)", label: "Modified" },
  { key: "staged", letter: "S", color: "var(--green)", label: "Staged" },
  { key: "untracked", letter: "?", color: "var(--text-muted)", label: "Untracked" },
  { key: "conflicted", letter: "C", color: "var(--red)", label: "Conflicted" },
];
const stateFilterTitle = (key: StateFilterKey, fallback: string): string =>
  (i18n.value.files as Record<string, string>)[key] || fallback;

const stateIcons: Record<string, { color: string; letter: string; i18nKey: string }> = {
  modified: { color: "var(--blue)", letter: "M", i18nKey: "stModified" },
  added: { color: "var(--green)", letter: "A", i18nKey: "stAdded" },
  deleted: { color: "var(--red)", letter: "D", i18nKey: "stDeleted" },
  renamed: { color: "var(--purple)", letter: "R", i18nKey: "stRenamed" },
  conflicted: { color: "var(--red)", letter: "C", i18nKey: "stConflicted" },
  untracked: { color: "var(--text-muted)", letter: "?", i18nKey: "stUntracked" },
  unchanged: { color: "var(--text-muted)", letter: "·", i18nKey: "stUnchanged" },
};

// Текстовая метка состояния файла (колонка State). Помимо
// state добавляем суффикс staged-статуса, чтобы было видно, попадёт ли файл
// в коммит: «Modified · staged» / «Modified · unstaged» / «Modified · partial».
function stateText(f: FileStatus): string {
  const fl = i18n.value.files as Record<string, string>;
  const base = fl[stateIcons[f.state]?.i18nKey] || f.state;
  if (f.state === "unchanged" || f.state === "untracked") return base;
  if (f.staged === "staged") return `${base} · ${i18n.value.files.staged}`;
  if (f.staged === "partial") return `${base} · ${i18n.value.files.partiallyStaged}`;
  return base;
}

function fileName(path: string): string {
  return path.split("/").pop() || path;
}

function fileDir(path: string): string {
  const parts = path.split("/");
  parts.pop();
  return parts.join("/");
}

async function selectFile(path: string, e?: MouseEvent) {
  // Выбор файла показывает его diff → переключаемся на вкладку Changes.
  closeLog();
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
    try {
      await diffFile(path, staged);
    } catch (err) {
      logError(`Diff failed: ${err}`);
    }
  }
}

async function selectCommitFile(path: string, e?: MouseEvent) {
  // Выбор файла показывает его diff → переключаемся на вкладку Changes.
  closeLog();
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
    try {
      await diffCommit(selectedCommit.value, path);
    } catch (err) {
      logError(`Diff failed: ${err}`);
    }
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
        <input
          v-model="fileFilter"
          type="text"
          :placeholder="i18n.files.filter"
          class="file-filter-input"
        />
        <div class="filter-tabs">
          <button
            v-for="f in stateFilterDefs"
            :key="f.key"
            class="state-filter"
            :class="{ active: stateFilters[f.key] }"
            :style="{ color: f.color }"
            :disabled="stateCounts[f.key] === 0"
            @click="toggleStateFilter(f.key)"
            :title="stateFilterTitle(f.key, f.label)"
          >
            {{ f.letter }}
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
            :class="{ selected: selectedDir === '', 'has-changes': hasAnyChanges }"
            @click="selectDir('')"
          >
            <span class="tree-chevron-spacer" />
            <span class="tree-name">{{ i18n.files.allFiles }}</span>
          </div>
          <div
            v-for="row in visibleTreeRows"
            :key="row.path"
            class="tree-row"
            :class="{ selected: selectedDir === row.path, 'has-changes': row.hasChanges }"
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
          <span
            class="state-text"
            :style="{ color: stateIcons[file.state]?.color }"
          >{{ stateText(file) }}</span>
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
          <span
            class="state-badge"
            :style="{ color: commitChangedPaths.has(cf.path) ? 'var(--blue)' : 'var(--text-muted)' }"
          >{{ commitChangedPaths.has(cf.path) ? 'M' : '·' }}</span>
          <span class="file-name" v-html="highlight(fileName(cf.path), fileFilter)" />
          <span class="file-dir" v-html="highlight(fileDir(cf.path), fileFilter)" />
          <span
            class="state-text"
            :style="{ color: commitChangedPaths.has(cf.path) ? 'var(--blue)' : 'var(--text-muted)' }"
          >{{ commitChangedPaths.has(cf.path) ? i18n.files.stModified : i18n.files.stUnchanged }}</span>
          <span v-if="commitChangedPaths.has(cf.path)" class="file-stats">
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
        <button class="ctx-item" :disabled="selectedPaths.length !== 1" @click="ctxHistory">
          <span class="ctx-label">{{ i18n.files.historyCtx }}</span>
        </button>
        <button class="ctx-item" :disabled="selectedPaths.length !== 1" @click="ctxBlame">
          <span class="ctx-label">{{ i18n.files.blameCtx }}</span>
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

.filter-tabs {
  display: flex;
  gap: 2px;
}

/* Тогглы фильтра по состоянию — буквенный бейдж в стиле кнопок тулбара.
   Выключенный = приглушённый ч/б; включённый = в своём цвете + фон. */
.state-filter {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  font-weight: 700;
  border-radius: var(--radius);
  filter: grayscale(1);
  opacity: 0.4;
}
.state-filter:hover {
  background: var(--bg-hover);
  opacity: 0.75;
}
.state-filter.active {
  background: var(--bg-surface);
  filter: none;
  opacity: 1;
}
.state-filter:disabled {
  opacity: 0.18;
  cursor: not-allowed;
  background: none;
  filter: grayscale(1);
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
/* В режиме дерева — отступ списка файлов от вертикального разделителя. */
.file-list-main.split .file-list-body {
  padding-left: 8px;
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

/* Видимый вертикальный разделитель + ручка ресайза.
   Ширина даёт отступ, линия по центру (border) — само разделение. */
.tree-resizer {
  width: 9px;
  flex-shrink: 0;
  cursor: col-resize;
  background: transparent;
  border-left: 1px solid var(--border);
  margin-left: 8px;
}
.tree-resizer:hover {
  border-left-color: var(--accent);
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
  /* "Структурные" папки (без изменений) приглушены... */
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
}
/* ...а папки, содержащие изменённые файлы, подсвечены — как в SmartGit. */
.tree-row.has-changes .tree-name {
  color: var(--text-primary);
  font-weight: 600;
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

/* Текстовая метка состояния — «колонка» State справа.
   Цвет наследует state, чтобы статус читался без расшифровки буквенного бейджа. */
.state-text {
  margin-left: auto;
  padding-left: 12px;
  font-size: var(--font-size-xs);
  white-space: nowrap;
  flex-shrink: 0;
}

.file-stats {
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
