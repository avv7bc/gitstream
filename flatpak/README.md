# Публикация GitStream в Flathub

Сборка приложения происходит **на серверах Flathub** из манифеста
`io.github.avv7bc.gitstream.yml`. Локально `flatpak-builder` запускать не нужно —
всё проверяется в CI при открытии PR.

App-ID: **`io.github.avv7bc.gitstream`** · упаковка: готовый `.deb` из GitHub Release.

## Файлы

| Файл | Назначение |
|------|-----------|
| `io.github.avv7bc.gitstream.yml` | Манифест Flathub |
| `io.github.avv7bc.gitstream.desktop` | Ярлык приложения |
| `io.github.avv7bc.gitstream.metainfo.xml` | AppStream-метаданные (обяз. для Flathub) |
| `screenshots/main.png` | Скриншот для metainfo |

## Хэши (уже заполнены под релиз v1.1.23)

Все `sha256` в манифесте посчитаны и вписаны:

- `.deb` (`GitStream_1.1.23_amd64.deb`) — `b2a7c9cb…cd6a50`, бинарник внутри `usr/bin/gitstream`
- `git-2.47.1.tar.xz` — `f3d8f9bb…6ed310`
- иконка `256x256.png` — `9f05e87a…074b09`

Пересчитывать нужно только при новой версии `.deb` (см. «Обновления» ниже — это делает бот).

## Порядок публикации

1. Закоммить папку `flatpak/` в `avv7bc/gitstream` (нужно для raw-ссылок на иконку и скриншот).
2. Форкни `github.com/flathub/flathub`, создай ветку `io.github.avv7bc.gitstream`.
3. Скопируй в корень форка: манифест `.yml`, `.desktop`, `.metainfo.xml`
   (скриншот и иконка тянутся по URL — копировать не надо).
4. Заполни `REPLACE_ME`-хэши, открой PR в `flathub/flathub`.
5. **Buildbot соберёт и протестирует прямо в PR.** Тут проверится, что prebuilt-бинарник
   стартует с webkit из рантайма и что бандл git работает. Если чего-то не хватает
   (например `openssh` для SSH-remote'ов) — правишь манифест в том же PR, пересборка
   автоматическая. Локально ничего гонять не нужно.
6. После апрува мейнтейнеров PR мёржат → создаётся твой репозиторий
   `flathub/io.github.avv7bc.gitstream`, ты его мейнтейнер.

## Обновления (нулевая ручная работа)

Блок `x-checker-data` в манифесте включает бота Flathub: при каждом новом теге
`vX.Y.Z` он сам открывает PR с новым URL+sha256 `.deb`. Тебе останется:
1. добавить `<release>` в `metainfo.xml` (можно тоже автоматизировать),
2. смёржить PR бота — Flathub пересоберёт и опубликует.

## Известные риски (проверяются бесплатно в PR-сборке)

- **webkit**: `.deb` слинкован с `webkit2gtk-4.1` из Ubuntu 22.04; рантайм GNOME даёт
  свой webkit2gtk-4.1 — обычно совпадает по soname. Если бинарник не стартует —
  переходим на сборку из исходников (`flatpak-cargo-generator` + `flatpak-node-generator`).
- **SSH-remote'ы**: если нужен `ssh`, добавить модуль `openssh` в манифест.
- **runtime-version `47`**: при необходимости поднять до актуального GNOME-рантайма.
