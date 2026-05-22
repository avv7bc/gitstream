# Поиск/фильтрация в логе коммитов — Design

**Status:** Draft
**Date:** 2026-05-21
**Scope:** MVP серверного поиска по логу + задел под расширенные опции

---

## Проблема

В `CommitGraph` сейчас есть поле "Filter" (`graphFilter`), которое фильтрует
**только локально загруженные коммиты** (по умолчанию первые 500, через
infinite scroll — больше, но в пределах загруженного). На репозитории с
тысячами коммитов поиск не находит то, что находится за пределами
загруженной страницы.

При активном фильтре infinite scroll отключён, поэтому пользователь не может
"догрузить пока не найдётся".

## Цель

Поиск должен работать по **всей истории** репозитория. Ввод в поле "Filter"
должен делать `git log --grep=… --author=… -i` через бэк и возвращать
совпадающие коммиты независимо от того, сколько страниц лога уже загружено.

UX поля поиска визуально не меняется в MVP — то же поле, тот же placeholder.
Меняется только семантика: поиск стал серверным и охватывает весь репозиторий.

## Out of scope (MVP)

Расширенные опции (отдельные чекбоксы Message/Author/File/Content, regex,
case-sensitivity, фильтр по пути, поиск по diff через `-S`/`-G`) — отдельный
следующий шаг. Архитектура этого MVP должна допускать их добавление без
переписывания.

## Архитектура

### Backend (Rust)

Новая структура `LogFilter` в `git/types.rs`:

```rust
#[derive(Serialize, Deserialize, Default)]
pub struct LogFilter {
    pub query: String,
    // Заделы под расширение (в MVP игнорируются или = default):
    // pub case_sensitive: bool,
    // pub regex: bool,
    // pub fields: Vec<LogField>,  // [Message, Author, File, Content]
    // pub path: Option<String>,
}
```

В MVP сериализуем только `query`. Остальные поля добавим, когда понадобятся
(serde позволяет добавлять опциональные поля без поломки совместимости).

`query::log()` принимает дополнительный параметр `filter: Option<&LogFilter>`:

```rust
pub fn log(
    repo_path: &Path,
    limit: usize,
    filter: Option<&LogFilter>,
) -> Result<Vec<CommitInfo>, GitError>
```

Если `filter` отсутствует или `query` пустой → текущая логика без изменений.

Если `query` непустой:

1. Собираем аргументы `git log`:
   - `--format=…` (как сейчас)
   - `--grep=<Q>` — поиск в теле сообщения
   - `--author=<Q>` — поиск в авторе (имя + email)
   - `--regexp-ignore-case` — case-insensitive (MVP — всегда вкл)
   - `--extended-regexp` — расширенный regex
   - **БЕЗ `--all-match`** — нужен OR между grep и author, не AND
2. **Hash-префикс:** если query является валидным hex-префиксом
   (`^[0-9a-fA-F]{3,40}$`), отдельно вызываем `git log -1 <Q> --format=…`.
   Если возвращает коммит — кладём его первым в результат (дедуп по oid).
3. `--max-count=<limit>` — общий лимит совпадений (по умолчанию 500, при
   поиске можно увеличить — см. ниже).
4. После получения коммитов снова прогоняем `super::graph::assign_lanes()`.

**Безопасность шелла:** аргументы передаются через `&[&str]` в `run_git`,
без шелл-интерполяции — инъекции невозможны. Спецсимволы regex в
`--grep`/`--author` интерпретируются git'ом как расширенный regex, что
приемлемо для MVP (расширенный regex — частично intuitive; см. известные
ограничения ниже).

### Tauri command

`commands::get_log` принимает дополнительный аргумент:

```rust
#[tauri::command]
pub async fn get_log(
    repo_path: String,
    limit: Option<usize>,
    filter: Option<LogFilter>,
) -> Result<Vec<CommitInfo>, String>
```

Обратная совместимость: оба новых поля опциональны, текущие вызовы без
изменений работают.

### Frontend (Vue)

**`useLog.ts`** — добавить:

- `filter: Ref<string>` — текущая строка фильтра (module-global, как `commits`)
- `isFiltering: ComputedRef<boolean>` — `filter.value.trim() !== ""`
- При изменении `filter` (с debounce 300ms) → `refresh()`
- `refresh()` передаёт `filter: { query: filter.value }` в invoke, когда
  фильтр непустой
- При активном фильтре `loadMore` отключается полностью (он и сейчас
  отключён через `!graphFilter`, но логика перенесётся в useLog)
- При `clear()` (смена репо) — сбрасывать `filter` тоже

**`CommitGraph.vue`** — упростить:

- Удалить локальный `graphFilter` ref и `filteredCommits` computed
- Привязать input к `useLog().filter`
- Везде, где используется `filteredCommits` (включая range selection,
  навигацию стрелками), использовать `commits` напрямую — фильтрация теперь
  серверная, и `commits` уже содержит только совпадения
- `isUnpushed(idx)` — оставить как есть (работает по `firstRemoteIdx`,
  который теперь считается по фильтрованной выборке — это корректно, т.к.
  отображаются только эти коммиты)
- Подсветка `highlight()` остаётся, использует `useLog().filter`
- Debounce 300ms — реализовать в watcher `useLog`, не в компоненте

### Debounce и race conditions

Существующий `refreshSeq` уже защищает от race между перекрывающимися
refresh'ами — он покрывает и filter-driven refresh без изменений.

Debounce реализуем как простой `setTimeout`-based в `useLog`:

```ts
let filterDebounce: ReturnType<typeof setTimeout> | null = null;
watch(filter, () => {
  if (filterDebounce) clearTimeout(filterDebounce);
  filterDebounce = setTimeout(() => { refresh(); }, 300);
});
```

При очистке поля refresh всё равно идёт через debounce — это даёт
консистентный UX без специальных веток.

### Лимит результатов при поиске

При обычном `refresh()` лимит = `Math.max(commits.length, PAGE_SIZE)` (текущая
логика).

При активном фильтре нет смысла "копить" предыдущие коммиты — это другой
набор. Используем фиксированный лимит для поиска: **1000**. Если совпадений
больше — индикатор в UI ("+ more matches, refine your filter").

Реализация:
```ts
const target = isFiltering.value ? 1000 : Math.max(commits.length, PAGE_SIZE);
```

### Подсветка совпадений

`highlight()` остаётся — он подсвечивает substring совпадения query в
message/author/refs/oid/date. Поскольку у git `--grep` regex-семантика, для
визуальной подсветки в MVP используем простой substring (как сейчас).
Расхождение «нашли по regex, но не подсветили» — приемлемое ограничение
MVP.

## Поток данных

```
[User types in Filter input]
       ↓ v-model
   useLog.filter
       ↓ watch + debounce 300ms
   refresh()
       ↓ invoke("get_log", { repoPath, limit: 1000, filter: { query } })
   Tauri command get_log
       ↓ spawn_blocking
   query::log(path, 1000, Some(filter))
       ↓
   run_git(["log", "--grep=Q", "--author=Q", "-i", "--extended-regexp", "--max-count=1000", "--format=…"])
       ↓
   [hash-prefix add-on if applicable]
       ↓
   assign_lanes()
       ↓
   Vec<CommitInfo> → JSON → frontend
       ↓
   commits.value = data
       ↓
   CommitGraph renders + highlight()
```

## Обработка ошибок

- Если `query` содержит невалидный regex (например, `[`) → git вернёт
  ошибку. Ловим в `run_git` и в `useLog.refresh()` показываем "Invalid
  filter pattern" вместо пустого списка. Существующий `classify_git_error`
  не покрывает этот случай — добавим минимально (по подстроке "fatal:" +
  "regex").

- Сетевые ошибки тут нерелевантны — операция локальная.

## Безопасность

- Аргументы передаются массивом, не строкой — нет шелл-инъекции.
- Regex DOS теоретически возможен (catastrophic backtracking в очень
  патологических шаблонах), но это локальный процесс пользователя — git
  его убьёт по таймауту, если зависнет. Дополнительной защиты не нужно.

## Тестирование

- Ручной smoke-тест на репо gitstream (≈30 коммитов): фильтр находит
  существующие коммиты по message/author/hash.
- Ручной smoke-тест на большом репо (>5000 коммитов): фильтр работает по
  всей истории, не только по первой странице.
- Проверка debounce: быстрое нажатие клавиш не вызывает множественные git
  процессы (счётчик в DevTools Network / `refreshSeq` логирование).
- Очистка поля → возврат к paged log с возможностью infinite scroll.
- Hash-префикс: ввод 7-символьного префикса найденного коммита возвращает
  его как первый элемент.
- Невалидный regex → понятное сообщение об ошибке, не падение.

Автоматизированных тестов в проекте нет (нет test runner'а), поэтому пишем
ручной smoke checklist.

## Известные ограничения

1. **Lane allocation на фильтрованной выборке.** `assign_lanes` рассчитывает
   колонки и линии графа на основе родительских связей внутри выборки. Если
   родители коммита отфильтрованы, линии графа будут выглядеть «обрывисто».
   Это приемлемо для MVP — пользователь понимает, что видит подмножество.
2. **Regex по умолчанию.** Git `--grep` — расширенный regex; пользователь,
   вводящий `(fix)`, может неожиданно получить regex-семантику. В
   расширенных опциях добавим toggle "literal/regex" с `--fixed-strings`.
3. **Подсветка по substring, не regex.** При regex-поиске подсветка может не
   находить совпадение визуально. Принято как ограничение MVP.
4. **Working Tree row** в `CommitGraph` остаётся всегда — он не относится к
   фильтру.

## Дальнейшие шаги (вне MVP)

- Раскрывающаяся панель опций под полем фильтра:
  - Чекбоксы: Message / Author / File path / Content (`-S` или `-G`)
  - Toggle: Case sensitive
  - Toggle: Regex / Literal
  - Поле: Path filter (`-- <path>`)
- Сохранение последних фильтров (history) в settings.
- Подсветка по regex (использовать тот же regex, что git, через JS RegExp с
  POSIX-flavor mapping или мини-парсер).
