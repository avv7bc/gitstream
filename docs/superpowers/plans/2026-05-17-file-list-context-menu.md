# Контекстное меню панели файлов — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Добавить мультивыделение (Ctrl/Shift+клик) и контекстное меню (Stage, Unstage, Commit, Discard, Remove, Delete) в панель файлов рабочего дерева.

**Architecture:** Состояние выделения локально в `FileList.vue`. Контекстное меню — инлайн `<Teleport to="body">` по образцу `CommitGraph.vue` (переиспуются CSS-классы). Две новые backend-команды: `remove_files` (`git rm` + фоллбэк для untracked) и `delete_files` (удаление с диска). Commit открывает существующий `CommitDialog` через emit.

**Tech Stack:** Rust (Tauri 2, git CLI), Vue 3 Composition API + TypeScript.

Спека: `docs/superpowers/specs/2026-05-17-file-list-context-menu-design.md`

Применяемые правила пользователя: коммиты без AI-атрибуции; self-check (`npm run build`, `cargo test`) перед сдачей; бамп patch-версии в `src/components/StatusBar.vue` (сейчас `0.1.39` → `0.1.40`); поведение по образцу эталонного клиента.

---

### Task 1: Backend — функции `remove` и `delete` в mutation.rs (TDD)

**Files:**
- Modify: `src-tauri/src/git/mutation.rs` (добавить функции после `discard`, ~строка 115; тесты — в существующий `mod tag_tests`, ~строка 514)

- [ ] **Step 1: Написать падающие тесты**

В `src-tauri/src/git/mutation.rs` внутри `mod tag_tests` (после `deletes_tag`, перед `fetch_args_basic`) добавить:

```rust
    #[test]
    fn remove_tracked_file_stages_deletion() {
        let dir = temp_repo();
        remove(&dir, &["a.txt".to_string()]).unwrap();
        assert!(!dir.join("a.txt").exists());
        let out = Command::new("git")
            .current_dir(&dir)
            .args(["status", "--porcelain"])
            .output()
            .unwrap();
        // staged deletion -> "D  a.txt"
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "D  a.txt");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn remove_untracked_file_falls_back_to_disk_delete() {
        let dir = temp_repo();
        fs::write(dir.join("u.txt"), "x").unwrap();
        remove(&dir, &["u.txt".to_string()]).unwrap();
        assert!(!dir.join("u.txt").exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_removes_file_from_disk_only() {
        let dir = temp_repo();
        delete(&dir, &["a.txt".to_string()]).unwrap();
        assert!(!dir.join("a.txt").exists());
        let out = Command::new("git")
            .current_dir(&dir)
            .args(["status", "--porcelain"])
            .output()
            .unwrap();
        // unstaged deletion -> " D a.txt"
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "D a.txt");
        fs::remove_dir_all(&dir).ok();
    }
```

- [ ] **Step 2: Запустить тесты — убедиться, что не компилируются/падают**

Run: `cd src-tauri && cargo test --lib remove_tracked_file_stages_deletion delete_removes_file_from_disk_only remove_untracked_file_falls_back_to_disk_delete`
Expected: ошибка компиляции `cannot find function remove`/`delete`.

- [ ] **Step 3: Реализовать `remove` и `delete`**

В `src-tauri/src/git/mutation.rs` сразу после функции `discard` (после строки `}` на ~строке 115) добавить:

```rust
/// `git rm` для tracked-файлов (удаляет с диска + стейджит удаление).
/// Untracked-файл `git rm` не берёт — для него фоллбэк: удалить с диска.
pub fn remove(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    let mut args = vec!["rm", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    match run_git_mut(repo_path, &args) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Вероятно среди файлов untracked — удаляем их с диска напрямую.
            for f in files {
                std::fs::remove_file(repo_path.join(f)).ok();
            }
            Ok(())
        }
    }
}

/// Удаляет файлы только с диска (git не трогаем — tracked станет
/// "deleted, unstaged").
pub fn delete(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    for f in files {
        std::fs::remove_file(repo_path.join(f))
            .map_err(|e| GitError::new("io", e.to_string()))?;
    }
    Ok(())
}
```

Примечание: проверить сигнатуру конструктора `GitError`. Открыть `src-tauri/src/git/error.rs`, найти как создаётся `GitError` (конструктор/變ант). Если нет `GitError::new(kind, msg)`, использовать фактический способ создания из этого файла (напр. `GitError { ... }` или `classify_git_error`). Заменить `GitError::new("io", e.to_string())` на корректный вызов.

- [ ] **Step 4: Запустить тесты — убедиться, что проходят**

Run: `cd src-tauri && cargo test --lib remove_tracked_file_stages_deletion delete_removes_file_from_disk_only remove_untracked_file_falls_back_to_disk_delete`
Expected: 3 passed.

- [ ] **Step 5: Полный прогон тестов backend**

Run: `cd src-tauri && cargo test --lib`
Expected: все тесты проходят (0 failed).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/git/mutation.rs
git commit -m "feat(files): backend remove (git rm) и delete (с диска)"
```

---

### Task 2: Tauri-команды `remove_files` / `delete_files`

**Files:**
- Modify: `src-tauri/src/commands.rs` (после `discard_files`, ~строка 160)
- Modify: `src-tauri/src/main.rs` (`invoke_handler`, после `commands::discard_files`, ~строка 34)

- [ ] **Step 1: Добавить команды**

В `src-tauri/src/commands.rs` сразу после функции `discard_files` (после её закрывающей `}`) добавить:

```rust
#[tauri::command]
pub fn remove_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::remove(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_files(repo_path: String, files: Vec<String>) -> Result<(), String> {
    mutation::delete(Path::new(&repo_path), &files).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Зарегистрировать в invoke_handler**

В `src-tauri/src/main.rs` найти строку `commands::discard_files,` (~строка 34) и сразу после неё добавить:

```rust
            commands::remove_files,
            commands::delete_files,
```

- [ ] **Step 3: Проверить сборку backend**

Run: `cd src-tauri && cargo check`
Expected: компилируется без ошибок.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "feat(files): tauri-команды remove_files и delete_files"
```

---

### Task 3: Composable — `removeFiles` / `deleteFiles` в useFiles.ts

**Files:**
- Modify: `src/composables/useFiles.ts` (после `discardFiles`, ~строка 61; и блок `return`, ~строка 96)

- [ ] **Step 1: Добавить функции**

В `src/composables/useFiles.ts` сразу после функции `discardFiles` (после её закрывающей `}` на ~строке 61) добавить:

```typescript
  async function removeFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("remove_files", { repoPath: repoPath.value, files: paths });
    await refreshAfterMutation();
  }

  async function deleteFiles(paths: string[]) {
    if (!repoPath.value) return;
    await invoke("delete_files", { repoPath: repoPath.value, files: paths });
    await refreshAfterMutation();
  }
```

- [ ] **Step 2: Экспортировать в return**

В `src/composables/useFiles.ts` в объекте `return { ... }` (~строка 96) после `discardFiles,` добавить:

```typescript
    removeFiles,
    deleteFiles,
```

- [ ] **Step 3: Проверка типов**

Run: `npx vue-tsc --noEmit`
Expected: без новых ошибок типов.

- [ ] **Step 4: Commit**

```bash
git add src/composables/useFiles.ts
git commit -m "feat(files): useFiles removeFiles/deleteFiles"
```

---

### Task 4: Мультивыделение в FileList.vue (Ctrl/Shift+клик)

**Files:**
- Modify: `src/components/FileList.vue` (`<script setup>` и working-tree `v-for`, строки ~12-189)

- [ ] **Step 1: Добавить состояние и логику выделения в `<script setup>`**

В `src/components/FileList.vue` после строки `const commitFiles = ref<FileDiff[]>([]);` (~строка 20) добавить:

```typescript
const selectedPaths = ref<string[]>([]);
const anchorPath = ref<string | null>(null);
```

Заменить функцию `selectFile` (строки ~100-107) на:

```typescript
async function selectFile(path: string, e?: MouseEvent) {
  const list = filteredFiles.value.map((f) => f.path);
  if (e?.shiftKey && anchorPath.value) {
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
  if (isWorkingTree.value) {
    const f = files.value.find((x) => x.path === path);
    const staged = f?.staged === "staged";
    await diffFile(path, staged);
  }
}
```

- [ ] **Step 2: Прокинуть событие клика и подсветку в шаблоне**

В `src/components/FileList.vue` в working-tree `v-for` (`<div ... class="file-item" ... @click="selectFile(file.path)">`, ~строка 164) заменить:
- `@click="selectFile(file.path)"` → `@click="selectFile(file.path, $event)"`
- `:class="{ selected: selectedFile === file.path }"` → `:class="{ selected: selectedPaths.includes(file.path) }"`

(Commit-файлы `v-for` ниже — НЕ трогать, там остаётся `selectedFile === cf.path` и `selectCommitFile`.)

- [ ] **Step 3: Сбрасывать выделение при смене working-tree/коммита**

В `src/components/FileList.vue` в существующем `watch(selectedCommit, ...)` (~строка 28), в ветке сброса (`if (!oid || oid === "__worktree__" ...)` блок и при установке commitFiles) — добавить сразу первой строкой внутри `watch`-колбэка:

```typescript
  selectedPaths.value = [];
  anchorPath.value = null;
```

(Ставить в самое начало async-колбэка `watch`, до проверок.)

- [ ] **Step 4: Проверка типов и сборка**

Run: `npm run build`
Expected: сборка проходит без ошибок.

- [ ] **Step 5: Ручная проверка**

Запустить `npm run tauri dev`, открыть репозиторий с несколькими изменёнными файлами:
- обычный клик — выделяется один файл, показывается diff;
- Ctrl+клик — добавляет/убирает файлы из выделения (несколько подсвечены);
- Shift+клик — выделяет диапазон от первого кликнутого.
Expected: поведение соответствует описанному.

- [ ] **Step 6: Commit**

```bash
git add src/components/FileList.vue
git commit -m "feat(files): мультивыделение Ctrl/Shift+клик в панели файлов"
```

---

### Task 5: Контекстное меню + действия + проводка в App.vue + версия

**Files:**
- Modify: `src/components/FileList.vue` (`<script setup>`, шаблон, `<style scoped>`)
- Modify: `src/App.vue` (`<FileList />`, ~строка 350)
- Modify: `src/components/StatusBar.vue` (версия, строка 49)

- [ ] **Step 1: Добавить emit, методы меню и состояние в `<script setup>`**

В `src/components/FileList.vue` в начале `<script setup>` (после импортов, перед `const { files, selectedFile } = useFiles();`) добавить:

```typescript
const emit = defineEmits<{ commit: [] }>();
```

Расширить деструктуризацию `useFiles` (строка ~12) — заменить:

```typescript
const { files, selectedFile } = useFiles();
```

на:

```typescript
const { files, selectedFile, stageFiles, unstageFiles, discardFiles, removeFiles, deleteFiles } = useFiles();
```

После `const anchorPath = ref<string | null>(null);` (из Task 4) добавить:

```typescript
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
    window.alert(`Action failed: ${err}`);
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && ctxMenu.value) closeCtxMenu();
}
</script>
```

Примечание: НЕ дублировать закрывающий `</script>` — он уже есть в файле; добавить только `onKeydown` перед ним и зарегистрировать слушатель. Для регистрации добавить в импорт из `vue` (строка 2) `onMounted, onUnmounted` и после определения функций добавить:

```typescript
onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
```

- [ ] **Step 2: Добавить `@contextmenu` на working-tree файлы**

В `src/components/FileList.vue` в working-tree `v-for` (`<div ... class="file-item" ...>`, ~строка 159-166) добавить атрибут:

```
@contextmenu="onFileContextMenu($event, file.path)"
```

(Существующий `@contextmenu.prevent` на `.file-list-body` оставить — он гасит дефолтное меню на пустой области.)

- [ ] **Step 3: Добавить разметку меню перед закрывающим `</div>` шаблона**

В `src/components/FileList.vue` сразу перед закрывающим `</div>` элемента `.file-list` (перед `</template>`, ~строка 211) вставить:

```html
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
          <span class="ctx-label">Commit…</span>
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
```

- [ ] **Step 4: Добавить CSS классов меню в `<style scoped>`**

В `src/components/FileList.vue` в конец блока `<style scoped>` (перед `</style>`, ~строка 347) добавить (скопировано из паттерна `CommitGraph.vue` для единого вида):

```css
.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
}
.ctx-menu {
  position: fixed;
  z-index: 1000;
  min-width: 180px;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius);
  padding: 4px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
}
.ctx-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 5px 10px;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  border-radius: var(--radius);
  text-align: left;
  width: 100%;
}
.ctx-item:hover:not(:disabled) {
  background: var(--bg-hover);
}
.ctx-item:disabled {
  color: var(--text-muted);
  opacity: 0.5;
  cursor: default;
}
.ctx-item.ctx-danger:not(:disabled) {
  color: var(--red);
}
.ctx-separator {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}
```

Примечание: открыть `src/components/CommitGraph.vue`, найти там реальные определения `.ctx-menu/.ctx-item/.ctx-separator/.ctx-backdrop` и при расхождении переменных/значений привести добавляемый CSS в точное соответствие, чтобы вид меню совпадал 1-в-1.

- [ ] **Step 5: Прокинуть `@commit` в App.vue**

В `src/App.vue` заменить `<FileList />` (~строка 350) на:

```html
<FileList @commit="showCommitDialog = true" />
```

- [ ] **Step 6: Бамп версии**

В `src/components/StatusBar.vue` строка 49 заменить `0.1.39` на `0.1.40`.

- [ ] **Step 7: Сборка и проверка типов**

Run: `npm run build`
Expected: сборка проходит без ошибок.

- [ ] **Step 8: Ручная проверка**

`npm run tauri dev`, репозиторий с изменениями:
- ПКМ по файлу вне выделения → он становится единственным выделением, меню открывается у курсора;
- ПКМ по файлу внутри мультивыделения → меню действует на всё выделение;
- Stage/Unstage активны/неактивны по состоянию файлов; после действия список обновляется;
- Commit открывает диалог коммита;
- Discard/Remove/Delete показывают confirm (с количеством при N>1), выполняют действие;
- меню закрывается по клику вне, Esc, после действия.
Expected: всё соответствует спеке.

- [ ] **Step 9: Финальный self-check**

Run: `cd src-tauri && cargo test --lib && cd .. && npm run build`
Expected: Rust-тесты проходят, фронтенд собирается.

- [ ] **Step 10: Commit**

```bash
git add src/components/FileList.vue src/App.vue src/components/StatusBar.vue
git commit -m "feat(files): контекстное меню Stage/Unstage/Commit/Discard/Remove/Delete + bump 0.1.40"
```

---

## Самопроверка плана (выполнено автором)

- **Покрытие спеки:** backend remove/delete — Task 1-2; composable — Task 3; мультивыделение Ctrl/Shift — Task 4; меню, enable/disable, confirm, emit commit, версия — Task 5. Все пункты спеки покрыты.
- **Плейсхолдеры:** нет «TBD/TODO»; весь код приведён. Два примечания требуют сверки с фактическими определениями (`GitError` конструктор, CSS-классы в `CommitGraph.vue`) — это явные шаги проверки, не плейсхолдеры.
- **Согласованность типов/имён:** `removeFiles/deleteFiles` (useFiles) ↔ `remove_files/delete_files` (Rust команды) ↔ `remove/delete` (mutation.rs); `selectedPaths/anchorPath/ctxMenu/onFileContextMenu/closeCtxMenu/ctxRun/ctxLabel/canStage/canUnstage` — единообразны между Task 4 и 5; emit `commit` ↔ `@commit` в App.vue.
