# GitHub Actions Release Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Автоматически собирать нативные инсталляторы для Linux/macOS/Windows при пуше тега и загружать их в черновой GitHub Release — без дополнительных секретов, только GITHUB_TOKEN.

**Architecture:** Один workflow-файл с тремя параллельными джобами (по платформе). Каждый джоб устанавливает зависимости, компилирует приложение через `npm run tauri build` и загружает артефакты в черновой GitHub Release через `softprops/action-gh-release@v2` — эта action идемпотентна при параллельном запуске.

**Tech Stack:** GitHub Actions, `actions/checkout@v4`, `actions/setup-node@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `softprops/action-gh-release@v2`

---

### Task 1: Создать workflow-файл

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Создать директорию**

```bash
mkdir -p .github/workflows
```

- [ ] **Step 2: Создать `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

permissions:
  contents: write

jobs:
  build-linux:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            build-essential \
            curl \
            wget \
            file \
            libxdo-dev \
            libssl-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev

      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - run: npm ci

      - name: Build
        run: npm run tauri build

      - name: Upload Linux artifacts
        uses: softprops/action-gh-release@v2
        with:
          draft: true
          files: |
            src-tauri/target/release/bundle/deb/*.deb
            src-tauri/target/release/bundle/rpm/*.rpm
            src-tauri/target/release/bundle/appimage/*.AppImage

  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - run: npm ci

      - name: Build
        run: npm run tauri build

      - name: Upload macOS artifacts
        uses: softprops/action-gh-release@v2
        with:
          draft: true
          files: |
            src-tauri/target/release/bundle/dmg/*.dmg
            src-tauri/target/release/bundle/macos/*.tar.gz

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - run: npm ci

      - name: Build
        run: npm run tauri build

      - name: Upload Windows artifacts
        uses: softprops/action-gh-release@v2
        with:
          draft: true
          files: |
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/nsis/*.exe
```

- [ ] **Step 3: Проверить синтаксис YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML valid"
```

Ожидаемый вывод: `YAML valid`

- [ ] **Step 4: Закоммитить**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow"
```

---

### Task 2: Обновить release.sh

**Files:**
- Modify: `release.sh` (строки 180–181)

- [ ] **Step 1: Найти строки**

```bash
grep -n "gh release create\|GitHub Release" release.sh
```

Ожидаемый вывод:
```
180:echo "GitHub Release:"
181:echo "  gh release create v$VERSION --draft src-tauri/target/release/bundle/*"
```

- [ ] **Step 2: Заменить подсказку**

В `release.sh` найти:
```bash
echo "GitHub Release:"
echo "  gh release create v$VERSION --draft src-tauri/target/release/bundle/*"
```

Заменить на:
```bash
echo "GitHub Release создаётся автоматически через GitHub Actions после:"
echo "  git push origin v$VERSION"
```

- [ ] **Step 3: Проверить, что старой строки нет**

```bash
grep "gh release create" release.sh && echo "FOUND — нужно убрать" || echo "OK"
```

Ожидаемый вывод: `OK`

- [ ] **Step 4: Закоммитить**

```bash
git add release.sh
git commit -m "chore: обновить подсказку в release.sh — release теперь через CI"
```

---

### Task 3: Проверить workflow в действии

- [ ] **Step 1: Запушить изменения на GitHub**

```bash
git push origin main
```

Если remote не настроен:
```bash
gh repo create gitstream --public --source=. --remote=origin --push
```

- [ ] **Step 2: Создать тестовый тег и запушить**

```bash
git tag v0.0.1-citest
git push origin v0.0.1-citest
```

- [ ] **Step 3: Наблюдать за запуском**

```bash
gh run list --limit 3
```

Подождать 1–2 минуты, затем:

```bash
gh run watch
```

Ожидаемый вывод в конце: все три джоба завершились с зелёным статусом (`✓`).

- [ ] **Step 4: Проверить черновой release**

```bash
gh release list
```

Ожидаемый вывод — строка с тегом `v0.0.1-citest`, статус `Draft`, и прикреплённые файлы `.deb`, `.AppImage`, `.dmg`, `.msi` и т.д.

- [ ] **Step 5: Удалить тестовый тег и release**

```bash
gh release delete v0.0.1-citest --yes
git push origin --delete v0.0.1-citest
git tag -d v0.0.1-citest
```
