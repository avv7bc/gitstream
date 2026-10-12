import type { CommandContext } from "./types";

/** Creates a context while keeping legacy component state loosely typed. */
export function createCommandContext(values: CommandContext = {}): CommandContext {
  return { ...values };
}

export function hasRepository(context: CommandContext): boolean {
  return typeof context.repoPath === "string" && context.repoPath.length > 0;
}

export function hasSelection(context: CommandContext): boolean {
  return Boolean(context.selectedFile || context.selectedCommit || context.selectedBranch);
}
