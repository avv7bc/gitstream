# Дизайн: Ref-чипы в CommitGraph (иконки, текущая ветка, ahead/behind)

**Дата:** 2026-05-16
**Статус:** Утверждён к планированию (фича A из разбиения CommitGraph-улучшений)

## Цель

Привести ref-метки коммитов в `CommitGraph` к виду как на референс-скриншоте:
иконка в чипе (ветка/remote/тег/stash), визуально выделенная текущая ветка
(`HEAD -> main`), и бейджи `+N` / `−N` (ahead/behind текущей ветки) у её HEAD-коммита.

Lane-линии графа — отдельная фича B, вне этой спеки.

## Текущее состояние

- `CommitGraph.vue` уже рендерит `commit.refs` как цветные текстовые чипы
  (`refClass` → `ref-local-branch`/`ref-remote-branch`/`ref-tag`/`ref-head`/`ref-stash`).
- Иконок в чипах нет. SVG-иконки веток/remote/тега/stash есть только в `BranchPanel.vue`
  (инлайн, дублируются по секциям).
- Бэкенд `parse_ref_labels` отбрасывает префикс `HEAD -> ` → текущая ветка
  неотличима от обычной локальной (kind `local-branch`). Standalone `HEAD`
  (detached) → kind `head`.
- ahead/behind есть только для текущей ветки через `useBranches().branches`
  (поля `ahead`/`behind`), используется в `StatusBar.vue`. Пер-коммит данных нет.

## Объём

В рамках фичи A:
- Иконки в ref-чипах (переиспользование иконок BranchPanel через общий компонент).
- Визуальное выделение текущей ветки (новый kind `current-branch`).
- Бейджи `+N` (ahead) и `−N` (behind) у HEAD-коммита текущей ветки.

Вне объёма: lane-линии графа; пер-коммит ahead/behind; изменения формата git-лога.

## Архитектура

Подход: бэкенд помечает текущую ветку отдельным kind; иконки выносятся в общий
компонент `RefIcon.vue` (единый источник, BranchPanel рефакторится на него);
бейджи ahead/behind — синтетические чипы из существующего `useBranches()`.

### 1. Бэкенд + типы

**`src-tauri/src/git/query.rs` — `parse_ref_labels`:** при разборе декорации
git (`%D`) строка вида `HEAD -> main` даёт `RefLabel { name: "main",
kind: "current-branch" }`. Standalone `HEAD` остаётся kind `head`. Остальная
логика (`tag: ` → tag, наличие `/` → remote-branch, иначе local-branch) — без изменений.

```rust
// внутри filter_map по r:
if r == "HEAD" { return Some(RefLabel { name: "HEAD".into(), kind: "head".into() }); }
if let Some(rest) = r.strip_prefix("HEAD -> ") {
    return Some(RefLabel { name: rest.to_string(), kind: "current-branch".into() });
}
if let Some(t) = r.strip_prefix("tag: ") {
    return Some(RefLabel { name: t.to_string(), kind: "tag".into() });
}
if r.contains('/') {
    Some(RefLabel { name: r.to_string(), kind: "remote-branch".into() })
} else {
    Some(RefLabel { name: r.to_string(), kind: "local-branch".into() })
}
```

`src-tauri/src/git/types.rs`: `RefLabel.kind: String` — структура не меняется.

**`src/types/index.ts`:** union расширяется значением `"current-branch"`:
```ts
kind: "local-branch" | "remote-branch" | "tag" | "head" | "stash" | "current-branch";
```

### 2. Общий компонент `RefIcon.vue`

`src/components/RefIcon.vue` — props `{ kind: RefLabel["kind"] }`. Рендерит инлайн
SVG (~12×12, `currentColor`, `flex-shrink:0`), перенесённый из `BranchPanel.vue`:

| kind | иконка |
|---|---|
| `local-branch`, `current-branch` | branch (две точки + соединитель) |
| `remote-branch` | remote (круг + основание) |
| `tag` | tag (ярлык + отверстие) |
| `stash` | stash (стопка) |
| `head` | branch (та же, что для веток) |

`BranchPanel.vue` рефакторится: инлайн-SVG в секциях Local/Remote/Tags/Stashes
заменяются на `<RefIcon :kind="..." />`; существующие CSS-классы цвета сохраняются.
Это убирает дублирование и даёт единый источник иконок.

### 3. CommitGraph: чипы + бейджи

**`src/components/CommitGraph.vue`:**

- `refClass(r)`: `current-branch` → `"ref-label ref-current-branch"` (остальные kind
  как раньше).
- Разметка чипа: вместо одиночного `<span v-html="highlight(r.name)">` —
  `<span :class="refClass(r)"><RefIcon :kind="r.kind" /><span v-html="highlight(r.name, graphFilter)" /></span>`.
- Новый CSS `.ref-current-branch`: как `.ref-local-branch`, но ярче и `font-weight:700`
  (насыщенный зелёный фон, выделяется среди обычных локальных веток).
- Бейджи ahead/behind: в скрипте — `const { branches } = useBranches()`,
  `currentBranch = computed(() => branches.value.find(b => b.is_current))`.
  Хелпер `isCurrentBranchRow(commit)` = в `commit.refs` есть kind `current-branch`.
  В `message-col`, перед чипами, для строки где `isCurrentBranchRow`:
  - если `currentBranch.ahead > 0` → `<span class="ref-label ref-ahead" :title="`${n} ahead`">+{{n}}</span>`
  - если `currentBranch.behind > 0` → `<span class="ref-label ref-behind" :title="`${n} behind`">−{{n}}</span>`
  Порядок как на скрине: `+14 ▶ main` (бейджи слева от чипа ветки).
- CSS `.ref-ahead` (зелёный фон/текст), `.ref-behind` (красный) — в стиле существующих
  `.ref-*` классов.

### Data flow

```
git log %D → parse_ref_labels → RefLabel{kind:"current-branch"} (бэкенд)
  → get_log → useLog.commits → CommitGraph row
  → refClass/RefIcon рисуют чип с иконкой и выделением текущей ветки
useBranches().branches → currentBranch (is_current) → ahead/behind
  → бейджи на строке с current-branch ref
```

## Обработка ошибок / краевые случаи

- Detached HEAD: только standalone `HEAD` (kind `head`), нет `current-branch` →
  бейджи не показываются; иконка head = branch.
- Текущая ветка без upstream: `ahead=behind=0` → бейджи скрыты.
- `ahead>0` и `behind>0` одновременно: показываются оба (`+N` затем `−N`).
- Несколько ref на одном коммите (например `current-branch` + `tag`): независимые
  чипы, каждый со своей иконкой.
- wt-row (`__worktree__`): ref'ов нет, бейджей нет — без изменений.
- Фильтр графа (`graphFilter`) и подсветка имени сохраняются (highlight на имени).

## Тестирование

- **Backend (Rust):** unit-тест `parse_ref_labels` в `query.rs` (стиль уже введённых
  `#[cfg(test)]` модулей): `HEAD -> main` → `current-branch`; standalone `HEAD` →
  `head`; `tag: v1.0` → `tag`; `origin/main` → `remote-branch`; `dev` → `local-branch`;
  комбинация `HEAD -> main, tag: v1, origin/main`.
- **Frontend:** проект без FE-тест-раннера → `npm run build` (vue-tsc + vite) +
  ручная проверка: иконки во всех секциях BranchPanel и в чипах графа; текущая
  ветка выделена; `+N`/`−N` у HEAD-коммита текущей ветки; detached HEAD без бейджей.
- Bump patch версии в `src-tauri/tauri.conf.json` (0.1.8 → 0.1.9) — правило проекта.

## Definition of Done

- Текущая ветка в графе визуально отличается от прочих локальных (kind
  `current-branch`, ярче/жирнее).
- Чипы веток/remote/тегов/stash содержат иконку; иконки — из общего `RefIcon.vue`,
  BranchPanel использует тот же компонент (нет дублирования SVG).
- У HEAD-коммита текущей ветки видны бейджи `+N`/`−N` при ненулевых ahead/behind.
- `npm run build` и `cargo test` (вкл. новый тест `parse_ref_labels`) зелёные.
