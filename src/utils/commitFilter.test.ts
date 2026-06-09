import { describe, it, expect } from "vitest";
import type { CommitInfo, RefLabel } from "@/types";
import { matchesCommitFilter, filterCommits } from "./commitFilter";

function commit(over: Partial<CommitInfo>): CommitInfo {
  return {
    oid: "0123456789abcdef0123456789abcdef01234567",
    short_oid: "0123456",
    message: "Initial commit",
    author: "Alice",
    author_email: "alice@example.com",
    date: "2026-06-09 12:00",
    parents: [],
    refs: [],
    column: 0,
    lines: [],
    unpushed: false,
    ...over,
  };
}

function ref(name: string): RefLabel {
  return { name, kind: "local-branch" };
}

describe("matchesCommitFilter", () => {
  it("matches empty query (everything passes)", () => {
    expect(matchesCommitFilter(commit({}), "")).toBe(true);
    expect(matchesCommitFilter(commit({}), "   ")).toBe(true);
  });

  it("matches by message, case-insensitively", () => {
    expect(matchesCommitFilter(commit({ message: "Fix the Bug" }), "bug")).toBe(true);
    expect(matchesCommitFilter(commit({ message: "Fix the Bug" }), "BUG")).toBe(true);
  });

  it("matches by author and email", () => {
    const c = commit({ author: "Bob Smith", author_email: "bob@corp.io" });
    expect(matchesCommitFilter(c, "smith")).toBe(true);
    expect(matchesCommitFilter(c, "corp.io")).toBe(true);
  });

  it("matches by full and short SHA", () => {
    const c = commit({ oid: "deadbeefcafe", short_oid: "deadbee" });
    expect(matchesCommitFilter(c, "deadbeefcafe")).toBe(true);
    expect(matchesCommitFilter(c, "deadbee")).toBe(true);
  });

  it("matches by date substring", () => {
    expect(matchesCommitFilter(commit({ date: "2026-06-09 12:00" }), "2026-06")).toBe(true);
  });

  it("matches by ref name", () => {
    const c = commit({ refs: [ref("main"), ref("feature/auth")] });
    expect(matchesCommitFilter(c, "feature/auth")).toBe(true);
    expect(matchesCommitFilter(c, "main")).toBe(true);
  });

  it("returns false when nothing matches", () => {
    expect(matchesCommitFilter(commit({}), "zzz-nope")).toBe(false);
  });

  it("trims the query before matching", () => {
    expect(matchesCommitFilter(commit({ message: "hello" }), "  hello  ")).toBe(true);
  });
});

describe("filterCommits", () => {
  it("returns all commits for an empty/whitespace query (same array)", () => {
    const list = [commit({ oid: "a" }), commit({ oid: "b" })];
    expect(filterCommits(list, "")).toBe(list);
    expect(filterCommits(list, "  ")).toBe(list);
  });

  it("filters down to matching commits", () => {
    const list = [
      commit({ short_oid: "aaa111", message: "alpha" }),
      commit({ short_oid: "bbb222", message: "beta" }),
    ];
    const out = filterCommits(list, "beta");
    expect(out).toHaveLength(1);
    expect(out[0].short_oid).toBe("bbb222");
  });
});
