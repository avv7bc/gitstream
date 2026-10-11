# Клавиатурные/терминальные Git-клиенты и новые UX-концепции (lazygit, gitui, tig, Magit, jj, GitLens, JetBrains, тренды 2025–2026)

Контекст для автора отчёта: в GitStream уже есть частичный stage строк/хунков, окно Git output (лог git-команд), squash, reword, reset/revert/cherry-pick из контекстного меню, blame, file history, worktree-подобного нет, undo нет, интерактивного rebase UI нет, 3-way merge нет, AI нет (по CLAUDE.md проекта). В разделах «Inferences» отмечено, что из найденного можно перенести в GitStream.

## 1. lazygit: какие механизмы делают его быстрым и любимым

### Takeaway
Сила lazygit в том, что каждая операция переписывания истории (squash/fixup/drop/move/amend старого коммита, удаление хунка из старого коммита) делается одной клавишей на выбранном коммите. Интерактивный rebase выполняется в фоне, а пользователь его не видит. Вместе с undo через reflog, copy/paste для cherry-pick, «mark as base» и видимым логом команд это делает рискованные операции дешёвыми.

### Cited Findings
- **Однокнопочная правка истории в панели Commits**: `s` squash (слить с коммитом ниже, оба сообщения сохраняются), `f` fixup (сообщение отбрасывается), `d` drop, `e` edit, `r` reword, `ctrl+j`/`ctrl+k` (`alt+down/up`) сдвиг коммита вниз/вверх, `t` revert, `g` меню reset (soft/mixed/hard), `T` тег, `b` меню bisect, `o` открыть коммит в браузере, `a` сменить автора или добавить co-author — [Keybindings_en.md](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Работает без явного старта rebase**: в README сказано, что эти действия доступны как разовые операции над одним коммитом, без предварительного `i`. Внутри интерактивного rebase TODO-список редактируется теми же клавишами, `m` открывает меню continue/abort, `shift+down` выделяет диапазон коммитов, чтобы двигать или fixup-ить их вместе — [lazygit README](https://github.com/jesseduffield/lazygit)
- **Amend старого коммита**: `A` (shift+a) на любом коммите вливает в него staged-изменения. «This runs an interactive rebase in the background» — [lazygit README](https://github.com/jesseduffield/lazygit); в keybindings: «Amends the commit with staged changes (uses rebase if not HEAD)» — [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Fixup-поток**: `F` создаёт коммит `fixup!` для выбранного коммита, `S` применяет все `fixup!`-коммиты выше выбранного (autosquash), `c` задаёт, какое сообщение оставить при fixup — [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Rebase onto marked base**: `B` помечает base-коммит, затем `r` на целевой ветке переносит только коммиты после base (`git rebase --onto`), `shift+p` пушит результат — [README](https://github.com/jesseduffield/lazygit), [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Cherry-pick как copy/paste**: `C` копирует коммит(ы), `V` вставляет их в текущую ветку, `ctrl+r` сбрасывает «буфер» скопированных коммитов — [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Undo/redo**: `z` / `Z`. «Undo is based on the reflog, so it covers commits and branches». Undo не откатывает изменения working tree и stash — [README](https://github.com/jesseduffield/lazygit), [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Построчный stage**: `<space>` стейджит хунк и сразу переходит к следующему, `a` переключает режим хунк/строка, `v` выделяет диапазон, `<tab>` переключает staged/unstaged-панели, `c` коммитит прямо из diff, `E` открывает хунк во внешнем редакторе, `n/N` переход к следующему/предыдущему файлу, `d` в staging discard-ит (в staged-панели unstage-ит) — [README](https://github.com/jesseduffield/lazygit), [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Custom patches (сборка патча из коммита)**: в diff коммита `<space>` добавляет хунк или строку в custom patch, `ctrl+p` открывает меню патча (например, вынести патч в новый коммит). `d` на хунке в diff старого коммита удаляет хунк из этого коммита, переписывая историю через rebase в фоне — [README](https://github.com/jesseduffield/lazygit)
- **Compare two refs**: `W` (shift+w) помечает коммит, ветку или ref. Выбор другого коммита показывает diff между ними, `<enter>` даёт список файлов, есть реверс направления и выход по Esc — [README](https://github.com/jesseduffield/lazygit)
- **Bisect**: `b` в коммитах помечает good/bad и запускает `git bisect` — [README](https://github.com/jesseduffield/lazygit)
- **Worktrees**: `w` в Branches создаёт worktree из ветки и переключается в него («work on several branches without stashing»). Отдельная панель Worktrees: new / switch / open in editor / remove / filter — [README](https://github.com/jesseduffield/lazygit), [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **Фильтр**: `/` фильтрует текущий список текстом, `ctrl+s` открывает опции фильтра лога (например, по пути или автору) — [Keybindings](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- **«Nuke» working tree**: `D` открывает меню reset, где есть вариант «nuke», который удаляет всё, что видно в `git status`, включая dirty submodules — [README](https://github.com/jesseduffield/lazygit)
- **Граф**: в увеличенном режиме цвет линий соответствует автору, родители выбранного коммита подсвечиваются — [README](https://github.com/jesseduffield/lazygit)
- **PR-индикаторы**: в панели Branches у ветки с PR есть иконка цвета состояния PR, `G` открывает PR в браузере (через `gh`) — [README](https://github.com/jesseduffield/lazygit)
- **Custom commands**: в YAML задаются `key`, `context` (files/commits/localBranches/global…), `command` (Go-шаблоны), `prompts` (`input` с `suggestions`, `confirm`, `menu`, `menuFromCommand` с regex-фильтром, `condition`), `output` (`none`/`terminal`/`log`/`logWithPty`/`popup`), `loadingText`, `after: checkForConflicts`. В шаблоны передаётся текущее выделение: `SelectedCommit`, `SelectedFile`, `SelectedLocalBranch`, `SelectedRemote`, `SelectedStashEntry`, `SelectedWorktree`, `CheckedOutBranch`, экранирование через `| quote`. Команда без `key` доступна через меню `?` — [Custom_Command_Keybindings.md](https://github.com/jesseduffield/lazygit/blob/master/docs/Custom_Command_Keybindings.md)
- **Что пользователи ценят больше всего (по словам автора)**: лог команд, видимый по умолчанию, «one of the things people like best about Lazygit»; он снимает претензию «UI делает что-то неожиданное». Custom commands используют состояние выделения. Исходный мотив автора: нажать `d` на файле, и файл «просто исчезает» независимо от того, untracked он, tracked или staged — [Lazygit 5 Years On (Jesse Duffield)](https://jesseduffield.com/Lazygit-5-Years-On/)
- **Принципы автора**: «state -> action -> new state» (показывать состояние и его изменение), discoverability («UIs make things more discoverable so you don't need to remember as much»), keyboard-centric, производительность, которая «doesn't come for free». Стандартный редактор interactive rebase автор называет «very difficult to use» — [Lazygit 5 Years On](https://jesseduffield.com/Lazygit-5-Years-On/)

### Inferences
- В GitStream уже есть аналог «command log» (окно Git output) и частичный stage, то есть два самых любимых механизма lazygit. Главного разрыва это не закрывает. Разрыв в **однокнопочной правке истории из контекстного меню графа**: Fixup into / Drop / Move up/down / Amend into this commit / Remove hunk from commit. Всё это реализуется через `git rebase -i` со сценарным `GIT_SEQUENCE_EDITOR`, а он в GitStream уже есть для reword не-HEAD коммита.
- Undo через reflog переносится напрямую: хранить «до/после» для HEAD и затронутых refs по каждой мутации GitStream. Это проще и надёжнее, чем парсить reflog, и похоже на подход jj (раздел 4). Backup-ref `refs/gitstream/backup/<branch>` в Sync Assistant уже является зачатком этого.
- «Copy/Paste» cherry-pick (Ctrl+C на коммитах → Ctrl+V на ветке) и «Mark as base → rebase onto» дают дешёвые выигрыши в рамках существующих мутаций.
- Пометить ref и сравнить с другим (`W`) можно сделать как «Compare with marked» в контекстном меню графа и веток. В GitStream уже есть File Compare, его можно расширить до сравнения целых коммитов.
- Custom commands: GitStream может хранить в settings список пользовательских команд с шаблонами `{selectedCommit}`, `{selectedFile}`, `{currentBranch}`, prompt-полями и выводом в Git output.

### Gaps
- Я не проверял changelog lazygit 2025–2026 и не знаю, что добавилось после текущего README (например, bulk-move коммитов, о котором автор писал как о плане).
- Меню custom patch (полный список действий `ctrl+p`) в keybindings-документе не описано, README перечисляет только часть.

## 2. gitui и tig: производительность, blame, поиск по логу

### Takeaway
tig остаётся эталоном **просмотрщика**: blame, refs и main/log/diff/tree views с единым поиском `/` по регулярным выражениям. gitui делает упор на скорость (Rust), но более свежих публичных бенчмарков, чем старый собственный замер автора, я не нашёл.

### Cited Findings
- В tig blame является отдельным first-class view («displays the file content annotated or blamed by commits»), refs view показывает ветки, remotes и теги — [tigmanual(7), Arch](https://man.archlinux.org/man/tigmanual.7.en)
- В tig `/` ищет контекстно: по сообщениям коммитов в main/log views, по коду в diff, по именам файлов в tree, по коду в blame. Поддерживаются regex — [tigmanual(7)](https://man.archlinux.org/man/tigmanual.7.en)
- tig задуман как просмотрщик: сложных операций с ветками и push в нём нет — [terminal.guide: tig](https://terminal.guide/tools/git-tool/tig)
- gitui позиционируется как «fast and intuitive keyboard-only control» с контекстной помощью. В старом собственном бенчмарке автора на парсинге всего репозитория Linux gitui занял 24 с, lazygit 57 с, tig 4 мин 20 с. Это старый самозамер, его стоит считать только ориентиром — [docs.rs gitui 0.16.2](https://docs.rs/gitui/0.16.2), [docs.rs gitui 0.14.0](https://docs.rs/gitui/0.14.0)
- В ранних документах gitui file blame (#484) и log search (#449, #429) значились в roadmap к 1.0 — [docs.rs gitui 0.14.0](https://docs.rs/gitui/0.14.0)

### Inferences
- Для GitStream перенести стоит **единый контекстный поиск/фильтр по `/` или Ctrl+F в каждой панели**: в графе по сообщениям, в diff по тексту, в blame по коду, в дереве файлов по имени. Сейчас фильтр есть в логе и списках веток, но не в diff и blame.
- Главный урок производительности: читать граф потоково и виртуализировать (в GitStream это уже есть). Отдельно стоит измерять загрузку больших репозиториев (уровня linux.git).

### Gaps
- Я не нашёл актуальных (2025–2026) данных о том, вошли ли blame и поиск по логу в стабильный gitui и какова его скорость сейчас. Нужен changelog gitui.

## 3. Magit: transient-меню, последовательности операций, instant fixup, spin-off

### Takeaway
Magit ценят за две вещи. Первая: **transient-меню**, где префикс-клавиша показывает все подкоманды и переключаемые аргументы, и это одновременно обучение и скорость. Вторая: составные «умные» операции в одно действие. Instant fixup сразу делает autosquash в нужный коммит, а spin-off уносит неотправленные коммиты в новую ветку и откатывает исходную.

### Cited Findings
- **Instant fixup** `c F` (`magit-commit-instant-fixup`) создаёт fixup-коммит и сразу вливает его в целевой коммит. Целевым становится коммит под курсором, иначе запрашивается. Под капотом выполняются `git commit --fixup=COMMIT --no-edit` и `git rebase --autosquash MERGE-BASE`. Цель может быть любой коммит, достижимый из HEAD, сообщение целевого коммита не меняется. `c S` (instant squash) работает так же, но просит автора написать итоговое сообщение — [Magit manual: Editing any reachable commit and rebasing immediately](https://docs.magit.vc/magit/Editing-any-reachable-commit-and-rebasing-immediately.html), [docstring magit-commit-instant-fixup](https://doc.emacsen.de/29.4/fun/magit-commit-instant-fixup.html)
- **Spin-off** `b s`: «This command creates and checks out a new branch starting at and tracking the current branch». Затем исходная ветка сбрасывается на последний коммит, общий с её upstream. Если upstream или неотправленных коммитов нет, создаётся только новая ветка — [Magit manual: Branching](https://docs.magit.vc/magit/Branching.html)
- **Spin-out** `b S`: то же, что spin-off, но текущая ветка не переключается — [Magit manual: Branching](https://docs.magit.vc/magit/Branching.html)
- **Branch reset** `b x` сбрасывает ветку (по умолчанию ветку под курсором) на другую ветку или коммит. Для текущей ветки это hard reset с подтверждением при наличии изменений — [Magit manual: Branching](https://docs.magit.vc/magit/Branching.html)
- **Transient-меню**: `b` открывает префикс `magit-branch`, временный буфер со всеми подкомандами. По умолчанию там же видны связанные git-переменные (например, upstream и pushRemote), и их можно менять прямо из меню (`magit-branch-direct-configure`) — [Magit manual: Branching](https://docs.magit.vc/magit/Branching.html)
- **Популярность**: по данным автора lazygit, у Magit меньше звёзд на GitHub (6.1k), но больше 3.8M загрузок, что «blows Lazygit out of the water» — [Lazygit 5 Years On](https://jesseduffield.com/Lazygit-5-Years-On/)
- Git сам умеет autosquash: `rebase.autoSquash = true`. Fixup сопоставляется с целью по subject или хэшу. Если в стеке два коммита с одинаковым subject, fixup уйдёт в самый старый — [поиск/обзор по autosquash, обсуждение git-absorb](https://github.com/tummychow/git-absorb/pull/5)

### Inferences
- Аналог transient для GUI: **командная палитра / which-key-панель**. После префикс-клавиши (например, `B` = Branch) показывается оверлей со всеми подкомандами, их клавишами и тогглами флагов (`--force-with-lease`, `--rebase`, `--include-untracked`). Это решает проблему discoverability горячих клавиш GitStream. Похожую which-key-строку статуса добавил jjui v0.9.11 (раздел 4).
- «Instant fixup» хорошо ложится на GitStream: контекстное меню коммита → «Fixup staged into this commit». Это `commit --fixup` + `rebase -i --autosquash` с неинтерактивным `GIT_SEQUENCE_EDITOR=true`.
- «Spin-off» стоит дать как пункт меню ветки «Move unpushed commits to new branch…». Это частый сценарий «закоммитил в main вместо feature».

### Gaps
- Описание infix-аргументов transient (например, `-f`, `--autostash` в меню rebase) я не получил: страница manual Rebasing вернула 404.

## 4. Jujutsu (jj) и его GUI: operation log, конфликты как объекты, авто-rebase потомков

### Takeaway
Три ключевые идеи jj: (1) **operation log**, то есть атомарный снимок всех refs и working copy после каждой операции, а значит undo/restore для чего угодно; (2) **конфликты хранятся в коммитах**, и rebase никогда не «останавливается»; (3) **автоматический rebase потомков** при переписывании коммита. GUI вроде gg превращают это в UX «вы всегда как будто посреди interactive rebase, но это хорошо», где drag-and-drop означает rebase/squash. На чистом git полностью можно повторить (1), частично (3) через `--update-refs`/autosquash. (2) повторяется только как UX-«отложенный конфликт», а не как модель данных.

### Cited Findings
- **Working copy является коммитом**, индекса нет: «The working copy is automatically committed». Роль staging выполняют `jj split` (разделить коммит) и `jj squash -i` (интерактивно перенести часть изменений в родителя) — [jj docs: Git comparison](https://docs.jj-vcs.dev/latest/git-comparison/)
- **Operation log** заменяет reflog и фиксирует атомарные обновления всех refs сразу. «The operation log powers e.g. the undo functionality» — [Git comparison](https://docs.jj-vcs.dev/latest/git-comparison/)
- Каждая операция хранит «view»: bookmarks, теги, git-refs, heads и working-copy commit каждого workspace, а также метаданные (время, пользователь, хост, описание). Есть `jj undo` (по шагу), `jj op restore` («restore the entire repo to the way it looked at an earlier point») и `--at-op` для read-only просмотра прошлого состояния. Операции lock-free: конкурентные правки дают «divergent» change, а не порчу репозитория — [jj docs: Operation log](https://docs.jj-vcs.dev/latest/operation-log/)
- **Конфликты first-class**: «Conflicts can be committed». В коммите хранится «a logical representation of the conflict, not conflict markers». Rebase с конфликтом завершается успешно, конфликт живёт в коммите, его можно отложить, а коммит с ним можно снова rebase-ить или merge-ить без вложенных маркеров — [jj docs: Conflicts](https://docs.jj-vcs.dev/latest/conflicts/), [Git comparison](https://docs.jj-vcs.dev/latest/git-comparison/)
- **Diff-style маркеры**: блок `+++++++` содержит снимок одной стороны, блок `%%%%%%%` содержит diff, который нужно применить к снимку. Есть `ui.conflict-marker-style` (snapshot или git diff3) — [jj docs: Conflicts](https://docs.jj-vcs.dev/latest/conflicts/)
- **Auto-rebase**: при переписывании коммита «all its descendants commits will automatically be rebased on top», bookmarks и working copy обновляются. Анонимные heads являются нормой («This is the normal state in Jujutsu»), все видимые heads отслеживаются, поэтому коммиты не теряются. «Текущей ветки» нет, bookmarks двигаются вручную — [Git comparison](https://docs.jj-vcs.dev/latest/git-comparison/)
- **gg (GUI for JJ)**: слева лог с запросом (revset), справа редактор выбранной ревизии (описание, родители, изменения). Принцип: «what if you were always in the middle of an interactive rebase, but this was actually a good thing?». Drag ревизии означает rebase, drop в родителей или из них добавляет merge или переносит поддерево. Drag файла на ревизию squash-ит его туда, на «родителя» откатывает его. Drag bookmark ставит или удаляет его. Двойной клик по изменяемой ревизии редактирует её, по неизменяемой создаёт дочернюю (иммутабельность задаётся `immutable_heads()`). Кнопка undo ⟲ в углу работает через op log («in theory... can't corrupt a repository»). Конфликты разрешаются во внешнем инструменте. На больших репозиториях часть функций отключается — [gg README](https://github.com/gulbanana/gg)
- **jjui** (TUI для jj) v0.9.11: почти полностью переписанная система представлений, большая часть UI кликабельна мышью, which-key-подобная строка статуса и **интерактивный target picker** для rebase/squash. Требует jj ≥ 0.37 — [newreleases: jjui v0.9.11](https://newreleases.io/project/github/idursun/jjui/release/v0.9.11)
- **VisualJJ** — закрытое (не open-source) расширение VS Code с нативной интеграцией jj без git-colocation. **Jujutsu Kaizen (jjk)** — MIT-альтернатива, повторяет функции Source Control view VS Code и умеет undo операций jj — [jj community tools](https://git.joshthomas.dev/mirrors/jj/src/branch/main/docs/community_tools.md)
- Автор lazygit называет jj «reboot of git», который может решить проблемы merge-конфликтов и undo — [Lazygit 5 Years On](https://jesseduffield.com/Lazygit-5-Years-On/)
- **Git-аналог авто-rebase стеков**: `git rebase --update-refs` (Git 2.38+) автоматически двигает все ветки, указывающие на переписываемые коммиты. В todo появляются строки `update-ref` — [adamj.eu: --update-refs](https://adamj.eu/tech/2022/10/15/how-to-rebase-stacked-git-branches/), [Viget](https://viget.com/articles/the-new-git-option-for-rebasing-multiple-branches-at-once)
- **git absorb** (порт hg absorb) сопоставляет каждый staged-хунк с коммитом в истории ветки и создаёт `fixup!`-коммиты, `--and-rebase` сразу делает autosquash. Ограничения: работает только со staged-изменениями, не умеет хунки, затрагивающие несколько исходных коммитов, и новый код без изменения существующих строк. По умолчанию просматривает 10 коммитов (`absorb.maxStack`) — [git-absorb man](https://www.mankier.com/1/git-absorb)

### Inferences
- **Operation log для git-GUI**: перед каждой мутацией GitStream снимать `git for-each-ref` + HEAD + (опционально) `git stash create` для working tree и писать запись в собственный журнал (`.git/gitstream/oplog`). Undo восстанавливает refs через `update-ref`, а при наличии snapshot-а и worktree. Это больше, чем reflog-undo lazygit, потому что откатывается ещё и working tree, и стоит сделать отдельную панель «Операции» с «Restore to here». Fetch и refs remote тоже можно включать, как в view jj.
- **Авто-rebase потомков** на git: все операции правки истории (amend старого коммита, drop, reword) выполнять с `--update-refs`, чтобы стек веток сверху не «отрывался». Это прямо полезно для stacked-PR (раздел 7).
- **«Absorb»** как кнопка в панели файлов: «Распределить staged-изменения по коммитам ветки» (своя реализация через blame строк или вызов git-absorb, если он установлен).
- **Drag-and-drop как в gg** естественно ложится на GitStream, где DnD уже есть в Repositories: коммит на коммит = rebase/fixup, файл из списка файлов коммита на другой коммит = перенос изменения, ветка-ярлык на коммит = reset/move branch (с подтверждением и undo).
- Конфликты как объекты в git воспроизвести нельзя. Достижимый аналог: не блокировать UI при конфликте, показывать «очередь» конфликтующих файлов и давать вернуться к ним позже (ConflictBar уже частично это делает), а также опцию стиля маркеров diff3/zdiff3.

### Gaps
- Документацию по change IDs, revsets и `jj absorb` я напрямую не получил (страница Git comparison их не покрывает).
- Собственной документации VisualJJ с перечнем функций не нашёл, только каталог и сторонний листинг.

## 5. VS Code и GitLens (бесплатные функции)

### Takeaway
Бесплатное ядро GitLens — это информация «в месте чтения кода»: inline-blame текущей строки, hover с деталями коммита и PR, CodeLens, файловые аннотации (blame/changes/heatmap), навигация по ревизиям файла, история файла и строки, а также Interactive Rebase Editor с drag-reorder. Commit Graph, Visual History и Worktrees бесплатны только на публичных репозиториях, Launchpad и AI платные.

### Cited Findings
- **Community (бесплатно, «on by default»)**: inline и status bar blame («unobtrusive, line-by-line authorship at the end of the current line»), rich hovers (детали коммита, связанные issues/PR, быстрые действия), Git CodeLens, file annotations («whole-file blame, recent changes, and a heatmap»), revision navigation (шаг вперёд/назад по истории файла), File and Line History «across renames and merges», Git Command Palette (пошаговый мастер git-команд) — [GitLens README](https://cdn.jsdelivr.net/gh/gitkraken/vscode-gitlens@main/README.md)
- **Interactive Rebase Editor (Community)**: «lets you drag commits to reorder them» и предупреждает о вероятных конфликтах до запуска rebase — [GitLens README](https://cdn.jsdelivr.net/gh/gitkraken/vscode-gitlens@main/README.md)
- **Commit Graph, Visual History, Worktrees** бесплатны на публичных репозиториях (нужен бесплатный аккаунт GitKraken), на приватных требуют Pro. У worktree в графе своя строка working changes — [GitLens README](https://cdn.jsdelivr.net/gh/gitkraken/vscode-gitlens@main/README.md)
- **Launchpad (Pro)**: «puts every pull request you're involved in on one list, ordered by what's actually blocking someone» — [GitLens README](https://cdn.jsdelivr.net/gh/gitkraken/vscode-gitlens@main/README.md)
- **AI (Pro)**: Generate Commit Message (также для stash и PR), Explain Changes (коммит, stash, ветка или рабочие изменения «in plain English»), Compose Commits (разложить изменения на атомарные коммиты), Automatic Rebase (rebase с разрешением конфликтов ИИ и показом уверенности). Также Agent sessions и Agent Kanban (experimental) — [GitLens README](https://cdn.jsdelivr.net/gh/gitkraken/vscode-gitlens@main/README.md)
- Тарификация: Pro примерно $8 на пользователя в месяц (по странице сравнения GitKraken). Сторонний листинг сообщает о «Version 19 (August 2026)», где Commit Graph перестроен под agent-review. Это **не подтверждено** первоисточником GitKraken — [GitKraken pro-features](https://gitkraken.com/gitlens/pro-features), [сводка поиска](https://www.gitkraken.com/answers/compare-the-branch-management-capabilities-of-github-desktop-vs-gitkraken-desktop)
- **Встроенный VS Code + Copilot**: кнопка-«искорка» в Source Control генерирует заголовок и описание коммита или PR (не experimental). «Resolve Merge Conflict with AI» — **experimental**: открывает Chat с агентным потоком, в контекст передаются merge base и изменения обеих сторон. Code review изменений PR через «Code Review» в Files Changed (нужно расширение GitHub PR) — [VS Code: AI smart actions](https://code.visualstudio.com/docs/copilot/copilot-smart-actions)

### Inferences
- Для GitStream (не редактора) аналог inline-blame — **blame-гуттер прямо в diff-панели и в File History**: hover по строке показывает коммит, автора и дату, клик переходит в граф. Blame-диалог уже есть, его можно сделать режимом diff-панели.
- **Heatmap** (давность изменения строк цветом) — дешёвое улучшение существующего Blame.
- **Drag-reorder в Interactive Rebase** и **предварительная оценка конфликтов** перед запуском (через `git merge-tree --write-tree` для каждого шага) — сильная фича для будущего rebase-диалога GitStream.
- «Git Command Palette» GitLens — ещё один аргумент за палитру команд (Ctrl+Shift+P) в GitStream.

### Gaps
- Актуальную (октябрь 2026) границу free/Pro я подтвердил только по README в ветке main. Утверждение о GitLens 19 не подтверждено.
- Детального описания «Home view» GitLens в README нет.

## 6. JetBrains IDE: changelists/shelves, interactive rebase, 3-way merge, Compare with branch, фильтры лога

### Takeaway
JetBrains силён тремя вещами. Первая — **3-панельный merge** (Local | Result | Theirs) с кнопками «Apply non-conflicting changes» и «Resolve simple conflicts», стрелками и крестиками у каждого чанка и списком файлов конфликта. Вторая — **правка истории из лога** (Fixup / Squash Into / Drop / Edit message / Extract changes в отдельный коммит) и диалог interactive rebase со стрелками перемещения и «Stop to Edit». Третья — changelists и shelves как гибкий частичный stash.

### Cited Findings
- **Conflicts dialog** появляется сам при конфликте: Accept Yours / Accept Theirs / Resolve Manually по каждому файлу — [JetBrains: Resolve conflicts](https://www.jetbrains.com/help/idea/resolve-conflicts.html)
- **Merge tool**: слева read-only local, справа read-only repository, в центре редактируемый результат, изначально равный base. Кнопка «Apply non-conflicting changes» (все сразу, только слева или только справа). У каждого конфликта действия Accept и Ignore, в контекстном меню «Resolve using Left/Right». Кнопка «Resolve simple conflicts» одним кликом сливает простые конфликты (например, правки начала и конца одной строки), которые bulk-действие пропускает. Меню сравнения Base/Local/Repository/Result. «Apply Changes» переводит к следующему файлу, «Save and Close» возвращает к списку, в списке есть «Revert conflict resolution», «Accept and Finish». Опция «Automatically apply non-conflicting changes» — [JetBrains: Resolve conflicts](https://www.jetbrains.com/help/idea/resolve-conflicts.html)
- **Правка истории из Log**: Edit Commit Message (F2), Fixup / Squash Into (затем «Commit and Rebase»), Squash Commits для нескольких выбранных, Drop Commit, Extract Selected Changes to Separate Commit (выбрать файлы в Changed Files коммита) — [JetBrains: Edit project history](https://www.jetbrains.com/help/idea/edit-project-history.html)
- **Interactively Rebase from Here**: выбирается самый старый коммит диапазона. Пункт недоступен, если у коммита несколько родителей, если он не в текущей ветке или если он запушен в protected-ветку. По умолчанию Pick, стрелки вверх/вниз меняют порядок, «Stop to Edit» останавливает на коммите, после чего уведомление предлагает continue/abort и действия Revert/Undo/Amend — [JetBrains 2023.2: Edit project history](https://www.jetbrains.com/help/idea/2023.2/edit-project-history.html)
- **Log filters**: по ветке или избранным веткам, пользователю, дате, папке/пути (в multi-root ещё и по корню), опция «Update Branch Filter» синхронизирует фильтр с выбором в панели Branches. Log открывается Alt+9 — [JetBrains: Investigate changes](https://www.jetbrains.com/help/idea/2025.2/investigate-changes.html)
- **Shelves vs stash**: stash создаёт git, shelve — это патчи IDE. При shelve можно выбрать часть изменений, а «stashing involves all uncommitted changes». Shelve Silently (Ctrl+Shift+H) использует имя changelist как имя полки. Полку можно применять многократно. При unshelve выбирается целевой changelist. «Automatically shelve base revision» сохраняет base для 3-way merge при конфликте при unshelve. Stash и shelves можно объединить в одну вкладку — [JetBrains: Shelve changes](https://www.jetbrains.com/help/idea/shelving-and-unshelving-changes.html)

### Inferences
- **3-панельный resolver** — самый ценный для GitStream пункт из JetBrains. Сейчас в GitStream только accept ours/theirs. Минимальная версия: `git merge-file -p --diff3` для base/ours/theirs, три колонки (переиспользовать SideBySideDiffView и useSyncScroll), стрелки «взять чанк слева/справа» и кнопки «Apply all non-conflicting» и «Resolve simple». Это также основа для будущего AI-разрешения.
- **Changelists** можно эмулировать как «именованные группы файлов» в панели файлов (локальное состояние GitStream) с коммитом группы. Shelves сводятся к stash с выбором файлов (`git stash push -- <paths>`), который применяется без удаления.
- «Extract Selected Changes to Separate Commit» — естественное продолжение частичного stage и custom patch lazygit: выделить строки в diff коммита → «Вынести в новый коммит».
- Фильтр лога **по пути** (`git log -- <path>`) и «Update Branch Filter» стоит добавить к существующему клиентскому фильтру GitStream: серверный `git log` с `--author/--since/--until/-- path`, а не только фильтрация загруженных коммитов.

### Gaps
- Страница Log tab (полный список фильтров, Compare with branch, «Show changes to selected files») не загрузилась (timeout). Описание «Compare with Branch» первоисточником не подтверждено.
- Полный список кнопок диалога interactive rebase в 2025–2026 (например, точная метка объединения коммитов «Unite»/«Squash») я не подтвердил.
- AI-merge в JetBrains (JetBrains AI Assistant) в найденной документации merge tool не упоминается, это пробел.

## 7. Тренды 2025–2026: AI-сообщения коммитов, AI-разрешение конфликтов, AI-ревью, stacked PRs, worktrees

### Takeaway
К концу 2026 года AI-генерация сообщений коммитов стала стандартом (VS Code/Copilot, GitHub Desktop 3.6, GitLens Pro). AI-разрешение конфликтов распространяется, но везде подаётся как **предложение для ревью**: experimental в VS Code, «review, accept, or edit» в GitHub Desktop, «показ уверенности» в GitLens. GitHub выпустил нативные stacked PRs (preview с апреля 2026, public preview с 30.07.2026, CLI `gh-stack`). Worktrees стали мейнстримом благодаря параллельным AI-агентам.

### Cited Findings
- **GitHub Desktop 3.6 (26.06.2026)**: меню «Current Worktree» рядом с Current Repository/Branch (switch/create linked worktrees) для работы «across multiple branches at once without repeatedly stashing changes», хорошо сочетается с coding-агентами в отдельных worktree. Генерация сообщений коммитов Copilot учитывает `.github/copilot-instructions.md`, `AGENTS.md` и правила commit metadata из rulesets репозитория, в диалоге коммита есть model picker. AI-merge: Copilot объясняет конфликтующие изменения и предлагает решение, которое можно «review, accept, or edit before completing the merge». Есть BYOK (свой ключ, локальная модель), всё на Copilot SDK — [GitHub Changelog: Desktop 3.6](https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/)
- **VS Code**: генерация сообщения коммита и PR через «искорку» (stable). «Resolve Merge Conflict with AI» (experimental, агентный поток, контекст: merge base и обе стороны) — [VS Code: AI smart actions](https://code.visualstudio.com/docs/copilot/copilot-smart-actions)
- **GitLens Pro AI**: Generate Commit Message, Explain Changes, Compose Commits (разбить изменения на атомарные коммиты), Automatic Rebase с разрешением конфликтов ИИ и показом уверенности, Agent sessions/Agent Kanban — [GitLens README](https://cdn.jsdelivr.net/gh/gitkraken/vscode-gitlens@main/README.md)
- **GitHub Stacked PRs**: private preview в апреле 2026, затем public preview с 30 июля 2026 (по вторичным источникам). PR может базироваться на предыдущем PR. CLI-расширение `gh extension install github/gh-stack`, его sync каскадно rebase-ит весь стек и атомарно force-push-ит ветки. Стек виден на github.com и в мобильном приложении, GitHub Desktop стеки **не поддерживает**. Поддержка merge queue запаздывает. Источники в основном вторичные, даты нужно сверить с changelog GitHub — [InfoQ (апрель 2026)](https://infoq.com/news/2026/04/github-stacked-prs), [Better Stack guide](https://betterstack.com/community/guides/linux/github-stacked/), [privatedevops](https://privatedevops.com/news/github-stacked-pull-requests-public-preview)
- **Механика стеков на чистом git**: `git rebase --update-refs` (Git 2.38+) двигает все промежуточные ветки стека за один rebase — [adamj.eu](https://adamj.eu/tech/2022/10/15/how-to-rebase-stacked-git-branches/)
- **lazygit** показывает статус PR у веток (иконка по состоянию, `G` открывает PR через `gh`) — [lazygit README](https://github.com/jesseduffield/lazygit)
- Сторонний гайд (DeployHQ, 06.03.2026) подчёркивает, что AI-разрешение конфликта — это предложение, а не гарантированное исправление — [DeployHQ](https://www.deployhq.com/git/resolving-merge-conflicts-with-ai)

### Inferences
- Для GitStream разумный порядок: (1) **AI-сообщение коммита** в CommitDialog. Diff staged → LLM. Провайдер выбирается в Settings (BYOK/локальная модель, как в Desktop 3.6). Учитывать `AGENTS.md`/конвенции репо и стиль последних N коммитов. (2) **«Explain commit/diff»** в CommitDetails. (3) **AI-разрешение конфликтов** только поверх собственного 3-way resolver, как предложение в средней колонке с явным accept/edit. Без resolver это небезопасно.
- **Worktrees** сейчас не опция, а ожидаемый базовый функционал (lazygit, GitLens, GitHub Desktop 3.6). В GitStream их естественно показывать как дочерние узлы репозитория в панели Repositories с отдельным статусом.
- **Stacked branches**: даже без интеграции с GitHub API полезно (а) делать все rebase/amend с `--update-refs` и (б) показывать в графе «стек» веток, базирующихся друг на друге, с действием «Rebase stack onto main». Интеграция с `gh-stack` и PR-статусами (как иконки PR в lazygit) — следующий шаг.
- Ни один из найденных инструментов не делает AI-действия «тихими»: везде нужен review и accept. Это согласуется с принципом Sync Assistant в GitStream (никаких тихих деструктивных действий).

### Gaps
- Не нашёл первоисточника (github.blog/changelog) с точной датой public preview stacked PRs. Дата 30.07.2026 взята из вторичных источников.
- Не исследованы AI-функции JetBrains AI Assistant (commit message, merge), SmartGit 2025–2026 и GitKraken Desktop: они вне явного фокуса этой заметки и, вероятно, покрыты другими исследователями.
- Количественных данных о том, как часто пользователи принимают AI-разрешения конфликтов, не найдено.
