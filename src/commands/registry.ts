import type { Command, CommandContext, CommandScope, ResolvedCommand } from "./types";

const commands = new Map<string, Command>();

const hasRepo = (context: CommandContext) => Boolean(context.repoPath);
const hasFile = (context: CommandContext) => Boolean(context.selectedFile);
const hasBranch = (context: CommandContext) => Boolean(context.selectedBranch);
const hasRemote = (context: CommandContext) => Boolean(context.selectedRemote);

/** Keep the Ctrl and Meta forms explicit for command palettes and menus. */
function ctrlMeta(key: string, suffix = ""): string[] {
  return [`Ctrl${suffix}+${key}`, `Meta${suffix}+${key}`];
}

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
  { id: "file.stage", label: "Stage", scope: ["file", "toolbar"], visible: hasFile, shortcut: ctrlMeta("T"), enabled: (ctx) => ctx.selectedFileStaged !== "staged", execute: action("file.stage") },
  { id: "file.unstage", label: "Unstage", scope: ["file", "toolbar"], visible: hasFile, shortcut: ctrlMeta("T", "+Shift"), enabled: (ctx) => ctx.selectedFileStaged === "staged" || ctx.selectedFileStaged === "partial", execute: action("file.unstage") },
  { id: "file.discard", label: "Discard", scope: "file", visible: hasFile, enabled: () => true, execute: action("file.discard") },
  { id: "commit.open", label: "Commit", scope: ["global", "toolbar", "file", "commit"], enabled: hasRepo, shortcut: ctrlMeta("K"), execute: action("commit.open") },
  { id: "branch.checkout", label: "Checkout", scope: ["toolbar", "branch"], enabled: hasRepo, shortcut: ctrlMeta("G"), execute: action("branch.checkout") },
  { id: "branch.checkout.local", label: "Checkout", scope: "toolbar", visible: hasRepo, enabled: hasRepo, execute: action("branch.checkout.local") },
  { id: "branch.merge", label: "Merge", scope: ["toolbar", "branch"], enabled: hasRepo, shortcut: ctrlMeta("M"), execute: action("branch.merge") },
  { id: "branch.rebase", label: "Rebase", scope: ["toolbar", "branch"], enabled: hasRepo, shortcut: ctrlMeta("D"), execute: action("branch.rebase") },
  { id: "branch.create", label: "Create branch", scope: "global", visible: hasRepo, shortcut: "F7", enabled: hasRepo, execute: action("branch.create") },
  { id: "tag.create", label: "Create tag", scope: "global", visible: hasRepo, shortcut: "Shift+F7", enabled: hasRepo, execute: action("tag.create") },
  { id: "remote.fetch", label: "Fetch", scope: ["global", "toolbar", "remote"], enabled: hasRepo, execute: action("remote.fetch") },
  { id: "remote.fetch.prune", label: "Fetch and prune", scope: "remote", visible: hasRemote, enabled: hasRepo, execute: action("remote.fetch.prune") },
  { id: "remote.fetchAll", label: "Fetch all", scope: ["toolbar", "remote"], enabled: hasRepo, execute: action("remote.fetchAll") },
  { id: "remote.fetchAllPrune", label: "Fetch all and prune", scope: ["toolbar", "remote"], enabled: hasRepo, execute: action("remote.fetchAllPrune") },
  { id: "remote.pull", label: "Pull", scope: ["global", "toolbar"], shortcut: "Alt+PageDown", enabled: hasRepo, execute: action("remote.pull") },
  { id: "remote.push", label: "Push", scope: ["global", "toolbar"], shortcut: "Alt+PageUp", enabled: hasRepo, execute: action("remote.push") },
  { id: "remote.edit", label: "Edit remote URL", scope: "remote", visible: hasRemote, enabled: hasRepo, execute: action("remote.edit") },
  { id: "remote.rename", label: "Rename remote", scope: "remote", visible: hasRemote, enabled: hasRepo, execute: action("remote.rename") },
  { id: "remote.remove", label: "Remove remote", scope: "remote", visible: hasRemote, enabled: hasRepo, execute: action("remote.remove") },
  { id: "remote.branch.delete", label: "Delete remote branch", scope: "branch", visible: hasBranch, enabled: hasRepo, execute: action("remote.branch.delete") },
  { id: "repository.clone", label: "Clone repository", scope: "toolbar", execute: action("repository.clone") },
  { id: "repository.add", label: "Add repository", scope: "toolbar", execute: action("repository.add") },
  { id: "repository.group.add", label: "Add group", scope: "toolbar", execute: action("repository.group.add") },
  { id: "stats.open", label: "Repository statistics", scope: "toolbar", enabled: hasRepo, execute: action("stats.open") },
  { id: "stash.open", label: "Stash", scope: "toolbar", enabled: hasRepo, execute: action("stash.open") },
  { id: "settings.open", label: "Settings", scope: ["global", "toolbar"], shortcut: ["Ctrl+P", "Meta+P", "Alt+P"], execute: action("settings.open") },
  { id: "palette.open", label: "Command Palette", scope: "global", shortcut: ctrlMeta("P", "+Shift"), execute: action("palette.open") },
  { id: "log.toggle", label: "Git output", scope: "global", shortcut: ["Ctrl+O", "Alt+O", "Meta+O"], execute: action("log.toggle") },
  { id: "oplog.undo", label: "Undo", scope: ["global", "toolbar"], shortcut: ["Ctrl+Z", "Meta+Z"], enabled: hasRepo, execute: action("oplog.undo") },
  { id: "oplog.redo", label: "Redo", scope: ["global", "toolbar"], shortcut: ["Ctrl+Shift+Z", "Meta+Shift+Z"], enabled: hasRepo, execute: action("oplog.redo") },
  { id: "oplog.open", label: "Operations", scope: ["global", "toolbar"], enabled: hasRepo, shortcut: "Ctrl+Alt+Z", execute: action("oplog.open") },
  { id: "history.interactive", label: "Interactive rebase", scope: ["global", "toolbar"], enabled: hasRepo, execute: action("history.interactive") },
  { id: "reflog.open", label: "Reflog", scope: ["global", "toolbar"], enabled: hasRepo, execute: action("reflog.open") },
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
  const requested = normalizeShortcut(shortcut);
  return shortcuts.some((item) => {
    const normalized = normalizeShortcut(item);
    if (normalized === requested) return true;
    // Ctrl is the cross-platform equivalent of Meta for legacy shortcuts.
    return normalized.replace(/(^|\+)meta(?=\+|$)/, "$1ctrl") === requested.replace(/(^|\+)meta(?=\+|$)/, "$1ctrl");
  });
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
