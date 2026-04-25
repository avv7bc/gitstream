# Keyboard Navigation in CommitGraph Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable Up/Down arrow keys to navigate commit selection in CommitGraph, moving focus between visible commits and Working Tree.

**Architecture:** Add keyboard event handler to CommitGraph container that calculates current position in the logical list (Working Tree + filtered commits), then moves selection up/down with boundary protection.

**Tech Stack:** Vue 3 (Composition API), TypeScript

---

## File Structure

### Modified Files
- `src/components/CommitGraph.vue` — Add keyboard navigation logic and event handler

No new files needed. All logic contained within existing component.

---

## Tasks

### Task 1: Add Helper Function to Get Current Position Index

**Files:**
- Modify: `src/components/CommitGraph.vue` (in `<script setup>`)

**Goal:** Implement `getCurrentIndex()` function that returns the current position in the logical list (Working Tree at -1, commits at 0+).

- [ ] **Step 1: Add `getCurrentIndex()` function in script**

Add this function after the `selectWorkingTree()` function (around line 53):

```typescript
function getCurrentIndex(): number {
  // Working Tree is at position -1
  if (selectedCommit.value === "__worktree__") {
    return changedCount.value > 0 ? -1 : 0; // fallback to 0 if WT not visible
  }
  
  // Find commit in filtered list
  const idx = filteredCommits.value.findIndex((c) => c.oid === selectedCommit.value);
  return idx >= 0 ? idx : 0; // fallback to 0 if not found
}
```

**Explanation:**
- Returns `-1` if Working Tree is selected and visible
- Returns index `0..n` if a commit from `filteredCommits` is selected
- Returns `0` as safety fallback if selected commit is not in filtered list (e.g., filter changed)

- [ ] **Step 2: Verify function placement**

Check that `getCurrentIndex()` is readable and positioned logically in the `<script setup>` block. No execution needed yet.

---

### Task 2: Add Navigation Function

**Files:**
- Modify: `src/components/CommitGraph.vue` (in `<script setup>`)

**Goal:** Implement `navigateCommits()` function that moves selection up/down with boundary checks.

- [ ] **Step 1: Add `navigateCommits()` function**

Add this function right after `getCurrentIndex()`:

```typescript
function navigateCommits(direction: 'up' | 'down'): void {
  const currentIdx = getCurrentIndex();
  const hasWorkingTree = changedCount.value > 0;
  const maxIdx = filteredCommits.value.length - 1;
  
  let newIdx: number;
  
  if (direction === 'up') {
    newIdx = currentIdx - 1;
    // Boundary: don't go above -1 (Working Tree) or 0 (first commit)
    if (newIdx < (hasWorkingTree ? -1 : 0)) {
      return; // no-op, stay at boundary
    }
  } else {
    // direction === 'down'
    newIdx = currentIdx + 1;
    // Boundary: don't go beyond last commit
    if (newIdx > maxIdx) {
      if (currentIdx === maxIdx) {
        return; // already at last commit, no-op
      }
      newIdx = maxIdx; // clamp to last commit
    }
  }
  
  // Apply new selection
  if (newIdx === -1) {
    selectWorkingTree();
  } else {
    selectedCommit.value = filteredCommits.value[newIdx].oid;
  }
}
```

**Explanation:**
- `direction` determines Up (decrement) or Down (increment)
- Up: move from WT (-1) → first commit (0) → earlier commits
- Down: move from earlier commits → last commit (maxIdx)
- Boundary checks prevent over-scrolling
- Selection update uses existing refs and functions

- [ ] **Step 2: Verify logic for edge cases**

Read through the function and mentally verify:
- Up from position 0 with WT visible: goes to -1 ✓
- Up from position 0 without WT: returns early (no-op) ✓
- Down from last commit: returns early (no-op) ✓
- Down from WT with commits available: goes to position 0 ✓

---

### Task 3: Add Keyboard Event Handler

**Files:**
- Modify: `src/components/CommitGraph.vue` (in `<script setup>`)

**Goal:** Implement `handleKeyDown()` event handler that captures Up/Down keys.

- [ ] **Step 1: Add `handleKeyDown()` function**

Add this function right after `navigateCommits()`:

```typescript
function handleKeyDown(e: KeyboardEvent): void {
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    navigateCommits('up');
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    navigateCommits('down');
  }
}
```

**Explanation:**
- Checks for ArrowUp/ArrowDown keys
- Calls `preventDefault()` to stop default browser behavior (scrolling)
- Delegates to `navigateCommits()` with appropriate direction

- [ ] **Step 2: Verify handler doesn't conflict with other keys**

Confirm this handler only intercepts Up/Down arrows. Other keys pass through. No changes needed.

---

### Task 4: Bind Keyboard Handler to Container

**Files:**
- Modify: `src/components/CommitGraph.vue` (in `<template>`)

**Goal:** Attach `@keydown` event listener to `.graph-body` div.

- [ ] **Step 1: Locate `.graph-body` in template**

Find the element around line 117:
```vue
<div class="graph-body">
```

- [ ] **Step 2: Add `@keydown` handler**

Modify it to:
```vue
<div class="graph-body" @keydown="handleKeyDown">
```

**Explanation:**
- `.graph-body` is the scrollable container for commits
- Event bubbles up from individual rows, captured by container
- Handler fires for any keydown event in the graph area
- Users click on a commit (sets focus conceptually) → press Up/Down → handler triggers

- [ ] **Step 3: Verify no other keydown handlers exist on this element**

Check if `.graph-body` or parent `.commit-graph` already have `@keydown` handlers. If yes, merge into single handler. (Expected: none currently exist.)

---

### Task 5: Manual Testing - Basic Navigation

**Testing Setup:**
- Open app with a repository that has multiple commits
- Have at least one uncommitted change (Working Tree visible)

- [ ] **Step 1: Test navigation through commits**

1. Click on the second-to-last commit in the list
2. Press Down arrow → should select last commit
3. Press Down arrow again → should stay on last commit (no-op)
4. Press Up arrow → should select second-to-last commit
5. Continue Up → commits move selection upward
6. Press Up when on first commit → should stay on first commit

**Expected:** Smooth keyboard navigation through commit list without wrapping.

- [ ] **Step 2: Test Working Tree integration**

1. Click on first commit
2. Press Up arrow → should select "Working Tree/Index" row
3. Press Up again → should stay on Working Tree (no-op)
4. Press Down arrow → should select first commit
5. Press Down until last commit

**Expected:** Working Tree is reachable and acts as the boundary above first commit.

- [ ] **Step 3: Record test results**

Document that basic navigation works. Note any unexpected behavior for next steps.

---

### Task 6: Manual Testing - Filter + Edge Cases

**Test Cases:**

- [ ] **Step 1: Test with active filter**

1. Type in the filter field to show only 2-3 commits
2. Navigate Up/Down through filtered list
3. Verify navigation only cycles through visible (filtered) commits, not hidden ones
4. Clear filter → verify navigation includes all commits again

**Expected:** Navigation respects filter dynamically.

- [ ] **Step 2: Test with empty repository or single commit**

1. Navigate to a repo with single commit or no commits
2. Press Up/Down → verify no errors, graceful handling

**Expected:** Graceful handling, no crashes or weird UI states.

- [ ] **Step 3: Test Working Tree disappear/appear**

1. Start with Working Tree visible
2. Navigate to it
3. Go to Files panel, discard all changes → WT disappears
4. Try pressing Up from first commit → should no-op (WT no longer a target)

**Expected:** Navigation adapts when WT visibility changes.

- [ ] **Step 4: Record all edge case results**

Document any issues found.

---

### Task 7: Commit Implementation

**Files:**
- `src/components/CommitGraph.vue`

- [ ] **Step 1: Review changes**

Run `git diff src/components/CommitGraph.vue` to verify:
- Three new functions added: `getCurrentIndex()`, `navigateCommits()`, `handleKeyDown()`
- `.graph-body` element has `@keydown="handleKeyDown"`
- No unintended deletions or modifications

- [ ] **Step 2: Stage and commit**

```bash
git add src/components/CommitGraph.vue
git commit -m "feat: add Up/Down keyboard navigation in CommitGraph

- Add getCurrentIndex() to calculate position in commit list or Working Tree
- Add navigateCommits() to move selection up/down with boundary protection
- Add handleKeyDown() event handler to capture arrow key presses
- Bind @keydown handler to .graph-body container
- Navigation respects filter: only moves through visible commits
- Boundaries: Up from first commit (or WT if no WT) is no-op, same for Down from last"
```

**Expected Output:**
```
[main xxx] feat: add Up/Down keyboard navigation in CommitGraph
 1 file changed, XX insertions(+)
```

---

## Self-Review Checklist

**Spec Coverage:**
- ✓ Input trigger (Up/Down keys) — Task 3, 4
- ✓ Navigation behavior (previous/next, stop at boundaries) — Task 2
- ✓ Position model (WT + filtered commits) — Task 1, 2
- ✓ Working Tree integration — Task 2
- ✓ Filter respect — Task 6 (tested), Task 2 (uses `filteredCommits`)
- ✓ No auto-scroll requirement — not implemented (as specified)

**Placeholder Scan:**
- No "TBD", "TODO", or incomplete steps
- All code blocks are complete and executable
- All test steps have explicit expected behavior
- No vague instructions ("add error handling", etc.)

**Type Consistency:**
- `getCurrentIndex()` returns `number` (used in `navigateCommits`)
- `navigateCommits(direction: 'up' | 'down')` parameter matches event check
- `selectedCommit.value` type is `string` (OID), matches assignment
- `changedCount.value` type is `number` (array length)

**Edge Cases Covered:**
- Boundary protection in Task 2
- WT visibility logic in Task 2
- Filter dynamism tested in Task 6
- Empty repo graceful handling in Task 6
- WT toggle in Task 6

No gaps found.

