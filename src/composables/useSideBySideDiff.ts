import diff_match_patch from "diff-match-patch";
import type { DiffHunk, DiffLine } from "@/types";

export interface WordDiffSpan {
  text: string;
  kind: "added" | "removed" | "context";
}

export interface DiffLineWithWordDiff extends DiffLine {
  wordDiffs?: WordDiffSpan[];
}

export interface DiffHunkWithWordDiff extends DiffHunk {
  lines: DiffLineWithWordDiff[];
}

export function useSideBySideDiff() {
  const dmp = new diff_match_patch();
  const SIMILARITY_THRESHOLD = 0.3;

  function matchRelatedLines(hunk: DiffHunk): Map<number, number> {
    // Map of removed line index -> added line index
    const mapping = new Map<number, number>();
    const usedAddedIndices = new Set<number>();
    const removedLines: Array<{ idx: number; content: string }> = [];
    const addedLines: Array<{ idx: number; content: string }> = [];

    for (let i = 0; i < hunk.lines.length; i++) {
      const line = hunk.lines[i];
      if (line.kind === "removed") {
        removedLines.push({ idx: i, content: line.content });
      } else if (line.kind === "added") {
        addedLines.push({ idx: i, content: line.content });
      }
    }

    // Simple matching: for each removed line, find the most similar added line
    for (const removed of removedLines) {
      let bestMatch = -1;
      let bestScore = 0;

      for (let j = 0; j < addedLines.length; j++) {
        const added = addedLines[j];
        const diffs = dmp.diff_main(removed.content, added.content);
        const similarity = computeSimilarity(diffs);
        if (similarity > bestScore && !usedAddedIndices.has(j) && similarity >= SIMILARITY_THRESHOLD) {
          bestScore = similarity;
          bestMatch = j;
        }
      }

      if (bestMatch >= 0 && bestScore >= SIMILARITY_THRESHOLD) {
        usedAddedIndices.add(addedLines[bestMatch].idx);
        mapping.set(removed.idx, addedLines[bestMatch].idx);
      }
    }

    return mapping;
  }

  function computeSimilarity(diffs: Array<[number, string]>): number {
    let sameCount = 0;
    let totalCount = 0;
    for (const [op] of diffs) {
      if (op === 0) sameCount++;
      totalCount++;
    }
    return totalCount > 0 ? sameCount / totalCount : 0;
  }

  function computeWordDiffs(oldText: string, newText: string): WordDiffSpan[] {
    const diffs = dmp.diff_main(oldText, newText);
    dmp.diff_cleanupSemantic(diffs);

    const spans: WordDiffSpan[] = [];
    for (const [op, text] of diffs) {
      if (op === 0) {
        spans.push({ text, kind: "context" });
      } else if (op === 1) {
        spans.push({ text, kind: "added" });
      } else if (op === -1) {
        spans.push({ text, kind: "removed" });
      }
    }
    return spans;
  }

  function enrichHunkWithWordDiff(
    hunk: DiffHunk
  ): DiffHunkWithWordDiff {
    const mapping = matchRelatedLines(hunk);
    const enrichedLines: DiffLineWithWordDiff[] = [];
    const usedAddedIndices = new Set(mapping.values());

    for (let i = 0; i < hunk.lines.length; i++) {
      const line = hunk.lines[i];
      const enrichedLine: DiffLineWithWordDiff = { ...line };

      if (line.kind === "removed" && mapping.has(i)) {
        const relatedIdx = mapping.get(i)!;
        const relatedLine = hunk.lines[relatedIdx];
        enrichedLine.wordDiffs = computeWordDiffs(
          line.content,
          relatedLine.content
        );
      } else if (line.kind === "added" && !usedAddedIndices.has(i)) {
        // Orphaned added line - highlight entire content as added
        enrichedLine.wordDiffs = [{ text: line.content, kind: "added" }];
      } else if (line.kind === "removed" && !mapping.has(i)) {
        // Orphaned removed line - highlight entire content as removed
        enrichedLine.wordDiffs = [{ text: line.content, kind: "removed" }];
      }

      enrichedLines.push(enrichedLine);
    }

    return { ...hunk, lines: enrichedLines };
  }

  function enrichAllHunks(hunks: DiffHunk[]): DiffHunkWithWordDiff[] {
    return hunks.map(enrichHunkWithWordDiff);
  }

  return {
    enrichHunkWithWordDiff,
    enrichAllHunks,
  };
}
