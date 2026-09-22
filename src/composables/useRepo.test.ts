import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { RepoInfo } from "@/types";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@/composables/useProgress", () => ({ invoke }));

const LAST_REPO_KEY = "gitstream:last-repo-path";

function info(path: string, branch = "main"): RepoInfo {
  return { path, current_branch: branch, head_oid: `${path}-${branch}` };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}

beforeEach(() => {
  vi.resetModules();
  invoke.mockReset();
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => { values.set(key, value); },
    removeItem: (key: string) => { values.delete(key); },
  });
});

afterEach(() => { vi.unstubAllGlobals(); });

describe("repository lifecycle", () => {
  it("keeps the active repository until the new path has been validated", async () => {
    const repo = (await import("./useRepo")).useRepo();
    invoke.mockResolvedValueOnce(info("/a"));
    await repo.openRepo("/a");
    const opened = vi.fn(() => {
      expect(repo.repoPath.value).toBe("/b");
      expect(repo.repoInfo.value).toEqual(info("/b"));
      return Promise.resolve();
    });
    repo.onRepoOpened(opened);
    const pending = deferred<RepoInfo>();
    invoke.mockReturnValueOnce(pending.promise);
    const opening = repo.openRepo("/b");

    expect(repo.repoPath.value).toBe("/a");
    expect(repo.repoInfo.value).toEqual(info("/a"));
    expect(localStorage.getItem(LAST_REPO_KEY)).toBe("/a");
    expect(opened).not.toHaveBeenCalled();

    pending.resolve(info("/b"));
    await opening;
    expect(opened).toHaveBeenCalledOnce();
    expect(localStorage.getItem(LAST_REPO_KEY)).toBe("/b");
  });

  it("preserves the active session when opening a missing path fails", async () => {
    const repo = (await import("./useRepo")).useRepo();
    invoke.mockResolvedValueOnce(info("/a"));
    await repo.openRepo("/a");
    const opened = vi.fn();
    repo.onRepoOpened(opened);
    invoke.mockRejectedValueOnce(new Error("Repository not found"));

    await expect(repo.openRepo("/missing")).rejects.toThrow("Repository not found");
    expect(repo.repoPath.value).toBe("/a");
    expect(repo.repoInfo.value).toEqual(info("/a"));
    expect(localStorage.getItem(LAST_REPO_KEY)).toBe("/a");
    expect(opened).not.toHaveBeenCalled();
  });

  it("removes a missing saved path without activating it", async () => {
    const repo = (await import("./useRepo")).useRepo();
    localStorage.setItem(LAST_REPO_KEY, "/missing");
    invoke.mockRejectedValueOnce(new Error("Repository not found"));

    await repo.restoreLastRepo();
    expect(repo.repoPath.value).toBeNull();
    expect(repo.repoInfo.value).toBeNull();
    expect(localStorage.getItem(LAST_REPO_KEY)).toBeNull();
  });

  it("does not let an older opening overwrite the latest selection", async () => {
    const repo = (await import("./useRepo")).useRepo();
    const older = deferred<RepoInfo>();
    invoke.mockReturnValueOnce(older.promise).mockResolvedValueOnce(info("/b"));
    const opened = vi.fn().mockResolvedValue(undefined);
    repo.onRepoOpened(opened);

    const openingA = repo.openRepo("/a");
    await repo.openRepo("/b");
    older.resolve(info("/a"));
    await openingA;

    expect(repo.repoPath.value).toBe("/b");
    expect(repo.repoInfo.value).toEqual(info("/b"));
    expect(localStorage.getItem(LAST_REPO_KEY)).toBe("/b");
    expect(opened).toHaveBeenCalledOnce();
  });

  it("does not erase a newer saved path when restoring an old path fails", async () => {
    const repo = (await import("./useRepo")).useRepo();
    localStorage.setItem(LAST_REPO_KEY, "/missing");
    const older = deferred<RepoInfo>();
    invoke.mockReturnValueOnce(older.promise).mockResolvedValueOnce(info("/b"));
    const restoring = repo.restoreLastRepo();
    await repo.openRepo("/b");
    older.reject(new Error("Repository not found"));
    await restoring;

    expect(repo.repoPath.value).toBe("/b");
    expect(repo.repoInfo.value).toEqual(info("/b"));
    expect(localStorage.getItem(LAST_REPO_KEY)).toBe("/b");
  });

  it("invalidates pending opens when the repository is closed", async () => {
    const repo = (await import("./useRepo")).useRepo();
    const pending = deferred<RepoInfo>();
    invoke.mockReturnValueOnce(pending.promise);
    const opened = vi.fn();
    repo.onRepoOpened(opened);
    const opening = repo.openRepo("/a");
    repo.closeRepo();
    pending.resolve(info("/a"));
    await opening;

    expect(repo.repoPath.value).toBeNull();
    expect(repo.repoInfo.value).toBeNull();
    expect(localStorage.getItem(LAST_REPO_KEY)).toBeNull();
    expect(opened).not.toHaveBeenCalled();
  });

  it("stops opening callbacks if the session is closed during one of them", async () => {
    const repo = (await import("./useRepo")).useRepo();
    const pending = deferred<void>();
    const started = deferred<void>();
    repo.onRepoOpened(() => { started.resolve(); return pending.promise; });
    const next = vi.fn();
    repo.onRepoOpened(next);
    invoke.mockResolvedValueOnce(info("/a"));
    const opening = repo.openRepo("/a");
    await started.promise;
    repo.closeRepo();
    pending.resolve();

    expect(await opening).toBe(false);
    expect(next).not.toHaveBeenCalled();
  });

  it("keeps a valid saved path when a post-open callback fails", async () => {
    const repo = (await import("./useRepo")).useRepo();
    localStorage.setItem(LAST_REPO_KEY, "/a");
    invoke.mockResolvedValueOnce(info("/a"));
    repo.onRepoOpened(() => Promise.reject(new Error("Log loading failed")));
    await repo.restoreLastRepo();

    expect(repo.repoPath.value).toBe("/a");
    expect(repo.repoInfo.value).toEqual(info("/a"));
    expect(localStorage.getItem(LAST_REPO_KEY)).toBe("/a");
  });

  it.each(["switch", "close"])("ignores an old info refresh after %s", async (action) => {
    const repo = (await import("./useRepo")).useRepo();
    invoke.mockResolvedValueOnce(info("/a"));
    await repo.openRepo("/a");
    const pending = deferred<RepoInfo>();
    invoke.mockReturnValueOnce(pending.promise);
    const refreshing = repo.refreshInfo();
    if (action === "switch") {
      invoke.mockResolvedValueOnce(info("/b"));
      await repo.openRepo("/b");
    } else {
      repo.closeRepo();
    }
    pending.resolve(info("/a", "old"));
    await refreshing;

    expect(repo.repoInfo.value).toEqual(action === "switch" ? info("/b") : null);
  });

  it("keeps the latest branch info when refreshes finish out of order", async () => {
    const repo = (await import("./useRepo")).useRepo();
    invoke.mockResolvedValueOnce(info("/a"));
    await repo.openRepo("/a");
    const older = deferred<RepoInfo>();
    invoke.mockReturnValueOnce(older.promise).mockResolvedValueOnce(info("/a", "new"));
    const refreshing = repo.refreshInfo();
    await repo.refreshInfo();
    older.resolve(info("/a", "old"));
    await refreshing;

    expect(repo.repoInfo.value).toEqual(info("/a", "new"));
  });
});
