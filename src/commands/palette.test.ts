import { describe, expect, it } from "vitest";
import { paletteCommandState } from "./palette";
import type { Command } from "./types";

describe("palette command availability", () => {
  const command: Command = { id: "file.stage", label: "Stage", scope: "file", visible: (ctx) => !!ctx.selectedFile, execute: () => {} };
  it("lists hidden commands as disabled in command mode", () => {
    expect(paletteCommandState(command, {}, true)).toEqual({ listed: true, enabled: false });
    expect(paletteCommandState(command, {}, false)).toEqual({ listed: false, enabled: false });
    expect(paletteCommandState(command, { selectedFile: "f" }, true)).toEqual({ listed: true, enabled: true });
  });
});
