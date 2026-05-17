# Публикация GitStream в Flathub

Дата: 2026-05-17. Статус: в работе (Этап 1).

## Ключевые решения

1. **App ID меняется** `com.gitstream.app` → `io.github.avv7bc.gitstream`
   (Flathub требует контроль над доменом; используем code-hosting префикс
   `io.github.<user>.<repo>`, репозиторий `github.com/avv7bc/gitstream` доступен).
2. **Песочница + Git — РЕШЕНО** (эталон: `flathub/com.github.git_cola.git-cola`,
   тоже Git-клиент на CLI `git`):
   - `git` собирается модулем из tarball kernel.org (`flatpak/git.json`);
     рантайм GNOME уже даёт curl/openssl/zlib/expat — доп. модули не нужны.
   - `--filesystem=home` (узкий scope, предпочитаемый Flathub; репозитории
     обычно в `$HOME`). Расширять до `--filesystem=host` только при
     необходимости. Плюс `--share=network`, `--socket=ssh-auth`,
     `--socket=gpg-agent` для remote/SSH/подписи.

## Этапы

### Этап 1. Подготовка репозитория (текущий)
- [x] План сохранён
- [ ] `identifier` в tauri.conf.json → `io.github.avv7bc.gitstream`, патч-бамп
- [ ] `flatpak/io.github.avv7bc.gitstream.desktop`
- [ ] `flatpak/io.github.avv7bc.gitstream.metainfo.xml` (нужны реальные скриншоты!)
- [ ] иконка с именем app-id (≥256×256 PNG / SVG)
- [ ] скелет манифеста `flatpak/io.github.avv7bc.gitstream.yml` + packaging README

### Этап 2. Offline-источники
- `flatpak-node-generator npm package-lock.json -o flatpak/generated-sources-node.json`
- `flatpak-cargo-generator src-tauri/Cargo.lock -o flatpak/generated-sources-cargo.json`
- (инфраструктура Flathub собирает без сети — зависимости вендорятся заранее)

### Этап 3. Финализация манифеста
- runtime `org.gnome.Platform` (актуальная поддерживаемая версия; Tauri-гайд → 46)
- sdk-extensions: `rust-stable`, `node20`
- finish-args: `--socket=wayland --socket=fallback-x11 --device=dri` + решение по ФС/Git
- установка LICENSE в `$FLATPAK_DEST/share/licenses/$FLATPAK_ID`

### Этап 4. Локальная проверка
```
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46
flatpak run org.flatpak.Builder --force-clean --sandbox --user \
  --install --repo=repo builddir flatpak/io.github.avv7bc.gitstream.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest <manifest>
flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo repo
flatpak run io.github.avv7bc.gitstream
```
Линтер обязан проходить без ошибок.

### Этап 5. Сабмишн
- форк `github.com/flathub/flathub`, клон ветки `--branch=new-pr`
- PR в base-ветку **`new-pr`** (не master), заголовок `Add io.github.avv7bc.gitstream`
- заполнить чек-лист (автор проекта; минимум permissions; ассеты 0BSD — редистрибутивны)
- правки в тот же PR; `bot, build` после закрытия замечаний

### Этап 6. После одобрения
- Flathub создаёт `flathub/io.github.avv7bc.gitstream`, инвайт принять за неделю
- включить 2FA на GitHub
- скриншоты/листинг появляются после мержа

### Этап 7. Обновления
- bump версии + новый `<release>` в metainfo
- перегенерировать generated-sources (если менялись зависимости)
- PR в `flathub/io.github.avv7bc.gitstream`

## Открытые вопросы
- ~~Изоляция Git~~ — РЕШЕНО (git-модуль + `--filesystem=home`, см. выше).
- Актуальная поддерживаемая версия GNOME runtime на момент сабмишна.
- Реальные скриншоты для metainfo (минимум 1, на стабильном хостинге).
- Имя/идентичность разработчика для `<developer>` в metainfo.
- Подтянуть актуальную версию git в `git.json` на момент сабмишна
  (сейчас 2.54.0, как у git-cola; x-checker-data обновит автоматически).
