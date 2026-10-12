# GitStream 2 — функциональная дорожная карта

Версия 2 — это полный функциональный апгрейд GitStream на основе текущего интерфейса. Ветка `v1` сохраняет стабильный legacy-продукт; визуальный редизайн A «Графит» выполняется отдельно в v3 после приёмки v2.

Источник требований — функциональные разделы `docs/SPEC-redesign-2026.md` и сравнительный отчёт `reports/Сравнение Git клиентов с GitStream.md`. HTML-макеты A не являются критерием приёмки v2.

## Правила линии v2

- Все новые команды и backend-возможности сначала получают тестируемый общий слой, затем legacy UI.
- Destructive-операции всегда имеют snapshot/Undo или явное подтверждение с описанием последствий.
- Версия v2 не меняет визуальную систему целиком; она исправляет поведение, доступность функций и обратную связь.
- Первый prerelease — `v2.0.0-alpha.1`; номер в исходниках поднимается только при подготовке prerelease.

## Этап 0 — ветки и обновления

Текущий `main` переименован в `v1`; `v2` создан от него. Проверка обновлений выбирает релиз внутри текущей major-линии и отдельно сообщает о следующей линии. CI запускается для `v1`, `v2` и будущего `v3`.

**Готово, когда:** ветки доступны на origin, старые настройки читаются без сброса, а prerelease не предлагаются стабильным пользователям.

## Этап 1 — общая команда и обратная связь

1. Единый реестр команд для toolbar, контекстных меню и хоткеев.
2. Достижимые Push, Merge и Rebase flows с выбором remote/upstream и превью git-команды.
3. Тосты успеха, прогресса и ошибки без переключения пользователя с Changes на Git output.
4. `ConfirmDialog` вместо `window.confirm`.
5. Встроенный commit composer при сохранении полного CommitDialog.

**Готово, когда:** каждая текущая операция доступна из legacy UI, `window.confirm` отсутствует, ошибки видны и в Git output, и через toast.

## Этап 2 — страховочная сеть

1. Oplog со снимками refs, HEAD, индекса и полного worktree, включая untracked-файлы.
2. Undo/Redo для мутаций: commit, amend, reset, revert, cherry-pick, merge, rebase, checkout, branch/tag, stash, discard и pull integration.
3. Reflog с переходом к коммиту, созданием ветки и безопасным reset.

**Готово, когда:** интеграционные тесты на временных репозиториях восстанавливают staged, unstaged, deleted, binary, symlink и untracked состояния.

## Этап 3 — история и рабочие деревья

1. Fixup, squash into, drop, move, amend staged into commit.
2. Interactive rebase с preview, autostash и `--update-refs`.
3. Worktrees в Repositories и Branches.

**Готово, когда:** операции работают для обычных и root-коммитов, конфликты передаются в ConflictBar, а worktree можно создать, открыть, удалить и prune.

## Этап 4 — конфликты и поиск

1. Прогноз конфликтов для merge/rebase/pull через `git merge-tree` при Git ≥ 2.38.
2. 3-way resolver Ours | Result | Theirs с сохранением и `git add`.
3. Палитра команд и быстрый поиск.
4. Syntax highlighting в worker для видимых строк diff.
5. Server-side log filters, `git grep`, branch DnD, stash subsets, ahead/behind, spin-off и branch hygiene.

**Готово, когда:** сложные операции доступны из legacy UI и имеют тесты на основные ошибки и ограничения старых версий Git.

## Приёмка v2

- `npm run build`, `npm test`, `npx vue-tsc --noEmit` проходят.
- `cargo clippy --all-targets -- -D warnings` и `cargo test` проходят.
- Ручная проверка выполнена на большом репозитории, detached HEAD, Cyrillic paths, partial stage, merge/rebase conflicts и нескольких worktrees.
- После приёмки создаётся ветка `v3` от принятого коммита v2.
