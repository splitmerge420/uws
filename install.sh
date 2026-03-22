#!/usr/bin/env bash
# install.sh — uws Universal Workspace CLI — One-Command Installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | sh -s -- --version 0.1.0
#   curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | sh -s -- --dir /usr/local/bin
#
# What it does:
#   1. Detects your OS and architecture
#   2. Downloads the pre-built binary from GitHub Releases
#   3. Falls back to 'cargo install' if no binary is available
#   4. Adds uws to your PATH
#
# To uninstall:
#   rm "$(which uws)"
#
# License: Apache-2.0

set -euo pipefail

REPO="splitmerge420/uws"
BIN_NAME="uws"
INSTALL_DIR=""
REQUESTED_VERSION=""
QUIET=0

# ── Argument parsing ────────────────────────────────────────────

while [ $# -gt 0 ]; do
  case "$1" in
    --version|-v) REQUESTED_VERSION="${2:-}"; shift 2 ;;
    --dir|-d)     INSTALL_DIR="${2:-}"; shift 2 ;;
    --quiet|-q)   QUIET=1; shift ;;
    --help|-h)
      echo "uws installer"
      echo "  --version VERSION   Install a specific version (default: latest)"
      echo "  --dir DIR           Install directory (default: /usr/local/bin or ~/.local/bin)"
      echo "  --quiet             Suppress output"
      exit 0 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# ── Helpers ─────────────────────────────────────────────────────

say() { [ "$QUIET" -eq 0 ] && printf '%s\n' "$*" || true; }
err() { printf '\033[31merror\033[0m: %s\n' "$*" >&2; exit 1; }
ok()  { [ "$QUIET" -eq 0 ] && printf '\033[32m✓\033[0m  %s\n' "$*" || true; }
warn(){ printf '\033[33mwarn\033[0m:  %s\n' "$*" >&2; }

need() { command -v "$1" >/dev/null 2>&1 || err "Required tool not found: $1 — please install it"; }
need "curl"
need "uname"

# ── Platform detection ──────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*)   OS_TAG="unknown-linux-musl" ;;
  Darwin*)  OS_TAG="apple-darwin" ;;
  MSYS*|MINGW*|CYGWIN*) OS_TAG="pc-windows-msvc" ;;
  *)        OS_TAG="unknown-linux-musl"; warn "Unknown OS: $OS — assuming Linux" ;;
esac

case "$ARCH" in
  x86_64)         ARCH_TAG="x86_64" ;;
  aarch64|arm64)  ARCH_TAG="aarch64" ;;
  armv7l)         ARCH_TAG="armv7"; OS_TAG="unknown-linux-musleabihf" ;;
  *)              ARCH_TAG="$ARCH"; warn "Unknown architecture: $ARCH" ;;
esac

PLATFORM="${ARCH_TAG}-${OS_TAG}"

say ""
say "  \033[1muws — Universal Workspace CLI\033[0m"
say "  https://github.com/${REPO}"
say ""
say "  Platform: ${PLATFORM}"

# ── Install directory ───────────────────────────────────────────

if [ -z "$INSTALL_DIR" ]; then
  if [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
    INSTALL_DIR="/usr/local/bin"
  elif [ -d "${HOME}/.local/bin" ]; then
    INSTALL_DIR="${HOME}/.local/bin"
  else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "$INSTALL_DIR"
  fi
fi

INSTALL_PATH="${INSTALL_DIR}/${BIN_NAME}"

# ── Resolve version ─────────────────────────────────────────────

if [ -z "$REQUESTED_VERSION" ]; then
  say "  Fetching latest release..."
  RESOLVED_VERSION="$(curl -fsSL \
    "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' 2>/dev/null || echo "")"
  if [ -z "$RESOLVED_VERSION" ]; then
    warn "Could not determine latest release — will fall back to cargo install"
    RESOLVED_VERSION=""
  fi
else
  RESOLVED_VERSION="v${REQUESTED_VERSION#v}"
fi

say "  Version:  ${RESOLVED_VERSION:-latest (source)}"

# ── Download pre-built binary ───────────────────────────────────

INSTALLED=0

try_download() {
  local url="$1"
  local archive="/tmp/uws-download.tar.gz"

  say "  Downloading ${url##*/}..."
  if curl -fsSL "$url" -o "$archive" 2>/dev/null; then
    mkdir -p /tmp/uws-extract
    tar -xzf "$archive" -C /tmp/uws-extract/ 2>/dev/null || return 1
    BINARY="$(find /tmp/uws-extract -name "$BIN_NAME" -type f 2>/dev/null | head -1)"
    if [ -z "$BINARY" ] && [ "$OS" = "Linux" ] && [ -d /tmp/uws-extract ]; then
      BINARY="$(find /tmp/uws-extract -maxdepth 2 -type f -executable 2>/dev/null | head -1)"
    fi
    if [ -n "$BINARY" ]; then
      install -m 755 "$BINARY" "$INSTALL_PATH"
      rm -rf /tmp/uws-extract "$archive"
      return 0
    fi
    rm -rf /tmp/uws-extract "$archive"
  fi
  return 1
}

if [ -n "$RESOLVED_VERSION" ]; then
  ARCHIVE_NAME="${BIN_NAME}-${PLATFORM}.tar.gz"
  URL="https://github.com/${REPO}/releases/download/${RESOLVED_VERSION}/${ARCHIVE_NAME}"
  if try_download "$URL"; then
    INSTALLED=1
    ok "Downloaded pre-built binary (${RESOLVED_VERSION})"
  else
    say "  Pre-built binary not available for ${PLATFORM} ${RESOLVED_VERSION}"
  fi
fi

# ── Fallback: cargo install ─────────────────────────────────────

if [ "$INSTALLED" -eq 0 ]; then
  warn "Pre-built binary not available — trying 'cargo install'..."

  if ! command -v cargo >/dev/null 2>&1; then
    say ""
    say "  \033[33mRust/cargo not found.\033[0m"
    say "  Install Rust with:"
    say "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    say ""
    say "  Then re-run this installer, or install uws directly with:"
    say "    cargo install --git https://github.com/${REPO}"
    err "Cannot install without a pre-built binary or Rust toolchain"
  fi

  say "  Building from source (this may take a few minutes)..."
  CARGO_HOME="${CARGO_HOME:-${HOME}/.cargo}"

  if [ -n "$RESOLVED_VERSION" ]; then
    TAG="${RESOLVED_VERSION}"
    cargo install --git "https://github.com/${REPO}" --tag "$TAG" \
      --root "$CARGO_HOME" --quiet 2>/dev/null || \
    cargo install --git "https://github.com/${REPO}" \
      --root "$CARGO_HOME" --quiet
  else
    cargo install --git "https://github.com/${REPO}" \
      --root "$CARGO_HOME" --quiet
  fi

  SRC="${CARGO_HOME}/bin/${BIN_NAME}"
  if [ -f "$SRC" ]; then
    install -m 755 "$SRC" "$INSTALL_PATH"
    INSTALLED=1
    ok "Built and installed from source"
  else
    err "Build succeeded but binary not found at ${SRC}"
  fi
fi

# ── Verify ───────────────────────────────────────────────────────

if [ ! -f "$INSTALL_PATH" ]; then
  err "Installation failed: binary not found at ${INSTALL_PATH}"
fi

INSTALLED_VERSION="$("$INSTALL_PATH" --version 2>/dev/null || echo "unknown")"
ok "uws ${INSTALLED_VERSION} installed at ${INSTALL_PATH}"

# ── PATH reminder ────────────────────────────────────────────────

if ! command -v uws >/dev/null 2>&1; then
  say ""
  say "  \033[33m${INSTALL_DIR} is not in your PATH.\033[0m"
  say "  Add it by running:"
  say ""
  case "${SHELL:-}" in
    */zsh)  say "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc && source ~/.zshrc" ;;
    */fish) say "    fish_add_path ${INSTALL_DIR}" ;;
    *)      say "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc && source ~/.bashrc" ;;
  esac
  say ""
fi

# ── Next steps ───────────────────────────────────────────────────

say ""
say "  \033[1mNext steps:\033[0m"
say ""
say "    uws auth setup       # Configure Google Workspace"
say "    uws ms-auth setup    # Configure Microsoft 365"
say "    uws github auth      # Configure GitHub (uses GITHUB_TOKEN)"
say ""
say "  \033[2mFor AI agent use:\033[0m"
say "    python mcp_server/server.py --transport stdio   # MCP mode"
say "    python toolchain/janus_runner.py boot           # Janus council"
say ""
say "  \033[2mDocs:\033[0m https://github.com/${REPO}"
say ""
