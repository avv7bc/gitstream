/** The surfaces where a command can be presented. */
export type CommandScope = "global" | "toolbar" | "file" | "branch" | "commit" | "repository" | "remote" | "stash";

/** Runtime state supplied to visibility, enabled and execute callbacks. */
export interface CommandContext {
  repoPath?: string | null;
  selectedFile?: string | null;
  selectedFiles?: string[];
  selectedCommit?: string | null;
  selectedBranch?: string | null;
  [key: string]: unknown;
}

export type CommandPredicate = (context: CommandContext) => boolean;
export type CommandAction = (context: CommandContext) => void | Promise<void>;

export interface Command {
  id: string;
  label: string;
  description?: string;
  scope: CommandScope | CommandScope[];
  shortcut?: string | string[];
  visible?: CommandPredicate;
  enabled?: CommandPredicate;
  execute: CommandAction;
}

export interface ResolvedCommand extends Omit<Command, "visible" | "enabled"> {
  visible: boolean;
  enabled: boolean;
}

export interface ContextMenuItem {
  id?: string;
  label?: string;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  separator?: boolean;
}
