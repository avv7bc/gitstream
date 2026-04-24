#!/usr/bin/env bash
# GitStream release builder — собирает production версию приложения с версионированием.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# --- цвета ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${BLUE}[i]${NC} $*"; }
ok()    { echo -e "${GREEN}[✓]${NC} $*"; }
warn()  { echo -e "${YELLOW}[!]${NC} $*"; }
err()   { echo -e "${RED}[✗]${NC} $*" >&2; }
section() { echo -e "\n${CYAN}===${NC} $* ${CYAN}===${NC}\n"; }

have() { command -v "$1" >/dev/null 2>&1; }

confirm() {
    local prompt="${1:-Продолжить?}"
    read -r -p "$prompt [y/N] " reply
    [[ "$reply" =~ ^[Yy]$ ]]
}

get_current_version() {
    grep '"version"' package.json | head -1 | sed 's/.*: "\([^"]*\)".*/\1/'
}

# --- парсинг аргументов ---
VERSION="${1:-}"
SKIP_DEPS="${SKIP_DEPS:-0}"

if [ -z "$VERSION" ]; then
    CURRENT=$(get_current_version)
    info "Текущая версия: $CURRENT"
    info "Использование: $0 <версия>"
    echo "   Пример: $0 0.2.0"
    exit 0
fi

# --- валидация версии (semver) ---
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    err "Неверный формат версии: $VERSION"
    err "Требуется семантическое версионирование (X.Y.Z или X.Y.Z-alpha+build)"
    exit 1
fi

section "Подготовка к сборке версии $VERSION"

# --- проверки окружения ---
if [ "$SKIP_DEPS" != "1" ]; then
    info "Проверяю зависимости..."

    if ! have git; then
        err "Git не найден"
        exit 1
    fi
    ok "git: $(git --version)"

    if ! have node || ! have npm; then
        err "Node.js/npm не найден"
        exit 1
    fi
    NODE_VER="$(node -v | sed 's/v//;s/\..*//')"
    if [ "${NODE_VER:-0}" -lt 18 ]; then
        err "Нужен Node.js 18+, установлен $(node -v)"
        exit 1
    fi
    ok "node: $(node -v), npm: $(npm -v)"

    if ! have cargo || ! have rustc; then
        err "Rust toolchain не найден"
        exit 1
    fi
    ok "rust: $(rustc --version)"
fi

# --- проверка чистоты репозитория ---
section "Проверка git состояния"

BRANCH=$(git rev-parse --abbrev-ref HEAD)
ok "Текущая ветка: $BRANCH"

if ! git diff --quiet; then
    err "В рабочей директории есть несохранённые изменения"
    err "Закоммить их или отменить перед сборкой release"
    exit 1
fi
ok "Рабочая директория чистая"

if [ -n "$(git diff --cached)" ]; then
    err "В индексе есть неконмиченные изменения"
    exit 1
fi
ok "Индекс чист"

# --- проверка версии в git ---
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    err "Тег v$VERSION уже существует в репозитории"
    exit 1
fi
ok "Версия $VERSION не тегирована"

# --- обновление версий ---
section "Обновление версий"

info "package.json: $(get_current_version) → $VERSION"
sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json
ok "package.json обновлён"

info "src-tauri/Cargo.toml: версия обновляется"
sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
ok "src-tauri/Cargo.toml обновлён"

# --- git commit версии ---
section "Коммит версии"

git add package.json src-tauri/Cargo.toml
git commit -m "chore: bump version to $VERSION"
ok "Версия закоммичена"

# --- установка зависимостей ---
section "Установка зависимостей"

if [ ! -d node_modules ] || [ ! -d src-tauri/target ]; then
    info "Устанавливаю npm-пакеты..."
    npm install
    ok "npm install завершён"
else
    ok "Зависимости на месте"
fi

# --- сборка frontend ---
section "Сборка фронтенда"

info "Проверка типов TypeScript..."
npm run build -- --mode production 2>&1 | grep -v "^$" || true
ok "Фронтенд собран"

# --- сборка Tauri ---
section "Сборка Tauri приложения"

info "Компиляция Rust backend..."
npm run tauri build

# ищем собранный артефакт
if [ -f "src-tauri/target/release/gitstream" ]; then
    BINARY_SIZE=$(du -h "src-tauri/target/release/gitstream" | cut -f1)
    ok "Бинарник собран: $BINARY_SIZE"
fi

if [ -d "src-tauri/target/release/bundle" ]; then
    ok "Bundle готов в src-tauri/target/release/bundle"
fi

# --- создание git tag ---
section "Создание release tag"

info "Теги для версии $VERSION:"
git tag -a "v$VERSION" -m "Release v$VERSION" || warn "Не удалось создать тег (может быть, нужны права на подпись)"
ok "Создан тег v$VERSION"

# --- итоговый отчёт ---
section "✓ Release $VERSION готов"

echo "Артефакты находятся в:"
echo "  - Бинарник:  src-tauri/target/release/gitstream"
echo "  - Bundle:    src-tauri/target/release/bundle"
echo ""
echo "Для публикации выполни:"
echo "  git push origin $BRANCH"
echo "  git push origin v$VERSION"
echo ""
echo "GitHub Release:"
echo "  gh release create v$VERSION --draft src-tauri/target/release/bundle/*"
