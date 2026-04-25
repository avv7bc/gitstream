# Keyboard Navigation in CommitGraph (Up/Down)

**Date:** 2026-04-25  
**Component:** CommitGraph.vue  
**Feature:** Navigate commit selection with keyboard arrows

---

## Overview

Add keyboard navigation to the CommitGraph component using Up/Down arrow keys. When focus is in the CommitGraph, users can move the selected commit up or down through the visible (filtered) list of commits.

---

## Requirements

### Functional Requirements

1. **Input Trigger**
   - Up/Down arrow keys trigger navigation
   - Only when focus is inside the CommitGraph component (after clicking a commit)

2. **Navigation Behavior**
   - Navigate through visible commits in `filteredCommits`
   - Up arrow moves to the previous commit in the list
   - Down arrow moves to the next commit in the list
   - Stop at boundaries: do not wrap around when reaching first/last commit

3. **Position Model**
   - Positions ordered as: Working Tree (if visible) → filteredCommits[0] → ... → filteredCommits[n-1]
   - Up from first commit: no change
   - Down from last commit: no change

4. **Working Tree Integration**
   - If Working Tree is visible (changedCount > 0) and selected: Down arrow → first commit
   - If Working Tree is visible and first commit selected: Up arrow → Working Tree
   - If Working Tree not visible: navigate only through commits

### Non-Functional Requirements

- No automatic scrolling to keep selected item visible
- Responsive to filter changes (navigate within currently visible commits)
- Graceful handling of edge cases (empty list, single commit)

---

## Architecture

### Component Modification

**File:** `src/components/CommitGraph.vue`

#### Add Keyboard Event Handler

```typescript
// In <script setup>
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
    e.preventDefault()
    navigateCommits(e.key === 'ArrowUp' ? 'up' : 'down')
  }
}

function navigateCommits(direction: 'up' | 'down') {
  // Determine current position
  // Calculate new position with boundary checks
  // Update selectedCommit or selectWorkingTree()
}
```

#### Bind Handler to Container

Attach `@keydown` listener to `.graph-body` element for capturing keyboard input when CommitGraph has focus.

### Navigation Logic

```
Current Selection → Find Current Index
  ├─ If "__worktree__" and visible → index = -1
  ├─ Otherwise find OID in filteredCommits → index = i
  └─ If not found → index = 0 (safety)

Calculate Next Index
  ├─ Direction UP:
  │   └─ new_index = max(-1, current_index - 1)
  ├─ Direction DOWN:
  │   └─ new_index = min(filteredCommits.length - 1, current_index + 1)

Apply Selection
  ├─ If new_index === -1 → selectWorkingTree()
  ├─ Otherwise → selectedCommit.value = filteredCommits[new_index].oid
```

### Edge Cases

| Case | Behavior |
|------|----------|
| No commits, WT visible | Navigate between WT and WT (no-op down) |
| No commits, WT invisible | No navigation |
| Single commit, WT visible | Up/Down cycles between WT and commit |
| Single commit, WT invisible | Up/Down no-op (stuck on commit) |
| Filter active | Navigate within filtered results only |
| WT not visible (no changes) | WT never selected, skip in navigation |

---

## Implementation Details

### Functions to Add

**`navigateCommits(direction: 'up' | 'down'): void`**
- Find current position in the logical list (WT + filtered commits)
- Calculate new position respecting boundaries
- Update selectedCommit or call selectWorkingTree()

**`getCurrentIndex(): number`**
- Returns -1 if Working Tree selected
- Returns index in filteredCommits if a commit selected
- Returns 0 if selectedCommit doesn't exist in current filteredCommits (safety fallback)

### Computed Values (Existing)

- `filteredCommits` — already exists, respects graphFilter
- `isWorkingTreeSelected` — already computed, check if selectedCommit === "__worktree__"
- `changedCount` — already exists, determines if WT visible

### State (Existing)

- `selectedCommit` — ref from useLog() composable, target of navigation updates

---

## Testing Strategy

### Manual Testing

1. **Basic Navigation**
   - Click on a commit
   - Press Up/Down, verify selection moves correctly
   - Press Up at first commit: no change
   - Press Down at last commit: no change

2. **Working Tree Integration**
   - Make a change (WT appears)
   - Click WT, press Down → first commit selected
   - Click first commit, press Up → WT selected
   - Discard changes (WT disappears): Up from first commit → no change

3. **Filter Interaction**
   - Apply filter
   - Navigate up/down: verify only visible (filtered) commits are targets
   - Clear filter: verify navigation includes all commits

4. **Edge Cases**
   - Empty repository: no commits, only WT → Down does nothing
   - Single commit: navigate between WT and commit

### Accessibility

- Keyboard users can fully navigate commit history
- No focus management issue (CommitGraph already focusable via click)

---

## Scope & Constraints

- **Scope:** CommitGraph keyboard navigation only
- **Future Work:** Could extend to other components, Page Up/Down support, or focus management via tabindex
- **No Changes:** useLog composable, CommitDetails, other components
