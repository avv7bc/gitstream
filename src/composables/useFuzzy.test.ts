import { describe, expect, it } from "vitest";
import { fuzzyScore } from "./useFuzzy";

describe("fuzzyScore", () => {
  it("matches subsequences and rewards contiguous words", () => {
    expect(fuzzyScore("branch.checkout", "bco")).toBeGreaterThan(0);
    expect(fuzzyScore("branch.checkout", "zzz")).toBe(-1);
    expect(fuzzyScore("branch.checkout", "checkout")).toBeGreaterThan(fuzzyScore("branch.checkout", "bco"));
  });
});
