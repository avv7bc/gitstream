import { describe, expect, it } from "vitest";
import { isMergePreviewEnabled, parseMergeTreePreview } from "./mergePreview";

describe("merge preview", () => {
  it("parses a clean merge-tree result", () => {
    expect(parseMergeTreePreview("0123456789012345678901234567890123456789\n")).toEqual({
      clean: true,
      conflicted_files: [],
      fast_forward: false,
    });
  });

  it("parses conflict paths and keeps fast-forward metadata", () => {
    const output = [
      "CONFLICT (content): Merge conflict in src/main.ts",
      "CONFLICT (add/add): Merge conflict in README.md",
    ].join("\n");
    expect(parseMergeTreePreview(output, true)).toEqual({
      clean: false,
      conflicted_files: ["src/main.ts", "README.md"],
      fast_forward: true,
    });
  });

  it("treats conflict output and non-zero status as conflicts without paths", () => {
    expect(parseMergeTreePreview("CONFLICT (content)")).toMatchObject({ clean: false });
    expect(parseMergeTreePreview("merge-tree failed", false, true)).toMatchObject({ clean: false });
  });

  it("only enables previews on Git 2.38 or newer", () => {
    expect(isMergePreviewEnabled("git version 2.37.9")).toBe(false);
    expect(isMergePreviewEnabled("git version 2.38.0")).toBe(true);
    expect(isMergePreviewEnabled("git version 3.0.0")).toBe(true);
  });
});
