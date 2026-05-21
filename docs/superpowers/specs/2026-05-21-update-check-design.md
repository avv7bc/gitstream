# Update Check — Design Spec

**Date:** 2026-05-21  
**Status:** Approved

---

## Overview

При запуске GitStream однократно проверяет GitHub Releases API на наличие новой версии. Если текущая версия устарела — показывает ненавязчивый плавающий баннер снизу-справа с предложением загрузить обновление.

---

## Architecture

```
App.vue (onMounted)
  └─ useUpdate.ts
       └─ invoke("check_for_update")
            └─ commands.rs: check_for_update()
                 └─ reqwest → api.github.com/repos/avv7bc/gitstream/releases/latest
                      └─ compare semver → return UpdateInfo | null
  └─ UpdateBanner.vue (v-if="updateInfo")
```

---

## Components

### Rust: `check_for_update` command

**Файл:** `src-tauri/src/commands.rs`  
**Зависимость:** `reqwest` с features `["json", "rustls-tls"]` (добавить в `Cargo.toml`)

```
async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String>
```

- Запрашивает `https://api.github.com/repos/avv7bc/gitstream/releases/latest`
- Парсит поле `tag_name` (формат `v0.4.2`) → версия строка
- Получает текущую версию через `app.config().version`
- Сравнивает строковым semver: если latest > current → возвращает `UpdateInfo`
- При любой ошибке сети/парсинга → возвращает `Ok(None)` (тихий фейл)

**Тип `UpdateInfo`** (добавить в `src-tauri/src/git/types.rs`):
```rust
pub struct UpdateInfo {
    pub version: String,       // "0.4.2"
    pub release_url: String,   // HTML URL релиза на GitHub
    pub changelog_url: String, // то же самое (releases/tag/v0.4.2)
}
```

**Сравнение версий:** разбиваем по точке → `Vec<u32>`, лексикографическое сравнение. Без внешних semver-крейтов.

---

### Composable: `useUpdate.ts`

**Файл:** `src/composables/useUpdate.ts`

```typescript
const updateInfo = ref<UpdateInfo | null>(null)
async function checkForUpdate(): Promise<void>  // вызывает invoke, пишет в updateInfo
```

- Вызывается один раз в `App.vue` в `onMounted`
- Ошибка invoke → тихо игнорируется, `updateInfo` остаётся `null`

---

### Component: `UpdateBanner.vue`

**Файл:** `src/components/UpdateBanner.vue`

```
┌────────────────────────────────────────────┐
│ ↑  Обновление GitStream доступно           │
│    (v0.4.2)                                │
│    Список изменений >                      │
│                                            │
│  [Загрузить]           [Отменить]          │
└────────────────────────────────────────────┘
```

- Позиция: `position: fixed; bottom: 24px; right: 24px`
- `z-index` выше панелей, ниже модальных диалогов
- «Список изменений >» → `window.__TAURI__.opener.openUrl()` через `@tauri-apps/plugin-opener` (если плагин уже есть) или новая Rust-команда `open_url(url)` через `std::process::Command` (`xdg-open`/`open`/`start`)
- «Загрузить» → открывает `release_url` в браузере, скрывает баннер
- «Отменить» → скрывает баннер (до следующего запуска)
- Стиль: соответствует существующей тёмной теме (`--bg-secondary`, `--text-primary`)

---

## Error Handling

- Ошибка сети при проверке → `Ok(None)`, баннер не показывается
- Таймаут запроса → 5 секунд
- Некорректный формат тега → трактуем как "нет обновления"

---

## Scope

**В рамках этой задачи:**
- Проверка при старте, баннер, открытие браузера

**За рамками (будущее):**
- Автоматическое скачивание/установка (tauri-plugin-updater)
- Проверка по расписанию / кнопка "Проверить обновления" в Settings
- Отключение уведомлений в настройках
