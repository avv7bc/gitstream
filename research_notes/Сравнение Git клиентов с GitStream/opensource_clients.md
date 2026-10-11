# Open-source desktop Git GUI клиенты: инвентаризация функций для сравнения с GitStream (по состоянию на 2026-10-11)

> Метаданные (звёзды, даты последнего push/релиза, язык, лицензия) получены через GitHub REST API 2026-10-11 и отражены ссылками на страницы репозиториев/релизов. Счётчики звёзд — снимок на эту дату.

## Покрытие базовых операций (partial staging, stash, ветки/теги/remotes, interactive rebase, конфликты, undo/oplog, reflog, bisect, submodules, LFS, worktrees, подпись, blame, история файла, поиск, граф, качество diff, патчи, PR/forge, AI, command palette, клавиатура, вкладки, терминал, внешние инструменты)

### Takeaway
Базовый слой (stage/unstage на уровне строк/хунков, stash, ветки/теги/remotes, merge/rebase, blame, история файла) сегодня есть почти у всех живых клиентов; дифференциация идёт по: interactive rebase UI, 3-way resolver, undo/oplog, worktrees, forge/PR-интеграции, AI и command palette. Самые «полные» из классических — Git Extensions (только Windows) и SourceGit; самые концептуально новые — GitButler (виртуальные/стековые ветки + oplog) и волна Tauri-клиентов 2025–2026 (GitWand, AngKorGit, Graft, GitDesktop, rgitui на GPUI).

### Cited Findings

**Git Extensions (C#/.NET, WinForms, только Windows)**
- Позиционируется как standalone Windows UI; интеграция с Windows Explorer (shell extension) и Microsoft Visual Studio — [README](https://github.com/gitextensions/gitextensions)
- Требует .NET 10 Desktop Runtime, рекомендуемый Git 2.53+, Windows 10+ — [v7.2.1 release](https://github.com/gitextensions/gitextensions/releases); [README](https://github.com/gitextensions/gitextensions)
- Встроенные плагины: Auto compile submodules, Create local tracking branches, Delete obsolete branches, Find large files, GitHub, Gource, Impact Graph, Periodic background fetch, Proxy Switcher, Release Notes Generator, Statistics, Plugin Manager; сторонний плагин Gerrit Code Review — [docs](https://git-extensions-documentation.readthedocs.io/en/main/)
- Scripts (пользовательские скрипты в Settings > Scripts), Submodules (manage/add), Worktrees, File history с вкладками Commit/Diff/View/Blame, Reflog (Browse Repository > Filter > Reflog), Merge conflicts, Create/Apply patches, Interactive rebase (autosquash, edit/reword), GPG, command line, интеграция VS и VS Code, Hotkeys, Revision graph с поиском/quick search/goto commit/фильтром истории — [docs](https://git-extensions-documentation.readthedocs.io/en/main/)
- v7.2.0: ahead/behind рисуется прямо в RevisionGrid, скрытие fetch-all UI при единственном remote — [v7.2.0 release](https://github.com/gitextensions/gitextensions/releases)
- v7.1.0: поддержка MinTTY как консольного эмулятора (встроенная консоль), поддержка global config в XDG_CONFIG_HOME — [v7.1.0 release](https://github.com/gitextensions/gitextensions/releases)
- v7.2.1: «session checkpoints» упоминаются как опция View ("Show session checkpoints") — [v7.2.1 release](https://github.com/gitextensions/gitextensions/releases)

**Git Cola (Python + Qt через QtPy: PyQt5/PyQt6/PySide2)**
- Status-виджет (modified/staged/unmerged/untracked; stage/unstage, revert, delete, add to .gitignore, запуск editor/difftool/mergetool); Diff editor со stage/unstage выбранных строк или хунков, revert выбранного — [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html)
- DAG viewer (`git cola dag` или отдельная команда `git dag`) — [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html); [README](https://github.com/git-cola/git-cola)
- Stash: save/apply/pop/drop/rename, опции Keep Index / Save Index — [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html)
- Grep (`git grep`), поиск коммитов, Browse tracked files, ветки деревом по `/`, commit editor с sign-off, GPG-подписью, amend, override даты/автора, индикатор длины строки — [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html)
- Spell check в редакторе сообщений (системные словари, hunspell, aspell) — [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html)
- Interactive rebase через собственный `git-cola-sequence-editor` (можно использовать отдельно как `GIT_SEQUENCE_EDITOR`) — [README](https://github.com/git-cola/git-cola)
- Применение патчей `git am` drag-n-drop файлов/папок; Blame viewer; Custom actions через `guitool.<name>.*` (prompt, confirm, shortcuts); Quick Open избранных/недавних репо; темы (QSS), сохранение layout'ов; виджет submodules; детект LFS; поддержка worktrees — [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html)
- Hotkeys задокументированы отдельно — [hotkeys](https://git-cola.gitlab.io/share/doc/git-cola/hotkeys.html)

**gitg (GNOME; Vala + GTK3/libhandy, libgit2-glib)**
- gitg 50 (2026-07-01): настраиваемые колонки истории, вкладка General в настройках, Custom actions, удаление remote-веток, полноценный push action, checkout detached, HEAD в истории, редактирование upstream, rename/resolve тегов, выбор diff по умолчанию Unified/Split, fetch all remotes, навигация по результатам поиска — [NEWS](https://gitlab.gnome.org/GNOME/gitg/-/blob/master/NEWS)
- gitg 44 (2023-08-03): подпись коммитов, создание патча из ref, edit/remove remotes, фильтр файлов в stage view, история submodule, external editor, core.hooksPath — [NEWS](https://gitlab.gnome.org/GNOME/gitg/-/blob/master/NEWS)

**GitAhead / Gittyup (C++/Qt, libgit2)**
- GitAhead: «no longer under active development», только low-level maintenance; авторы просят продолжать в rebranded-форке — [README](https://github.com/gitahead/gitahead)
- Gittyup — продолжение GitAhead; Qt ≥ 6.4, libgit2, cmark, libssh2 — [README](https://github.com/Murmele/Gittyup)
- Gittyup: вкладки репозиториев; single-branch view; staging отдельных строк чекбоксами или горячими клавишами S/U/R (stage/unstage/revert выделения); blame с интегрированной timeline; amend; разрешение конфликтов при rebase и continue; разрешение whole-file и бинарных конфликтов; «starred commits»; шаблоны сообщений коммита; кнопка открытия терминала и файлового менеджера; поддержка Gitea; настраиваемые hotkeys; превью изображений; фильтр first-parent; SHA256-репозитории и mmap-индексация в v2.0.0; RU в списке языков — [Gittyup site](https://murmele.github.io/Gittyup/)

**GitButler (Tauri + Svelte/TypeScript, Rust backend; CLI `but`)**
- Stacked branches (автоматический restack при amend/edit), Parallel (virtual) branches — работа над несколькими ветками одновременно без переключения — [README](https://github.com/gitbutlerapp/gitbutler)
- Управление коммитами drag-n-drop: uncommit, reword, amend, move, split, squash («забудьте про rebase -i») — [README](https://github.com/gitbutlerapp/gitbutler)
- Undo Timeline: логирование всех операций и откат любой — [README](https://github.com/gitbutlerapp/gitbutler); снапшоты состояния виртуальных веток, незакоммиченной работы и конфликтов перед крупными действиями, хранятся в object database Git, «Revert» на записи откатывает к состоянию до неё — [docs: Timeline](https://docs.gitbutler.com/features/timeline)
- First Class Conflicts: «rebases always succeed», коммиты помечаются conflicted и разрешаются в любое время в любом порядке — [README](https://github.com/gitbutlerapp/gitbutler)
- Forge integration GitHub/GitLab/Bitbucket: открытие/обновление PR, CI-статусы — [README](https://github.com/gitbutlerapp/gitbutler)
- AI: генерация commit messages, имён веток, PR descriptions; установка hooks/skills для агентных систем — [README](https://github.com/gitbutlerapp/gitbutler)
- v0.22.0 (2026-07-30): нативные GitHub stacked PRs; cherry-pick создаёт conflicted-коммиты вместо блокировки; переработка CLI (`but commit/amend/squash/move`, `--json`) — [releases](https://github.com/gitbutlerapp/gitbutler/releases)
- v0.22.1–0.22.3: «Create PR»/«Land» в контекстном меню ветки; при конфликте обновления от target с незакоммиченной работой сохраняются обе версии; есть TUI (`but tui`) с branch mode, поиском веток — [releases](https://github.com/gitbutlerapp/gitbutler/releases)

**Gitnuro (Kotlin + JetBrains Compose Multiplatform, JGit)**
- Заявленные функции: diff текстовых файлов, лог со всеми ветками, stage/unstage hunks и отдельных строк, clone, commit, reset, revert, amend, merge, rebase, interactive rebase, ветки/теги (локально), remote-ветки, pull/push, force push, удаление remote-веток, управление remotes, stash/pop, detached checkout, diff изображений side-by-side, init, поиск по сообщению/автору/id, blame, история файла, темы, side-by-side diff, submodules, смена tracking-ветки — [README](https://github.com/JetpackDuba/Gitnuro)
- В планах (не реализовано по README): create/apply patches, удаление тегов на remote, stashes в дереве лога, syntax highlighting в diff — [README](https://github.com/JetpackDuba/Gitnuro)
- 2.0 beta01 (2026-08-11): rename ветки, подпись коммитов/тегов SSH-ключами, git-credential-manager, multiselect в status, новый дизайн графа для сложных репо, сниженная RAM, ускоренный diff, file-watching по директориям с учётом .gitignore; хранилище настроек сбрасывается — [releases](https://github.com/JetpackDuba/Gitnuro/releases)
- 2.0 beta02–04: LFS-фиксы, hooks на Windows через Git Bash, «amend при отсутствии изменений», планируется экспериментальная native-сборка без Java; JAR требует Java 25 — [releases](https://github.com/JetpackDuba/Gitnuro/releases)

**Guitar (C++20, Qt 6; Windows/Linux)**
- Commit/stage, ветки (create/checkout/rename/delete local/remote), push/pull/fetch с `--prune`, push all tags, merge/rebase с abort, cherry-pick, stash, reset soft/hard, clone (в т.ч. прямо с GitHub); граф коммитов с аватарами; side-by-side и inline diff с «syntax-aware highlighting»; blame; история файла; reflog viewer; инкрементальный поиск по сообщениям и содержимому файлов; jump по хэшу; много репозиториев в сайдбаре с группировкой «working folders» — [README](https://github.com/soramimi/Guitar) (README помечен автором как сгенерированный AI и «может быть неточным»)

**QGit (C++/Qt5, ≥5.11)**
- Просмотр ревизий, diff, истории файлов, аннотаций (blame), archive tree; коммит с выбором файлов; apply/save patch series; drag-n-drop коммитов между двумя экземплярами qgit; custom actions (последовательности команд/скрипты с выводом в окно терминала); GUI для команд StGIT (push/pop/apply/save patches) — [README](https://github.com/tibirna/qgit)

**Ungit (Node.js web-UI + Electron-сборки)**
- Web-приложение (запускает сервер, UI в браузере, можно на удалённой машине `:8448`); Gerrit-плагин; PGP-подпись через git config (без ввода пароля GPG в UI); внешние merge tools (Kaleidoscope, p4merge); авто-refresh через рекурсивный watch с учётом .gitignore; требует Node ≥ 20.19 — [README](https://github.com/FredrikNoren/ungit)

**RelaGit (TypeScript/Solid.js, Electron)**
- «early beta»; поддержка submodules и commit graph в модальном окне отмечены как сделанные; рассматривается переход с git CLI на libgit2; расширяемость через «Workflows» — [README](https://github.com/relagit/relagit)
- v0.16.11: Electron обновлён с 29 до 39, добавлены поиск и прокрутка списка веток — [releases](https://github.com/relagit/relagit/releases)

**GitFiend (Electron shell + Rust core)**
- Архитектура: gitfiend-shell (Electron main process) + gitfiend-core (backend на чтение репо/действия); frontend включён в shell, но «minified for now» — [Support README](https://github.com/GitFiend/Support)
- gitfiend-shell использует electron ^39.2.1 — [gitfiend-shell](https://github.com/GitFiend/gitfiend-shell)
- v0.46.4: copy path, rebase branch, force push, auto-fetch — [releases](https://github.com/GitFiend/Support/releases)
- v1.0.0-preview (2026-09-27): переписанный UI поверх gitfiend-core, «has some rough edges» — [release](https://github.com/GitFiend/Support/releases/tag/v1.0.0-preview)

**SourceGit (C#, Avalonia; Win/macOS/Linux) — заметный «новый классический» клиент**
- Clone/fetch/pull/push, merge/rebase/reset/revert/cherry-pick, amend/reword/squash, interactive rebase, branches/remotes/tags/stashes, submodules, worktrees, archive, save as patch/apply, file histories, blame, revision diffs, branch diff, image diff (side-by-side/swipe/blend), лог git-команд, поиск коммитов, GitFlow, Git LFS, bisect, issue links, workspaces, custom actions, создание PR на GitHub/GitLab/Gitea/Gitee/Bitbucket, AI-генерация commit message, помощник conventional commits, кастомные темы, 14 языков вкл. русский — [README](https://github.com/sourcegit-scm/sourcegit)

**GitHub Desktop (TypeScript, Electron)**
- MIT, ~21.9k звёзд, активные бета-релизы (release-3.6.7-beta3, 2026-10-07) — [repo](https://github.com/desktop/desktop)

**Новые open-source клиенты 2024–2026 (Tauri/Electron/GPUI)**
- **GitWand** (Tauri 2 + Vue 3, MIT, создан 2026-04, v3.12.1 2026-10-10): stage строк/хунков; amend непушнутых коммитов прямо из графа; merge preview без изменения working tree (`git merge-base` + `git merge-file -p --diff3`) с классификацией файлов auto/partial/manual/add-delete; авторазрешение тривиальных конфликтов по реестру паттернов (diff3/diff2); diff с подсветкой 30+ языков, word-level diff (LCS), сворачивание неизменённого, canvas-minimap; file history + blame + time-travel diff; Workspaces (несколько репо в одном dashboard); «Today» — кросс-репо inbox; PR/MR GitHub/GitLab/Bitbucket/Azure DevOps с inline-комментариями и аннотациями CI; MCP-сервер, Agents panel (запуск Claude Code), встроенный WebGL-терминал с вкладками shell/claude/codex; CodeMirror-редактор файлов; CLI — [README](https://github.com/devlint/GitWand); [repo](https://github.com/devlint/GitWand)
- **AngKorGit** (Tauri v2 + Rust/libgit2, MIT, создан 2026-08, v0.22.0): виртуализированный граф (100k коммитов), drag ветки на ветку для merge/rebase; 3-pane конфликт (обе стороны с номерами строк, чекбокс на всю сторону, «+» на строку, редактируемый результат); worktrees как отдельные вкладки; stage файлов/хунков/строк; interactive rebase; cherry-pick одного/многих; submodules; несколько аккаунтов на хост и per-repo identity профили; AI через уже залогиненный AI CLI (commit message, объяснение коммита/конфликта, review, PR description); ⌘K command palette со всеми командами; Esc закрывает ровно одно окно; word-level diff, minimap; 16 тем; сознательно без in-app PR review, без телеметрии и аккаунтов — [README](https://github.com/cheat2001/angkorgit)
- **Graft** (Tauri + Rust + React/Tailwind, MIT; создан 2025-10, последний push 2025-12): command palette Ctrl+K (30+ команд), Quick Search Ctrl+P (коммиты/ветки/stash), interactive rebase drag-n-drop, Monaco diff, граф на 10k+ коммитов — [README](https://github.com/Dancode-188/graft)
- **GitDesktop** (Tauri 2 + React 19, Apache-2.0, создан 2026-06, v0.13.4 2026-10-08): «AI-native, keyboard-first»; полный PR-цикл (review/comment/approve/merge) для GitHub/GitLab/Bitbucket; offline «local PRs»; GitHub Actions cockpit; запуск coding-агентов (Claude Code, Codex, Copilot, opencode) в изолированных worktree/контейнерах; AI review/security audit; весь доступ к GitHub через `gh` CLI; один переключатель скрывает все AI-поверхности — [README](https://github.com/theBGuy/GitDesktop)
- **rgitui** (Rust + GPUI от Zed, MIT, создан 2026-03, «early, active development»): GPU-рендеринг 60fps; вкладки репо; stage/unstage/discard файл/хунк/строка; interactive rebase (pick/squash/reword/fixup/drop); bisect (start/good/bad/skip/reset); worktrees; submodules; undo stack и crash recovery; 3-way resolver Current/Result/Incoming с syntect; reflog; global search через `git grep`; GitHub device-flow, создание PR; command palette Ctrl+P; AI commit messages (Gemini) — [README](https://github.com/noahbclarkson/rgitui)
- **GitQuill** (Electron + Vue + Monaco, MIT, создан 2024-11): редактирование файлов прямо в diff; amend прошлых коммитов через rebase tool; ссылки на issue tracker; кнопки для произвольных git-команд; WIP через ветки вместо stash; при конфликте автоматически сбрасывает unmerged-состояние; нет file watching (refresh по фокусу); лог команд в `.git/.quill/app.log` — [README](https://github.com/adamsol/GitQuill)
- **gg — GUI for Jujutsu** (Tauri, Rust, Apache-2.0, v0.45.0 2026-09-23): GUI для jj, режимы native и web (`gg web`) — [README](https://github.com/gulbanana/gg)
- Прочие молодые Tauri-проекты по поиску GitHub: gitru (TypeScript, GPL-3.0, 286★, v0.0.0-beta.7), gitnado (Tauri + Lit + Rust, «no telemetry, fully offline»), GitCat («Safety Manager» снимает snapshot перед каждой мутацией), flow-git (Tauri v2 + Svelte 5, canvas-граф 60 FPS, «48h Safe Discard»), GitWave (Tauri 2 + Vue 3, AI-коммиты, 0★) — [gitru](https://github.com/ruru-m07/gitru); [gitnado](https://github.com/hegsie/gitnado); [GitCat](https://github.com/zangjiucheng/GitCat); [flow-git](https://github.com/longgoll/flow-git); [GitWave](https://github.com/ginuim/gitwave)
- Commit (m1guelpf; Tauri, MIT) — «command palette-style Git client», заброшен с 2023-08 — [repo](https://github.com/m1guelpf/commit)

### Inferences

Сводная матрица (✓ — подтверждено источниками выше; ~ — частично; ✗ — явно отсутствует/в планах; ? — не установлено). GitStream-колонка — по CLAUDE.md проекта.

| Функция | GitStream | Git Ext. | Git Cola | gitg 50 | Gittyup | GitButler | Gitnuro 2.0β | Guitar | QGit | Ungit | SourceGit | GitWand | AngKorGit | rgitui |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Stage строк/хунков | ✓ | ? | ✓ | ? | ✓ (строки) | ~ (hunk-assign к веткам) | ✓ | ? | ✗ (файлы) | ? | ? | ✓ | ✓ | ✓ |
| Stash | ✓ | ? | ✓ (+rename) | ~ (ref в sidebar) | ? | ? | ✓ | ✓ | ✗ | ? | ✓ | ? | ✓ | ✓ |
| Interactive rebase UI | ✗ (только reword/squash) | ✓ | ✓ (sequence editor) | ? | ✗ | ✓ (drag-n-drop мутации) | ✓ | ✗ | ✗ | ? | ✓ | ? | ✓ | ✓ |
| 3-way resolver | ✗ (ours/theirs) | ? (внешний tool) | ✗ (mergetool) | ? | ~ (whole-file/binary) | ~ (first-class conflicts) | ? | ✗ | ✗ | внешний | ? | ✓+авторазрешение | ✓ | ✓ |
| Undo/oplog | ✗ (backup-ref при force-push) | ~ (session checkpoints?) | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ? | ✗ | ? | ? | ✓ (undo stack) |
| Reflog | ✗ | ✓ | ? | ? | ? | ~ (oplog) | ? | ✓ | ? | ? | ? | ? | ? | ✓ |
| Bisect | ✗ | ? | ? | ? | ? | ? | ? | ? | ? | ? | ✓ | ? | ? | ✓ |
| Submodules | ✗ | ✓ | ✓ | ✓ (история) | ? | ? | ✓ | ? | ? | ? | ✓ | ? | ✓ | ✓ |
| LFS | ✗ | ? | ~ (детект) | ? | ? | ? | ✓ | ? | ? | ? | ✓ | ? | ? | ? |
| Worktrees | ✗ | ✓ | ✓ | ? | ? | ? | ? | ? | ? | ? | ✓ | ? | ✓ (вкладки) | ✓ |
| Подпись | ✗ | ✓ (GPG) | ✓ (GPG) | ✓ | ? | ? | ✓ (SSH) | ? | ? | ~ (через config) | ? | ✓ | ? | ? |
| Blame / история файла | ✓/✓ | ✓/✓ | ✓/? | ? | ✓/✓ | ? | ✓/✓ | ✓/✓ | ✓/✓ | ? | ✓/✓ | ✓/✓ | ? | ✓/✓ |
| Word-diff / подсветка синтаксиса | ✗/✗ | ? | ? | ? | ? | ? | ✗ (в планах) | ?/✓ | ? | ? | ? | ✓/✓ | ✓/? | ?/✓ |
| Image diff | ✓ (превью) | ? | ? | ? | ✓ (превью) | ? | ✓ (side-by-side) | ? | ? | ? | ✓ (swipe/blend) | ? | ? | ? |
| Патчи | ✗ | ✓ | ✓ (am drag-n-drop) | ✓ (create) | ? | ? | ✗ (в планах) | ? | ✓ | ? | ✓ | ? | ? | ? |
| PR/forge | ✗ | ~ (плагины GitHub/Gerrit) | ✗ | ✗ | ~ (Gitea) | ✓ (GH/GL/BB, stacked PR) | ✗ | ~ (clone с GitHub) | ✗ | ~ (Gerrit) | ~ (создание PR) | ✓ (4 форджа) | ✗ (намеренно) | ~ (GitHub) |
| AI | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ (commit msg) | ✓ (MCP, агенты) | ✓ (через CLI) | ✓ (Gemini) |
| Command palette | ✗ | ? | ✗ | ✗ | ✗ | ? | ✗ | ✗ | ✗ | ✗ | ? | ? | ✓ | ✓ |
| Мульти-репо | ✓ (treeview) | ? | ✓ (bookmarks) | ? | ✓ (вкладки) | ✓ (окна) | ✓ (вкладки) | ✓ (sidebar+группы) | ✗ | ✗ | ✓ (workspaces) | ✓ (workspaces) | ✓ | ✓ (вкладки) |
| Терминал | ✗ | ✓ (MinTTY) | ✗ | ✗ | ✓ (кнопка) | ✗ | ✗ | ? | ✓ (custom actions) | ✗ | ? | ✓ (встроенный) | ? | ? |
| Custom actions/скрипты | ✗ | ✓ (скрипты+плагины) | ✓ (guitool) | ✓ (v50) | ✗ | ✗ | ✗ | ? | ✓ | ✗ | ✓ | ? | ✗ (намеренно) | ✗ |
| Лог git-команд | ✓ (Git output) | ? | ? | ✗ | ? | ? | ? | ? | ? | ? | ✓ | ? | ? | ? |
| Auth/askpass в GUI | ✓ | ? | ? | ? | ~ (credential storage) | ✓ (forge OAuth) | ✓ (git-credential-manager) | ? | ? | ✗ (GPG) | ✓ (SSH per remote) | ✓ (OAuth device flow) | ✓ (multi-account) | ✓ (device flow) |

- Наиболее явные пробелы GitStream относительно лидеров: interactive rebase UI (есть у Git Extensions, Git Cola, Gitnuro, SourceGit, AngKorGit, rgitui, Graft), 3-way resolver, reflog/undo, worktrees, submodules/LFS, подпись коммитов, syntax highlight/word-diff, command palette.
- Сильные стороны GitStream, которые редки у конкурентов: Sync Assistant (диагностика non-fast-forward с безопасными remedy и backup-ref) — у прямых аналогов ближе всего GitButler (oplog) и GitCat (snapshot перед мутацией); askpass-мост с паузой сетевого таймаута; окно Git output (аналог есть у SourceGit «Git command logs» и GitQuill `.git/.quill/app.log`).
- GitWand — практически прямой «стековый близнец» GitStream (Tauri 2 + Vue 3 + system git), и уже имеет PR/MCP/терминал; это главный референс для позиционирования.

### Gaps
- Для Git Extensions не удалось из оглавления документации подтвердить: LFS, bisect, встроенный 3-way merge (исторически используется KDiff3 — не подтверждено источниками в этой сессии), word-diff, line-level staging.
- Для gitg не подтверждены stage строк, blame, stash UI — источником был только NEWS последних двух версий.
- GitButler: не подтверждены детали line-level staging (в GitButler модель «assign hunk to branch»), blame, история файла, image diff, command palette; сведения о retention снапшотов oplog в документации не нашлись.
- Для Ungit, RelaGit, GitFiend подробных feature-листов в источниках не найдено (README Ungit не перечисляет операции; сайт gitfiend.com отдаёт SPA без текста).
- Guitar: README сам предупреждает, что сгенерирован AI и может быть неточен — функции требуют перепроверки.
- Статус Gittyup «Hunk-level staging» и LFS не описан на сайте.

## Отличительные концепции и UX-идеи

### Takeaway
Ключевые «идеи-доноры»: виртуальные/стековые ветки + oplog с откатом любой операции (GitButler); «конфликт — это состояние коммита, а не блокировка» (GitButler, теперь и для cherry-pick); merge preview и авторазрешение тривиальных конфликтов (GitWand); расширяемость через плагины/скрипты/custom actions (Git Extensions, Git Cola guitool, gitg 50, QGit, SourceGit); keyboard-first + ⌘K palette (AngKorGit, Graft, rgitui); worktrees-как-вкладки для AI-агентов (AngKorGit, GitDesktop).

### Cited Findings
- GitButler: параллельные ветки в одном working tree, stacked branches с auto-restack, drag-n-drop мутации коммитов (uncommit/move/split/squash) вместо `rebase -i` — [README](https://github.com/gitbutlerapp/gitbutler)
- GitButler Timeline: снапшоты перед крупными действиями в Git ODB, кнопка Revert на любой записи, просмотр файлов, изменённых в снапшоте — [docs: Timeline](https://docs.gitbutler.com/features/timeline)
- GitButler: «Rebases always succeed», conflicted-коммиты разрешаются позже в любом порядке; с v0.22.0 cherry-pick тоже создаёт conflicted-коммиты; нативные GitHub stacked PRs — [README](https://github.com/gitbutlerapp/gitbutler); [releases](https://github.com/gitbutlerapp/gitbutler/releases)
- GitButler поставляет GUI, CLI `but` и TUI на одном Rust-движке; есть skills/hooks для AI-агентов — [README](https://github.com/gitbutlerapp/gitbutler); [releases](https://github.com/gitbutlerapp/gitbutler/releases)
- Git Extensions: Plugin Manager и набор плагинов (Statistics, Gource, Impact Graph, Find large files, Delete obsolete branches, Periodic background fetch, Release Notes Generator) + пользовательские Scripts — [docs](https://git-extensions-documentation.readthedocs.io/en/main/)
- Git Cola: spellcheck в commit editor; `git dag`; «Diff Mode» — unstage/revert правок из произвольного коммита обратно в worktree; клавиша `s` стейджит выделение/хунк под курсором при просмотре diff; Alt-drag отдаёт пути для командной строки; сообщение коммита в `.git/GIT_COLA_MSG` синхронизируется с внешними инструментами/агентами; Turbo mode для огромных репо; Remote mode (`connect`/`server`) — [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html)
- Gittyup: «starred commits», шаблоны commit message, blame с интегрированной timeline, клавиши S/U/R для stage/unstage/revert выделенного текста — [Gittyup site](https://murmele.github.io/Gittyup/)
- Gitnuro: цель — мультиплатформенный клиент «without relying on web technologies»; 2.0 — новый граф для сложных репо и watcher по директориям с учётом .gitignore (меньше RAM и inotify-лимитов на Linux) — [README](https://github.com/JetpackDuba/Gitnuro); [releases](https://github.com/JetpackDuba/Gitnuro/releases)
- QGit: drag-n-drop коммитов между двумя окнами qgit (перенос патчей), GUI для StGIT patch stacks — [README](https://github.com/tibirna/qgit)
- Ungit: браузерный UI, запускаемый на удалённой/headless машине — [README](https://github.com/FredrikNoren/ungit)
- GitWand: merge preview до merge; auto-resolve тривиальных конфликтов (паттерн `same_change` и др.); «Intelligence panel» — предсказание конфликтов через `git merge-tree`, hotspot-анализ; кросс-репо inbox «Today» — [README](https://github.com/devlint/GitWand)
- AngKorGit: «Escape closes exactly one thing at a time», shortcuts напечатаны рядом с пунктами меню, per-repo identity profiles, сознательный отказ от in-app PR review/телеметрии/маркетплейса — [README](https://github.com/cheat2001/angkorgit)
- GitDesktop: offline «local PRs» между любыми двумя ветками с промоутом в реальный PR; один переключатель скрывает весь AI — [README](https://github.com/theBGuy/GitDesktop)
- GitQuill: WIP через ветки вместо stash; редактирование файлов в diff; кнопки произвольных git-команд — [README](https://github.com/adamsol/GitQuill)
- SourceGit: image diff в режимах side-by-side/swipe/blend; встроенный помощник conventional commits; issue links; workspaces — [README](https://github.com/sourcegit-scm/sourcegit)
- GitCat: «Safety Manager» делает snapshot перед каждой мутацией; flow-git: «48h Safe Discard» — [GitCat](https://github.com/zangjiucheng/GitCat); [flow-git](https://github.com/longgoll/flow-git)

### Inferences
- Для GitStream с его философией «никаких тихих деструктивных действий» (Sync Assistant + backup-ref) естественное продолжение — общий oplog/undo (по модели GitButler: snapshot в refs/ODB перед каждой мутацией) и «safe discard» (по модели flow-git/GitCat) — это сегодня видимый тренд у новых Tauri-клиентов.
- Merge preview через `git merge-tree` (GitWand) хорошо ложится на CLI-архитектуру GitStream (query.rs, без мутаций) и на SmartGit-паритет.
- Command palette (⌘K/Ctrl+K) стала де-факто стандартом у новых клиентов (AngKorGit, Graft, rgitui); у классических (Git Cola, gitg, Gittyup, Gitnuro) её нет.
- AI в 2026 — почти обязательная галочка у новых клиентов, но с трендом «opt-in / один выключатель / через уже установленный CLI агента» (AngKorGit, GitDesktop), а не собственный облачный аккаунт.

### Gaps
- Не найдено первоисточника о конкретной реализации «session checkpoints» в Git Extensions 7.2 (упомянуто только как пункт меню в known issues).
- Не проверено, есть ли у Gitnuro/Gittyup/Git Cola command palette — в источниках не упоминается (отмечено как отсутствующее по умолчанию в inference, а не как факт).

## Активность, статус поддержки, лицензии

### Takeaway
Активно развиваются (релиз в 2026): Git Extensions 7.2.1, Git Cola 4.19.0, gitg 50, GitButler 0.22.3, Gitnuro 2.0 beta, Guitar 1.4.1, Ungit 1.5.30, RelaGit 0.16.11, SourceGit v2026.21, GitFiend 1.0.0-preview, плюс новые Tauri-проекты. Фактически заморожены: GitAhead (maintenance-only, последний релиз 2023-12) и m1guelpf/commit (2023). QGit — медленный, но живой (2.13, 2025-10). Gittyup — стабильный 2.0.0 (2025-11) + rolling dev-сборки.

### Cited Findings
- **Git Extensions**: v7.2.1 (2026-08-18), push 2026-10-09, ~8.6k★, лицензия GPL (LICENSE.md — GNU GPL; API показывает NOASSERTION) — [releases](https://github.com/gitextensions/gitextensions/releases); [LICENSE](https://github.com/gitextensions/gitextensions/blob/master/LICENSE.md)
- **Git Cola**: v4.19.0 (2026-07-12), минимальный Python 3.9 с v4.18.0, GPL-2.0, ~2.6k★, основной хостинг на GitLab — [releases](https://github.com/git-cola/git-cola/releases)
- **gitg**: тег v50 (2026-07-01), предыдущий v44 (2023-08-03); в master уже версия 51.alpha; GPL-2.0; GitHub — read-only зеркало GitLab GNOME — [tags](https://gitlab.gnome.org/GNOME/gitg/-/tags); [meson.build](https://gitlab.gnome.org/GNOME/gitg/-/blob/master/meson.build); [mirror](https://github.com/GNOME/gitg)
- **GitAhead**: последний релиз v2.7.1 (2023-12-20); последние коммиты — мелкие фиксы (2025-09 совместимость с macOS Tahoe, 2026-08 «Fix warning»); README: «no longer under active development»; MIT; репозиторий формально НЕ архивирован (archived=false по API) — [repo](https://github.com/gitahead/gitahead); [releases](https://github.com/gitahead/gitahead/releases)
- **Gittyup**: стабильный gittyup_v2.0.0 (2025-11-30), v1.4.0 (2024-05-14); rolling «development» (2026-03-28); push 2026-10-05; MIT; ~2.3k★ — [releases](https://github.com/Murmele/Gittyup/releases)
- **GitButler**: release/0.22.3 (2026-08-29), push 2026-10-11, ~21.8k★; лицензия FSL-1.1-MIT (Functional Source License с переходом в MIT) — т.е. fair-source, не OSI open source — [releases](https://github.com/gitbutlerapp/gitbutler/releases); [LICENSE](https://github.com/gitbutlerapp/gitbutler/blob/master/LICENSE.md)
- **Gitnuro**: 2.0.0-beta04 (2026-10-08), план 4 беты до RC; GPL-3.0; ~2.8k★ — [releases](https://github.com/JetpackDuba/Gitnuro/releases)
- **Guitar**: v1.4.1 (2026-07-09), GPL-2.0, ~1.7k★; mac-версии не будет, CI и nightly прекращены (2026-04-07), AppImage планируется прекратить, автор ищет мейнтейнера (2026-07-02) — [README](https://github.com/soramimi/Guitar); [releases](https://github.com/soramimi/Guitar/releases)
- **QGit**: qgit-2.13 (2025-10-15), push 2026-09-07, ~214★; баги принимаются через git@vger.kernel.org — [releases](https://github.com/tibirna/qgit/releases); [README](https://github.com/tibirna/qgit)
- **Ungit**: v1.5.30 (2026-03-03), MIT, ~10.6k★ — [releases](https://github.com/FredrikNoren/ungit/releases)
- **RelaGit**: v0.16.11 (2026-04-30), до этого v0.16.9 (2025-03) — релизы в основном dependabot-апдейты; LGPL-3.0; «early beta» — [releases](https://github.com/relagit/relagit/releases)
- **GitFiend**: v1.0.0-preview (2026-09-27), v0.46.4 (2025-03-15); публичный репо GitFiend/Support — «root repo» c issues; frontend минифицирован; репозиторий gitfiend-core по прямому raw-URL отдаёт 404 (возможно, переименован/закрыт) — [releases](https://github.com/GitFiend/Support/releases); [Support README](https://github.com/GitFiend/Support)
- **SourceGit**: v2026.21 (2026-09-21), MIT, ~6.1k★ — [repo](https://github.com/sourcegit-scm/sourcegit)
- **gg (jj)**: v0.45.0 (2026-09-23), Apache-2.0, ~865★ — [repo](https://github.com/gulbanana/gg)
- Для референса TUI: lazygit v0.66.0 (2026-10-05, ~83k★), gitui v0.28.1 (2026-03-24, ~22.5k★) — [lazygit](https://github.com/jesseduffield/lazygit); [gitui](https://github.com/gitui-org/gitui)
- Новые Tauri-клиенты по поиску GitHub (created > 2024): GitWand 181★ (MIT), GitDesktop 234★ (Apache-2.0), AngKorGit 145★ (MIT), gitru 286★ (GPL-3.0), Graft 67★ (MIT, последний push 2025-12), rgitui 262★ (MIT, GPUI) — [GitHub search](https://github.com/search?q=git+client+tauri+created%3A%3E2024-01-01&type=repositories&s=stars)

### Inferences
- «Abandoned/фактически заморожены»: GitAhead (рекомендует форк Gittyup), m1guelpf/commit, Graft (нет push с 2025-12 — ранний признак остановки). Под риском: Guitar (один автор, сокращает платформы), RelaGit (активность только через зависимости), GitFiend (закрытость кода фронтенда, частичный open source).
- GitButler — не «free open source» в строгом смысле (FSL), это важно для честной сравнительной матрицы.

### Gaps
- Не удалось получить лицензию QGit (COPYING по raw-URL 404; API показывает NOASSERTION) — исторически GPL-2.0, но в этой сессии не подтверждено.
- Не установлена последняя стабильная версия Gitnuro 1.x (API вернул только беты 2.0; в notes упомянута «v1.5»).
- Лицензия GitFiend (бесплатный, но open-source ли полностью) — не подтверждена.

## Технологический стек и производительность

### Takeaway
Стек распределяется так: нативный C++/Qt (Gittyup, GitAhead, Guitar, QGit), Python/Qt (Git Cola), Vala/GTK3 (gitg), .NET WinForms (Git Extensions, только Windows), .NET Avalonia (SourceGit), Kotlin/Compose+JGit (Gitnuro), Electron (GitHub Desktop, RelaGit, GitFiend, Ungit-сборки, GitQuill), Tauri (GitButler, GitWand, AngKorGit, Graft, GitDesktop, gg, gitnado), GPUI (rgitui). Новые клиенты почти все Tauri 2, при этом делятся на «system git CLI» (GitWand, GitDesktop, как GitStream) и «libgit2/gix в Rust» (AngKorGit, gigit).

### Cited Findings
- Gitnuro: Compose + JGit; JAR 2.0 требует Java 25; планируется native-сборка без Java; 2.0 снижает RAM и ускоряет diff; watcher по директориям для обхода лимита inotify на Linux — [README](https://github.com/JetpackDuba/Gitnuro); [releases](https://github.com/JetpackDuba/Gitnuro/releases)
- Gittyup: C++/Qt ≥6.4, libgit2; v2.0.0 — mmap-based indexing — [README](https://github.com/Murmele/Gittyup); [Gittyup site](https://murmele.github.io/Gittyup/)
- gitg: Vala, зависимость libhandy-1 (GTK3-эпоха), glib ≥2.68 — [meson.build](https://gitlab.gnome.org/GNOME/gitg/-/blob/master/meson.build); язык Vala — [mirror](https://github.com/GNOME/gitg)
- Git Cola: Python ≥3.9 + QtPy (PyQt5/PyQt6/PySide2); Turbo mode для больших репо — [README](https://github.com/git-cola/git-cola); [docs](https://git-cola.readthedocs.io/en/latest/git-cola.html)
- Git Extensions: C# 14 / .NET 10 / Visual Studio 2026, Windows 10+ — [README](https://github.com/gitextensions/gitextensions)
- QGit: Qt5 ≥5.11; загрузка `git log` через временный файл (быстрее всего на tmpfs), fallback на QProcess — [README](https://github.com/tibirna/qgit)
- Guitar: C++20, Qt 6, «fast, lightweight» — [README](https://github.com/soramimi/Guitar)
- GitButler: Tauri + Svelte/TypeScript, Rust backend; тот же движок в CLI `but` — [README](https://github.com/gitbutlerapp/gitbutler)
- GitFiend: Electron ^39 + Rust core — [gitfiend-shell](https://github.com/GitFiend/gitfiend-shell); [Support README](https://github.com/GitFiend/Support)
- RelaGit: Electron (обновлён до 39), Solid.js, git CLI (обсуждается libgit2) — [releases](https://github.com/relagit/relagit/releases); [README](https://github.com/relagit/relagit)
- Ungit: Node.js-сервер + браузер; Electron-пакеты опционально — [README](https://github.com/FredrikNoren/ungit)
- SourceGit: C#, кроссплатформенный (Linux тестируется только на Debian 12 X11/Wayland); требует Git ≥2.25.1; MSYS Git на Windows не поддерживается — [README](https://github.com/sourcegit-scm/sourcegit)
- GitWand: Tauri 2 + Vue 3, система git; граф — SVG DAG с trunk-pinning; на Linux dev-режим требует `WEBKIT_DISABLE_COMPOSITING_MODE=1` — [README](https://github.com/devlint/GitWand)
- AngKorGit: Tauri v2, Rust + libgit2, системный webview, виртуализированный граф на 100k коммитов, auto-update с проверкой подписи — [README](https://github.com/cheat2001/angkorgit)
- rgitui: GPUI (UI-фреймворк Zed) — GPU-рендеринг 60fps; граф с кривыми Безье; syntect для подсветки — [README](https://github.com/noahbclarkson/rgitui)
- GitQuill: Electron + Monaco; без file watching — refresh по фокусу окна — [README](https://github.com/adamsol/GitQuill)
- Graft: Tauri + React + Monaco, «sub-second startup», граф 10k+ коммитов — [README](https://github.com/Dancode-188/graft)

### Inferences
- По стеку GitStream (Vue 3 + Tauri 2 + system git CLI) ближайшие аналоги — GitWand (идентичный стек) и GitWave (Tauri 2 + Vue 3, spawn git); GitButler и AngKorGit идут через Rust-библиотеки (libgit2/gix), что даёт меньше накладных расходов на spawn, но хуже совместимость с git-конфигурацией (hooks, credential helpers) — GitStream здесь выигрывает за счёт CLI + askpass.
- Типичные заявленные перф-приёмы конкурентов, применимые к GitStream: виртуализация графа (AngKorGit, Graft), canvas/GPU-рендер графа (flow-git, rgitui), watcher с учётом .gitignore (Gitnuro 2.0, Ungit), refresh по фокусу вместо watch (GitQuill), mmap-индексация (Gittyup).
- Linux/WebKitGTK-специфика (флаг compositing у GitWand) — общий риск всех Tauri-клиентов на Linux, включая GitStream.

### Gaps
- Количественных бенчмарков (RAM/время старта/скорость на больших репо) в первоисточниках нет — только качественные заявления авторов («fast», «sub-second», «60 fps»); независимых замеров не найдено.
- Не подтверждено, переведён ли gitg на GTK4 в ветке 51.alpha (meson.build показывает libhandy-1 → GTK3).
- Стек нового UI GitFiend 1.0.0-preview в release notes не указан (shell на npm-версии 0.46.5 — Electron).
