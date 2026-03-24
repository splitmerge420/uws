#!/usr/bin/env sh
# uws — Universal Workspace CLI — one-line installer
# Usage: curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | sh
# Or:    curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | sh -s -- --prefix /usr/local

set -eu

UWS_REPO="splitmerge420/uws"
UWS_BINARY="uws"
PREFIX="${PREFIX:-$HOME/.local}"

# ── Parse flags ──────────────────────────────────────────────────────────────
for arg in "$@"; do
  case "$arg" in
    --prefix=*) PREFIX="${arg#--prefix=}" ;;
    --prefix)   shift; PREFIX="$1" ;;
    --help|-h)
      echo "Usage: install.sh [--prefix DIR]"
      echo "  --prefix DIR  Install to DIR/bin (default: \$HOME/.local)"
      exit 0
      ;;
  esac
done

INSTALL_DIR="${PREFIX}/bin"

# ── Detect platform ──────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *)
        echo "ERROR: Unsupported Linux architecture: $ARCH" >&2
        echo "Install from source: cargo install --git https://github.com/${UWS_REPO}" >&2
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-apple-darwin" ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      *)
        echo "ERROR: Unsupported macOS architecture: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "ERROR: Unsupported OS: $OS" >&2
    echo "On Windows, download the installer from:" >&2
    echo "  https://github.com/${UWS_REPO}/releases/latest" >&2
    exit 1
    ;;
esac

# ── Fetch latest release tag ─────────────────────────────────────────────────
echo "→ Fetching latest release..."
if command -v curl >/dev/null 2>&1; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${UWS_REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
elif command -v wget >/dev/null 2>&1; then
  VERSION="$(wget -qO- "https://api.github.com/repos/${UWS_REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
else
  echo "ERROR: curl or wget is required" >&2
  exit 1
fi

if [ -z "$VERSION" ]; then
  echo "ERROR: Could not determine latest release version." >&2
  echo "Visit https://github.com/${UWS_REPO}/releases to download manually." >&2
  exit 1
fi

echo "→ Latest version: $VERSION"

# ── Download binary archive ──────────────────────────────────────────────────
# cargo-dist produces archives named: uws-{version}-{target}.tar.gz
ARCHIVE_NAME="${UWS_BINARY}-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${UWS_REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "→ Downloading $ARCHIVE_NAME..."
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE_NAME}"
else
  wget -qO "${TMP_DIR}/${ARCHIVE_NAME}" "$DOWNLOAD_URL"
fi

# ── Extract and install ──────────────────────────────────────────────────────
echo "→ Extracting..."
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"

BIN_SRC="$(find "$TMP_DIR" -name "$UWS_BINARY" -type f | head -n1)"
if [ -z "$BIN_SRC" ]; then
  echo "ERROR: Could not find '$UWS_BINARY' binary in archive." >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
cp "$BIN_SRC" "${INSTALL_DIR}/${UWS_BINARY}"
chmod +x "${INSTALL_DIR}/${UWS_BINARY}"

echo "→ Installed ${UWS_BINARY} ${VERSION} to ${INSTALL_DIR}/${UWS_BINARY}"

# ── PATH check ───────────────────────────────────────────────────────────────
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "⚠  ${INSTALL_DIR} is not in your PATH."
    echo "   Add it by running one of these:"
    echo ""
    echo "   # bash"
    echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
    echo ""
    echo "   # zsh"
    echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
    echo ""
    ;;
esac

echo ""
echo "✓ uws ${VERSION} installed successfully!"
echo ""
echo "  Quick start:"
echo "    uws --version"
echo "    uws auth setup              # Google Workspace"
echo "    uws ms-auth setup           # Microsoft 365"
echo "    GITHUB_TOKEN=\$TOKEN uws github repos list"
echo ""
echo "  Full docs: https://github.com/${UWS_REPO}#readme"
