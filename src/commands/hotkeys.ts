import { commandMatchesShortcut, allCommands } from "./registry";
import type { Command, CommandContext } from "./types";

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
  const shortcut = eventShortcut(event);
  const command = allCommands().find((item) => {
    if (!commandMatchesShortcut(item, shortcut)) return false;
    if (item.visible && !item.visible(context)) return false;
    return !(item.enabled && !item.enabled(context));
  });
  if (!command) return undefined;
  event.preventDefault();
  void command.execute(context);
  return command;
}

export { eventShortcut };
