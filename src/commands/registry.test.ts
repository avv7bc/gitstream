import { describe, expect, it, vi } from "vitest";
import {
  commandsForScope,
  getCommand,
  registerCommand,
  unregisterCommand,
} from "@/commands/registry";
import { dispatchHotkey } from "@/commands/hotkeys";

describe("command registry", () => {
  it("filters commands by visibility and enabled predicates", () => {
    const execute = vi.fn();
    registerCommand({
      id: "test.visible",
      label: "Visible",
      scope: "file",
      visible: (ctx) => Boolean(ctx.selectedFile),
      enabled: (ctx) => ctx.selectedFile === "ready.txt",
      execute,
    });

    expect(commandsForScope("file", {}).find((item) => item.id === "test.visible")).toBeUndefined();
    expect(commandsForScope("file", { selectedFile: "busy.txt" }).find((item) => item.id === "test.visible")?.enabled).toBe(false);
    expect(commandsForScope("file", { selectedFile: "ready.txt" }).find((item) => item.id === "test.visible")?.enabled).toBe(true);
    expect(execute).not.toHaveBeenCalled();

    unregisterCommand("test.visible");
  });

  it("dispatches a matching enabled shortcut and prevents browser defaults", () => {
    const execute = vi.fn();
    registerCommand({
      id: "test.shortcut",
      label: "Shortcut",
      scope: "global",
      shortcut: "Ctrl+Shift+K",
      execute,
    });
    const event = { key: "k", code: "KeyK", ctrlKey: true, metaKey: false, altKey: false, shiftKey: true, preventDefault: vi.fn() } as unknown as KeyboardEvent;
    const prevent = vi.spyOn(event, "preventDefault");
    const command = dispatchHotkey(event, {});

    expect(command?.id).toBe("test.shortcut");
    expect(execute).toHaveBeenCalledWith({});
    expect(prevent).toHaveBeenCalled();
    unregisterCommand("test.shortcut");
  });

  it("keeps unavailable shortcuts inert", () => {
    const execute = vi.fn();
    registerCommand({
      id: "test.disabled",
      label: "Disabled",
      scope: "global",
      shortcut: "Ctrl+Shift+J",
      enabled: () => false,
      execute,
    });
    const event = { key: "j", code: "KeyJ", ctrlKey: true, metaKey: false, altKey: false, shiftKey: true, preventDefault: vi.fn() } as unknown as KeyboardEvent;
    const prevent = vi.spyOn(event, "preventDefault");

    expect(dispatchHotkey(event, {})).toBeUndefined();
    expect(execute).not.toHaveBeenCalled();
    expect(prevent).not.toHaveBeenCalled();
    unregisterCommand("test.disabled");
  });

  it("exposes built-in legacy commands", () => {
    expect(getCommand("file.stage")?.scope).toContain("file");
    expect(getCommand("branch.checkout")?.scope).toContain("branch");
    expect(getCommand("settings.open")?.shortcut).toContain("Ctrl+P");
  });
});

describe("toast state", () => {
  it("creates, updates and dismisses toast notifications", async () => {
    const { toast, toasts, updateToast, dismissToast } = await import("@/composables/useToast");
    const id = toast("Failed", "error", { duration: 0 });
    expect(toasts.value.find((item) => item.id === id)?.kind).toBe("error");
    updateToast(id, { title: "Updated" });
    expect(toasts.value.find((item) => item.id === id)?.title).toBe("Updated");
    dismissToast(id);
    expect(toasts.value.some((item) => item.id === id)).toBe(false);
  });
});
