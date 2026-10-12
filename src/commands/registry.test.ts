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

  it("preserves Meta aliases for legacy Ctrl shortcuts", () => {
    const execute = vi.fn();
    registerCommand({
      id: "test.meta",
      label: "Meta alias",
      scope: "global",
      shortcut: "Ctrl+K",
      execute,
    });
    const event = {
      key: "k", code: "KeyK", ctrlKey: false, metaKey: true, altKey: false,
      shiftKey: false, preventDefault: vi.fn(), target: null,
    } as unknown as KeyboardEvent;

    expect(dispatchHotkey(event, {})).toMatchObject({ id: "test.meta" });
    expect(execute).toHaveBeenCalled();
    unregisterCommand("test.meta");
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

  it("suppresses dispatch in modal, composing, and editable contexts", () => {
    const execute = vi.fn();
    registerCommand({ id: "test.suppressed", label: "Suppressed", scope: "global", shortcut: "Ctrl+L", execute });
    const base = { key: "l", code: "KeyL", ctrlKey: true, metaKey: false, altKey: false, shiftKey: false, preventDefault: vi.fn(), target: null };
    expect(dispatchHotkey(base as unknown as KeyboardEvent, { modalOpen: true })).toBeUndefined();
    expect(dispatchHotkey({ ...base, isComposing: true } as unknown as KeyboardEvent, {})).toBeUndefined();
    expect(dispatchHotkey({ ...base, target: { tagName: "INPUT", isContentEditable: false } } as unknown as KeyboardEvent, {})).toBeUndefined();
    expect(execute).not.toHaveBeenCalled();
    unregisterCommand("test.suppressed");
  });

  it("logs synchronous and asynchronous command failures", async () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => undefined);
    registerCommand({ id: "test.failure", label: "Failure", scope: "global", shortcut: "Ctrl+L", execute: async () => { throw new Error("boom"); } });
    const event = { key: "l", code: "KeyL", ctrlKey: true, metaKey: false, altKey: false, shiftKey: false, preventDefault: vi.fn(), target: null } as unknown as KeyboardEvent;
    expect(dispatchHotkey(event, {})?.id).toBe("test.failure");
    await Promise.resolve();
    expect(error).toHaveBeenCalledWith("Command execution failed", expect.any(Error));
    error.mockRestore();
    unregisterCommand("test.failure");
  });

  it("exposes built-in legacy commands", () => {
    expect(getCommand("file.stage")?.scope).toContain("file");
    expect(getCommand("branch.checkout")?.scope).toContain("branch");
    expect(getCommand("settings.open")?.shortcut).toContain("Ctrl+P");
    expect(getCommand("file.stage")?.shortcut).toContain("Meta+T");
    const toolbarWithoutRepo = commandsForScope("toolbar", {});
    expect(toolbarWithoutRepo.find((item) => item.id === "commit.open")?.enabled).toBe(false);
    expect(toolbarWithoutRepo.find((item) => item.id === "file.stage")).toBeUndefined();
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
