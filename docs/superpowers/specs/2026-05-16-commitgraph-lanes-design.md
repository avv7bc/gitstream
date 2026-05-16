# Дизайн: Lane-линии графа коммитов (фича B)

**Дата:** 2026-05-16
**Статус:** Утверждён к планированию (пользователь делегировал решения; per-section гейт пропущен)

## Цель

Заменить плоский «прямой вертикальной линией» рендер графа в `CommitGraph`
на настоящий граф с колонками (lane allocation): ветвления (fork) и слияния
(merge) рисуются кривыми между колонками, как в GitKraken-подобных
клиентах.

ref-чипы — отдельная фича A, вне этой спеки.

## Текущее состояние

- `CommitGraph.vue`: `graph-col` (80px) на строку рисует SVG 80×24 с одной
  вертикальной линией и кружком-узлом по центру (`x=8, y=12, r=4`). Ветвлений нет.
- `CommitInfo` (Rust `types.rs` / TS `types/index.ts`) содержит `parents:
  Vec<String>` — данных о топологии достаточно.
- TS `types/index.ts` уже содержит **неиспользуемые** `GraphRow` и `GraphLine`
  (`{ from_column, to_column, color, style: "straight"|"merge-left"|"merge-right"|"fork" }`).
  Rust-аналогов нет.
- `query::log` возвращает `Vec<CommitInfo>` в порядке `git log` (новейшие сверху,
  топологически согласовано).

## Объём

- Алгоритм lane allocation на бэкенде (Rust, тестируемый) — заполняет на каждый
  коммит `column` и `lines`.
- Рендер графа в `CommitGraph` по этим данным (динамическая ширина колонки,
  кривые fork/merge, цвет по колонке).
- Rust unit-тесты алгоритма.

Вне объёма: пересчёт графа под активный фильтр (как и сейчас, фильтр деградирует
граф — известное ограничение); анимации; перетаскивание; цветовая
непрерывность строго «по ветке» (цвет — детерминированный по индексу колонки).

## Архитектура

Логика — на бэкенде (паттерн проекта: вычисления в Rust с `cargo test`, Vue —
тонкий рендер). Алгоритм заполняет поля прямо в `CommitInfo` (одна выборка,
выровненные данные), без отдельной команды.

### 1. Типы

**`src-tauri/src/git/types.rs`** — новая структура и два поля в `CommitInfo`:

```rust
#[derive(Serialize, Clone, Debug)]
pub struct GraphLine {
    pub from_column: u32,
    pub to_column: u32,
    pub color: u32,
    pub style: String, // "straight" | "merge-left" | "merge-right" | "fork"
}

// в CommitInfo добавить:
pub column: u32,
pub lines: Vec<GraphLine>,
```

**`src/types/index.ts`:** в `CommitInfo` добавить `column: number;` и
`lines: GraphLine[];`. Существующий тип `GraphLine` оставить как есть
(union стилей совпадает). Неиспользуемый `GraphRow` удалить (становится лишним —
поля инлайнятся в `CommitInfo`; чистка во избежание путаницы).

### 2. Модуль `graph.rs`

`src-tauri/src/git/graph.rs`, регистрируется в `src-tauri/src/git/mod.rs`
(`pub mod graph;`). Чистая функция:

```rust
pub fn assign_lanes(commits: &mut [CommitInfo])
```

Состояние: `lanes: Vec<Option<String>>` — `lanes[c] = Some(oid)` означает, что
колонка `c` «ждёт» коммит с этим oid (ссылка-родитель от уже выведенного
потомка). Колонки индексируются стабильно (без компакции).

Палитра цветов: `color = column % N` (N = число цветов рендера). Детерминированно;
строгая непрерывность цвета по ветке — вне объёма (зафиксировано как ограничение).

Для каждого коммита (в порядке списка, новейшие первыми) с oid `o`, родителями `P`:

1. **Колонка узла `col`:** наименьший `i` с `lanes[i] == Some(o)`; если нет —
   наименьший свободный (`None`) индекс; если и таких нет — `lanes.push`, `col =
   len-1` (это «верхушка», не достижимая загруженными потомками).
2. **Входящие (верхняя половина ячейки):** для каждого `i` с `lanes[i] ==
   Some(o)`:
   - `i == col`: вклад в вертикаль узла (см. п.5).
   - `i != col`: `GraphLine { from_column: i, to_column: col, style:
     if i > col {"merge-left"} else {"merge-right"}, color: color_of(i) }`;
     затем `lanes[i] = None` (стренд слит в узел).
3. `lanes[col] = None` (узел разрешён, временно).
4. **Сквозные:** для каждого `i` с `lanes[i] == Some(x)` где `x != o` (не
   тронуты): `GraphLine { from_column: i, to_column: i, style: "straight",
   color: color_of(i) }` (стренд проходит всю ячейку).
5. **Исходящие (нижняя половина) и вертикаль узла:**
   - первый родитель `p0` (если есть): `lanes[col] = Some(p0)`.
   - всегда эмитим вертикаль узла: `GraphLine { from_column: col, to_column:
     col, style: "straight", color: color_of(col) }` (полная вертикаль через
     ячейку; для верхушки/корня визуально — короткий стержень, это нормально и
     стандартно).
   - каждый дополнительный родитель `pk` (k≥1, мерж/octopus): `j` = наименьший
     свободный индекс (или `push`); если `pk` уже присутствует в каком-то lane —
     не дублировать (ветки сойдутся позже), иначе `lanes[j] = Some(pk)` и
     `GraphLine { from_column: col, to_column: j, style: "fork", color:
     color_of(j) }`.
   - корень (нет родителей): `lanes[col]` остаётся `None`.
6. Записать `commits[r].column = col`, `commits[r].lines = <собранные линии>`.

`query::log`: при конструировании `CommitInfo` задавать `column: 0, lines:
Vec::new()`, затем перед `Ok(commits)` вызвать `graph::assign_lanes(&mut commits)`.

### 3. Рендер (CommitGraph.vue)

Константы: `COL_W = 14`, `NODE_R = 4`, высота ячейки `H = 24`, левый отступ
`PAD = 8`. `X(c) = PAD + c * COL_W`. `maxCol = max(commit.column, max
to/from_column во всех lines)`; ширина `graph-col` = `max(80, X(maxCol) + 16)`
(вычисляется как `computed`, применяется через CSS-переменную, аналогично
существующей `--author-col-w`).

На строку SVG `width = ширина graph-col`, `height = 24`. Для каждой
`commit.lines`:
- `straight` (`from==to`): `<line x1=X(from) y1=0 x2=X(from) y2=24>`.
- `merge-left|merge-right` (верхняя половина): кубическая кривая
  `(X(from),0) → (X(to),12)` (control points сглаживают S-образно).
- `fork` (нижняя половина): кубическая кривая `(X(col),12) → (X(to),24)`.
Цвет линии: `palette[color % palette.length]`. Узел: `<circle cx=X(commit.column)
cy=12 r=4>`; заливка — сохранить текущее поведение: `unpushed → var(--yellow)`,
иначе `palette[commit.column % …]`; обводка `var(--bg-primary)`; `selected`
подсветка строки — без изменений.

Палитра (CSS-переменные тёмной темы, без `--red` — он зарезервирован за WT):
`[--blue, --green, --mauve, --yellow, --teal, --sapphire]` (или эквивалентные
существующие; точный список — на этапе плана из `styles/`).

**WT-строка:** сохранить особый верхний ряд; узел `var(--red)` в колонке
`commits[0].column` (обычно 0), вертикаль вниз к первому коммиту. Минимально.

Существующий наивный `graph-col` SVG (одиночные `<line v-if> + <circle>`)
заменяется новым рендером для строк коммитов; WT-строка — упрощённый вариант.

## Data flow

```
git log %P → CommitInfo.parents → graph::assign_lanes (Rust)
  → CommitInfo.column / .lines → get_log → useLog.commits
  → CommitGraph рендерит SVG по lines (straight/merge/fork) + узел
```

## Краевые случаи

- Линейная история: все `column = 0`, на строку одна `straight` линия.
- Корневой коммит (нет родителей): `column` назначен, нет `fork`, lane
  закрывается (ниже узла линии нет — допустимо, рисуется «стержень» узла).
- Родитель вне загруженных 500: lane остаётся открытым, `straight` уходит за
  нижнюю границу (родитель за экраном) — приемлемо.
- Octopus merge (>2 родителей): каждый доп. родитель → новый lane + `fork`.
- Несколько потомков у коммита (сходящиеся ветки): несколько lane с одним oid →
  узел в наименьшей колонке, прочие — `merge-*`, затем закрываются.
- Активный фильтр графа: граф деградирует (как и сейчас) — известное
  ограничение, не чиним.

## Тестирование

- **Rust `#[cfg(test)]` в `graph.rs`** (хелпер строит `CommitInfo` с заданными
  oid/parents):
  1. Линейная A←B←C (порядок C,B,A): у всех `column==0`; у каждого есть
     `straight` линия с `from==to==0`.
  2. Ветка+мерж: `M` (parents `[P1, P2]`), затем `P1`, `P2`, `Base`
     (P1,P2 parent = Base): `M.column==0`; у `M` есть `fork` на колонку 1;
     `P2.column==1`; в строке `Base` обе ветки сходятся (`merge-*`).
  3. Корень: последний коммит без родителей → `column` задан, нет `fork`.
  4. Верхушка не достижима потомками → новый lane (column>0 при занятой 0).
  Проверяем колонки и наличие/тип линий, не пиксели.
- **Frontend:** нет FE-тест-раннера → `npm run build` (vue-tsc + vite) + ручная
  проверка на репозитории gitstream (линейная история + мерж-коммит
  tag-operations): видны колонки, кривая мержа, цвета по колонкам, WT-строка.
- Bump patch версии `src-tauri/tauri.conf.json` 0.1.8 → 0.1.9 (правило проекта).
  Примечание: фича A (в своей ветке) тоже поднимает до 0.1.9 — при будущем
  объединении веток конфликт версии разрешает пользователь.

## Definition of Done

- `assign_lanes` заполняет `column`/`lines`; Rust-тесты (≥4 кейса) зелёные.
- `CommitGraph` рисует мульти-колоночный граф с кривыми fork/merge и цветом по
  колонке; ширина графа адаптивная; WT-строка корректна.
- Линейная история выглядит как одна вертикаль (без регресса).
- `npm run build`, `cargo test`, `cargo build` зелёные.
