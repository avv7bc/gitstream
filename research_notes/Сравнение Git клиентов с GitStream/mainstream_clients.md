# Mainstream free / free-tier / evaluation desktop Git GUI clients: feature inventory 2025–2026 (GitHub Desktop, Sourcetree, GitKraken, Fork, Sublime Merge, SmartGit, Tower)

Research date: 2026-10-11. Method: official release notes and changelogs where reachable (sublimemerge.com/download, github.com/desktop/desktop/releases, github.blog changelog, smartgit.dev/whats-new). Where official pages timed out, I used release aggregators (git-fork.com and fork.dev release notes timed out, so Fork data comes from releasebot.io; help.gitkraken.com release notes timed out, so GitKraken data comes from componentsource.com and the gitkraken.com blog). Search-engine summaries are marked as such. Prior-knowledge items I could not verify this session are listed under **Gaps** and are not presented as findings.

---

## Q1. Core and advanced feature inventory per client, with paid-only status

### Takeaway
All seven clients cover the everyday basics: commit/amend, hunk or line staging, stash, branches/tags, push/pull/fetch, cherry-pick/revert/merge/rebase. They differ on the advanced layer: visual interactive rebase, a built-in 3-way conflict resolver, reflog/undo, worktrees, LFS, AI and PR integration. GitHub Desktop is the most minimal, with drag-and-drop squash/reorder instead of a full interactive-rebase editor. Fork, SmartGit, Sublime Merge and Tower are the "power" clients. GitKraken restricts its free tier to local and public repositories.

### Cited Findings

**GitHub Desktop (free, open source; macOS and Windows; latest stable 3.6.6, beta 3.6.7)**
- Drag a commit onto another to squash (with a new message for the combined commit). Drag commits anywhere in history to reorder. Amend the last commit's message or contents. "Undo last commit" returns its changes to the working directory. Revert creates an inverse commit. Squash/reorder/amend shipped in 2.9 (2021). — [GitHub blog 2.9](https://github.blog/news-insights/product-news/github-desktop-2-9-squashing-reordering-amending/); [GitHub Docs: managing commits](https://docs.github.com/en/enterprise-server@3.17/desktop/managing-commits/options-for-managing-commits-in-github-desktop)
- Cherry-pick was added in 2.7 and copies a commit from one branch to another. — [GitHub Docs](https://docs.github.com/en/enterprise-server@3.16/desktop/managing-commits/options-for-managing-commits-in-github-desktop) (via search summary)
- **Worktrees (3.6.0, 2026-06-26):** a "Current Worktree" toolbar menu next to Current Repository and Current Branch lists the main and linked worktrees, with a "New Worktree" button. It is pitched explicitly for parallel coding-agent sessions. — [GitHub Changelog 2026-06-26](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/)
- **Copilot commit-message generation:** existed in 3.5. In 3.6 it reads `.github/copilot-instructions.md` and `AGENTS.md` and follows the repo's ruleset commit-metadata rules. — [GitHub Changelog](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/); 3.5 origin from [issoh.co.jp](https://www.issoh.co.jp/tech/details/12796/) (secondary)
- **Copilot merge-conflict assistance (3.6):** explains the conflicting changes and suggests a resolution that the user can review, accept or edit. A later beta added delete-vs-modify conflicts ("keep" or "delete"). — [GitHub Changelog](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/); delete/modify per search summary of [kingy.ai](https://kingy.ai/ai-launch-tracker/github-desktop-3-6-copilot-integration-2026-06-26/)
- Every Copilot feature has a model picker and BYOK support (third-party provider or local model). Copilot features need GitHub Copilot access, so they are paid or quota-bound even though Desktop is free. — [GitHub Changelog](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/)
- The PR list shows CI status icons, with tooltips added in 3.6.7-beta3. A shortcut and menu item open the repo in GitHub Copilot. Copilot no longer adds itself as co-author on generated messages. Git LFS progress tracking exists (crash fixed in 3.6.7-beta2). An "Open in Cygwin" shell integration exists. — [GitHub Desktop releases](https://github.com/desktop/desktop/releases)
- 3.6.5 fixed stdin-based hooks on Windows, so Desktop runs hooks. 3.6.6-beta1 moved to Electron 44.1.1 and dropped macOS 12. Desktop bundles Git for Windows 2.53.0.windows.4 and Git Credential Manager 2.9.0 (3.6.4). — [GitHub Desktop releases](https://github.com/desktop/desktop/releases)

**Atlassian Sourcetree (free; Windows 3.4.x; macOS 4.x)**
- 3.4.23 (2025-03-26) added "Ignore Whitespace" to the Diff tab and numeric version sorting for tags. — [Sourcetree Windows release notes](https://product-downloads.atlassian.com/software/sourcetree/windows/ga/ReleaseNotes_3.4.30.html) (dates and contents via search summary)
- 3.4.24 (2025-07-19) added stashing of selected or staged files only, API-token authentication and lower SSH-Agent CPU use. — [same](https://product-downloads.atlassian.com/software/sourcetree/windows/ga/ReleaseNotes_3.4.30.html)
- 3.4.25 (2025-09-12) moved to Git 2.51.0 and Git LFS 3.7.0, so LFS is bundled. — [ReleaseNotes_3.4.25](https://product-downloads.atlassian.com/software/sourcetree/windows/ga/ReleaseNotes_3.4.25.html)
- 3.4.27 (2025-12-04) added keyboard shortcuts for custom actions and selectable branch filtering in log/history. It restored double-click diff and fixed interactive rebase of child commits, which confirms an interactive-rebase UI exists. — [ReleaseNotes_3.4.27](https://product-downloads.atlassian.com/software/sourcetree/windows/ga/ReleaseNotes_3.4.27.html)
- 3.4.29 (2026-02-26) replaced deprecated Bitbucket cross-workspace APIs (flagged as a breaking change). 3.4.30 (2026-03-07) is refresh fixes only. — [ReleaseNotes_3.4.30](https://product-downloads.atlassian.com/software/sourcetree/windows/ga/ReleaseNotes_3.4.30.html)

**GitKraken Desktop (freemium; Windows/macOS/Linux; 11.x → 12.x in 2026)**
- **Free tier limits:** "Community" works only with local repos and public remotes. Private repos and cloud issue-tracker integrations need Pro. — [GitKraken pricing via search summary](https://www.gitkraken.com/pricing); [thesoftwarescout 2026](https://thesoftwarescout.com/gitkraken-pricing-2026-plans-costs-is-it-really-worth-it/) (secondary)
- Reviewers list the merge conflict editor and Launchpad among Pro features. Pro costs about $4.95/user/month billed annually; sources conflict ($4, $49/yr). — [thesoftwarescout](https://thesoftwarescout.com/gitkraken-pricing-2026-plans-costs-is-it-really-worth-it/); [toolradar](https://toolradar.com/tools/gitkraken/pricing) (both third-party)
- **Undo/Redo button:** 11.8 extended undo/redo to interactive rebases, cherry-picks and AI-composed commits. — [GitKraken blog: 11.8 "undo when it doesn't"](https://www.gitkraken.com/blog/gitkraken-desktop-11-8-visibility-where-it-matters-undo-when-it-doesnt)
- Interactive rebase supports Pick, Reword, Squash and Drop. — [gitkraken.com/lp/e3-v2](https://www.gitkraken.com/lp/e3-v2); [help: interactive rebase](https://help.gitkraken.com/gitkraken-client/interactive-rebase)
- Built-in merge tool: side-by-side view with editable output, plus "Auto-resolve with AI" giving explanations and confidence levels. — [gitkraken.com/git-client](https://gitkraken.com/git-client) (via search summary)
- GitKraken AI generates commit messages, stash messages, changelogs and PR descriptions. — [gitkraken.com/git-client](https://gitkraken.com/git-client) (via search summary)
- 11.8.0 (2026-01-14) added ARM builds for Linux/Windows and shallow clone. 11.9.0 (2026-02-03) added shallow-clone options in the Clone dialog, word-wrap in diff and merge views, and an experimental squash-on-merge. — [ComponentSource 11.8](https://www.componentsource.com/product/gitkraken/releases/2943716); [11.9](https://www.componentsource.com/product/gitkraken/releases/2946766)
- 11.10.0 (2026-03-03) added branch pinning to the left of the Commit Graph, a "smart branch filtering" mode, restoring files from any commit into the working dir, sparse checkout, Gemini 3 Flash for GitKraken AI and bundled Git 2.53.0. It removed GitHub Actions from the Left Panel. — [ComponentSource 11.10](https://www.componentsource.com/product/gitkraken/releases/2951151)
- **12.0.0 (2026-04-14):** adds an **Agent Sessions View**. Each worktree is a card showing branch, pending changes, PR status and the Claude Code agent's state (working, waiting, error). "New Agent Session" creates a worktree, runs setup commands and launches an agent (Claude Code, Codex CLI, OpenCode, Copilot CLI or Gemini CLI). The embedded terminal keeps one session per worktree. — [ComponentSource 12.0.0](https://www.componentsource.com/product/gitkraken/releases/2959401)
- 12.3.1 (2026-07-21) has the tagline "monitoring agents". No details were retrievable. — [GitKraken release notes (search summary)](https://support.gitkraken.com/release-notes/current/)
- "Commit Composer" belongs to GitLens 17.4 (VS Code, Sept 2025), not GitKraken Desktop 12.0. — [ComponentSource brand news](https://www.componentsource.com/brand/git-kraken/news) (via search summary)

**Fork (paid $59.99 with free evaluation; macOS and Windows)**
- Homepage features: fetch/pull/push, commit/amend, branches/tags, create/delete remote repos, checkout, cherry-pick, revert, merge, rebase, stashes, submodules, visual interactive rebase (edit/reorder/squash), merge-conflict helper and built-in resolver, image diffs, side-by-side diff, file/directory history, blame, line-by-line staging, recent commit messages, file tree browsing at any commit, **Reflog to restore lost commits**, stashes in the commit list, Git-flow, Git LFS, GitHub notifications, and a repository manager with summary and statistics. — [fork.dev](https://fork.dev)
- Price is $59.99 with a free evaluation. A one-time license covers 1 user on up to 3 machines (third-party claim). Homepage shows Mac 2.66.7 and Windows 2.21.1; Mac and Windows have separate version numbering. — [fork.dev](https://fork.dev); license scope per [itechguides](https://www.itechguides.com/products/fork/) (secondary)
- Bisect UI exists: good/bad coloring (2.56), clickable bisect refs (2.64), improved bisect UX (2.67). — [releasebot Fork](https://releasebot.io/updates/git-fork)
- Interactive rebase gained `--update-refs` (2.61 rebase dialog, 2.63 interactive rebase) and avatars (2.61). — [releasebot](https://releasebot.io/updates/git-fork)
- Worktrees: a worktree icon on branches checked out elsewhere (2.59), background worktree removal (2.61), a Create Worktree dialog and Quick Launch entry plus optional worktree removal on branch delete (2.63), worktree tab disambiguation (2.65), and "checkout remote branch as worktree" (2.68). — [releasebot](https://releasebot.io/updates/git-fork)
- Patches: paste a patch from the clipboard (2.64) and Save as Patch from the stash menu (2.69). — [releasebot](https://releasebot.io/updates/git-fork)
- Quick Launch (command palette) gained fuzzy file-path search (2.57). — [releasebot](https://releasebot.io/updates/git-fork)

**Sublime Merge (paid with unlimited evaluation; Windows/macOS/Linux; build 2132)**
- Homepage features: line-by-line and hunk staging (select lines to split hunks), commit editing, side-by-side diffs, syntax highlighting (Sublime Text engine, 40+ languages, plus installed third-party syntaxes), character-level diffs, draggable hunk edges for more context, instant as-you-type search over commit message/author/path/contents, a built-in merge tool, a command palette, blame and file history, submodule management, Git Flow, command-line integration, display of the exact Git commands run, a custom high-performance Git reading library, and themes. — [sublimemerge.com](https://www.sublimemerge.com/)
- The free evaluation has no time limit, but a license is "required for continued use". The 2018 evaluation restriction was dark theme only. A personal license is $99 once with 3 years of updates. Business seats are about $65 down to $50 per seat per year (older listing $75). — [Sublime HQ store](https://www.sublimehq.com/store); [TNW 2018](https://thenextweb.com/dd/2018/09/21/sublime-git-is-a-clean-code-editor-for-merge-conflicts/) (old)
- Git LFS lock/unlock (build 2125), "Edit Tracked Pattern" for LFS (2102), worktree management in the locations bar (2130), reftables (2130), SHA-256 repositories and split-index mode (2121), HDR image diff (2096), and a refined character-level diff (2096). — [Sublime Merge changelog](https://www.sublimemerge.com/download)

**SmartGit (non-commercial license free with registration; Windows/macOS/Linux; 26.1 stable, 26.2 preview)**
- Free non-commercial licenses cover open-source developers (about 250 contributions per year minimum), public education, NGOs and hobby use. Requests are reviewed manually. The hobby tier allows only repos where you are the sole author or that are open source, and excludes enterprise integrations for self-hosted GitHub/GitLab/Bitbucket/Azure DevOps. Commercial use starts at about $5/user/month or a perpetual license. — [smartgit.dev register-non-commercial](https://www.smartgit.dev/register-non-commercial/); [smartgit.dev purchase](https://www.smartgit.dev/purchase/); [syntevo blog 2025-06-26](https://blog.syntevo.com/smartgit/2025/06/26/non-commercial-full-version.html) ("full feature set" for academic/OSS/charity)
- SmartGit has three main windows. Standard and Log are named; the third is not named on the 26.1 page (likely Working Tree, unverified). 26.1 gave each window a distinct icon with quick switching in the top-right corner. — [SmartGit 26.1 What's New](https://www.smartgit.dev/whats-new/26_1/)
- 26.1 adds SHA-256 and reftable support (Git 3.0 readiness). It integrates GitLab merge requests into the Standard window, with CI pipeline state in "My History" and commit messages generated from selected GitLab issues. Worktree-aware Branches/Graph shows clean/modified state, and clean worktree refs can be dragged in the Graph. A **Commit Overlap** column in the Graph rates each commit against a pinned commit (none, slight, heavy, subset, superset, same files) with tooltips listing common files and possible conflicts. Provider configuration can be set per remote. The Pull popup defaults to the current remote or all remotes. — [SmartGit 26.1](https://www.smartgit.dev/whats-new/26_1/)
- AI: 25.1 introduced AI commit-message generation, rewording and automatic stash descriptions, with an explicit consent step before sending data. 26.1 adds "Ask AI" with a custom prompt over multiple commits (including merge commits), `{{ branch }}` placeholders, `@ai-marker` templates, an AI button in Edit Message, and AI reword that keeps commit signatures. — [SmartGit 25.1](https://www.smartgit.dev/whats-new/25_1/); [SmartGit 26.1](https://www.smartgit.dev/whats-new/26_1/)
- 26.2 preview (26.2.004, 2026-08-19): the Graph flags unread generated notes. — [changelog-26.2](https://www.smartgit.dev/changelogs/changelog-26.2.txt) (via search summary)

**Tower (reference only; paid with 30-day trial; macOS 17.x, Windows 13.x)**
- Features: universal Undo (Cmd+Z) covering local changes, reverted commits, deleted commits and deleted branches. Drag-and-drop actions, visual merge-conflict solving, an unsynced-commits view, built-in SSH/credential/token handling and GitHub PRs. Pro add-ons: stacked branches, custom workflows (start/finish), Graphite support, automatic branch cleanup, self-hosted repos and PR-based merge detection. The 30-day trial unlocks all Pro features. — [git-tower.com/features](https://www.git-tower.com/features)
- Windows 9.1 (Jul 2025) added Gitea support and Gitmoji. Windows 10 (Nov 2025) added comprehensive Graphite support. Windows 11 (Feb 2026) added commit templates and ARM-native builds. Windows 13.1 (2026-07-16) added branch pinning, "Fully Merged"/"Stale" badges and an Archived Branches view with auto-archiving. Mac 17.0 (Jul 2026) improved Fully-Merged detection, including merged-PR detection. — [AlternativeTo Tower 9.1](https://alternativeto.net/news/2025/7/git-client-tower-9-1-for-windows-adds-gitea-support-gitmoji-integration-and-other-updates/); [AlternativeTo Tower 10](https://alternativeto.net/news/2025/11/powerful-git-client-tower-10-for-windows-introduces-comprehensive-graphite-support); [softantenna](https://softantenna.com/softwares/7518-tower-for-windows) (secondary)
- Tower has an AI product area in its docs. — [git-tower.com/docs/ai](https://www.git-tower.com/docs/ai/product-overview/) (content not fetched)

### Inferences
- **Interactive rebase UI:** Fork (visual, `--update-refs`), GitKraken (pick/reword/squash/drop), Sourcetree and Tower all have one. GitHub Desktop offers only drag-and-drop squash/reorder plus amend. GitStream currently has squash and reword only, so a visual interactive-rebase editor is a parity gap against Fork and SmartGit.
- **Undo:** GitKraken (undo/redo, including rebases) and Tower (Cmd+Z for nearly everything) are the leaders. Fork offers reflog-based recovery, and GitHub Desktop only "undo last commit". GitStream's backup ref `refs/gitstream/backup/<branch>` is a partial analogue; a reflog UI or undo stack would close the gap.
- **Worktrees** were the dominant 2025–2026 trend. GitHub Desktop 3.6, GitKraken 12 (agent sessions), Fork 2.59–2.68, Sublime Merge 2130 and SmartGit 26.1 all added or deepened them, driven by parallel AI coding agents.
- **Git 3.0 readiness (SHA-256 + reftable)** shipped in Sublime Merge (2121/2130), SmartGit 26.1 and Fork (reftable fix in 2.70) during 2025–2026.
- **Paid-only map:** GitHub Desktop and Sourcetree are fully free, but GitHub Desktop's AI needs Copilot. GitKraken free is limited to local/public repos, with the merge editor, Launchpad and cloud integrations in Pro. Fork and Sublime Merge are paid but nagware-style evaluations without hard feature locks (Sublime: unlimited time). SmartGit is free only for eligible non-commercial use, with full features except enterprise integrations for hobby users. Tower has a 30-day trial only.

### Gaps
- I could not reach the official Fork release notes (git-fork.com and fork.dev timed out), so the Windows-specific Fork feature list (2.x Windows numbering) is unverified. The fork.dev homepage version (Mac 2.66.7) conflicts with releasebot listing Mac 2.70 on 2026-09-04; the homepage was likely stale or cached.
- GitKraken 12.1–12.x details, the official current pricing table and exact free-tier AI quotas were not retrieved (help.gitkraken.com and gitkraken.com/pricing timed out).
- Sourcetree for Mac 2025–2026 release notes were not found.
- Not verified this session (prior knowledge only, to confirm before using in a matrix):
  - GitHub Desktop: co-author trailers in the commit box, no built-in 3-way merge editor, no blame/reflog/bisect UI.
  - Sublime Merge: 3-way merge tool and fully keyboard-driven UI.
  - SmartGit: built-in Conflict Solver (3-way), git-flow and git-flow-lite, "Distributed Reviews", interactive rebase via drag-and-drop in the Log graph, and support for submodules, LFS, bisect and GPG signing.
  - GitKraken: GPG/SSH signing, LFS, submodules, git-flow, Workspaces (multi-repo, Pro for cloud), Launchpad (cross-repo PRs/issues) and the Command Palette.
  - Fork: GPG/SSH commit signing.
  - Tower: bisect, reflog and LFS support.

---

## Q2. Notable UX ideas unique to each client

### Takeaway
Each client has a signature UX idea:
- **GitKraken:** one-click Undo/Redo and an agent-session dashboard.
- **Sublime Merge:** speed, a command palette and as-you-type content search.
- **Fork:** Quick Launch, visual interactive rebase and a repository manager.
- **GitHub Desktop:** extreme simplicity, drag-to-squash/reorder and Copilot.
- **SmartGit:** three specialized windows (Standard/Log/…) and Commit Overlap analysis.
- **Tower:** universal Cmd+Z and branch-hygiene automation.

### Cited Findings
- **GitKraken:** Undo/Redo also reverses interactive rebases, cherry-picks and AI-composed commits (11.8). — [GitKraken blog 11.8](https://www.gitkraken.com/blog/gitkraken-desktop-11-8-visibility-where-it-matters-undo-when-it-doesnt)
- **GitKraken:** worktree cards show live AI-agent status, and one action creates a worktree, runs setup and launches an agent (12.0). — [ComponentSource 12.0.0](https://www.componentsource.com/product/gitkraken/releases/2959401)
- **GitKraken:** branch pinning to the left of the graph and "smart branch filtering" (11.10). — [ComponentSource 11.10](https://www.componentsource.com/product/gitkraken/releases/2951151)
- **Sublime Merge:** "When you're using Sublime Merge, you're using Git" — it shows the exact Git commands. It also offers instant search across message, author, path and contents, a custom high-performance Git reading library, and draggable hunk edges to expand context. — [sublimemerge.com](https://www.sublimemerge.com/)
- **Sublime Merge:** the locations bar filter supports `and`, `or`, `not`, `before:` and `after:` (2102). Its command palette includes Switch Repository (2096) and Open in Terminal (2121). — [Sublime Merge changelog](https://www.sublimemerge.com/download)
- **Sublime Merge:** "keep local changes when resetting" option (2132) and "skip empty commits when cherry-picking multiple" (2130). — [changelog](https://www.sublimemerge.com/download)
- **Fork:** Quick Launch with fuzzy file-path search and worktree actions. Its tab UX has rename/color tabs, restore closed tabs and vertical tabs (2.68–2.70). It shows ahead/behind counts on the pull/push toolbar buttons (2.62), offers "Select Stale Local Branches" (2.62) and "File History for selected code" (line-range history, 2.59), and shows diffs for conflicts resolved in external mergers (2.58). — [releasebot Fork](https://releasebot.io/updates/git-fork)
- **GitHub Desktop:** drag one commit onto another to squash, and drag to reorder. — [GitHub blog 2.9](https://github.blog/news-insights/product-news/github-desktop-2-9-squashing-reordering-amending/)
- **GitHub Desktop:** the worktree switcher sits as a third toolbar segment beside Repository and Branch. — [GitHub Changelog](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/)
- **SmartGit:** three main windows with distinct icons and quick switching. A Commit Overlap column finds reorder/squash candidates and predicts upstream conflicts. Pull vs Pull All is configurable, with optional deletion of merged branches after pull. — [SmartGit 26.1](https://www.smartgit.dev/whats-new/26_1/)
- **Tower:** "Undo anything" with Cmd+Z and drag-and-drop. Tower Workflows offer configurable branching models with start/finish actions. Stale/Fully-Merged badges and automatic archiving keep branches tidy. — [git-tower.com/features](https://www.git-tower.com/features); [softantenna](https://softantenna.com/softwares/7518-tower-for-windows) (secondary)

### Inferences
- These ideas map most directly onto GitStream's SmartGit-parity goal and its existing architecture:
  - Ahead/behind on the push/pull buttons and "select stale branches" (Fork)
  - Line-range file history (Fork)
  - Commit Overlap (SmartGit)
  - Search by file content (Sublime Merge)
  - Undo stack (GitKraken/Tower)
  - Show the executed git command, which GitStream already partly does through its Git output window

### Gaps
- The free-tier status of GitKraken's Command Palette and Workspaces was not verified.

---

## Q3. What's new in 2025–2026 releases (chronological highlights)

### Takeaway
2025–2026 releases cluster around four themes. Ordered by how many clients shipped them:
1. **Worktrees and AI coding-agent workflows:** all of GitHub Desktop, GitKraken, Fork, Sublime Merge and SmartGit.
2. **AI commit messages, conflict help and code review:** GitHub Desktop, GitKraken, Fork and SmartGit.
3. **Git 3.0 readiness (SHA-256/reftable):** Sublime Merge and SmartGit, with reftable work in Fork.
4. **Performance:** mainly Sublime Merge.

### Cited Findings
- **GitHub Desktop 3.6.0 (2026-06-26):** worktrees; Copilot SDK; repo-aware commit messages; AI conflict resolution; model picker; BYOK. — [GitHub Changelog](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/)
- **GitHub Desktop 3.6.4–3.6.7-beta (Aug–Oct 2026):** Git for Windows 2.53, Electron 44, an option to keep the worktree toolbar segment visible, and an Open in Copilot shortcut. — [releases](https://github.com/desktop/desktop/releases)
- **GitKraken 11.8 (Jan 2026):** ARM builds, shallow clone, undo/redo for rebases.
- **GitKraken 11.9 (Feb 2026):** diff word-wrap, squash-on-merge.
- **GitKraken 11.10 (Mar 2026):** sparse checkout, pinned branches, Gemini 3 Flash.
- **GitKraken 12.0 (Apr 2026):** Agent Sessions View.
- **GitKraken 12.3.1 (Jul 2026):** agent monitoring.
- GitKraken sources: [ComponentSource](https://www.componentsource.com/product/gitkraken/releases/2959401); [GitKraken blog](https://www.gitkraken.com/blog/gitkraken-desktop-11-8-visibility-where-it-matters-undo-when-it-doesnt)
- **Fork (Mac), Aug 2025 – Sep 2026, versions 2.56 → 2.70:**
  - AI commit messages and AI code review via external agents: Claude (2.59/2.60), Codex (2.64) and Cursor (2.70), with editable agent requests (2.64) and AI review of SHA ranges (2.61).
  - Worktrees end-to-end.
  - Bisect improvements.
  - Patch paste/save.
  - .gitignore template picker (2.65).
  - Sidebar keyboard navigation (2.66).
  - Collapsible commit details (2.66), chunk navigation (2.67), restore closed tabs (2.67).
  - Pixel diff for 16-bit RGBA images (2.68).
  - Submodule icon in file views (2.69).
  - Vertical tabs (2.70).
  - Source: [releasebot Fork](https://releasebot.io/updates/git-fork)
- **Sublime Merge:**
  - **Build 2110 (2025-07-28):** faster refresh, diff and branch commands; Add Remote suggests a name from the URL; bundled Git 2.50.1.
  - **Build 2121 (2025-12-18):** much faster repo loading and blame; SHA-256; split-index; Enter in blame jumps to the commit.
  - **Build 2125 (2026-04-14):** LFS lock/unlock.
  - **Build 2130 (2026-09-14):** reftables; worktree management; click-to-copy hash; `submodule_head` variable for custom commands.
  - **Build 2132 (2026-09-18):** keep local changes on reset.
  - No builds between Oct 2024 and Jul 2025.
  - Source: [Sublime Merge changelog](https://www.sublimemerge.com/download)
- **SmartGit 25.1 (2025):** AI commit messages, reword and stash descriptions with a consent flow. **SmartGit 26.1 (GA 2026-05-18):** UI/icon refresh, SHA-256/reftable, GitLab MRs in the Standard window, Commit Overlap, worktree-aware graph, Ask AI over commits. **26.2 preview** in Aug 2026. — [SmartGit 25.1](https://www.smartgit.dev/whats-new/25_1/); [26.1](https://www.smartgit.dev/whats-new/26_1/); [26.1 release blog](https://blog.syntevo.com/smartgit/2026/05/18/smartgit-26-1-released.html)
- **Sourcetree Windows 3.4.23–3.4.30 (Mar 2025 – Mar 2026):** maintenance-level. Notable items are ignore-whitespace in diff, stash of selected/staged files, custom-action shortcuts, Git 2.51/LFS 3.7 and Bitbucket API migration. — [Sourcetree release notes](https://product-downloads.atlassian.com/software/sourcetree/windows/ga/ReleaseNotes_3.4.30.html)
- **Tower (Windows 9.1 → 13.1, Mac → 17.x):** Gitea, Gitmoji, Graphite, commit templates, ARM, branch pinning/archiving, merged-PR detection. — [AlternativeTo](https://alternativeto.net/news/2025/11/powerful-git-client-tower-10-for-windows-introduces-comprehensive-graphite-support); [softantenna](https://softantenna.com/softwares/6978-tower) (secondary)

### Inferences
- Sourcetree looks to be in maintenance mode, with no major features in 2025–2026. GitKraken and GitHub Desktop are pivoting toward "AI agent orchestration" around worktrees. Fork integrates external CLI agents (Claude/Codex/Cursor) instead of running its own AI backend, a lightweight model that suits a Tauri app like GitStream.

### Gaps
- Exact GitHub Desktop 3.5 release date and contents were not retrieved.
- Tower's 2026 release-note specifics come from secondary sites. The official git-tower.com release notes were not fetched.

---

## Q4. AI features, PR integration and hosting integrations (cross-cutting)

### Takeaway
AI commit-message generation is now table stakes: GitHub Desktop, GitKraken, Fork and SmartGit have it; Sublime Merge, Sourcetree, and possibly Tower do not. GitHub Desktop and GitKraken add AI conflict resolution. PR integration is strongest in GitKraken (Launchpad, Pro), SmartGit (GitHub/GitLab in the Standard window) and Tower.

### Cited Findings
- GitHub Desktop: Copilot commit messages and conflict explanation/resolution, a model picker, BYOK/local models, and CI status in the PR list. — [GitHub Changelog](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/); [releases](https://github.com/desktop/desktop/releases)
- GitKraken: AI commit, stash, changelog and PR descriptions; AI auto-resolve of conflicts with confidence levels; Gemini 3 Flash available. — [gitkraken.com/git-client](https://gitkraken.com/git-client); [ComponentSource 11.10](https://www.componentsource.com/product/gitkraken/releases/2951151)
- Fork: AI commit messages and branch/SHA-range code review delegated to Claude, Codex or Cursor; GitHub notifications. — [releasebot](https://releasebot.io/updates/git-fork); [fork.dev](https://fork.dev)
- SmartGit: AI commit/reword/stash descriptions, Ask AI, GitLab MR + CI pipeline integration, and commit messages from GitLab issues. — [SmartGit 26.1](https://www.smartgit.dev/whats-new/26_1/)
- Sourcetree: Bitbucket integration (API migration in 3.4.29). — [release notes](https://product-downloads.atlassian.com/software/sourcetree/windows/ga/ReleaseNotes_3.4.30.html)
- Tower: GitHub PRs, Graphite review, Gitea. — [git-tower.com/features](https://www.git-tower.com/features); [AlternativeTo](https://alternativeto.net/news/2025/7/git-client-tower-9-1-for-windows-adds-gitea-support-gitmoji-integration-and-other-updates/)

### Inferences
- Sublime Merge's changelog shows no AI features through build 2132, which is a deliberate differentiation (speed and purity).

### Gaps
- Tower's AI feature set: the docs page exists, but its contents were not fetched.

---

## Q5. Licensing / free-tier summary

### Takeaway
- **Fully free:** GitHub Desktop (OSS; AI needs Copilot) and Sourcetree.
- **Free tier with hard limits:** GitKraken (no private repos; Pro for merge editor, Launchpad and cloud integrations).
- **Free with eligibility:** SmartGit (non-commercial, registration and manual review).
- **Paid with soft evaluation:** Fork ($59.99) and Sublime Merge ($99 personal, unlimited evaluation).
- **Trial only:** Tower (30 days).

### Cited Findings
- See citations in Q1: [fork.dev](https://fork.dev); [Sublime HQ store](https://www.sublimehq.com/store); [SmartGit purchase](https://www.smartgit.dev/purchase/); [SmartGit non-commercial](https://www.smartgit.dev/register-non-commercial/); [GitKraken pricing (via search)](https://www.gitkraken.com/pricing); [git-tower.com/features](https://www.git-tower.com/features); [GitHub Changelog](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/)

### Inferences
- A free, open-source, cross-platform (including Linux) client with SmartGit-level features would sit in a gap. Among the free options:
  - GitHub Desktop has no Linux build and is simple by design.
  - Sourcetree has no Linux build and is stagnant.
  - GitKraken free is crippled for private repos.
  - SmartGit free is eligibility-gated.

### Gaps
- Official current GitKraken prices and Tower prices were not retrieved.
- The Sublime Merge business price is conflicting ($75/yr vs a $65–$50 tiered schedule).
