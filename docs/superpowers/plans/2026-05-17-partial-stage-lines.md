# Частичный stage отдельных строк — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Дать возможность делать stage / unstage / discard выбранного диапазона строк (а не только целого хунка) прямо в `SideBySideDiffView`.

**Architecture:** Синтез частичного unified-патча выполняется в Rust (чистая, тестируемая функция), результат применяется через существующий `apply_patch`. Фронтенд хранит выделение строк (клик / Shift / Ctrl) и передаёт в новую команду `apply_lines` сырые хунки + порядковые номера выбранных изменённых строк. Трансформация хунка идентична для всех трёх операций — различаются только флаги `git apply` (`--reverse`/`--cached`).

**Tech Stack:** Rust (Tauri 2, `std::process::Command`, `#[cfg(test)]`), Vue 3 Composition API + TypeScript.

**Спецификация:** `docs/superpowers/specs/2026-05-17-partial-stage-lines-design.md`

---

## File Structure

**Backend (Rust):**
- `src-tauri/src/git/mutation.rs` — добавить типы `LineHunkSelection`, `LineOp`; функции `parse_hunk_header`, `rebuild_hunk`, `build_partial_patch`, `apply_lines`; новый тест-модуль `mod partial_line_tests`.
- `src-tauri/src/commands.rs` — добавить `#[tauri::command] apply_lines`.
- `src-tauri/src/main.rs` — зарегистрировать `commands::apply_lines` в `generate_handler!`.

**Frontend (TS/Vue):**
- `src/types/index.ts` — типы `LineOp`, `LineHunkSelection`.
- `src/composables/useFiles.ts` — функция `applyLines`.
- `src/components/SideBySideDiffView.vue` — состояние выделения, обработчики клика, payload-билдер, привязка кнопок, CSS.

**Версия:**
- `src/components/StatusBar.vue:49` — bump `0.1.37` → `0.1.38`.

> **Замечание о TDD:** в проекте нет фронтенд тест-фреймворка (нет `test`-скрипта, нет vitest). Поэтому TDD с автотестами применяется к Rust-логике (вся рискованная арифметика патча). Фронтенд проверяется `npm run build` (включает `vue-tsc`) + ручная проверка по чеклисту в Task 9.

> **Отклонение от спецификации (осознанное):** для подтверждения discard выбранных строк используется `window.confirm` — так уже сделано для разрушающих операций в `CommitGraph.vue:112` (hard reset) и `BranchPanel.vue:142`. Vue-`ConfirmDialog` смонтирован только на уровне `App.vue` и его проброс в `SideBySideDiffView` вне объёма этой задачи.

---

## Task 1: Парсинг заголовка хунка (Rust)

**Files:**
- Modify: `src-tauri/src/git/mutation.rs` (добавить функцию перед строкой `#[cfg(test)]`, ~line 350)
- Test: тот же файл, новый модуль `mod partial_line_tests` в конце файла

- [ ] **Step 1: Написать падающий тест**

В конец `src-tauri/src/git/mutation.rs` добавить новый тест-модуль (рядом с существующим `mod tag_tests`):

```rust
#[cfg(test)]
mod partial_line_tests {
    use super::*;

    #[test]
    fn parses_header_with_section_tail() {
        let (os, ns, tail) = parse_hunk_header("@@ -12,7 +12,8 @@ fn foo()").unwrap();
        assert_eq!(os, 12);
        assert_eq!(ns, 12);
        assert_eq!(tail, " fn foo()");
    }

    #[test]
    fn parses_header_without_counts_and_tail() {
        let (os, ns, tail) = parse_hunk_header("@@ -1 +1 @@").unwrap();
        assert_eq!(os, 1);
        assert_eq!(ns, 1);
        assert_eq!(tail, "");
    }
}
```

- [ ] **Step 2: Запустить тест — убедиться, что не компилируется/падает**

Run: `cd src-tauri && cargo test partial_line_tests -- --nocapture`
Expected: ошибка компиляции `cannot find function parse_hunk_header`.

- [ ] **Step 3: Реализовать `parse_hunk_header`**

Добавить в `src-tauri/src/git/mutation.rs` непосредственно перед финальным `#[cfg(test)]`:

```rust
/// Разбирает заголовок хунка "@@ -a,b +c,d @@ tail".
/// Возвращает (old_start, new_start, tail) — где tail включает ведущий пробел.
fn parse_hunk_header(h: &str) -> Option<(usize, usize, String)> {
    let rest = h.strip_prefix("@@ ")?;
    let close = rest.find(" @@")?;
    let ranges = &rest[..close];
    let tail = &rest[close + 3..];
    let mut it = ranges.split(' ');
    let old = it.next()?;
    let new = it.next()?;
    let old_start: usize = old
        .trim_start_matches('-')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    let new_start: usize = new
        .trim_start_matches('+')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    Some((old_start, new_start, tail.to_string()))
}
```

- [ ] **Step 4: Запустить тест — убедиться, что проходит**

Run: `cd src-tauri && cargo test partial_line_tests -- --nocapture`
Expected: PASS (2 теста).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/git/mutation.rs
git commit -m "feat(partial-stage): парсинг заголовка хунка для частичного патча"
```

---

## Task 2: Пересборка хунка из выбранных строк (Rust)

**Files:**
- Modify: `src-tauri/src/git/mutation.rs`
- Test: `mod partial_line_tests` в том же файле

- [ ] **Step 1: Написать падающие тесты**

Добавить в `mod partial_line_tests`:

```rust
    #[test]
    fn rebuild_keeps_only_selected_added_line() {
        // Хунк: контекст, +A (ord 0), +B (ord 1). Выбрана только +A.
        let raw = "@@ -1,1 +1,3 @@\n ctx\n+A\n+B\n";
        let out = rebuild_hunk(raw, &[0]).unwrap();
        // +B выброшен; old=1 (ctx), new=2 (ctx + A)
        assert_eq!(out, "@@ -1,1 +1,2 @@\n ctx\n+A\n");
    }

    #[test]
    fn rebuild_unselected_removal_becomes_context() {
        // -X (ord 0) не выбрана → становится контекстом; -Y (ord 1) выбрана.
        let raw = "@@ -1,3 +1,1 @@\n ctx\n-X\n-Y\n";
        let out = rebuild_hunk(raw, &[1]).unwrap();
        // old=3 (ctx + X + Y), new=2 (ctx + X-as-context)
        assert_eq!(out, "@@ -1,3 +1,2 @@\n ctx\n X\n-Y\n");
    }

    #[test]
    fn rebuild_returns_none_when_nothing_selected() {
        let raw = "@@ -1,1 +1,2 @@\n ctx\n+A\n";
        assert!(rebuild_hunk(raw, &[]).is_none());
    }

    #[test]
    fn rebuild_no_newline_marker_follows_kept_line() {
        // +A выбрана, далее маркер "\ No newline" — должен сохраниться.
        let raw = "@@ -0,0 +1,1 @@\n+A\n\\ No newline at end of file\n";
        let out = rebuild_hunk(raw, &[0]).unwrap();
        assert_eq!(out, "@@ -0,0 +1,1 @@\n+A\n\\ No newline at end of file\n");
    }

    #[test]
    fn rebuild_no_newline_marker_dropped_with_unselected_line() {
        // +A не выбрана → строка выброшена, маркер тоже не должен попасть.
        let raw = "@@ -1,1 +1,2 @@\n ctx\n+A\n\\ No newline at end of file\n";
        assert!(rebuild_hunk(raw, &[]).is_none());
    }

    #[test]
    fn rebuild_preserves_header_tail() {
        let raw = "@@ -10,2 +10,3 @@ fn foo()\n ctx\n+A\n ctx2\n";
        let out = rebuild_hunk(raw, &[0]).unwrap();
        assert_eq!(out, "@@ -10,2 +10,3 @@ fn foo()\n ctx\n+A\n ctx2\n");
    }
```

- [ ] **Step 2: Запустить — убедиться, что падает**

Run: `cd src-tauri && cargo test partial_line_tests -- --nocapture`
Expected: ошибка компиляции `cannot find function rebuild_hunk`.

- [ ] **Step 3: Реализовать `rebuild_hunk`**

Добавить в `src-tauri/src/git/mutation.rs` рядом с `parse_hunk_header`:

```rust
/// Пересобирает один хунк, оставляя только выбранные изменённые (+/-) строки.
/// `selected` — 0-based порядковые номера +/- строк тела хунка (в порядке raw).
/// Невыбранные '-' превращаются в контекст, невыбранные '+' выбрасываются.
/// Возвращает None, если в результате не осталось ни одной +/- строки.
fn rebuild_hunk(raw: &str, selected: &[usize]) -> Option<String> {
    let mut lines = raw.split('\n');
    let header = lines.next()?;
    let (old_start, new_start, tail) = parse_hunk_header(header)?;

    let sel: std::collections::HashSet<usize> = selected.iter().copied().collect();
    let mut body = String::new();
    let mut old_count = 0usize;
    let mut new_count = 0usize;
    let mut changed_ord = 0usize;
    let mut kept_any = false;
    let mut last_kept = false;

    for line in lines {
        if line.is_empty() {
            continue; // хвостовой пустой элемент после последнего '\n'
        }
        let tag = line.as_bytes()[0] as char;
        match tag {
            ' ' => {
                body.push_str(line);
                body.push('\n');
                old_count += 1;
                new_count += 1;
                last_kept = true;
            }
            '-' => {
                let is_sel = sel.contains(&changed_ord);
                changed_ord += 1;
                old_count += 1;
                if is_sel {
                    body.push_str(line);
                    body.push('\n');
                    kept_any = true;
                    last_kept = true;
                } else {
                    body.push(' ');
                    body.push_str(&line[1..]);
                    body.push('\n');
                    new_count += 1;
                    last_kept = true;
                }
            }
            '+' => {
                let is_sel = sel.contains(&changed_ord);
                changed_ord += 1;
                if is_sel {
                    body.push_str(line);
                    body.push('\n');
                    new_count += 1;
                    kept_any = true;
                    last_kept = true;
                } else {
                    last_kept = false;
                }
            }
            '\\' => {
                if last_kept {
                    body.push_str(line);
                    body.push('\n');
                }
            }
            _ => {
                body.push_str(line);
                body.push('\n');
                last_kept = true;
            }
        }
    }

    if !kept_any {
        return None;
    }

    let new_header = format!(
        "@@ -{},{} +{},{} @@{}",
        old_start, old_count, new_start, new_count, tail
    );
    Some(format!("{}\n{}", new_header, body))
}
```

- [ ] **Step 4: Запустить — убедиться, что проходит**

Run: `cd src-tauri && cargo test partial_line_tests -- --nocapture`
Expected: PASS (все тесты Task 1 + Task 2).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/git/mutation.rs
git commit -m "feat(partial-stage): пересборка хунка из выбранных строк"
```

---

## Task 3: Сборка патча и команда `apply_lines` (Rust)

**Files:**
- Modify: `src-tauri/src/git/mutation.rs`
- Test: `mod partial_line_tests` (юнит) + `tempdir` интеграционный тест

- [ ] **Step 1: Написать падающие тесты**

Добавить в `mod partial_line_tests`. Хелпер `temp_repo` повторяет паттерн из `mod tag_tests` (репозиторий с одним коммитом):

```rust
    use std::fs;
    use std::process::Command as ProcCommand;

    fn temp_repo_pl() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_pl_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            ProcCommand::new("git").current_dir(&dir).args(args).output().unwrap();
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t.t"]);
        run(&["config", "user.name", "t"]);
        fs::write(dir.join("f.txt"), "l1\nl2\nl3\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "init"]);
        dir
    }

    fn staged_content(dir: &std::path::Path) -> String {
        let out = ProcCommand::new("git")
            .current_dir(dir)
            .args(["show", ":f.txt"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    #[test]
    fn build_partial_patch_none_when_empty() {
        let hunks = vec![LineHunkSelection {
            raw: "@@ -1,1 +1,2 @@\n l1\n+x\n".to_string(),
            selected: vec![],
        }];
        assert!(build_partial_patch("HEADER\n", &hunks).is_none());
    }

    #[test]
    fn build_partial_patch_prepends_header_and_skips_empty_hunks() {
        let hunks = vec![
            LineHunkSelection {
                raw: "@@ -1,1 +1,2 @@\n l1\n+x\n".to_string(),
                selected: vec![0],
            },
            LineHunkSelection {
                raw: "@@ -5,1 +6,2 @@\n l5\n+y\n".to_string(),
                selected: vec![], // пустой → пропущен
            },
        ];
        let p = build_partial_patch("HDR\n", &hunks).unwrap();
        assert_eq!(p, "HDR\n@@ -1,1 +1,2 @@\n l1\n+x\n");
    }

    #[test]
    fn apply_lines_stages_only_selected_line() {
        let dir = temp_repo_pl();
        // Рабочее дерево: добавляем строку после l2 и меняем l3.
        fs::write(dir.join("f.txt"), "l1\nl2\nNEW\nl3X\n").unwrap();
        // Дифф против индекса (unstaged). Заголовок файла + один хунк.
        let file_header =
            "diff --git a/f.txt b/f.txt\nindex 0000000..1111111 100644\n--- a/f.txt\n+++ b/f.txt\n";
        // Хунк: ctx l1, ctx l2, +NEW (ord0), -l3 (ord1), +l3X (ord2).
        let raw = "@@ -1,3 +1,4 @@\n l1\n l2\n+NEW\n-l3\n+l3X\n";
        let hunks = vec![LineHunkSelection {
            raw: raw.to_string(),
            selected: vec![0], // только +NEW
        }];
        apply_lines(&dir, file_header, &hunks, LineOp::Stage).unwrap();
        // В индексе должна появиться NEW, но l3 остаться неизменной.
        assert_eq!(staged_content(&dir), "l1\nl2\nNEW\nl3\n");
        fs::remove_dir_all(&dir).ok();
    }
```

- [ ] **Step 2: Запустить — убедиться, что падает**

Run: `cd src-tauri && cargo test partial_line_tests -- --nocapture`
Expected: ошибка компиляции (`LineHunkSelection`, `build_partial_patch`, `apply_lines`, `LineOp` не найдены).

- [ ] **Step 3: Реализовать типы, `build_partial_patch`, `apply_lines`**

Добавить в `src-tauri/src/git/mutation.rs` рядом с `rebuild_hunk` (типы — ближе к началу файла, после `use`-секции):

```rust
#[derive(serde::Deserialize)]
pub struct LineHunkSelection {
    pub raw: String,
    pub selected: Vec<usize>,
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LineOp {
    Stage,
    Unstage,
    Discard,
}

/// Собирает частичный патч: file_header + непустые пересобранные хунки.
/// None, если ни в одном хунке ничего не выбрано.
fn build_partial_patch(file_header: &str, hunks: &[LineHunkSelection]) -> Option<String> {
    let mut out = String::new();
    let mut any = false;
    for h in hunks {
        if let Some(rebuilt) = rebuild_hunk(&h.raw, &h.selected) {
            out.push_str(&rebuilt);
            any = true;
        }
    }
    if !any {
        return None;
    }
    Some(format!("{}{}", file_header, out))
}

/// Применяет выбранные строки. Трансформация хунка одинакова для всех op —
/// различаются только флаги git apply.
pub fn apply_lines(
    repo_path: &Path,
    file_header: &str,
    hunks: &[LineHunkSelection],
    op: LineOp,
) -> Result<(), GitError> {
    let patch = match build_partial_patch(file_header, hunks) {
        Some(p) => p,
        None => return Ok(()),
    };
    let (reverse, cached) = match op {
        LineOp::Stage => (false, true),
        LineOp::Unstage => (true, true),
        LineOp::Discard => (true, false),
    };
    apply_patch(repo_path, &patch, reverse, cached)
}
```

- [ ] **Step 4: Запустить — убедиться, что проходит**

Run: `cd src-tauri && cargo test partial_line_tests -- --nocapture`
Expected: PASS (все юнит-тесты + интеграционный `apply_lines_stages_only_selected_line`).

- [ ] **Step 5: Запустить весь набор тестов backend**

Run: `cd src-tauri && cargo test`
Expected: PASS, регрессий в `tag_tests` и прочих нет.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/git/mutation.rs
git commit -m "feat(partial-stage): build_partial_patch + apply_lines с тестами"
```

---

## Task 4: Tauri-команда `apply_lines` (Rust)

**Files:**
- Modify: `src-tauri/src/commands.rs` (после `discard_hunk`, ~line 176)
- Modify: `src-tauri/src/main.rs:37` (после `commands::discard_hunk,`)

- [ ] **Step 1: Добавить команду в `commands.rs`**

После функции `discard_hunk` в `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn apply_lines(
    repo_path: String,
    file_header: String,
    hunks: Vec<mutation::LineHunkSelection>,
    op: mutation::LineOp,
) -> Result<(), String> {
    mutation::apply_lines(Path::new(&repo_path), &file_header, &hunks, op)
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Зарегистрировать в `main.rs`**

В `src-tauri/src/main.rs`, в `tauri::generate_handler![ ... ]`, сразу после строки `commands::discard_hunk,` добавить:

```rust
            commands::apply_lines,
```

- [ ] **Step 3: Проверить сборку backend**

Run: `cd src-tauri && cargo build`
Expected: компилируется без ошибок и предупреждений о неиспользуемом коде.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "feat(partial-stage): IPC-команда apply_lines"
```

---

## Task 5: Типы и composable на фронте

**Files:**
- Modify: `src/types/index.ts` (после `FileDiff`, ~line 82)
- Modify: `src/composables/useFiles.ts`

- [ ] **Step 1: Добавить типы**

В `src/types/index.ts` после интерфейса `FileDiff`:

```ts
export type LineOp = "stage" | "unstage" | "discard";

export interface LineHunkSelection {
  raw: string;
  selected: number[];
}
```

- [ ] **Step 2: Добавить `applyLines` в `useFiles`**

В `src/composables/useFiles.ts` добавить функцию рядом с `applyHunk` и экспортировать её из `return`:

```ts
  async function applyLines(
    op: "stage" | "unstage" | "discard",
    fileHeader: string,
    hunks: { raw: string; selected: number[] }[],
  ) {
    if (!repoPath.value) return;
    if (hunks.length === 0) return;
    const { reloadDiff } = useDiff();
    await invoke("apply_lines", {
      repoPath: repoPath.value,
      fileHeader,
      hunks,
      op,
    });
    await refresh();
    await reloadDiff();
  }
```

В объекте `return { ... }` добавить `applyLines,` рядом с `stageHunk, unstageHunk, discardHunk`.

- [ ] **Step 3: Проверить типы/сборку**

Run: `npm run build`
Expected: сборка успешна (vue-tsc без ошибок).

- [ ] **Step 4: Commit**

```bash
git add src/types/index.ts src/composables/useFiles.ts
git commit -m "feat(partial-stage): тип LineHunkSelection и useFiles.applyLines"
```

---

## Task 6: Состояние выделения строк в SideBySideDiffView

**Files:**
- Modify: `src/components/SideBySideDiffView.vue`

> Инвариант: порядок `hunk.lines` совпадает с порядком тела `hunk.raw` (`parse_diff_single` строит `lines` и `raw` в одном проходе; `enrichAllHunks` лишь добавляет `wordDiffs`, порядок сохраняет). Поэтому ordinal среди `+/-` строк в `hunk.lines` == ordinal в backend.

- [ ] **Step 1: Добавить состояние и хелперы в `<script setup>`**

В `src/components/SideBySideDiffView.vue` в секцию `<script setup>` добавить (после `const busyHunk = ref<number | null>(null);`):

```ts
import { watch } from "vue";

const { applyLines } = useFiles();

// id строки = "<hunkIdx>:<lineIdxInHunkLines>"
const selectedLines = ref<Set<string>>(new Set());
const selAnchor = ref<string | null>(null);

const hasSelection = computed(() => selectedLines.value.size > 0);

function isSelectable(kind: string) {
  return kind === "added" || kind === "removed";
}

// Плоский список выделяемых строк в порядке отображения (для Shift-диапазона).
const selectableFlat = computed(() => {
  const arr: string[] = [];
  enrichedHunks.value.forEach((h, hi) => {
    h.lines.forEach((ln, li) => {
      if (isSelectable(ln.kind)) arr.push(`${hi}:${li}`);
    });
  });
  return arr;
});

function clearSelection() {
  selectedLines.value = new Set();
  selAnchor.value = null;
}

function onLineClick(hi: number, li: number, kind: string, ev: MouseEvent) {
  if (!isSelectable(kind)) return;
  const id = `${hi}:${li}`;
  if (ev.shiftKey && selAnchor.value) {
    const flat = selectableFlat.value;
    const a = flat.indexOf(selAnchor.value);
    const b = flat.indexOf(id);
    if (a !== -1 && b !== -1) {
      const [lo, hiIdx] = a <= b ? [a, b] : [b, a];
      selectedLines.value = new Set(flat.slice(lo, hiIdx + 1));
    }
  } else if (ev.ctrlKey || ev.metaKey) {
    const next = new Set(selectedLines.value);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedLines.value = next;
    selAnchor.value = id;
  } else {
    selectedLines.value = new Set([id]);
    selAnchor.value = id;
  }
}

// Группирует выделение по хункам, считая ordinal среди +/- строк хунка.
function buildSelectionPayload(): { raw: string; selected: number[] }[] {
  const result: { raw: string; selected: number[] }[] = [];
  enrichedHunks.value.forEach((h, hi) => {
    let ord = -1;
    const selected: number[] = [];
    h.lines.forEach((ln, li) => {
      if (isSelectable(ln.kind)) {
        ord++;
        if (selectedLines.value.has(`${hi}:${li}`)) selected.push(ord);
      }
    });
    if (selected.length > 0 && currentDiff.value) {
      result.push({ raw: currentDiff.value.hunks[hi].raw, selected });
    }
  });
  return result;
}

// Сброс выделения при смене файла/контекста.
watch([currentDiff, diffContext], clearSelection);
```

- [ ] **Step 2: Проверить сборку (промежуточно)**

Run: `npm run build`
Expected: успешно (новый код пока не используется в шаблоне — допустимо, без ошибок типов).

- [ ] **Step 3: Commit**

```bash
git add src/components/SideBySideDiffView.vue
git commit -m "feat(partial-stage): состояние выделения строк в SideBySideDiffView"
```

---

## Task 7: Обработчик действий над выделением + привязка кнопок

**Files:**
- Modify: `src/components/SideBySideDiffView.vue`

- [ ] **Step 1: Добавить обработчик действий над выделением**

В `<script setup>` `SideBySideDiffView.vue` добавить после `buildSelectionPayload`:

```ts
async function onSelectionAction(action: "stage" | "unstage" | "discard") {
  if (!hasSelection.value || busyHunk.value !== null) return;
  const payload = buildSelectionPayload();
  if (payload.length === 0) return;
  if (action === "discard") {
    const n = selectedLines.value.size;
    if (!window.confirm(`Отменить изменения в выбранных строках (${n})? Это действие необратимо.`)) {
      return;
    }
  }
  busyHunk.value = -1; // блокирует все кнопки на время операции
  try {
    await applyLines(action, currentDiff.value?.header ?? "", payload);
    clearSelection();
  } catch (e) {
    console.error("Selection action failed:", e);
  } finally {
    busyHunk.value = null;
  }
}

// Маршрутизатор: при наличии выделения — по строкам, иначе — по хунку.
function onStageBtn(hi: number) {
  if (hasSelection.value) onSelectionAction("stage");
  else onHunkAction("stage", hi);
}
function onUnstageBtn(hi: number) {
  if (hasSelection.value) onSelectionAction("unstage");
  else onHunkAction("unstage", hi);
}
function onDiscardBtn(hi: number) {
  if (hasSelection.value) onSelectionAction("discard");
  else onHunkAction("discard", hi);
}
```

- [ ] **Step 2: Перепривязать кнопки в шаблоне**

В `<template>` `SideBySideDiffView.vue` заменить три обработчика кнопок:

- `@click="onHunkAction('stage', hi)"` → `@click="onStageBtn(hi)"`
- `@click="onHunkAction('discard', hi)"` → `@click="onDiscardBtn(hi)"`
- `@click="onHunkAction('unstage', hi)"` → `@click="onUnstageBtn(hi)"`

- [ ] **Step 3: Добавить клик по строкам + счётчик выделения**

В `<template>`, в блоке `New Version`, на элемент строки добавить обработчик и класс выделения. Заменить:

```html
          <div
            v-for="line in hunk.lines"
            :key="`${line.content}-${line.kind}-new`"
            class="diff-line"
            :class="[line.kind, { hidden: line.kind === 'removed' }]"
          >
```

на (используем индекс строки `li`):

```html
          <div
            v-for="(line, li) in hunk.lines"
            :key="`${line.content}-${line.kind}-new`"
            class="diff-line"
            :class="[
              line.kind,
              {
                hidden: line.kind === 'removed',
                'line-selected': selectedLines.has(`${hi}:${li}`),
                selectable: isSelectable(line.kind),
              },
            ]"
            @click="onLineClick(hi, li, line.kind, $event)"
          >
```

Аналогично в блоке `Old Version` заменить:

```html
          <div
            v-for="line in hunk.lines"
            :key="`${line.content}-${line.kind}-old`"
            class="diff-line"
            :class="[line.kind, { hidden: line.kind === 'added' }]"
          >
```

на:

```html
          <div
            v-for="(line, li) in hunk.lines"
            :key="`${line.content}-${line.kind}-old`"
            class="diff-line"
            :class="[
              line.kind,
              {
                hidden: line.kind === 'added',
                'line-selected': selectedLines.has(`${hi}:${li}`),
                selectable: isSelectable(line.kind),
              },
            ]"
            @click="onLineClick(hi, li, line.kind, $event)"
          >
```

В шапку панели (`.diff-actions`, рядом с `hunk-counter`) добавить индикатор выделения перед навигационными кнопками:

```html
        <span class="sel-counter" v-if="hasSelection">
          выбрано {{ selectedLines.size }}
        </span>
```

- [ ] **Step 4: Добавить CSS**

В `<style scoped>` `SideBySideDiffView.vue` добавить:

```css
.diff-line.selectable {
  cursor: pointer;
}

.diff-line.line-selected {
  background: var(--accent-soft, rgba(137, 180, 250, 0.25));
  box-shadow: inset 2px 0 0 var(--blue, #89b4fa);
}

.sel-counter {
  font-size: var(--font-size-xs);
  color: var(--blue, #89b4fa);
  user-select: none;
}
```

- [ ] **Step 5: Проверить сборку**

Run: `npm run build`
Expected: успешно, без ошибок типов/шаблона.

- [ ] **Step 6: Commit**

```bash
git add src/components/SideBySideDiffView.vue
git commit -m "feat(partial-stage): выделение строк кликом и действия Stage/Unstage/Discard"
```

---

## Task 8: Bump версии

**Files:**
- Modify: `src/components/StatusBar.vue:49`

- [ ] **Step 1: Поднять patch-версию**

В `src/components/StatusBar.vue` строка 49: заменить `<span class="version">0.1.37</span>` на `<span class="version">0.1.38</span>`.

- [ ] **Step 2: Commit**

```bash
git add src/components/StatusBar.vue
git commit -m "chore: bump версии 0.1.38 (частичный stage строк)"
```

---

## Task 9: Финальная проверка и ручной чеклист

**Files:** (без изменений кода — только верификация)

- [ ] **Step 1: Полная сборка backend + тесты**

Run: `cd src-tauri && cargo test && cargo build`
Expected: все тесты PASS, сборка без предупреждений о неиспользуемом коде.

- [ ] **Step 2: Полная сборка frontend**

Run: `npm run build`
Expected: успешно, без ошибок.

- [ ] **Step 3: Ручной чеклист (запустить `npm run tauri dev`)**

Проверить в реальном репозитории с изменениями:

- [ ] Файл с unstaged-изменениями: клик по `+`-строке выделяет её (подсветка + полоса слева), курсор pointer.
- [ ] Shift-клик по другой `+/-`-строке выделяет диапазон (в т.ч. через границу хунков).
- [ ] Ctrl/Cmd-клик добавляет/убирает отдельную строку, не сбрасывая остальные.
- [ ] Счётчик «выбрано N» в шапке отражает число строк.
- [ ] Кнопка **Stage** при выделении: в индекс попадают только выбранные строки (проверить `git diff --cached`); невыбранные изменения остаются в рабочем дереве.
- [ ] Контекст `staged` (клик по staged-файлу): кнопка **Unstage** убирает из индекса только выбранные строки.
- [ ] Кнопка **Discard** при выделении: показывает `window.confirm`; после подтверждения выбранные строки откатываются в рабочем дереве, остальные изменения целы.
- [ ] Без выделения кнопки Stage/Unstage/Discard работают по хунку целиком (регрессия не сломана).
- [ ] Выделение сбрасывается при смене файла и после успешной операции.
- [ ] Файл без финального перевода строки (`\ No newline at end of file`): частичный stage не ломает файл (содержимое индекса корректно).
- [ ] Просмотр коммита (`diffContext === 'commit'`): кнопок действий нет, клики по строкам безвредны.

- [ ] **Step 2 → если все пункты пройдены: финальный коммит при необходимости**

Если ручной чеклист потребовал правок — закоммитить их отдельным коммитом с описанием. Иначе задача завершена.

---

## Self-Review

**1. Spec coverage:**
- §1 Архитектура/поток — Tasks 4–7 (команда + composable + UI).
- §2 Бэкенд синтез патча (таблица трансформации, заголовок, пропуск пустых, маппинг op→флаги) — Tasks 1–3 с юнит-тестами на каждый случай таблицы.
- §3 Фронтенд выделение (клик/Shift/Ctrl, только изменённые строки, сброс, подсветка, Discard с подтверждением, мульти-хунк) — Tasks 6–7. Отклонение: `window.confirm` вместо Vue-`ConfirmDialog` — задокументировано в шапке плана.
- §4 Крайние случаи (`git apply` упал → existing classify_git_error/reloadDiff; пустое выделение → по хунку; `\ No newline`; CRLF — байт-в-байт из raw; бинарные — нет +/- строк; гонки — existing sequence-guard) — покрыто тестами Task 2/3 и логикой Task 5/7; ручной чеклист Task 9.
- §5 Тестирование — все 8 классов тестов спеки распределены: смешанный (T2), только `+` (T2), только `-` (T2), пересчёт `@@`/невыбранные (T2), `\ No newline` (T2, 2 теста), мульти-хунк/пустой (T3), хвост заголовка (T1/T2), интеграционный stage (T3). Bump версии — T8.

**2. Placeholder scan:** плейсхолдеров нет; весь код приведён целиком.

**3. Type consistency:** `LineHunkSelection { raw, selected }` и `LineOp` (`stage|unstage|discard`, serde lowercase) согласованы Rust↔TS. Tauri camelCase: `repo_path`↔`repoPath`, `file_header`↔`fileHeader` (как в существующем коде). `applyLines(op, fileHeader, hunks)` — единая сигнатура в useFiles и вызове из компонента. `busyHunk` переиспользуется (значение `-1` как «занято выделением») — совместимо с существующими `:disabled="busyHunk !== null"`.

---

## Execution Handoff

Выбери способ выполнения (см. ниже).
