#!/bin/sh
# ==============================================================================
# PULT (Пульт) — Estação de Trabalho TUI Soberana & Mediadora de Contexto AST
# Instalador Automatizado Soberano
# «ПОЕХАЛИ!»
# ==============================================================================

set -eu

# Cores
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD="\033[1m"
    GREEN="\033[1;32m"
    YELLOW="\033[1;33m"
    RED="\033[1;31m"
    CYAN="\033[1;36m"
    RESET="\033[0m"
else
    BOLD=""
    GREEN=""
    YELLOW=""
    RED=""
    CYAN=""
    RESET=""
fi

show_help() {
    cat <<EOF
${BOLD}☭ PULT (Пульт) — Instalador Automatizado${RESET}

Uso:
  ./install.sh [OPÇÕES]

Opções:
  --prefix <DIR>     Diretório base de instalação (padrão: \$HOME/.local ou /usr/local para root)
  --bin-dir <DIR>    Diretório de destino do binário (padrão: <prefix>/bin)
  --channel <NOME>   Canal de release: stable ou nightly (padrão: stable)
  --build            Forçar compilação local via cargo em vez de baixar binário pré-compilado
  -h, --help         Exibir esta mensagem de ajuda

Exemplos:
  curl -sSL https://pult.2lp.in/install.sh | sh
  ./install.sh --prefix /usr/local
EOF
}

PREFIX="${PREFIX:-}"
BIN_DIR="${BINDIR:-}"
CHANNEL="stable"
FORCE_BUILD=0

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --bin-dir)
            BIN_DIR="$2"
            shift 2
            ;;
        --channel)
            CHANNEL="$2"
            shift 2
            ;;
        --build)
            FORCE_BUILD=1
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "${RED}Opção desconhecida: $1${RESET}" >&2
            show_help
            exit 1
            ;;
    esac
done

if [ -z "$PREFIX" ]; then
    if [ "$(id -u)" -eq 0 ]; then
        PREFIX="/usr/local"
    else
        PREFIX="$HOME/.local"
    fi
fi

if [ -z "$BIN_DIR" ]; then
    BIN_DIR="$PREFIX/bin"
fi

mkdir -p "$BIN_DIR"

# Detecção de Sistema e Arquitetura
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="linux-amd64"
        EXT="tar.gz"
        ;;
    aarch64|arm64)
        TARGET_ARCH="linux-arm64"
        EXT="tar.gz"
        ;;
    *)
        echo "${YELLOW}Arquitetura $ARCH não possui binários pré-compilados na Banca.${RESET}"
        FORCE_BUILD=1
        ;;
esac

if [ "$OS" != "linux" ]; then
    echo "${YELLOW}Sistema operacional $OS: recomendada compilação nativa via cargo.${RESET}"
    FORCE_BUILD=1
fi

INSTALLED=0

if [ "$FORCE_BUILD" -eq 0 ]; then
    TMP_DIR="$(mktemp -d /tmp/pult-install.XXXXXX)"
    trap 'rm -rf "$TMP_DIR"' EXIT

    DOWNLOAD_URL="https://bucket.2lp.in/downloads/pult/${CHANNEL}-latest/pult-${TARGET_ARCH}.${EXT}"
    echo "★ ${CYAN}Baixando Pult (${CHANNEL} / ${TARGET_ARCH})...${RESET}"
    
    if command -v curl >/dev/null 2>&1; then
        if curl -sSL -f "$DOWNLOAD_URL" -o "$TMP_DIR/pkg.${EXT}" 2>/dev/null; then
            tar -xzf "$TMP_DIR/pkg.${EXT}" -C "$TMP_DIR"
            if [ -f "$TMP_DIR/pult" ]; then
                install -m 755 "$TMP_DIR/pult" "$BIN_DIR/pult"
                INSTALLED=1
            fi
        fi
    fi
fi

if [ "$INSTALLED" -eq 0 ]; then
    echo "★ ${YELLOW}Compilando localmente via Cargo...${RESET}"
    if [ -f "crates/pult-tui/Cargo.toml" ]; then
        cargo build --release -p pult-tui
        install -m 755 "target/release/pult" "$BIN_DIR/pult"
        INSTALLED=1
    elif command -v cargo >/dev/null 2>&1; then
        cargo install --git "https://gambi.2lp.in/lumen/pult.git" pult-tui --root "$PREFIX"
        INSTALLED=1
    else
        echo "${RED}Erro: Não foi possível baixar o binário nem encontrar o Cargo para compilação.${RESET}" >&2
        exit 1
    fi
fi

echo "✔ ${GREEN}${BOLD}PULT (Пульт) instalado com sucesso em $BIN_DIR/pult!${RESET}"
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "${YELLOW}Aviso: Adicione $BIN_DIR ao seu PATH: export PATH=\"\$PATH:$BIN_DIR\"${RESET}" ;;
esac
