#!/usr/bin/env bash
# uws installer
# Usage: curl -sSfL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | sh
#
# Installs the uws Universal Workspace CLI binary to /usr/local/bin (or ~/.local/bin if
# root access is unavailable).  Supports Linux (x86_64, aarch64) and macOS (x86_64, arm64).
set -euo pipefail

REPO="splitmerge420/uws"
BINARY="uws"
INSTALL_DIR="/usr/local/bin"

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)          TRIPLE="x86_64-unknown-linux-musl" ;;
      aarch64|arm64)   TRIPLE="aarch64-unknown-linux-musl" ;;
      *)               echo "error: unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    EXT="tar.gz"
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)          TRIPLE="x86_64-apple-darwin" ;;
      arm64|aarch64)   TRIPLE="aarch64-apple-darwin" ;;
      *)               echo "error: unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    EXT="tar.gz"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    TRIPLE="x86_64-pc-windows-msvc"
    EXT="zip"
    ;;
  *)
    echo "error: unsupported OS: $OS" >&2
    echo "       Install manually: https://github.com/${REPO}/releases" >&2
    exit 1
    ;;
esac

# Resolve the download URL (latest release)
if command -v curl &>/dev/null; then
  LATEST=$(curl -sSfL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
elif command -v wget &>/dev/null; then
  LATEST=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
else
  echo "error: curl or wget is required" >&2
  exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}-${TRIPLE}.${EXT}"
echo "Installing uws ${LATEST} for ${TRIPLE}..."

# Download and extract
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

if command -v curl &>/dev/null; then
  curl -sSfL "$DOWNLOAD_URL" -o "$TMP/archive.${EXT}"
else
  wget -qO "$TMP/archive.${EXT}" "$DOWNLOAD_URL"
fi

if [ "$EXT" = "tar.gz" ]; then
  tar -xzf "$TMP/archive.${EXT}" -C "$TMP"
else
  unzip -q "$TMP/archive.${EXT}" -d "$TMP"
fi

BINARY_PATH=$(find "$TMP" -type f -name "uws" -o -name "uws.exe" | head -1)
if [ -z "$BINARY_PATH" ]; then
  echo "error: could not find uws binary in downloaded archive" >&2
  exit 1
fi
chmod +x "$BINARY_PATH"

# Install binary
if [ -w "$INSTALL_DIR" ] || sudo -n true 2>/dev/null; then
  sudo install -m 755 "$BINARY_PATH" "$INSTALL_DIR/$BINARY" 2>/dev/null \
    || install -m 755 "$BINARY_PATH" "$INSTALL_DIR/$BINARY"
  INSTALLED="$INSTALL_DIR/$BINARY"
else
  mkdir -p "$HOME/.local/bin"
  install -m 755 "$BINARY_PATH" "$HOME/.local/bin/$BINARY"
  INSTALLED="$HOME/.local/bin/$BINARY"
  echo ""
  echo "  Installed to $INSTALLED"
  echo "  Make sure ~/.local/bin is in your PATH:"
  echo '    export PATH="$HOME/.local/bin:$PATH"'
fi

echo ""
echo "✓ uws ${LATEST} installed to ${INSTALLED}"
echo ""
echo "  Get started:"
echo "    uws auth setup          # Google Workspace"
echo "    uws ms-auth setup       # Microsoft 365"
echo "    uws --help              # All commands"
echo ""
echo "  Docs: https://github.com/${REPO}"
