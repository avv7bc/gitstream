---
name: Side-by-Side Diff Redesign
description: Replace unified diff view with two-panel side-by-side view featuring synchronized scroll, word-level highlighting, and hunk navigation
type: design
---

# Side-by-Side Diff View Redesign

## Overview

Replace the current `DiffView.vue` (unified mode + side-by-side mode toggle) with a unified two-panel side-by-side diff view. The left panel shows the old version ("как было"), the right panel shows the new version ("как стало"). Changes are highlighted at both line and word level using `diff-match-patch` library.

## Requirements

### Core Display
- **Two-panel layout**: Left (old version) and Right (new version)
- **Line numbers**: Both panels show line numbers aligned by hunk
- **Horizontal scroll**: Long lines scroll horizontally in each panel independently
- **Word-level highlighting**: Changed words/characters within lines are highlighted with distinct colors
- **Line-level highlighting**: Entire lines are colored (green for added, red for removed, neutral for context)

### Interaction
- **Synchronized scroll**: Scrolling one panel vertically scrolls the other (keeps lines aligned)
- **Hunk navigation**: Previous/Next hunk buttons in header, navigate between changed blocks
- **Single mode only**: Remove unified mode toggle; two-panel is the only view

### Data Model

Diff hunks from backend remain unchanged (from `useDiff` composable). Each hunk contains:
```typescript
{
  header: string;        // "@@ -10,5 +10,6 @@"
  lines: DiffLine[];
}
```

DiffLine structure:
```typescript
{
  kind: "added" | "removed" | "context";
  old_lineno?: number;
  new_lineno?: number;
  content: string;
}
```

### Word-Level Diffing

Use `diff-match-patch` library to identify character-level changes within each changed line:

1. For each added/removed line, compare it with its counterpart in the hunk (e.g., a removed line with the added line that follows it, if they're related)
2. Use `DiffMatchPatch.diff_main()` to get word/character-level changes
3. Render changed spans with different coloring/styling

**Example:**
- Old line: `const name = "John";`
- New line: `const name = "Jane";`
- Word-level diff highlights: `"John"` (red) → `"Jane"` (green)

## Architecture

### Component Structure

- **SideBySideDiffView.vue**: Main component (replace DiffView.vue)
  - Header with file info and hunk navigation buttons
  - Two-panel container (left/right)
  - Manages synchronized scroll state
  
- **DiffPanel.vue**: Single panel (left or right)
  - Renders line numbers and code content
  - Receives hunk data and word-level diff info
  - Handles horizontal scroll
  
- **DiffLinesPair.vue**: Single row pair (one line from left + right)
  - Renders both old and new versions side-by-side
  - Applies line and word-level highlighting

### Composables

**`useSideBySideDiff.ts`** (new):
- `parseHunkWithWordDiff(hunk)`: For each changed line in the hunk, compute word-level differences
- `matchRelatedLines(hunk)`: Match removed lines with added lines to identify what-changed-to-what
- Returns enriched hunk structure with word-level change info

**`useSyncScroll.ts`** (new):
- `syncScroll(leftRef, rightRef)`: Synchronize vertical scroll between two panels
- `unsyncScroll()`: Clean up listeners

### Styling

**Line highlighting:**
- Added lines: `background: var(--diff-added-bg)` (green-ish, existing)
- Removed lines: `background: var(--diff-removed-bg)` (red-ish, existing)
- Context lines: neutral background

**Word highlighting:**
- Added words: `background: var(--diff-word-added-bg)` (more intense green, new CSS var)
- Removed words: `background: var(--diff-word-removed-bg)` (more intense red, new CSS var)
- Can use padding/margin for subtle visual separation if needed

### Dependencies

Add to `package.json`:
```json
"diff-match-patch": "^20240101"
```

## Implementation Notes

### Synchronized Scroll

Attach scroll event listeners to the left panel. When it scrolls, update the right panel's scroll position. Avoid infinite loops by disabling listeners during updates.

### Word-Level Diff Matching

Challenge: When a block has multiple removed lines followed by multiple added lines, which removed line maps to which added line?

**Simple heuristic:** For each removed line, find the most similar added line (by edit distance or diff length). This is good enough for typical changes.

### Hunk Navigation

Track current hunk index. Buttons scroll to the hunk header and highlight it briefly.

### Edge Cases

- **Empty hunks**: Skip in navigation
- **Very long lines**: Horizontal scroll handles this
- **Binary files**: Don't attempt diff (skip this feature for binary)
- **Deleted/added whole file**: Display either all-green or all-red

## File Changes

- **Delete:** `src/components/DiffView.vue` (replace with new version)
- **Create:** `src/components/DiffPanel.vue` (new panel component)
- **Create:** `src/components/DiffLinesPair.vue` (new line pair component)
- **Create:** `src/composables/useSideBySideDiff.ts` (new word-level diff logic)
- **Create:** `src/composables/useSyncScroll.ts` (new scroll sync logic)
- **Update:** `src/styles/main.css` (add `--diff-word-added-bg`, `--diff-word-removed-bg`)
- **Update:** `package.json` (add `diff-match-patch`)

## Success Criteria

1. Side-by-side view displays old and new versions
2. Line numbers align correctly within hunks
3. Word-level highlighting identifies and colors changed words
4. Vertical scroll is synchronized between panels
5. Horizontal scroll works independently per panel
6. Hunk navigation (Prev/Next) works
7. All existing file states (working tree, commit history) work with new view
