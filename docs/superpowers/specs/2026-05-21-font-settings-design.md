# Font Settings Design

## Goal

Add 4 font-related settings to the Settings dialog (Appearance category), styled like VS Code.

## New Settings

| ID | Label | Type | Default | Range |
|---|---|---|---|---|
| `workbench-font-family` | Workbench: Font Family | text input | `Ubuntu, -apple-system, BlinkMacSystemFont, sans-serif` | — |
| `workbench-font-size` | Workbench: Font Size | number spinner | `15` | 11–20 |
| `editor-font-family` | Editor: Font Family | text input | `Ubuntu Mono, Courier New, monospace` | — |
| `editor-font-size` | Editor: Font Size | number spinner | `13` | 10–24 |

All 4 appear in the existing **Внешний вид** category, below Color Theme.

## CSS Application

Changes are applied immediately via `document.documentElement.style.setProperty`:

| Setting | CSS variable(s) |
|---|---|
| Workbench Font Family | `--font-sans` |
| Workbench Font Size N | `--font-size: Npx`, `--font-size-sm: (N-2)px`, `--font-size-xs: (N-3)px` |
| Editor Font Family | `--font-mono` |
| Editor Font Size | `--font-size-diff` (new variable) |

`DiffView.vue` switches from `--font-size-sm` to `--font-size-diff`.

## Persistence

### Rust — `settings.rs`

```rust
pub struct AppSettings {
    pub network_timeout_secs: u64,
    pub workbench_font_family: String,
    pub workbench_font_size: u8,   // 11–20
    pub editor_font_family: String,
    pub editor_font_size: u8,      // 10–24
}
```

Defaults: font families match current CSS defaults; sizes 15 and 13.  
Serialized with `serde(default = ...)` so existing `settings.json` without these fields still loads correctly.

### Frontend — `useSettings.ts`

4 new `ref`s exposed. Each watched with 300 ms debounce before `invoke("set_settings", ...)`.  
On load: CSS variables are applied immediately after `get_settings` resolves.

## Controls (VS Code style)

**Font Family** — full-width `<input type="text">` (320 px wide, same as existing `vs-select`).

**Font Size** — `<input type="number">` with custom increment buttons (▲▼), value display, no native browser spinner. Width ~100 px.

## Files Changed

1. `src-tauri/src/settings.rs` — add 4 fields to `AppSettings`, update `Default`
2. `src/composables/useSettings.ts` — add 4 refs, watchers, CSS application
3. `src/components/dialogs/SettingsDialog.vue` — add 4 settings items + controls
4. `src/components/DiffView.vue` — switch monospace font-size to `--font-size-diff`
