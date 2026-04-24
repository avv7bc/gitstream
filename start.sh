#!/usr/bin/env bash
# GitStream launcher — проверяет зависимости, доустанавливает недостающие и запускает приложение.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# --- цвета ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${BLUE}[i]${NC} $*"; }
ok()    { echo -e "${GREEN}[✓]${NC} $*"; }
warn()  { echo -e "${YELLOW}[!]${NC} $*"; }
err()   { echo -e "${RED}[✗]${NC} $*" >&2; }

have() { command -v "$1" >/dev/null 2>&1; }

confirm() {
    local prompt="${1:-Продолжить?}"
    read -r -p "$prompt [y/N] " reply
    [[ "$reply" =~ ^[Yy]$ ]]
}

# --- определение ОС ---
OS="$(uname -s)"
DISTRO=""
PKG=""
PKG_INSTALL=""

detect_linux() {
    if [ -f /etc/os-release ]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        DISTRO="$ID"
    fi
    if have apt-get; then
        PKG="apt"
        PKG_INSTALL="sudo apt-get install -y"
    elif have dnf; then
        PKG="dnf"
        PKG_INSTALL="sudo dnf install -y"
    elif have pacman; then
        PKG="pacman"
        PKG_INSTALL="sudo pacman -S --noconfirm"
    elif have zypper; then
        PKG="zypper"
        PKG_INSTALL="sudo zypper install -y"
    else
        warn "Не удалось определить пакетный менеджер. Системные пакеты придётся ставить вручную."
    fi
}

case "$OS" in
    Linux*)  detect_linux ;;
    Darwin*) ok "macOS обнаружена" ;;
    *)       warn "Платформа $OS не поддерживается скриптом — продолжаю как смогу" ;;
esac

# --- git ---
if ! have git; then
    err "Git не найден. Установи его и повтори запуск."
    exit 1
fi
ok "git: $(git --version)"

# --- Node.js ---
if ! have node || ! have npm; then
    err "Node.js/npm не найден."
    echo "   Установи Node 18+: https://nodejs.org/ или через nvm:"
    echo "   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash"
    exit 1
fi
NODE_VER="$(node -v | sed 's/v//;s/\..*//')"
if [ "${NODE_VER:-0}" -lt 18 ]; then
    err "Нужен Node.js 18+, установлен $(node -v)"
    exit 1
fi
ok "node: $(node -v), npm: $(npm -v)"

# --- Rust ---
if ! have cargo || ! have rustc; then
    warn "Rust toolchain не найден."
    if confirm "Установить через rustup (официальный установщик)?"; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        # shellcheck disable=SC1091
        . "$HOME/.cargo/env"
        ok "Rust установлен: $(rustc --version)"
    else
        err "Без Rust невозможно собрать Tauri backend. Выхожу."
        exit 1
    fi
else
    ok "rust: $(rustc --version)"
fi

# подгружаем cargo env если запущено в свежей оболочке после установки
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi

# --- системные зависимости Tauri (Linux) ---
check_linux_deps() {
    local missing=()
    # проверка через pkg-config / существование библиотек
    have pkg-config || missing+=("pkg-config")
    pkg-config --exists webkit2gtk-4.1 2>/dev/null || pkg-config --exists webkit2gtk-4.0 2>/dev/null || missing+=("webkit2gtk")
    pkg-config --exists gtk+-3.0 2>/dev/null || missing+=("gtk3")
    pkg-config --exists librsvg-2.0 2>/dev/null || missing+=("librsvg2")

    if [ ${#missing[@]} -eq 0 ]; then
        ok "Системные зависимости Tauri на месте"
        return 0
    fi

    warn "Отсутствуют системные пакеты: ${missing[*]}"
    if [ -z "$PKG_INSTALL" ]; then
        err "Установи вручную: https://tauri.app/start/prerequisites/"
        return 1
    fi

    local pkgs=""
    case "$PKG" in
        apt)
            pkgs="libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev build-essential curl wget file libssl-dev pkg-config"
            ;;
        dnf)
            pkgs="webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel curl wget file @development-tools"
            ;;
        pacman)
            pkgs="webkit2gtk-4.1 base-devel curl wget file openssl gtk3 libappindicator-gtk3 librsvg"
            ;;
        zypper)
            pkgs="webkit2gtk3-devel libopenssl-devel gtk3-devel libappindicator3-1 librsvg-devel"
            ;;
    esac

    echo "   Команда: $PKG_INSTALL $pkgs"
    if confirm "Установить сейчас?"; then
        # shellcheck disable=SC2086
        $PKG_INSTALL $pkgs
        ok "Системные зависимости установлены"
    else
        err "Tauri не соберётся без этих пакетов."
        return 1
    fi
}

if [ "$OS" = "Linux" ]; then
    check_linux_deps || exit 1
fi

# --- npm dependencies ---
if [ ! -d node_modules ]; then
    info "Устанавливаю npm-пакеты..."
    npm install
    ok "npm install завершён"
else
    ok "node_modules на месте"
fi

# --- запуск ---
MODE="${1:-dev}"
case "$MODE" in
    dev)
        info "Запускаю в режиме разработки (npm run tauri dev)..."
        exec npm run tauri dev
        ;;
    build)
        info "Собираю production-бандл (npm run tauri build)..."
        exec npm run tauri build
        ;;
    *)
        err "Неизвестный режим: $MODE"
        echo "Использование: $0 [dev|build]"
        exit 1
        ;;
esac
