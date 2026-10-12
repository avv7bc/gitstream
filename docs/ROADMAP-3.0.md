# GitStream 3 — дизайн A «Графит»

Ветка `v3` создаётся только от принятого baseline v2. Она сохраняет весь функциональный слой v2 и меняет presentation layer, навигацию и визуальную систему по офлайн-макетам `docs/design/A/`.

## Правила линии v3

- Backend-команды, типы, composables и тесты v2 переиспользуются.
- Визуальные изменения не должны отключать legacy-команды или менять семантику Git-операций.
- Первый prerelease — `v3.0.0-alpha.1`; версия в исходниках меняется только при подготовке prerelease.
- Критерий готовности — одинаково доступный функциональный набор v2 и соответствие четырём наборам макетов: main, dialogs, tools, states.

## Этап 1 — дизайн-система

1. Семантические токены `tokens.css`, темы по файлам, Graphite по умолчанию.
2. IBM Plex Sans и JetBrains Mono из локальных файлов.
3. Compact/comfortable density.
4. Stylelint/ESLint-проверки для цветов и строк.
5. `BaseDialog` и `DialogButton` с единым footer, Esc, default action, busy и draggable header.

**Готово, когда:** все диалоги используют один контракт, а компоненты не содержат hardcoded hex/rgba вне тем.

## Этап 2 — окно и состояния

1. Toolbar Graphite: Fetch, Pull, Push, Commit, Undo, palette, Stats, Settings.
2. Тематический status bar с ahead/behind, изменениями, операцией и последней ошибкой.
3. FileList toolbar с segmented-фильтрами, деревом и Stage/Unstage all.
4. Нижние вкладки Changes, Operations и Git output.
5. Welcome screen, empty states и skeletons.
6. Toast/progress/conflict/command-palette states из `states.html`.

**Готово, когда:** главное окно совпадает с `A/main.html`, а каждая пустая панель объясняет следующее действие.

## Этап 3 — миграция операционных окон

Перенести все существующие диалоги по порядку из спецификации: Confirm, Discard, Commit, Push/Pull, Checkout, Branch/Tag/Stash, Remote/Upstream, Add/Clone/Group/Rename, Credential, Squash/Reword, FileHistory/Blame, FileCompare, Settings/Stats.

**Готово, когда:** поведение v2 сохранено, а `dialogs.html` покрывает рабочие и error states.

## Этап 4 — инструменты и визуальная приёмка

1. Перенести tools: interactive rebase, resolver, file compare, reflog/oplog и Git output.
2. Проверить Graphite, Light и Dracula, compact/comfortable, keyboard-only navigation, reduced motion и narrow windows.
3. Сравнить screenshots с `docs/design/A/*.html` при одинаковом viewport.

**Готово, когда:** функциональная матрица v2 проходит без регрессий, visual diff проверен, CI зелёный.
