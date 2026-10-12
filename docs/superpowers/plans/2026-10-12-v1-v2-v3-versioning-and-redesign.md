# GitStream v1/v2/v3 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Сохранить текущий продукт в ветке `v1`, довести ветку `v2` до полного функционального набора на существующем интерфейсе, а затем создать `v3` с тем же функционалом и новым дизайном A «Графит».

**Architecture:** `v1` — замороженная стабильная линия с текущим UI. `v2` — функциональная линия без визуального редизайна; общие backend-команды, модели, composables и тесты добавляются здесь поверх существующих компонентов. `v3` создаётся только после приёмки v2 и переиспользует её функциональный слой, заменяя presentation layer на макеты `docs/design/A`.

**Tech Stack:** Vue 3, TypeScript, Vite, Tauri 2, Rust, git CLI, Vitest, Cargo test/clippy, GitHub Actions.

**Spec:** `docs/ROADMAP-2.0.md`, `docs/SPEC-redesign-2026.md`, `docs/design/A/`, `reports/Сравнение Git клиентов с GitStream.md`.

## Global Constraints

- Текущий `main` переименовывается в `v1`; история текущего HEAD не переписывается.
- `v2` создаётся от `v1`; `v3` создаётся только после завершения и приёмки v2.
- Ветка `v1` получает только исправления стабильности и безопасности; новые функции туда не переносятся.
- Ветка `v2` сохраняет legacy-визуальный стиль; дизайн A относится к v3.
- Точные выпуски фиксируются Git-тегами; до подготовки первого релиза версии в `package.json`, `src-tauri/Cargo.toml` и `src-tauri/tauri.conf.json` не меняются.
- Backend использует только системный git CLI через существующий Tauri/Rust слой; libgit2 не добавляется.
- Минимальная версия Git для preview конфликтов и `--update-refs` — 2.38; на более старых версиях функцию нужно скрывать.
- Перед каждым отчётом о готовности запускать `npx vue-tsc --noEmit`, `npm test`, `cargo clippy --all-targets -- -D warnings` и `cargo test` в `src-tauri`.
- Коммиты, push, создание тегов и удаление удалённых веток выполнять только после явного одобрения пользователя для этого плана.

---

### Task 1: Зафиксировать модель веток и дорожные карты

**Files:**
- Modify: `docs/ROADMAP-2.0.md`
- Create: `docs/ROADMAP-3.0.md`
- Modify: `docs/SPEC-redesign-2026.md`
- Modify: `CLAUDE.md`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-10-11-v1-v2-versioning.md`

**Interfaces:**
- Produces the single branch/version policy consumed by CI, release notes, and later implementation tasks.

- [ ] Step 1: Rewrite `ROADMAP-2.0.md` so v2 means functional maturity on the current UI: oplog/Undo/Redo, reflog, commit composer, reliable Push/Merge/Rebase flows, command registry, history editing, interactive rebase, resolver, worktrees, command palette, syntax highlighting, DnD, server-side log filters, and the remaining P1 items from the existing spec.
- [ ] Step 2: Create `ROADMAP-3.0.md` by moving the A-specific work there: semantic tokens, themes, fonts, `BaseDialog`, A toolbar/status bar/file panel, welcome and empty states, notification presentation, and visual QA against `docs/design/A`.
- [ ] Step 3: Update `SPEC-redesign-2026.md` so its functional sections are v2 deliverables and its visual sections are v3 deliverables; remove statements that v2 already equals design A.
- [ ] Step 4: Replace the old `2026-10-11-v1-v2-versioning.md` with a short superseded-plan note linking this plan; do not leave two active branch policies.
- [ ] Step 5: Document the policy in `CLAUDE.md` and `README.md`: `v1` stable legacy, `v2` functional line, `v3` design A; tags carry exact semantic versions.
- [ ] Step 6: Check for contradictory branch/version references.

Run: `rg -n 'release/1\\.x|main.*2|Версия 2 = дизайн|1\\.2\\.0|1\\.7\\.0' CLAUDE.md README.md docs`
Expected: no obsolete branch model remains outside the historical archive note.

---

### Task 2: Create the branch lines

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Produces local and remote `v1` and `v2` lines; `v3` is deliberately deferred until Task 12.

- [ ] Step 1: Verify the worktree is clean and record the current commit hash.

Run: `git status --short --branch && git rev-parse HEAD`
Expected: clean `main` worktree and the baseline hash recorded in the task log.

- [ ] Step 2: Rename the local `main` branch to `v1` and push it to `origin` with upstream tracking.
- [ ] Step 3: Set the remote default branch to `v1` with `gh repo edit avv7bc/gitstream --default-branch v1`, verify `git ls-remote --symref origin HEAD`, then delete remote `main` with `git push origin --delete main` only after `v1` is confirmed available.
- [ ] Step 4: Create `v2` from `v1` and push it with upstream tracking.
- [ ] Step 5: Update CI triggers to run on `v1`, `v2`, and later `v3`; keep tag-based releases independent of branch names.
- [ ] Step 6: Verify the branch graph and remote refs.

Run: `git branch -a -vv && git ls-remote --heads origin`
Expected: `v1` and `v2` point to the baseline; no work has been started on `v3`.

---

### Task 3: Make update checks major-line aware

**Files:**
- Create: `src-tauri/src/update.rs`
- Modify: `src-tauri/src/main.rs`, `src-tauri/src/git/types.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/settings.rs`
- Modify: `src/types/index.ts`, `src/composables/useUpdate.ts`, `src/components/UpdateBanner.vue`, `src/App.vue`
- Modify: `src/locales/ru.ts`, `src/locales/en.ts`
- Modify: `.github/workflows/ci.yml`, `.github/workflows/release.yml`

**Interfaces:**
- Rust: `pick_update(releases: &[ReleaseEntry], current: &Version) -> UpdateCheck`.
- IPC: `check_for_update() -> UpdateCheck` with same-major update and optional next-major notice.
- TypeScript: `UpdateCheck { update: UpdateInfo | null; next_major: UpdateInfo | null }`.
- Branch flow: implement the compatibility logic on `v1`, then bring the same commits to `v2`; `v3` inherits it from v2.

- [ ] Step 1: Add failing Rust tests for stable, prerelease, same-major, next-major, draft, and missing-URL release selection.
- [ ] Step 2: Implement `ReleaseEntry`, `UpdateInfo`, `UpdateCheck`, and `pick_update` without using GitHub `releases/latest`.
- [ ] Step 3: Add frontend tests for same-major banners, one-time next-major dismissal, and a later next-major replacing a dismissed version.
- [ ] Step 4: Wire the update command, banners, locales, and settings compatibility.
- [ ] Step 5: Test that newer settings fields are ignored instead of resetting v1 settings.

Run: `cargo test update` and `npm test -- src/composables/useUpdate.test.ts`
Expected: all update-selection and settings-compatibility tests pass.

---

### Task 4: Build the shared command and error surfaces on v2

**Files:**
- Create: `src/commands/types.ts`, `src/commands/registry.ts`, `src/commands/context.ts`, `src/commands/hotkeys.ts`
- Create: `src/components/ContextMenu.vue`, `src/composables/useToast.ts`, `src/components/ToastStack.vue`
- Modify: `src/App.vue`, `src/components/AppToolbar.vue`, `src/components/CommitGraph.vue`, `src/components/FileList.vue`, `src/components/BranchPanel.vue`, `src/components/RepositoriesPanel.vue`, `src/composables/useProgress.ts`
- Modify: `src/locales/ru.ts`, `src/locales/en.ts`

**Interfaces:**
- `CommandContext`, `Command`, `getCommand(id)`, `commandsForScope(scope, ctx)`, `dispatchHotkey(event)`.
- `toast`, `updateToast`, `dismissToast`; errors continue to be written to Git output and additionally receive an error toast.

- [ ] Step 1: Add registry tests for command visibility/enabled predicates and shortcut dispatch.
- [ ] Step 2: Move existing stage, unstage, discard, checkout, merge, rebase, commit, push, fetch, and settings actions into the registry while preserving behavior.
- [ ] Step 3: Replace duplicated context-menu lists with `ContextMenu.vue`; hide irrelevant commands and disable only applicable-but-unavailable commands.
- [ ] Step 4: Add toast state and stack; change `logError` so it does not steal the Changes view.
- [ ] Step 5: Route the current toolbar and global hotkeys through the registry.
- [ ] Step 6: Verify the existing legacy UI still supports every current keyboard shortcut.

Run: `npm test && npx vue-tsc --noEmit`
Expected: no duplicated command wiring remains in the touched components and all existing tests pass.

---

### Task 5: Complete v2 daily operations in the legacy UI

**Files:**
- Modify: `src/components/AppToolbar.vue`, `src/App.vue`
- Modify: `src/components/dialogs/PushDialog.vue`, `src/components/dialogs/PullDialog.vue`, `src/components/dialogs/ConfirmDialog.vue`
- Create: `src/components/dialogs/MergeDialog.vue`, `src/components/dialogs/RebaseDialog.vue`
- Create: `src/components/CommitComposer.vue`
- Modify: `src/components/FileList.vue`, `src/components/BranchPanel.vue`, `src/components/CommitGraph.vue`, `src/components/SideBySideDiffView.vue`
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/git/query.rs`, `src-tauri/src/git/mutation.rs`

**Interfaces:**
- Merge/Rebase preview returns `{ clean, conflicted_files, fast_forward }`.
- `CommitComposer` uses the existing commit composable and emits `committed` after a successful commit.
- All destructive actions use `ConfirmDialog`; no `window.confirm` remains.

- [ ] Step 1: Add tests for preview parsing and the legacy UI's enabled-state predicates.
- [ ] Step 2: Implement reachable Push, Merge, and Rebase dialogs with command previews and explicit remote/upstream choices.
- [ ] Step 3: Implement `preview_merge` and the Git-version guard for `git merge-tree`.
- [ ] Step 4: Add the inline commit composer while retaining the existing full commit dialog as a compatibility path.
- [ ] Step 5: Replace every `window.confirm` call and add tests for reset, discard, remove, delete, and rebase confirmations.
- [ ] Step 6: Add success/progress/error toasts without removing Git output.

Run: `rg -n 'window\\.confirm' src`
Expected: no matches; manual legacy UI check covers commit, push, pull, merge, rebase, reset, and discard.

---

### Task 6: Implement reliable operation snapshots, Undo/Redo, and Reflog

**Files:**
- Create: `src-tauri/src/git/oplog.rs`, `src-tauri/src/git/reflog.rs`
- Modify: `src-tauri/src/git/mod.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/git/mutation.rs`, `src-tauri/src/git/types.rs`, `src-tauri/src/main.rs`
- Create: `src/composables/useOplog.ts`, `src/components/OplogPanel.vue`
- Modify: `src/components/AppToolbar.vue`, `src/App.vue`, `src/components/ToastStack.vue`

**Interfaces:**
- Rust: `snapshot(repo, op) -> OpSnapshot`, `undo(repo, id)`, `redo(repo, id)`, `get_oplog(repo)`, `get_reflog(repo, refname, limit)`.
- `OpSnapshot` records HEAD, refs, index tree, and a complete worktree snapshot including untracked files.

- [ ] Step 1: Add integration tests for staged, unstaged, deleted, binary, symlink, and untracked files before implementation.
- [ ] Step 2: Implement worktree capture with a temporary index and explicit untracked-file blobs/metadata; do not rely on bare `git stash create`, which omits untracked files.
- [ ] Step 3: Implement ref and HEAD snapshots with protected `refs/gitstream/oplog/*` references and rotation to the latest 50 operations.
- [ ] Step 4: Wrap every local mutation, including discard, stash, commit, reset, branch/tag operations, merge/rebase, checkout, and pull integration.
- [ ] Step 5: Implement guarded undo/redo with clean-state checks and conflict-safe confirmation when the worktree changed.
- [ ] Step 6: Add Reflog commands and the v2 legacy panel/actions.
- [ ] Step 7: Run temporary-repository integration tests for reset-hard, branch delete, stash pop, discard, amend, undo, and redo.

Run: `cargo test oplog -- --nocapture`
Expected: tracked and untracked files, index state, refs, and detached HEAD states restore correctly.

---

### Task 7: Add history editing, interactive rebase, and worktrees

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/git/mutation.rs`, `src-tauri/src/git/query.rs`, `src-tauri/src/git/types.rs`
- Create: `src/components/dialogs/InteractiveRebaseDialog.vue`, `src/components/dialogs/WorktreeDialog.vue`
- Modify: `src/components/CommitGraph.vue`, `src/components/BranchPanel.vue`, `src/components/RepositoriesPanel.vue`, `src/components/ConflictBar.vue`
- Modify: `src/composables/useBranches.ts`, `src/types/index.ts`, `src/locales/ru.ts`, `src/locales/en.ts`

**Interfaces:**
- `do_rebase_plan(repo, base, plan: TodoItem[])` is the shared backend entry point for fixup, drop, move, squash, reword, and interactive rebase.
- `get_worktrees`, `do_worktree_add`, `do_worktree_remove`, `do_worktree_prune` expose porcelain worktree data.

- [ ] Step 1: Add Rust tests for todo-plan validation, root commits, `--update-refs`, autostash, and protected pushed commits.
- [ ] Step 2: Implement `do_rebase_plan` through the existing sequence-editor helper and record it in oplog.
- [ ] Step 3: Add legacy interactive-rebase and one-action history commands with conflict handoff to `ConflictBar`.
- [ ] Step 4: Implement worktree query/mutation commands and tests for linked, detached, locked, and prunable worktrees.
- [ ] Step 5: Add worktree nodes and dialogs to the legacy repositories/branches UI.

Run: `cargo test rebase worktree && npm test`
Expected: history operations and worktree operations work without changing the existing visual system.

---

### Task 8: Add resolver, diff/search, and remaining v2 capabilities

**Files:**
- Create: `src/components/dialogs/ConflictResolverDialog.vue`, `src/workers/highlight.ts`, `src/composables/useFuzzy.ts`
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/git/query.rs`, `src-tauri/src/git/mutation.rs`
- Modify: `src/components/ConflictBar.vue`, `src/components/SideBySideDiffView.vue`, `src/components/DiffPanel.vue`, `src/components/CommitGraph.vue`, `src/components/BranchPanel.vue`, `src/App.vue`
- Modify: `src/types/index.ts`, `src/locales/ru.ts`, `src/locales/en.ts`, `package.json`, `package-lock.json`

**Interfaces:**
- `get_conflict_sides(repo, path)`, `save_resolved(repo, path, content, mark_resolved)`.
- `get_log(repo, limit, filter)` with author/date/grep/path/pickaxe/all-branches filters.
- `CommandPalette` consumes the command registry and returns selected commands/refs/files.

- [ ] Step 1: Add backend tests for conflict stages, add/add conflicts, binary files, log filters, and `git grep`.
- [ ] Step 2: Implement 3-way side extraction and safe resolved-file writes with oplog entries.
- [ ] Step 3: Build the resolver by reusing `useSyncScroll` and existing side-by-side data types.
- [ ] Step 4: Add worker-based syntax highlighting with visible-row limits and large-file fallback.
- [ ] Step 5: Add command palette, quick search, branch DnD, server-side log filters, stash subsets, ahead/behind, spin-off, and branch hygiene.
- [ ] Step 6: Verify older Git versions hide unsupported previews without failing.

Run: `npm test && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: every functional item listed in `ROADMAP-2.0.md` has a working legacy UI path and automated coverage for its critical behavior.

---

### Task 9: Stabilize and accept v2

**Files:**
- Modify: `docs/ROADMAP-2.0.md`, `README.md`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- Modify: release metadata only when preparing the first v2 prerelease.

- [ ] Step 1: Run the full frontend/backend verification suite and fix failures.
- [ ] Step 2: Test v2 manually on a repository with 50k+ commits, detached HEAD, Cyrillic paths, untracked files, partial stage, merge conflicts, rebase conflicts, multiple remotes, and multiple worktrees.
- [ ] Step 3: Verify update checks from v1 and v2 against stable and prerelease tags.
- [ ] Step 4: Prepare the first v2 prerelease by setting all three version files to `2.0.0-alpha.1`, creating tag `v2.0.0-alpha.1`, and publishing only after explicit release approval.
- [ ] Step 5: Record v2 acceptance and freeze its functional baseline before creating v3.

Run: `npm run build && npm test && (cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test)`
Expected: clean build, passing tests, and a documented v2 acceptance checklist.

---

### Task 10: Create v3 from the accepted v2 baseline

**Files:**
- Modify: branch refs only; no source changes before the branch point.

- [ ] Step 1: Confirm v2 acceptance, clean worktree, and the v2 baseline tag.
- [ ] Step 2: Create `v3` from the accepted v2 commit and push it with upstream tracking.
- [ ] Step 3: Run the complete v2 test suite on v3 before changing UI code.

Expected: v3 initially behaves identically to v2.

---

### Task 11: Implement design A on v3

**Files:**
- Create: `src/styles/tokens.css`, `src/styles/themes/` theme files, `src/components/dialogs/BaseDialog.vue`, `src/components/DialogButton.vue`, `src/components/EmptyState.vue`, `src/components/Skeleton.vue`, `src/components/WelcomeScreen.vue`
- Create: `src/components/files/FileListToolbar.vue`
- Modify: `src/styles/main.css`, `src/styles/dialogs.css`, `src/App.vue`, `src/components/AppToolbar.vue`, `src/components/StatusBar.vue`, `src/components/FileList.vue`, `src/components/RepositoriesPanel.vue`, `src/components/BranchPanel.vue`, `src/components/CommitGraph.vue`, `src/components/DiffPanel.vue`, `src/components/GitOutputPanel.vue`, `src/components/ConflictBar.vue`
- Migrate: all files under `src/components/dialogs/` to `BaseDialog` without changing their v2 command behavior.
- Create: `stylelint.config.mjs`, `eslint.config.mjs`
- Modify: `package.json`, `package-lock.json`

**Interfaces:**
- `BaseDialog` owns open/close, title/subtitle, size, resizable, busy, dismissable, draggable header, and modal registration.
- Semantic tokens are the only color interface used by v3 components; command registry and composables remain unchanged.

- [ ] Step 1: Add token/theme tests and a style check that rejects component-level hex and rgba values.
- [ ] Step 2: Add local IBM Plex Sans/JetBrains Mono font loading and density tokens.
- [ ] Step 3: Implement `BaseDialog`/`DialogButton`, migrate dialogs in the existing documented order, and preserve Esc/default-action behavior.
- [ ] Step 4: Implement the A toolbar, status bar, file-list header, lower tabs, and commit composer layout from `docs/design/A/main.html`.
- [ ] Step 5: Implement welcome, empty, skeleton, toast, progress, conflict, command palette, and operation states from `docs/design/A/states.html` and `tools.html`.
- [ ] Step 6: Run the v2 functional tests after each UI migration to ensure no command behavior regresses.

Run: `npm run build && npm test && npx vue-tsc --noEmit`
Expected: v3 has the A layout while all v2 commands and workflows remain available.

---

### Task 12: Visual and release acceptance for v3

**Files:**
- Modify: `docs/ROADMAP-3.0.md`, `README.md`, release metadata only when preparing the first v3 prerelease.

- [ ] Step 1: Compare v3 screenshots at the same viewport against `docs/design/A/main.html`, `dialogs.html`, `tools.html`, and `states.html`.
- [ ] Step 2: Manually test light, Dracula, and Graphite themes, compact/comfortable density, keyboard-only navigation, reduced motion, and narrow window sizes.
- [ ] Step 3: Run the complete functional matrix from v2 on v3, including undo/redo, resolver, rebase, worktrees, and update checks.
- [ ] Step 4: Prepare the first v3 prerelease by setting all three version files to `3.0.0-alpha.1`, creating tag `v3.0.0-alpha.1`, and publishing only after explicit release approval.
- [ ] Step 5: Document the branch/release workflow and the known support boundary between v1, v2, and v3.

Run: `npm run build && npm test && (cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test)`
Expected: v3 is visually aligned with design A and functionally equivalent to accepted v2.

---

## Plan self-review

- **Coverage:** Tasks 1–3 cover branch/version/update infrastructure; Tasks 4–8 cover all functional sections of the existing v2 roadmap; Tasks 9–12 cover v2 acceptance, v3 branching, design A, and v3 acceptance.
- **Known correction:** The old roadmap incorrectly combined v2 functionality with the A visual redesign; Task 1 separates them into v2 and v3.
- **Safety correction:** The old oplog proposal relied on bare `git stash create`; Task 6 explicitly tests and replaces that assumption for untracked files.
- **Version policy:** Development branches retain their current package version until a prerelease is prepared; release tags carry `2.0.0-alpha.1` and `3.0.0-alpha.1`.
- **No placeholders:** Every task names the files, interfaces, test command, and acceptance result needed for execution.
