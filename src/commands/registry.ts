import type { Command, CommandContext, CommandScope, ResolvedCommand } from "./types";

const commands = new Map<string, Command>();

function action(id: string) {
  return (context: CommandContext) => {
    const actions = context.actions;
    if (actions && typeof actions === "object") {
      const callback = (actions as Record<string, unknown>)[id];
      if (typeof callback === "function") return (callback as (ctx: CommandContext) => void | Promise<void>)(context);
    }
  };
}

const builtIns: Command[] = [
  { id: "file.stage", label: "Stage", scope: ["file", "toolbar"], shortcut: "Ctrl+T", enabled: (ctx) => Boolean(ctx.selectedFile) && ctx.selectedFileStaged !== "staged", execute: action("file.stage") },
  { id: "file.unstage", label: "Unstage", scope: ["file", "toolbar"], shortcut: "Ctrl+Shift+T", enabled: (ctx) => Boolean(ctx.selectedFile) && (ctx.selectedFileStaged === "staged" || ctx.selectedFileStaged === "partial"), execute: action("file.unstage") },
  { id: "file.discard", label: "Discard", scope: "file", enabled: (ctx) => Boolean(ctx.selectedFile), execute: action("file.discard") },
  { id: "commit.open", label: "Commit", scope: ["global", "toolbar", "file", "commit"], shortcut: "Ctrl+K", enabled: (ctx) => Boolean(ctx.repoPath), execute: action("commit.open") },
  { id: "branch.checkout", label: "Checkout", scope: ["toolbar", "branch"], shortcut: "Ctrl+G", enabled: (ctx) => Boolean(ctx.repoPath), execute: action("branch.checkout") },
  { id: "branch.merge", label: "Merge", scope: ["toolbar", "branch"], shortcut: "Ctrl+M", enabled: (ctx) => Boolean(ctx.repoPath), execute: action("branch.merge") },
  { id: "branch.rebase", label: "Rebase", scope: ["toolbar", "branch"], shortcut: "Ctrl+D", enabled: (ctx) => Boolean(ctx.repoPath), execute: action("branch.rebase") },
  { id: "remote.fetch", label: "Fetch", scope: ["global", "toolbar", "remote"], execute: action("remote.fetch") },
  { id: "remote.pull", label: "Pull", scope: ["global", "toolbar"], shortcut: "Alt+PageDown", enabled: (ctx) => Boolean(ctx.repoPath), execute: action("remote.pull") },
  { id: "remote.push", label: "Push", scope: ["global", "toolbar"], shortcut: "Alt+PageUp", enabled: (ctx) => Boolean(ctx.repoPath), execute: action("remote.push") },
  { id: "settings.open", label: "Settings", scope: ["global", "toolbar"], shortcut: ["Ctrl+P", "Meta+P", "Alt+P"], execute: action("settings.open") },
];

for (const command of builtIns) commands.set(command.id, command);

export function registerCommand(command: Command): void {
  commands.set(command.id, command);
}

export function unregisterCommand(id: string): boolean {
  return commands.delete(id);
}

export function getCommand(id: string): Command | undefined {
  return commands.get(id);
}

export function allCommands(): Command[] {
  return [...commands.values()];
}

export function commandsForScope(scope: CommandScope, context: CommandContext = {}): ResolvedCommand[] {
  return allCommands()
    .filter((command) => Array.isArray(command.scope) ? command.scope.includes(scope) : command.scope === scope)
    .filter((command) => command.visible?.(context) ?? true)
    .map((command) => ({
      ...command,
      visible: true,
      enabled: command.enabled?.(context) ?? true,
    }));
}

export function commandMatchesShortcut(command: Command, shortcut: string): boolean {
  const shortcuts = Array.isArray(command.shortcut) ? command.shortcut : command.shortcut ? [command.shortcut] : [];
  return shortcuts.some((item) => normalizeShortcut(item) === normalizeShortcut(shortcut));
}

export function normalizeShortcut(shortcut: string): string {
  return shortcut
    .split("+")
    .map((part) => part.trim().toLowerCase())
    .filter(Boolean)
    .map((part) => part === "cmd" ? "meta" : part)
    .sort((a, b) => {
      const order = ["ctrl", "meta", "alt", "shift"];
      const ai = order.indexOf(a); const bi = order.indexOf(b);
      if (ai >= 0 && bi >= 0) return ai - bi;
      if (ai >= 0) return -1;
      if (bi >= 0) return 1;
      return a.localeCompare(b);
    })
    .join("+");
}

export function contextForCommand(context: CommandContext = {}): CommandContext {
  return context;
}
