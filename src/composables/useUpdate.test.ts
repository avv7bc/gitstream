import { beforeEach, describe, expect, it, vi } from "vitest";

import { invoke } from "@tauri-apps/api/core";
import { useUpdate } from "./useUpdate";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const v1 = { version: "1.1.35", release_url: "https://example.test/v1.1.35" };
const v2 = { version: "2.0.0", release_url: "https://example.test/v2.0.0" };

beforeEach(() => {
  mockedInvoke.mockReset();
  vi.stubGlobal("localStorage", {
    getItem: vi.fn(() => null),
    setItem: vi.fn(),
    removeItem: vi.fn(),
  });
});

describe("useUpdate", () => {
  it("exposes same-major update and next-major notice separately", async () => {
    mockedInvoke.mockResolvedValue({ update: v1, next_major: v2 });
    const state = useUpdate();

    await state.checkForUpdate();

    expect(state.updateInfo.value).toEqual(v1);
    expect(state.nextMajorInfo.value).toEqual(v2);
  });

  it("dismisses only the next-major notice", async () => {
    mockedInvoke.mockResolvedValue({ update: null, next_major: v2 });
    const state = useUpdate();

    await state.checkForUpdate();
    state.dismissNextMajor();

    expect(state.updateInfo.value).toBeNull();
    expect(state.nextMajorInfo.value).toBeNull();
    expect(localStorage.setItem).toHaveBeenCalledWith("gitstream.dismissed-major", "2.0.0");
  });

  it("does not hide a later next-major version after an earlier dismissal", async () => {
    const storage = {
      getItem: vi.fn(() => "2.0.0"),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    };
    vi.stubGlobal("localStorage", storage);
    const v3 = { version: "3.0.0", release_url: "https://example.test/v3.0.0" };
    mockedInvoke.mockResolvedValue({ update: null, next_major: v3 });
    const state = useUpdate();

    await state.checkForUpdate();

    expect(state.nextMajorInfo.value).toEqual(v3);
  });
});
