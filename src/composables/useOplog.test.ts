import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, repoPath, opened } = vi.hoisted(() => ({
  invoke: vi.fn(),
  repoPath: { value: null as string | null, __v_isRef: true },
  opened: vi.fn(),
}));
vi.mock("@/composables/useProgress", () => ({ invoke }));
vi.mock("@/composables/useRepo", () => ({ useRepo: () => ({ repoPath, onRepoOpened: opened }) }));

function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>((r) => { resolve = r; }); return { promise, resolve }; }

describe("useOplog", () => {
  beforeEach(() => { vi.resetModules(); invoke.mockReset(); opened.mockReset(); repoPath.value = null; });

  it("does not let a slow old-repository response overwrite the current entries", async () => {
    const old = deferred<unknown[]>();
    invoke.mockReturnValueOnce(old.promise).mockResolvedValueOnce([{ id: 2, kind: "new", label: "new", at: 2, undone: false, eligible: true }]);
    const { useOplog } = await import("./useOplog");
    const oplog = useOplog();
    repoPath.value = "/old";
    const first = oplog.refresh();
    repoPath.value = "/new";
    await oplog.refresh();
    old.resolve([{ id: 1, kind: "old", label: "old", at: 1, undone: false, eligible: true }]);
    await first;
    expect(oplog.entries.value[0].id).toBe(2);
  });

  it("uses summary eligibility for undo and redo affordances", async () => {
    invoke.mockResolvedValueOnce([
      { id: 3, kind: "undo", label: "undo", at: 3, undone: false, eligible: false },
      { id: 2, kind: "edit", label: "edit", at: 2, undone: false, eligible: true },
    ]);
    const { useOplog } = await import("./useOplog");
    const oplog = useOplog();
    repoPath.value = "/repo";
    await oplog.refresh();
    expect(oplog.canUndo.value).toBe(true);
    expect(oplog.canRedo.value).toBe(false);
  });
});
