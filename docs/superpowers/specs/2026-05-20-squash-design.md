# Squash Commits — Design Spec

**Date:** 2026-05-20  
**Status:** Approved

## Overview

Allow squashing a range of consecutive commits into one directly from the CommitGraph context menu. Phase 1 covers only commits reachable from HEAD (using `git reset --soft`). Phase 2 (squash in middle of history via `git rebase -i`) is deferred.

## UX Flow

1. Click a commit → normal single select (existing behaviour unchanged).
2. Shift+click another commit → selects the inclusive range between the two; all rows in range are highlighted.
3. Right-click any commit when a range is active → context menu shows **Squash** item, enabled only when the range includes HEAD.
4. **SquashDialog** opens with:
   - Header: "Squash N commits"
   - Textarea pre-filled with all selected commit messages joined by blank lines (newest first)
   - OK / Cancel
5. OK → backend squash → graph refresh.

## Architecture

### Frontend

| File | Change |
|---|---|
| `CommitGraph.vue` | `rangeAnchor ref<string\|null>`, `selectedOids ref<string[]>`. Shift+click computes range. Highlight rows in `selectedOids`. Squash ctx-menu item enabled when `selectedOids.length >= 2 && headInRange`. Emits `squash` event with `{oids, commits}`. |
| `SquashDialog.vue` | New dialog. Props: `oids: string[]`, `commits: CommitInfo[]`. Emits `confirm(message)` / `close`. |
| `useLog.ts` | `squashCommits(oids: string[], message: string)` → invoke `do_squash` → refresh. |
| `App.vue` | `squashPayload ref`, handle `@squash` from CommitGraph, show SquashDialog. |

### Backend (Rust)

| File | Change |
|---|---|
| `mutation.rs` | `pub fn squash(path, oids: &[String], message: &str)` — validates oids non-empty, resolves parent of oldest OID via `git rev-parse <oid>^`, runs `git reset --soft <parent>` then `git commit -m <message>`. |
| `commands.rs` | `async fn do_squash(repo_path, oids, message)` via `spawn_blocking`. |
| `main.rs` | Register `do_squash`. |

## Constraints

- Squash is disabled unless HEAD (`commits[0].oid`) is in the selection.
- Range must be consecutive (enforced by the linear range-select UI).
- After squash: reload log + branches + status.
- Phase 2 (non-HEAD squash via rebase -i) is out of scope.
