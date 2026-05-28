---
name: delete-remote-branch
description: Mechanics for deleting remote branches from BranchPanel Remote Branches section
metadata:
  type: project
---

# Delete Remote Branch

## Summary

Add the ability to delete remote branches directly from the "Remote Branches" section in BranchPanel, with batch support and a confirmation dialog.

## Scope

- Context menu on remote branch items: **Check out** + **Delete Remote Branch**
- Batch delete (Ctrl/Shift multi-select, as with local branches)
- ConfirmDialog (danger) before deletion
- New Tauri command `do_delete_remote_branch` → `git push <remote> --delete <branch>`

## Architecture

### Pattern

Mirrors the `tagCtxMenu` pattern — separate state/handlers for remote branches, not merged with the local-branch menu.

### Frontend — BranchPanel.vue

**New state:**
```ts
const remoteCtxMenu = ref<{ x: number; y: number } | null>(null);
const remoteCtxBranch = ref<BranchInfo | null>(null);
const showDeleteRemoteConfirm = ref(false);
const targetRemoteBranches = ref<BranchInfo[]>([]);
```

**New computed:**
```ts
const selectedRemoteBranches = computed(() =>
  remoteBranches.value.filter((b) => selectedKeys.value.has(`remote:${b.name}`))
);
```

**Name parser:**
```ts
function parseRemoteBranch(fullName: string): { remote: string; branch: string } {
  const idx = fullName.indexOf("/");
  return idx > 0
    ? { remote: fullName.slice(0, idx), branch: fullName.slice(idx + 1) }
    : { remote: fullName, branch: fullName };
}
```

**Handlers:**
- `onRemoteBranchContextMenu(e, branch)` — right-click, selects branch if not already selected, opens menu
- `closeRemoteCtxMenu()`
- `handleCheckoutRemoteCtx()` — emit('checkoutRemote', branch.name)
- `handleDeleteRemoteCtx()` — set targetRemoteBranches, show confirm
- `confirmDeleteRemote()` — iterate, parse, call deleteRemoteBranch per branch, emit branchesChanged

**Template additions:**
- `@contextmenu="onRemoteBranchContextMenu($event, branch)"` on each remote branch item
- Context menu block (Teleport to body): Checkout + separator + Delete (danger)
- ConfirmDialog for showDeleteRemoteConfirm

### useBranches.ts

```ts
async function deleteRemoteBranch(remote: string, branch: string) {
  if (!repoPath.value) return;
  await invoke("do_delete_remote_branch", {
    repoPath: repoPath.value,
    remote,
    branch,
    timeoutSecs: networkTimeoutSecs.value,
  });
}
```

### Rust — mutation.rs

```rust
pub fn delete_remote_branch_args(remote: &str, branch: &str) -> Vec<String> {
    vec!["push".into(), remote.into(), "--delete".into(), branch.into()]
}
```

### Rust — commands.rs

```rust
#[tauri::command]
pub async fn do_delete_remote_branch(
    app: tauri::AppHandle,
    repo_path: String,
    remote: String,
    branch: String,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let args = mutation::delete_remote_branch_args(&remote, &branch);
    run_network_git(&app, Some(Path::new(&repo_path)), &args, timeout_secs, "push").await
}
```

### main.rs

Register `commands::do_delete_remote_branch`.

### i18n

Add to `branches` namespace in both en.ts / ru.ts:
- `deleteRemoteBranch: "Delete Remote Branch"` / `"Удалить remote-ветку"`

## Edge Cases

- Branch name with `/` (e.g. `origin/feature/foo`) → `indexOf('/')` splits only on first slash → correct
- Network errors → `window.alert(...)` consistent with push error handling
- Timeout → `networkTimeoutSecs` passed to Tauri command
