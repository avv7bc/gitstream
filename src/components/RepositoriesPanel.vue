<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@/composables/useProgress";
import { useRepo } from "@/composables/useRepo";
import { useI18n } from "@/composables/useI18n";
import type { RepoInfo } from "@/types";
import { useSettings } from "@/composables/useSettings";
import { logError } from "@/composables/useProgress";
import AddGroupDialog from "@/components/dialogs/AddGroupDialog.vue";
import AddRepositoryDialog from "@/components/dialogs/AddRepositoryDialog.vue";
import CloneRepositoryDialog from "@/components/dialogs/CloneRepositoryDialog.vue";
import RenameNodeDialog from "@/components/dialogs/RenameNodeDialog.vue";

const { i18n } = useI18n();
const { networkTimeoutSecs } = useSettings();

const emit = defineEmits<{
  // Добавлен новый существующий репозиторий — App откроет его и сделает pull.
  "repo-added": [path: string];
}>();

// --- Tree node types ---
interface RepoNode {
  id: string;
  type: "repo";
  name: string;
  path: string;
  branch: string;
  hasChanges: boolean;
}

interface FolderNode {
  id: string;
  type: "folder";
  name: string;
  children: TreeNode[];
  expanded: boolean;
}

type TreeNode = RepoNode | FolderNode;

// --- Persistence ---
const TREE_KEY = "gitstream:repo-tree";
const TREE_NEXT_ID_KEY = "gitstream:repo-tree-next-id";

function loadTree(): TreeNode[] {
  try {
    const raw = localStorage.getItem(TREE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch { return []; }
}

function saveTree() {
  localStorage.setItem(TREE_KEY, JSON.stringify(tree.value));
  localStorage.setItem(TREE_NEXT_ID_KEY, String(nextId));
}

// --- State ---
const { repoPath, openRepo, closeRepo } = useRepo();

const tree = ref<TreeNode[]>(loadTree());

const selectedId = ref("");

// --- Drag & Drop state ---
const draggedId = ref<string | null>(null);
const dropTargetId = ref<string | null>(null);
const dropPosition = ref<"inside" | "before" | "after" | null>(null);

// --- Helpers ---
function findNode(nodes: TreeNode[], id: string): TreeNode | null {
  for (const n of nodes) {
    if (n.id === id) return n;
    if (n.type === "folder") {
      const found = findNode(n.children, id);
      if (found) return found;
    }
  }
  return null;
}

function removeNode(nodes: TreeNode[], id: string): TreeNode | null {
  for (let i = 0; i < nodes.length; i++) {
    if (nodes[i].id === id) {
      return nodes.splice(i, 1)[0];
    }
    const n = nodes[i];
    if (n.type === "folder") {
      const found = removeNode(n.children, id);
      if (found) return found;
    }
  }
  return null;
}

function isDescendant(parentId: string, childId: string, nodes: TreeNode[]): boolean {
  const parent = findNode(nodes, parentId);
  if (!parent || parent.type !== "folder") return false;
  if (parent.children.some((c) => c.id === childId)) return true;
  return parent.children.some(
    (c) => c.type === "folder" && isDescendant(c.id, childId, nodes)
  );
}

function insertAt(nodes: TreeNode[], targetId: string, node: TreeNode, pos: "before" | "after") {
  for (let i = 0; i < nodes.length; i++) {
    if (nodes[i].id === targetId) {
      nodes.splice(pos === "before" ? i : i + 1, 0, node);
      return true;
    }
    const n = nodes[i];
    if (n.type === "folder") {
      if (insertAt(n.children, targetId, node, pos)) return true;
    }
  }
  return false;
}

// Папки всегда отображаются по алфавиту. Сортируем рекурсивно и in-place, не
// трогая позиции репозиториев: переставляем только узлы-папки в занятых ими
// слотах. Ссылки на объекты узлов сохраняются — expanded/выбор/реактивность
// не ломаются.
function sortFolders(nodes: TreeNode[]) {
  for (const n of nodes) {
    if (n.type === "folder") sortFolders(n.children);
  }
  const slots: number[] = [];
  const folders: FolderNode[] = [];
  nodes.forEach((n, i) => {
    if (n.type === "folder") {
      slots.push(i);
      folders.push(n);
    }
  });
  folders.sort((a, b) =>
    a.name.localeCompare(b.name, undefined, { sensitivity: "base" })
  );
  slots.forEach((idx, k) => {
    nodes[idx] = folders[k];
  });
}

function applyFolderSort() {
  sortFolders(tree.value);
  saveTree();
}

// --- Toggle folder ---
function toggleFolder(node: FolderNode) {
  node.expanded = !node.expanded;
}

// --- Select ---
function selectNode(node: TreeNode) {
  selectedId.value = node.id;
}

// Активный (открытый) репозиторий подсвечивается всегда, независимо от
// selectedId: клик по другому узлу лишь выделяет его, но не должен скрывать,
// какой репозиторий сейчас открыт.
function isActiveRepo(node: TreeNode): boolean {
  return node.type === "repo" && !!repoPath.value && node.path === repoPath.value;
}

// --- Double-click: open repo ---
async function openRepoNode(node: TreeNode) {
  if (node.type === "repo") {
    selectedId.value = node.id;
    await openRepo(node.path);
  }
}

// --- Drag start ---
function onDragStart(e: DragEvent, node: TreeNode) {
  draggedId.value = node.id;
  e.dataTransfer!.effectAllowed = "move";
  e.dataTransfer!.setData("text/plain", node.id);
}

function onDragEnd() {
  draggedId.value = null;
  dropTargetId.value = null;
  dropPosition.value = null;
}

// --- Drag over: compute drop position ---
function onDragOver(e: DragEvent, node: TreeNode) {
  e.preventDefault();
  if (!draggedId.value || draggedId.value === node.id) return;

  // Don't allow dropping a folder into its own descendant
  if (isDescendant(draggedId.value, node.id, tree.value)) return;

  e.dataTransfer!.dropEffect = "move";
  dropTargetId.value = node.id;

  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const y = e.clientY - rect.top;
  const h = rect.height;

  if (node.type === "folder") {
    if (y < h * 0.25) {
      dropPosition.value = "before";
    } else if (y > h * 0.75) {
      dropPosition.value = "after";
    } else {
      dropPosition.value = "inside";
    }
  } else {
    dropPosition.value = y < h / 2 ? "before" : "after";
  }
}

function onDragLeave(_e: DragEvent, node: TreeNode) {
  if (dropTargetId.value === node.id) {
    dropTargetId.value = null;
    dropPosition.value = null;
  }
}

// --- Drop ---
function onDrop(e: DragEvent, targetNode: TreeNode) {
  e.preventDefault();
  if (!draggedId.value || !dropPosition.value) return;
  if (draggedId.value === targetNode.id) return;
  if (isDescendant(draggedId.value, targetNode.id, tree.value)) return;

  const dragged = removeNode(tree.value, draggedId.value);
  if (!dragged) return;

  if (dropPosition.value === "inside" && targetNode.type === "folder") {
    targetNode.children.push(dragged);
    targetNode.expanded = true;
  } else if (dropPosition.value === "before" || dropPosition.value === "after") {
    insertAt(tree.value, targetNode.id, dragged, dropPosition.value);
  }

  applyFolderSort();
  draggedId.value = null;
  dropTargetId.value = null;
  dropPosition.value = null;
}

// --- Drop on root (empty area) ---
function onDropRoot(e: DragEvent) {
  e.preventDefault();
  if (!draggedId.value) return;
  const dragged = removeNode(tree.value, draggedId.value);
  if (dragged) {
    tree.value.push(dragged);
  }
  applyFolderSort();
  draggedId.value = null;
  dropTargetId.value = null;
  dropPosition.value = null;
}

// --- Drop indicator class ---
function dropClass(node: TreeNode) {
  if (dropTargetId.value !== node.id || !dropPosition.value) return {};
  return {
    "drop-before": dropPosition.value === "before",
    "drop-after": dropPosition.value === "after",
    "drop-inside": dropPosition.value === "inside",
  };
}

// --- Context menu ---
const ctxMenu = ref<{ x: number; y: number } | null>(null);
const ctxTargetId = ref<string | null>(null);

let nextId = Number(localStorage.getItem(TREE_NEXT_ID_KEY)) || 100;

watch(tree, saveTree, { deep: true });

function onContextMenu(e: MouseEvent, node?: TreeNode) {
  e.preventDefault();
  ctxMenu.value = { x: e.clientX, y: e.clientY };
  ctxTargetId.value = node?.id ?? null;
}

function closeCtxMenu() {
  ctxMenu.value = null;
  ctxTargetId.value = null;
}

const showAddRepoDialog = ref(false);
const addRepoTargetId = ref<string | null>(null);

function addRepository() {
  addRepoTargetId.value = ctxTargetId.value;
  closeCtxMenu();
  showAddRepoDialog.value = true;
}

const showCloneDialog = ref(false);
const cloneTargetId = ref<string | null>(null);

function cloneRepository() {
  cloneTargetId.value = ctxTargetId.value;
  closeCtxMenu();
  showCloneDialog.value = true;
}

async function confirmClone(url: string, dest: string, name: string) {
  const target = cloneTargetId.value;
  showCloneDialog.value = false;
  cloneTargetId.value = null;

  // Клон может идти минуты — поднимаем нижний порог таймаута до 5 минут.
  const timeoutSecs = Math.max(networkTimeoutSecs.value, 300);
  try {
    await invoke<string>("do_clone", { url, dest, timeoutSecs });
  } catch (e) {
    logError(String(e));
    return;
  }
  selectedId.value = await addRepoToTree(dest, name, target);
}

function findAllRepos(nodes: TreeNode[]): RepoNode[] {
  const result: RepoNode[] = [];
  for (const n of nodes) {
    if (n.type === "repo") result.push(n);
    else result.push(...findAllRepos(n.children));
  }
  return result;
}

// Добавляет узел репозитория в дерево (в целевую папку targetId или в корень),
// подгружает текущую ветку. Возвращает id добавленного узла или существующего
// при дубликате пути.
async function addRepoToTree(path: string, name: string, targetId: string | null): Promise<string> {
  const existing = findAllRepos(tree.value).find((r) => r.path === path);
  if (existing) return existing.id;

  const id = `r${nextId++}`;
  const node: RepoNode = { id, type: "repo", name, path, branch: "", hasChanges: false };

  // Load branch info
  try {
    const info = await invoke<RepoInfo>("get_repo_info", { repoPath: path });
    node.branch = info.current_branch;
  } catch { /* leave empty */ }

  const target = targetId ? findNode(tree.value, targetId) : null;
  if (target?.type === "folder") {
    target.children.push(node);
    target.expanded = true;
  } else {
    tree.value.push(node);
  }
  return id;
}

async function confirmAddRepository(path: string, name: string, isGitRepo: boolean) {
  const target = addRepoTargetId.value;
  showAddRepoDialog.value = false;
  addRepoTargetId.value = null;

  // Пустая/не-git папка → инициализируем новый репозиторий
  if (!isGitRepo) {
    try {
      await invoke<RepoInfo>("do_init", { path });
    } catch (e) {
      logError(String(e));
      return;
    }
  }

  selectedId.value = await addRepoToTree(path, name, target);

  // Существующий репозиторий → открываем и сразу подтягиваем изменения с
  // remote. Для свежесозданного через init remote'а ещё нет — pull пропускаем.
  if (isGitRepo) emit("repo-added", path);
}

const showAddGroupDialog = ref(false);
const addGroupTargetId = ref<string | null>(null);

const showRenameDialog = ref(false);
const renameTargetId = ref<string | null>(null);

function addGroup() {
  addGroupTargetId.value = ctxTargetId.value;
  closeCtxMenu();
  showAddGroupDialog.value = true;
}

function confirmAddGroup(name: string) {
  const id = `f${nextId++}`;
  const node: FolderNode = { id, type: "folder", name, children: [], expanded: true };
  if (addGroupTargetId.value) {
    const target = findNode(tree.value, addGroupTargetId.value);
    if (target?.type === "folder") {
      target.children.push(node);
      target.expanded = true;
    } else {
      tree.value.push(node);
    }
  } else {
    tree.value.push(node);
  }
  selectedId.value = id;
  applyFolderSort();
  showAddGroupDialog.value = false;
  addGroupTargetId.value = null;
}

// --- Refresh repo info on load ---
async function refreshRepoNodes(nodes: TreeNode[]) {
  for (const n of nodes) {
    if (n.type === "repo" && n.path) {
      try {
        const info = await invoke<RepoInfo>("get_repo_info", { repoPath: n.path });
        n.branch = info.current_branch;
      } catch { /* repo may be gone */ }
    } else if (n.type === "folder") {
      await refreshRepoNodes(n.children);
    }
  }
}

onMounted(() => {
  if (tree.value.length > 0) {
    applyFolderSort();
    refreshRepoNodes(tree.value);
  }
  // Select the active repo in treeview
  if (repoPath.value) {
    const active = findAllRepos(tree.value).find((r) => r.path === repoPath.value);
    if (active) selectedId.value = active.id;
  }
});

// Watch for repo changes from outside (e.g. restoreLastRepo)
watch(repoPath, (path) => {
  if (path) {
    const active = findAllRepos(tree.value).find((r) => r.path === path);
    if (active) selectedId.value = active.id;
  }
});

function deleteNode() {
  if (ctxTargetId.value) {
    const node = findNode(tree.value, ctxTargetId.value);
    const isActiveRepo = node?.type === "repo" && node.path === repoPath.value;
    removeNode(tree.value, ctxTargetId.value);
    if (selectedId.value === ctxTargetId.value) {
      selectedId.value = "";
    }
    if (isActiveRepo) {
      closeRepo();
    }
  }
  closeCtxMenu();
}

function renameNode() {
  const node = findNode(tree.value, ctxTargetId.value || "");
  if (node) {
    renameTargetId.value = ctxTargetId.value;
    closeCtxMenu();
    showRenameDialog.value = true;
  }
}

function confirmRename(newName: string) {
  if (renameTargetId.value) {
    const node = findNode(tree.value, renameTargetId.value);
    if (node) {
      node.name = newName;
      // Rename may change a folder's alphabetical position.
      applyFolderSort();
      // Trigger reactivity
      tree.value = [...tree.value];
      saveTree();
    }
  }
  showRenameDialog.value = false;
  renameTargetId.value = null;
}

function triggerAddRepository() {
  addRepoTargetId.value = selectedId.value;
  showAddRepoDialog.value = true;
}

function triggerAddGroup() {
  addGroupTargetId.value = selectedId.value;
  showAddGroupDialog.value = true;
}

function triggerCloneRepository() {
  cloneTargetId.value = selectedId.value;
  showCloneDialog.value = true;
}

function triggerDeleteNode() {
  if (selectedId.value) {
    ctxTargetId.value = selectedId.value;
    deleteNode();
  }
}

defineExpose({
  selectedId,
  triggerAddRepository,
  triggerCloneRepository,
  triggerAddGroup,
  triggerDeleteNode,
});
</script>

<template>
  <div class="repos-panel" @click="closeCtxMenu">
    <div class="panel-title-bar">
      <span class="panel-title">{{ i18n.repos.title }}</span>
    </div>
    <div
      class="repos-tree"
      @dragover.prevent
      @drop="onDropRoot"
      @selectstart.prevent
      @contextmenu="onContextMenu($event)"
    >
      <template v-for="node in tree" :key="node.id">
        <!-- Folder -->
        <div
          v-if="node.type === 'folder'"
          class="tree-node folder-node"
          :class="{ selected: selectedId === node.id, dragging: draggedId === node.id, ...dropClass(node) }"
          draggable="true"
          @dragstart="onDragStart($event, node)"
          @dragend="onDragEnd"
          @dragover="onDragOver($event, node)"
          @dragleave="onDragLeave($event, node)"
          @drop.stop="onDrop($event, node)"
          @click="selectNode(node)"
          @contextmenu.stop="onContextMenu($event, node)"
        >
          <svg
            class="chevron"
            :class="{ expanded: node.expanded }"
            width="10" height="10" viewBox="0 0 12 12"
            @click.stop="toggleFolder(node)"
          >
            <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <svg width="14" height="14" viewBox="0 0 16 16" class="folder-icon">
            <path d="M2 4h5l1.5-2H14v10H2V4z" fill="none" stroke="currentColor" stroke-width="1.2"/>
          </svg>
          <span class="node-name">{{ node.name }}</span>
        </div>

        <!-- Folder children -->
        <div v-if="node.type === 'folder' && node.expanded" class="folder-children">
          <template v-for="child in node.children" :key="child.id">
            <!-- Nested folder -->
            <div
              v-if="child.type === 'folder'"
              class="tree-node folder-node indented"
              :class="{ selected: selectedId === child.id, dragging: draggedId === child.id, ...dropClass(child) }"
              draggable="true"
              @dragstart="onDragStart($event, child)"
              @dragend="onDragEnd"
              @dragover="onDragOver($event, child)"
              @dragleave="onDragLeave($event, child)"
              @drop.stop="onDrop($event, child)"
              @click="selectNode(child)"
              @contextmenu.stop="onContextMenu($event, child)"
            >
              <svg
                class="chevron"
                :class="{ expanded: (child as any).expanded }"
                width="10" height="10" viewBox="0 0 12 12"
                @click.stop="toggleFolder(child as any)"
              >
                <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              <svg width="14" height="14" viewBox="0 0 16 16" class="folder-icon">
                <path d="M2 4h5l1.5-2H14v10H2V4z" fill="none" stroke="currentColor" stroke-width="1.2"/>
              </svg>
              <span class="node-name">{{ child.name }}</span>
            </div>

            <!-- Repo inside folder -->
            <div
              v-else
              class="tree-node repo-node indented"
              :class="{ selected: selectedId === child.id, active: isActiveRepo(child), dragging: draggedId === child.id, ...dropClass(child) }"
              draggable="true"
              @dragstart="onDragStart($event, child)"
              @dragend="onDragEnd"
              @dragover="onDragOver($event, child)"
              @dragleave="onDragLeave($event, child)"
              @drop.stop="onDrop($event, child)"
              @click="selectNode(child)"
              @dblclick="openRepoNode(child)"
              @contextmenu.stop="onContextMenu($event, child)"
            >
              <svg width="14" height="14" viewBox="0 0 16 16" class="repo-icon" :class="{ changes: (child as RepoNode).hasChanges }">
                <path d="M3 2h10v12H3z" fill="none" stroke="currentColor" stroke-width="1.2"/>
                <path d="M5 5h6M5 8h6M5 11h4" fill="none" stroke="currentColor" stroke-width="1"/>
              </svg>
              <span class="node-name">{{ child.name }}</span>
              <span v-if="(child as RepoNode).branch" class="repo-branch">({{ (child as RepoNode).branch }})</span>
              <span v-if="(child as RepoNode).hasChanges" class="changes-dot" />
            </div>
          </template>
        </div>

        <!-- Top-level repo -->
        <div
          v-if="node.type === 'repo'"
          class="tree-node repo-node"
          :class="{ selected: selectedId === node.id, active: isActiveRepo(node), dragging: draggedId === node.id, ...dropClass(node) }"
          draggable="true"
          @dragstart="onDragStart($event, node)"
          @dragend="onDragEnd"
          @dragover="onDragOver($event, node)"
          @dragleave="onDragLeave($event, node)"
          @drop.stop="onDrop($event, node)"
          @click="selectNode(node)"
          @dblclick="openRepoNode(node)"
          @contextmenu.stop="onContextMenu($event, node)"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" class="repo-icon" :class="{ changes: node.hasChanges }">
            <path d="M3 2h10v12H3z" fill="none" stroke="currentColor" stroke-width="1.2"/>
            <path d="M5 5h6M5 8h6M5 11h4" fill="none" stroke="currentColor" stroke-width="1"/>
          </svg>
          <span class="node-name">{{ node.name }}</span>
          <span v-if="node.branch" class="repo-branch">({{ node.branch }})</span>
          <span v-if="node.hasChanges" class="changes-dot" />
        </div>
      </template>
    </div>

    <!-- Context menu -->
    <Teleport to="body">
      <div
        v-if="ctxMenu"
        class="ctx-menu"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" @click="addRepository">{{ i18n.repos.addRepository }}</button>
        <button class="ctx-item" @click="cloneRepository">{{ i18n.repos.cloneRepository }}</button>
        <button class="ctx-item" @click="addGroup">{{ i18n.repos.addGroup }}</button>
        <button v-if="ctxTargetId" class="ctx-item" @click="renameNode">{{ i18n.repos.rename }}</button>
        <div v-if="ctxTargetId" class="ctx-separator" />
        <button v-if="ctxTargetId" class="ctx-item ctx-danger" @click="deleteNode">{{ i18n.repos.delete }}</button>
      </div>
      <div v-if="ctxMenu" class="ctx-backdrop" @click="closeCtxMenu" @contextmenu.prevent="closeCtxMenu" />

      <AddGroupDialog
        v-if="showAddGroupDialog"
        @close="showAddGroupDialog = false"
        @confirm="confirmAddGroup"
      />

      <AddRepositoryDialog
        v-if="showAddRepoDialog"
        @close="showAddRepoDialog = false"
        @confirm="confirmAddRepository"
      />

      <CloneRepositoryDialog
        v-if="showCloneDialog"
        @close="showCloneDialog = false"
        @confirm="confirmClone"
      />

      <RenameNodeDialog
        v-if="showRenameDialog && renameTargetId"
        :old-name="findNode(tree, renameTargetId)?.name || ''"
        :node-type="findNode(tree, renameTargetId)?.type === 'repo' ? 'repo' : 'folder'"
        @close="showRenameDialog = false"
        @confirm="confirmRename"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.repos-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
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
.repos-tree {
  flex: 1;
  padding: 4px 0;
  overflow-y: auto;
  min-height: 40px;
}

/* --- Tree nodes --- */
.tree-node {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  cursor: grab;
  font-size: var(--font-size-sm);
  user-select: none;
  position: relative;
}
.tree-node:hover {
  background: var(--bg-hover);
}
.tree-node.selected {
  background: var(--bg-surface);
}
/* Неактивные репозитории слегка приглушены */
.repo-node .node-name {
  color: var(--text-secondary);
}
/* Открытый репозиторий: линия слева и жирное имя, фон не фиксируется */
.tree-node.active {
  box-shadow: inset 2px 0 0 var(--accent);
}
.tree-node.active .node-name {
  color: var(--text-primary);
  font-weight: 600;
}
.tree-node.dragging {
  opacity: 0.4;
}
.tree-node.indented {
  padding-left: 28px;
}

/* --- Drop indicators --- */
.tree-node.drop-before::before {
  content: "";
  position: absolute;
  top: -1px;
  left: 4px;
  right: 4px;
  height: 2px;
  background: var(--accent);
  border-radius: 1px;
}
.tree-node.drop-after::after {
  content: "";
  position: absolute;
  bottom: -1px;
  left: 4px;
  right: 4px;
  height: 2px;
  background: var(--accent);
  border-radius: 1px;
}
.tree-node.drop-inside {
  outline: 1.5px solid var(--accent);
  outline-offset: -1.5px;
  border-radius: var(--radius);
}

/* --- Folder --- */
.chevron {
  transition: transform 0.15s;
  flex-shrink: 0;
  cursor: pointer;
}
.chevron.expanded {
  transform: rotate(90deg);
}
.folder-icon {
  color: var(--yellow);
  flex-shrink: 0;
}

/* --- Repo --- */
.repo-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}
.repo-icon.changes {
  color: var(--yellow);
}
.repo-node {
  cursor: pointer;
}
.repo-node:active {
  cursor: grabbing;
}

.node-name {
  color: var(--text-primary);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.repo-branch {
  color: var(--text-muted);
  font-size: var(--font-size-xs);
  flex-shrink: 0;
}

.changes-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--yellow);
  flex-shrink: 0;
  margin-left: auto;
}

.folder-children {
  /* no extra styling, just a wrapper */
}

/* --- Context menu --- */
.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
}
.ctx-menu {
  position: fixed;
  z-index: 1000;
  min-width: 160px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
}
.ctx-item {
  display: block;
  width: 100%;
  padding: 6px 12px;
  text-align: left;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  background: none;
  border: none;
  cursor: pointer;
}
.ctx-item:hover {
  background: var(--bg-hover);
}
.ctx-danger {
  color: var(--red);
}
.ctx-danger:hover {
  background: rgba(243, 139, 168, 0.1);
}
.ctx-separator {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}
</style>
