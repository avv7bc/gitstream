# GitHub Actions Release Pipeline

## Цель

Автоматическая сборка нативных инсталляторов для Linux, macOS и Windows при пуше тега, с загрузкой артефактов в черновой GitHub Release.

## Триггер

Файл: `.github/workflows/release.yml`

Срабатывает только на `push` тегов формата `v*.*.*`. На обычные коммиты и PR не реагирует.

## Структура workflow

### Джоб `create-release`

Выполняется первым. Создаёт черновой (draft) GitHub Release с именем тега. Возвращает `upload_url`, который используют платформенные джобы для загрузки артефактов.

### Джобы matrix (параллельные)

Зависят от `create-release`. Запускаются одновременно после создания release.

| Джоб | Runner | Артефакты |
|------|--------|-----------|
| `build-linux` | `ubuntu-22.04` | `.deb`, `.rpm`, `.AppImage` |
| `build-macos` | `macos-latest` | `.dmg`, `.app.tar.gz` |
| `build-windows` | `windows-latest` | `.msi`, `.exe` (NSIS) |

Каждый джоб:
1. Checkout репозитория
2. Системные зависимости (Linux: `libwebkit2gtk-4.1-dev`, `build-essential`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`)
3. Node.js 20 (`actions/setup-node`)
4. Rust stable (`dtolnay/rust-toolchain`)
5. Кэш Cargo (`Swatinem/rust-cache`) — ускоряет повторные сборки
6. `npm ci` + `npm run tauri build`
7. Загрузка всех найденных артефактов из `src-tauri/target/release/bundle/**` в GitHub Release

## Обнаружение артефактов

На каждой платформе — glob по расширениям:
- Linux: `*.deb`, `*.rpm`, `*.AppImage`
- macOS: `*.dmg`, `*.tar.gz`
- Windows: `*.msi`, `*.exe`

Используется `actions/upload-release-asset` или `softprops/action-gh-release` (проще с glob).

## Интеграция с существующим release.sh

`release.sh` остаётся без изменений. Workflow запускается командой, которую скрипт уже печатает в конце:

```
git push origin v0.2.14
```

Строку `gh release create` из `release.sh` убрать — release теперь создаёт workflow.

## Секреты и права

- `GITHUB_TOKEN` — автоматически доступен во всех Actions, отдельных секретов не нужно
- `permissions: contents: write` в workflow (для создания release и загрузки артефактов)

## Результат

После прогона workflow на странице GitHub Releases появляется черновик с артефактами всех трёх платформ. Автор вручную добавляет описание и публикует.

## Что не входит в скоуп

- Авто-обновления (Tauri Updater) — отдельная задача, требует подписи
- Подпись бинарников (macOS Notarization, Windows Code Signing) — требует платных сертификатов
- Pre-release сборки на каждый PR — не нужно пока нет внешних контрибьюторов
