# GitStream

A desktop Git GUI focused on everyday Git workflows — commits, branches, push/pull, diff review — without blocking the UI or unnecessary complexity.

## Features

### File operations
- **Stage / Unstage / Discard** with a single click or via context menu
- File statuses: modified, added, deleted, renamed, untracked, conflicted
- **Unified** and **side-by-side** diff views with a toggle

### Commits
- Commit dialog with a file table and checkboxes — pick what to commit without manually touching the index
- Subject line length indicator (50/72)
- **Amend** to fix the last commit
- **Commit & Push** button — commit and push in one action
- **Partial stage** — select individual lines/hunks in the side-by-side diff, then stage/unstage/discard the selection
- **Multi-select** (Shift/Ctrl+click) for batch stage, unstage, commit, discard, or delete

### History
- Commit graph with branches, tags, HEAD, and remote tracking
- Highlights unpushed commits
- Working Tree / Index row above the graph — double-click opens the commit dialog
- Selected commit details: author, date, message, changed files
- Filter commits by message, author, SHA, date, or branch

### Branches, tags, stash
- Local and remote branch list with ahead/behind indicators
- Checkout, rename, delete branches (including batch delete)
- Checkout a remote branch as a new local branch — upstream tracking set automatically
- Tags and stash entries in the left panel
- Create, delete, and push tags; batch delete/push selected tags
- Stash save (with message and `--include-untracked`), apply, pop, drop

### Remote operations
- **Pull** — merge or rebase, with remote selection; upstream configured automatically
- **Push** — always sets upstream on first push; force push with confirmation
- **Fetch** — fetch without integrating
- Network progress indicator with remote name and timeout countdown
- Configurable network timeout

### Merge / Rebase / Conflict resolution
- Merge branch, rebase branch onto branch
- Continue / Abort controls (ConflictBar) for merge, rebase, cherry-pick, revert
- Accept ours / Accept theirs per file

### Log operations
- Reset (soft / mixed / hard), Revert, Cherry-pick from the commit graph context menu
- Squash multiple commits, Reword commit message

### Repository manager
- Treeview of open repositories with folder grouping
- Drag-and-drop: repos into groups, groups into groups
- Double-click to switch repository
- Context menu: Add Repository, Create Group, Delete

### Keyboard shortcuts
- **Ctrl+K** — open commit dialog
- **Ctrl+T** — stage selected files
- **Shift+Ctrl+T** — unstage selected files
- **Ctrl+M** — merge branch
- **Ctrl+D** — rebase branch
- **F7** — create branch
- **Shift+F7** — create tag
- **Ctrl+A** — select all files in the file panel

### Interface
- Toolbar with **Repository▾**, **Local▾**, **Branch▾** dropdown menus and Pull/Push/Fetch buttons in the center
- Dark theme (Catppuccin-inspired)
- Draggable modal dialogs
- Resizable panels
- Status bar: current branch, ahead/behind, operation progress
- i18n: Russian / English

## Tech stack

- **Frontend:** Vue 3 (Composition API) + TypeScript + Vite
- **Backend:** Tauri 2 (Rust)
- **Git:** git CLI under the hood, `--porcelain` / `--format` parsing

## Download

Latest release: **v0.6.3**

| Platform | Package | Link |
|----------|---------|------|
| Linux | `.deb` (Debian / Ubuntu) | [GitStream_0.6.3_amd64.deb](https://github.com/avv7bc/gitstream/releases/download/v0.6.3/GitStream_0.6.3_amd64.deb) |
| Linux | `.rpm` (Fedora / RHEL) | [GitStream-0.6.3-1.x86_64.rpm](https://github.com/avv7bc/gitstream/releases/download/v0.6.3/GitStream-0.6.3-1.x86_64.rpm) |
| Linux | `.AppImage` | [GitStream_0.6.3_amd64.AppImage](https://github.com/avv7bc/gitstream/releases/download/v0.6.3/GitStream_0.6.3_amd64.AppImage) |
| macOS | `.dmg` (Apple Silicon) | [GitStream_0.6.3_aarch64.dmg](https://github.com/avv7bc/gitstream/releases/download/v0.6.3/GitStream_0.6.3_aarch64.dmg) |
| Windows | `.exe` installer | [GitStream_0.6.3_x64-setup.exe](https://github.com/avv7bc/gitstream/releases/download/v0.6.3/GitStream_0.6.3_x64-setup.exe) |
| Windows | `.msi` | [GitStream_0.6.3_x64_en-US.msi](https://github.com/avv7bc/gitstream/releases/download/v0.6.3/GitStream_0.6.3_x64_en-US.msi) |

All releases: [github.com/avv7bc/gitstream/releases](https://github.com/avv7bc/gitstream/releases)

## Getting started

### Prerequisites

- **Git** — must be in `PATH` (GitStream calls `git` directly)
- **Node.js** 18+ and **npm**
- **Rust toolchain** — install via [rustup.rs](https://rustup.rs/)
- **Tauri system dependencies** — see the [official prerequisites guide](https://tauri.app/start/prerequisites/):
  - **Linux:** `webkit2gtk`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `build-essential`
  - **macOS:** Xcode Command Line Tools
  - **Windows:** Microsoft Visual Studio C++ Build Tools + WebView2

### Quick start (automatic)

The included `start.sh` script checks all dependencies, installs missing ones (Rust via rustup, system packages via the system package manager), and launches the app:

```bash
./start.sh          # development mode
./start.sh build    # production build
```

Supports apt, dnf, pacman, zypper, and macOS. Asks for confirmation before each installation step.

### Manual setup

```bash
git clone <repo-url> gitstream
cd gitstream
npm install
```

### Development mode

Starts the Vite dev server and Tauri with hot reload:

```bash
npm run tauri dev
```

The first run is slow — Rust compiles all dependencies. Subsequent runs are fast.

### Production build

Builds an optimized bundle and a native installer for the current platform:

```bash
npm run tauri build
```

Output:
- **Linux:** `src-tauri/target/release/bundle/{deb,rpm,appimage}/`
- **macOS:** `src-tauri/target/release/bundle/{dmg,macos}/`
- **Windows:** `src-tauri/target/release/bundle/{msi,nsis}/`

### First run

1. Open the app — you'll see an empty repository manager on the left
2. Right-click in the `Repositories` panel → `Add Repository` → select a local Git repository path
3. Double-click a repository to switch to it

### Useful commands

```bash
npm run dev          # Vite only (no Tauri, for UI debugging)
npm run build        # type-check + build frontend
npx vue-tsc --noEmit # TypeScript check without building
```

## License

[MIT](LICENSE)
