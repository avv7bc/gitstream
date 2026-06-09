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

// Свыше этого числа строк word-diff пропускается целиком: пользы от него на
// гигантских диффах нет, а diff_main по всем строкам ощутимо тормозит отрисовку.
export const WORD_DIFF_LINE_LIMIT = 20000;

const dmp = new diff_match_patch();

// Сопоставляет удалённые строки с добавленными для построчного word-diff.
// Внутри блока изменений (подряд идущие -/+ строки между контекстом) git
// выдаёт сначала все `-`, затем все `+`; парные правки совпадают по позиции
// в блоке. Позиционное сопоставление — O(n) против прежнего O(removed×added)
// со сравнением diff_main каждой пары, из-за которого крупные файлы зависали.
export function matchRelatedLines(hunk: DiffHunk): Map<number, number> {
  const mapping = new Map<number, number>();
  let i = 0;
  while (i < hunk.lines.length) {
    if (hunk.lines[i].kind === "context") {
      i++;
      continue;
    }
    // Границы блока изменений: подряд идущие added/removed строки.
    const removed: number[] = [];
    const added: number[] = [];
    while (i < hunk.lines.length && hunk.lines[i].kind !== "context") {
      if (hunk.lines[i].kind === "removed") removed.push(i);
      else if (hunk.lines[i].kind === "added") added.push(i);
      i++;
    }
    const pairs = Math.min(removed.length, added.length);
    for (let k = 0; k < pairs; k++) mapping.set(removed[k], added[k]);
  }
  return mapping;
}

export function computeWordDiffs(oldText: string, newText: string): WordDiffSpan[] {
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

export function enrichHunkWithWordDiff(hunk: DiffHunk): DiffHunkWithWordDiff {
  const mapping = matchRelatedLines(hunk);
  const enrichedLines: DiffLineWithWordDiff[] = [];
  const usedAddedIndices = new Set(mapping.values());

  for (let i = 0; i < hunk.lines.length; i++) {
    const line = hunk.lines[i];
    const enrichedLine: DiffLineWithWordDiff = { ...line };

    if (line.kind === "removed" && mapping.has(i)) {
      const relatedIdx = mapping.get(i)!;
      const relatedLine = hunk.lines[relatedIdx];
      enrichedLine.wordDiffs = computeWordDiffs(line.content, relatedLine.content);
    } else if (line.kind === "added" && !usedAddedIndices.has(i)) {
      // Orphaned added line — highlight entire content as added.
      enrichedLine.wordDiffs = [{ text: line.content, kind: "added" }];
    } else if (line.kind === "removed" && !mapping.has(i)) {
      // Orphaned removed line — highlight entire content as removed.
      enrichedLine.wordDiffs = [{ text: line.content, kind: "removed" }];
    }

    enrichedLines.push(enrichedLine);
  }

  return { ...hunk, lines: enrichedLines };
}

export function enrichAllHunks(hunks: DiffHunk[]): DiffHunkWithWordDiff[] {
  const total = hunks.reduce((n, h) => n + h.lines.length, 0);
  if (total > WORD_DIFF_LINE_LIMIT) return hunks;
  return hunks.map(enrichHunkWithWordDiff);
}
