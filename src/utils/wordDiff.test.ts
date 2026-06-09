import { describe, it, expect } from "vitest";
import type { DiffHunk, DiffLine } from "@/types";
import {
  matchRelatedLines,
  computeWordDiffs,
  enrichHunkWithWordDiff,
  enrichAllHunks,
  WORD_DIFF_LINE_LIMIT,
} from "./wordDiff";

function line(kind: DiffLine["kind"], content: string): DiffLine {
  return { kind, old_lineno: null, new_lineno: null, content };
}

function hunk(lines: DiffLine[]): DiffHunk {
  return { header: "@@", raw: "", lines };
}

describe("matchRelatedLines", () => {
  it("pairs removed with added by position within a change block", () => {
    const h = hunk([
      line("context", "a"),
      line("removed", "old1"),
      line("removed", "old2"),
      line("added", "new1"),
      line("added", "new2"),
      line("context", "b"),
    ]);
    const m = matchRelatedLines(h);
    // removed idx 1→added idx 3, removed idx 2→added idx 4
    expect(m.get(1)).toBe(3);
    expect(m.get(2)).toBe(4);
    expect(m.size).toBe(2);
  });

  it("pairs only up to the smaller count (extra added stay unpaired)", () => {
    const h = hunk([
      line("removed", "old1"),
      line("added", "new1"),
      line("added", "new2"),
    ]);
    const m = matchRelatedLines(h);
    expect(m.get(0)).toBe(1);
    expect(m.size).toBe(1); // added idx 2 left unpaired
  });

  it("keeps separate change blocks independent", () => {
    const h = hunk([
      line("removed", "a"),
      line("added", "A"),
      line("context", "—"),
      line("removed", "b"),
      line("added", "B"),
    ]);
    const m = matchRelatedLines(h);
    expect(m.get(0)).toBe(1);
    expect(m.get(3)).toBe(4);
    expect(m.size).toBe(2);
  });

  it("returns empty mapping for context-only hunk", () => {
    expect(matchRelatedLines(hunk([line("context", "x")])).size).toBe(0);
  });
});

describe("computeWordDiffs", () => {
  it("marks a changed word as removed+added around shared context", () => {
    const spans = computeWordDiffs("hello world", "hello there");
    expect(spans[0]).toEqual({ text: "hello ", kind: "context" });
    // remaining spans contain the divergent words
    const kinds = spans.map((s) => s.kind);
    expect(kinds).toContain("removed");
    expect(kinds).toContain("added");
    // reconstructing old text from context+removed
    const oldText = spans
      .filter((s) => s.kind !== "added")
      .map((s) => s.text)
      .join("");
    expect(oldText).toBe("hello world");
  });

  it("returns a single context span for identical text", () => {
    expect(computeWordDiffs("same", "same")).toEqual([
      { text: "same", kind: "context" },
    ]);
  });
});

describe("enrichHunkWithWordDiff", () => {
  it("attaches word diffs to paired removed lines and leaves context untouched", () => {
    const h = hunk([
      line("context", "ctx"),
      line("removed", "foo bar"),
      line("added", "foo baz"),
    ]);
    const out = enrichHunkWithWordDiff(h);
    expect(out.lines[0].wordDiffs).toBeUndefined();
    expect(out.lines[1].wordDiffs).toBeDefined();
    // paired removed line carries the word-level diff
    expect(out.lines[1].wordDiffs!.some((s) => s.kind === "removed")).toBe(true);
  });

  it("marks orphaned added/removed lines as whole-line spans", () => {
    const h = hunk([line("removed", "gone"), line("added", "fresh"), line("added", "extra")]);
    const out = enrichHunkWithWordDiff(h);
    // idx0 removed is paired with idx1 added → has computed diff
    expect(out.lines[0].wordDiffs).toBeDefined();
    // idx2 added is orphaned → whole-line added span
    expect(out.lines[2].wordDiffs).toEqual([{ text: "extra", kind: "added" }]);
  });

  it("does not mutate the input hunk", () => {
    const h = hunk([line("removed", "x"), line("added", "y")]);
    const snapshot = JSON.stringify(h);
    enrichHunkWithWordDiff(h);
    expect(JSON.stringify(h)).toBe(snapshot);
  });
});

describe("enrichAllHunks", () => {
  it("enriches small diffs", () => {
    const out = enrichAllHunks([hunk([line("removed", "a"), line("added", "b")])]);
    expect(out[0].lines[0].wordDiffs).toBeDefined();
  });

  it("skips word-diff entirely above the line limit", () => {
    const many = Array.from({ length: WORD_DIFF_LINE_LIMIT + 1 }, () => line("context", "x"));
    const out = enrichAllHunks([hunk(many)]);
    // returned as-is, no wordDiffs attached
    expect(out[0].lines.every((l) => l.wordDiffs === undefined)).toBe(true);
  });
});
