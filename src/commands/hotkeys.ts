import { commandMatchesShortcut, allCommands } from "./registry";
import type { Command, CommandContext } from "./types";

function reportCommandError(error: unknown): void {
  // Keep global keyboard dispatch independent from the Tauri progress module;
  // command implementations may still surface the error in Git output/toast.
  console.error("Command execution failed", error);
}

function eventShortcut(event: KeyboardEvent): string {
  const modifiers: string[] = [];
  if (event.ctrlKey) modifiers.push("Ctrl");
  if (event.metaKey) modifiers.push("Meta");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");
  const physical = event.code.startsWith("Key") ? event.code.slice(3)
    : event.code.startsWith("Digit") ? event.code.slice(5)
    : event.code;
  const key = physical && physical !== "Unidentified"
    ? physical
    : event.key === " " ? "Space" : event.key.length === 1 ? event.key.toUpperCase() : event.key;
  return [...modifiers, key].join("+");
}

export function dispatchHotkey(event: KeyboardEvent, context: CommandContext = {}): Command | undefined {
  if (event.defaultPrevented || event.isComposing || (event as KeyboardEvent & { keyCode?: number }).keyCode === 229) return undefined;
  if (context.modalOpen) return undefined;
  const target = event.target as HTMLElement | null;
  if (target && (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))) return undefined;
  const shortcut = eventShortcut(event);
  const command = allCommands().find((item) => {
    if (!commandMatchesShortcut(item, shortcut)) return false;
    if (item.visible && !item.visible(context)) return false;
    return !(item.enabled && !item.enabled(context));
  });
  if (!command) return undefined;
  event.preventDefault();
  try {
    const result = command.execute(context);
    if (result && typeof (result as Promise<void>).catch === "function") {
      void (result as Promise<void>).catch(reportCommandError);
    }
  } catch (error) {
    reportCommandError(error);
  }
  return command;
}

export { eventShortcut };
