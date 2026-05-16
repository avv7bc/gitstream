# CommitGraph Lane-Lines Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.
>
> **HARD CONSTRAINT — NO GIT COMMITS.** Standing user rule: never `git add`/`commit`/`merge`/`push` without explicit consent. Every task implements + verifies ONLY. Leave changes uncommitted. Controller asks about commit once at the end.

**Goal:** Заменить плоский граф в CommitGraph настоящим lane-графом (колонки, кривые fork/merge), вычисляемым в Rust.

**Architecture:** Чистый алгоритм `graph::assign_lanes` (Rust, тестируемый cargo) заполняет `column`/`lines` в `CommitInfo`; `query::log` вызывает его; Vue рисует SVG по этим данным с адаптивной шириной.

**Tech Stack:** Rust (Tauri), Vue 3 + TS, Vite. Worktree `/home/avv/projects/gitstream/.worktrees/lanes` (ветка `feature/commitgraph-lanes`).

**Spec:** `docs/superpowers/specs/2026-05-16-commitgraph-lanes-design.md`

**Verify:** BE `cargo test` / `cargo build`; FE `npm run build`.

---

### Task 1: Rust-типы — `GraphLine` + поля `CommitInfo`

**Files:**
- Modify: `src-tauri/src/git/types.rs`
- Modify: `src-tauri/src/git/query.rs` (конструирование `CommitInfo` в `log`)

- [ ] **Step 1: Тип и поля** — В `src-tauri/src/git/types.rs` сразу ПОСЛЕ структуры `CommitInfo` (после её закрывающей `}`, ~строка 26) добавить:

```rust
#[derive(Serialize, Clone, Debug)]
pub struct GraphLine {
    pub from_column: u32,
    pub to_column: u32,
    pub color: u32,
    pub style: String,
}
```

И в саму `struct CommitInfo` (после поля `pub refs: Vec<RefLabel>,`) добавить две строки:

```rust
    pub column: u32,
    pub lines: Vec<GraphLine>,
```

- [ ] **Step 2: Дефолты в `query::log`** — В `src-tauri/src/git/query.rs`, в функции `log`, в литерале `CommitInfo { ... }` (где `parents, refs,`) добавить инициализацию новых полей. Заменить блок:

```rust
        commits.push(CommitInfo {
            oid: parts[0].to_string(), short_oid: parts[1].to_string(),
            message: parts[2].to_string(), author: parts[3].to_string(),
            author_email: parts[4].to_string(), date: parts[5].to_string(),
            parents, refs,
        });
```

на:

```rust
        commits.push(CommitInfo {
            oid: parts[0].to_string(), short_oid: parts[1].to_string(),
            message: parts[2].to_string(), author: parts[3].to_string(),
            author_email: parts[4].to_string(), date: parts[5].to_string(),
            parents, refs,
            column: 0,
            lines: Vec::new(),
        });
```

- [ ] **Step 3: Verify compile** — `cd src-tauri && cargo build 2>&1 | tail -5` — Expected: компилируется без ошибок (новые поля сериализуются, пока не заполняются).

- [ ] **Step 4: НЕ коммитить.** Сообщить статус и изменённые файлы.

---

### Task 2: Модуль `graph.rs` — алгоритм + тесты + хук в `log`

**Files:**
- Create: `src-tauri/src/git/graph.rs`
- Modify: `src-tauri/src/git/mod.rs` (регистрация модуля)
- Modify: `src-tauri/src/git/query.rs` (вызов `assign_lanes`)

- [ ] **Step 1: Создать `src-tauri/src/git/graph.rs`** с тестами и реализацией (TDD: тесты в том же файле):

```rust
use super::types::{CommitInfo, GraphLine};

const COLORS: u32 = 6;

/// Назначает каждому коммиту колонку (`column`) и набор линий (`lines`)
/// для отрисовки lane-графа. Коммиты ожидаются в порядке `git log`
/// (новейшие первыми, топологически согласовано).
pub fn assign_lanes(commits: &mut [CommitInfo]) {
    let mut lanes: Vec<Option<String>> = Vec::new();

    for idx in 0..commits.len() {
        let oid = commits[idx].oid.clone();
        let parents = commits[idx].parents.clone();
        let mut lines: Vec<GraphLine> = Vec::new();

        // 1. колонка узла
        let col = match lanes.iter().position(|l| l.as_deref() == Some(oid.as_str())) {
            Some(c) => c,
            None => match lanes.iter().position(|l| l.is_none()) {
                Some(c) => c,
                None => {
                    lanes.push(None);
                    lanes.len() - 1
                }
            },
        };

        // 2. входящие мержи: другие lane, указывающие на oid
        for i in 0..lanes.len() {
            if i != col && lanes[i].as_deref() == Some(oid.as_str()) {
                let style = if i > col { "merge-left" } else { "merge-right" };
                lines.push(GraphLine {
                    from_column: i as u32,
                    to_column: col as u32,
                    color: (i as u32) % COLORS,
                    style: style.to_string(),
                });
                lanes[i] = None;
            }
        }

        // 3. узел разрешён
        lanes[col] = None;

        // 4. сквозные lane
        for i in 0..lanes.len() {
            if i != col && lanes[i].is_some() {
                lines.push(GraphLine {
                    from_column: i as u32,
                    to_column: i as u32,
                    color: (i as u32) % COLORS,
                    style: "straight".to_string(),
                });
            }
        }

        // 5. вертикаль узла (всегда)
        lines.push(GraphLine {
            from_column: col as u32,
            to_column: col as u32,
            color: (col as u32) % COLORS,
            style: "straight".to_string(),
        });

        // исходящие родители
        if let Some((first, rest)) = parents.split_first() {
            lanes[col] = Some(first.clone());
            for p in rest {
                if lanes.iter().any(|l| l.as_deref() == Some(p.as_str())) {
                    continue;
                }
                let j = match lanes.iter().position(|l| l.is_none()) {
                    Some(c) => c,
                    None => {
                        lanes.push(None);
                        lanes.len() - 1
                    }
                };
                lanes[j] = Some(p.clone());
                lines.push(GraphLine {
                    from_column: col as u32,
                    to_column: j as u32,
                    color: (j as u32) % COLORS,
                    style: "fork".to_string(),
                });
            }
        }
        // корень (нет родителей): lanes[col] остаётся None

        commits[idx].column = col as u32;
        commits[idx].lines = lines;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::CommitInfo;

    fn c(oid: &str, parents: &[&str]) -> CommitInfo {
        CommitInfo {
            oid: oid.to_string(),
            short_oid: oid.to_string(),
            message: String::new(),
            author: String::new(),
            author_email: String::new(),
            date: String::new(),
            parents: parents.iter().map(|s| s.to_string()).collect(),
            refs: Vec::new(),
            column: 0,
            lines: Vec::new(),
        }
    }

    #[test]
    fn linear_history_single_column() {
        let mut v = vec![c("C", &["B"]), c("B", &["A"]), c("A", &[])];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0);
        assert_eq!(v[1].column, 0);
        assert_eq!(v[2].column, 0);
        for row in &v {
            assert!(row
                .lines
                .iter()
                .any(|l| l.style == "straight" && l.from_column == 0 && l.to_column == 0));
        }
    }

    #[test]
    fn branch_and_merge() {
        // M — мерж (parents P1, P2); P1, P2 → Base
        let mut v = vec![
            c("M", &["P1", "P2"]),
            c("P1", &["Base"]),
            c("P2", &["Base"]),
            c("Base", &[]),
        ];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0); // M
        assert!(v[0]
            .lines
            .iter()
            .any(|l| l.style == "fork" && l.from_column == 0 && l.to_column == 1));
        assert_eq!(v[2].column, 1); // P2 на lane 1
        assert_eq!(v[3].column, 0); // Base
        assert!(v[3]
            .lines
            .iter()
            .any(|l| l.style == "merge-left" && l.from_column == 1 && l.to_column == 0));
    }

    #[test]
    fn root_commit_no_fork() {
        let mut v = vec![c("R", &[])];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0);
        assert!(!v[0].lines.iter().any(|l| l.style == "fork"));
        assert!(v[0]
            .lines
            .iter()
            .any(|l| l.style == "straight" && l.from_column == 0));
    }

    #[test]
    fn unreferenced_tip_gets_new_lane() {
        // A держит lane0 занятым (родитель B), затем независимый C → lane1
        let mut v = vec![c("A", &["B"]), c("C", &["D"]), c("B", &[]), c("D", &[])];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0); // A
        assert_eq!(v[1].column, 1); // C — lane0 занят B
    }
}
```

- [ ] **Step 2: Зарегистрировать модуль** — В `src-tauri/src/git/mod.rs` добавить строку (рядом с прочими `pub mod ...;`, по алфавиту/после `error`):

```rust
pub mod graph;
```

- [ ] **Step 3: Run tests, verify PASS** — `cd src-tauri && cargo test --lib graph 2>&1 | tail -10` — Expected: `test result: ok. 4 passed`.

- [ ] **Step 4: Хук в `query::log`** — В `src-tauri/src/git/query.rs`, в функции `log`, заменить финальную строку `Ok(commits)` на:

```rust
    super::graph::assign_lanes(&mut commits);
    Ok(commits)
```

(переменная уже объявлена как `let mut commits = Vec::new();` — мутабельна.)

- [ ] **Step 5: Verify** — `cd src-tauri && cargo build 2>&1 | tail -3 && cargo test 2>&1 | tail -5` — Expected: build ok; все тесты `ok` (graph 4 + ранее существующие).

- [ ] **Step 6: НЕ коммитить.** Сообщить статус, изменённые/созданные файлы.

---

### Task 3: TS-типы

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: Поля CommitInfo + удалить GraphRow** — В `src/types/index.ts`:
  - В интерфейс `CommitInfo` после строки `refs: RefLabel[];` добавить:
    ```ts
      column: number;
      lines: GraphLine[];
    ```
  - Удалить целиком неиспользуемый интерфейс `GraphRow`:
    ```ts
    export interface GraphRow {
      commit: CommitInfo;
      column: number;
      lines: GraphLine[];
    }
    ```
  - Интерфейс `GraphLine` оставить без изменений.

- [ ] **Step 2: Verify** — `npm run build 2>&1 | tail -5` — Expected: `✓ built`, без vue-tsc ошибок (если где-то использовался `GraphRow` — ошибка; тогда сообщить, такой код в проекте не ожидается).

- [ ] **Step 3: НЕ коммитить.** Сообщить статус.

---

### Task 4: Рендер графа в `CommitGraph.vue`

**Files:**
- Modify: `src/components/CommitGraph.vue` (script: импорт типа + хелперы; template: graph-col WT и коммитов; style: ширина колонки)

- [ ] **Step 1: Script — хелперы** — В `<script setup>` `CommitGraph.vue`:
  - Убедиться, что `computed` импортирован из `vue` (строка 2 `import { computed, ref } from "vue";` — уже да).
  - Добавить импорт типа рядом с `import type { RefLabel } from "@/types";` (или к существующему type-импорту из `@/types`): итоговая строка должна импортировать и `RefLabel`, и `GraphLine`, например:
    ```ts
    import type { RefLabel, GraphLine } from "@/types";
    ```
    (Если `RefLabel` импортируется отдельной строкой — добавить рядом `import type { GraphLine } from "@/types";`.)
  - После строки `const { commits, selectedCommit } = useLog();` добавить блок:
    ```ts
    const GRAPH_PALETTE = ["--blue", "--green", "--purple", "--teal", "--orange", "--yellow"];
    const GRAPH_COL_W = 14;
    const GRAPH_PAD = 8;
    const GRAPH_ROW_H = 24;

    function laneX(c: number): number {
      return GRAPH_PAD + c * GRAPH_COL_W;
    }
    function laneColor(colorIdx: number): string {
      return `var(${GRAPH_PALETTE[colorIdx % GRAPH_PALETTE.length]})`;
    }
    function linePath(l: GraphLine): string {
      const x1 = laneX(l.from_column);
      const x2 = laneX(l.to_column);
      const mid = GRAPH_ROW_H / 2;
      if (l.style === "straight") {
        return `M ${x1} 0 L ${x1} ${GRAPH_ROW_H}`;
      }
      if (l.style === "fork") {
        const cy = (mid + GRAPH_ROW_H) / 2;
        return `M ${x1} ${mid} C ${x1} ${cy} ${x2} ${cy} ${x2} ${GRAPH_ROW_H}`;
      }
      // merge-left | merge-right (верхняя половина → центр узла)
      const cy = mid / 2;
      return `M ${x1} 0 C ${x1} ${cy} ${x2} ${cy} ${x2} ${mid}`;
    }
    const graphMaxCol = computed(() => {
      let m = 0;
      for (const c of commits.value) {
        if (c.column > m) m = c.column;
        for (const l of c.lines) {
          if (l.from_column > m) m = l.from_column;
          if (l.to_column > m) m = l.to_column;
        }
      }
      return m;
    });
    const graphColW = computed(() => Math.max(80, laneX(graphMaxCol.value) + 16));
    const wtCol = computed(() => commits.value[0]?.column ?? 0);
    ```

- [ ] **Step 2: Template — корневой стиль** — Заменить открывающий div:

```vue
  <div class="commit-graph" :style="{ '--author-col-w': (maxAuthorLen + 2) + 'ch' }" @contextmenu.prevent>
```

на:

```vue
  <div class="commit-graph" :style="{ '--author-col-w': (maxAuthorLen + 2) + 'ch', '--graph-col-w': graphColW + 'px' }" @contextmenu.prevent>
```

- [ ] **Step 3: Template — WT graph-col** — Заменить блок (в `wt-row`):

```vue
        <div class="graph-col">
          <svg width="80" height="24" class="graph-svg">
            <line x1="8" y1="12" x2="8" y2="24" stroke="var(--blue)" stroke-width="2" />
            <circle cx="8" cy="12" r="4" fill="var(--red)" stroke="var(--bg-primary)" stroke-width="1.5" />
          </svg>
        </div>
```

на:

```vue
        <div class="graph-col">
          <svg :width="graphColW" height="24" class="graph-svg">
            <line :x1="laneX(wtCol)" y1="12" :x2="laneX(wtCol)" y2="24" stroke="var(--red)" stroke-width="2" />
            <circle :cx="laneX(wtCol)" cy="12" r="4" fill="var(--red)" stroke="var(--bg-primary)" stroke-width="1.5" />
          </svg>
        </div>
```

- [ ] **Step 4: Template — commit graph-col** — Заменить блок (в строке коммита):

```vue
        <!-- Graph column with SVG lines -->
        <div class="graph-col">
          <svg width="80" height="24" class="graph-svg">
            <line v-if="idx > 0 || changedCount > 0" x1="8" y1="0" x2="8" y2="24" stroke="var(--blue)" stroke-width="2" />
            <circle cx="8" cy="12" r="4" :fill="isUnpushed(idx) ? 'var(--yellow)' : 'var(--blue)'" stroke="var(--bg-primary)" stroke-width="1.5" />
          </svg>
        </div>
```

на:

```vue
        <!-- Graph column with SVG lane lines -->
        <div class="graph-col">
          <svg :width="graphColW" height="24" class="graph-svg">
            <path
              v-for="(ln, li) in commit.lines"
              :key="li"
              :d="linePath(ln)"
              :stroke="laneColor(ln.color)"
              stroke-width="2"
              fill="none"
            />
            <circle
              :cx="laneX(commit.column)"
              cy="12"
              r="4"
              :fill="isUnpushed(idx) ? 'var(--yellow)' : laneColor(commit.column)"
              stroke="var(--bg-primary)"
              stroke-width="1.5"
            />
          </svg>
        </div>
```

- [ ] **Step 5: Style — ширина колонки** — В `<style scoped>` заменить правило:

```css
.graph-col {
  width: 80px;
  flex-shrink: 0;
  overflow: hidden;
}
```

на:

```css
.graph-col {
  width: var(--graph-col-w, 80px);
  flex-shrink: 0;
  overflow: hidden;
}
```

- [ ] **Step 6: Verify** — `npm run build 2>&1 | tail -5` — Expected: `✓ built`, без vue-tsc ошибок.

- [ ] **Step 7: НЕ коммитить.** Сообщить статус, изменённые файлы.

---

### Task 5: Bump версии + финальная проверка

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Bump** — В `src-tauri/tauri.conf.json` заменить `"version": "0.1.8",` на `"version": "0.1.9",` (правило проекта; конфликт с фичей A при будущем merge разрешит пользователь).

- [ ] **Step 2: Полная проверка** — из корня worktree:
  `npm run build 2>&1 | tail -3 && cd src-tauri && cargo test 2>&1 | tail -6 && cargo build 2>&1 | tail -3`
  Expected: `✓ built`; все `cargo test` `ok` (graph 4 + tag_tests 4); `cargo build` без ошибок.

- [ ] **Step 3: НЕ коммитить.** Сообщить итог и полный список изменённых/созданных файлов.

---

## Self-Review

**Покрытие спеки:**
- Типы `GraphLine` + `CommitInfo.column/lines` (Rust+TS) — Task 1, 3. ✓
- Алгоритм `assign_lanes` (lane state, incoming/passthrough/node/outgoing, styles) — Task 2. ✓
- Хук в `query::log` — Task 2 Step 4. ✓
- Rust-тесты ≥4 (linear, branch+merge, root, unreferenced tip) — Task 2. ✓
- Рендер: адаптивная ширина, straight/merge/fork кривые, цвет по колонке, узел, WT-строка — Task 4. ✓
- Удаление неиспользуемого `GraphRow` — Task 3. ✓
- Bump версии + примечание о конфликте с A — Task 5. ✓
- Краевые случаи (корень/верхушка/octopus/несколько потомков) покрыты алгоритмом Task 2 и тестами (root, unreferenced tip, branch+merge).

**Плейсхолдеры:** нет — весь код приведён дословно (включая алгоритм и тесты).

**Согласованность типов:** Rust `GraphLine{from_column,to_column,color:u32,style:String}` ↔ TS `GraphLine{from_column,to_column,color:number,style:"straight"|"merge-left"|"merge-right"|"fork"}` (значения из алгоритма строго в этом множестве). `CommitInfo.column:u32`↔`number`, `lines:Vec<GraphLine>`↔`GraphLine[]`. `assign_lanes(&mut [CommitInfo])` вызывается в `query::log`. Рендер использует `commit.column`, `commit.lines`, `l.from_column/to_column/color/style` — все определены в Task 1/3. `laneColor`/`laneX`/`linePath`/`graphColW`/`wtCol` объявлены в Task 4 Step 1 и используются в Step 2–5.
