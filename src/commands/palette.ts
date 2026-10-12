import type { Command, CommandContext } from "./types";

/** Command mode lists unavailable commands so users can discover the prerequisite. */
export function paletteCommandState(command: Command, context: CommandContext, commandMode: boolean) {
  const visible = command.visible?.(context) ?? true;
  return { listed: commandMode || visible, enabled: visible && (command.enabled?.(context) ?? true) };
}
