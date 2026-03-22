#!/usr/bin/env bash
# install.sh — uws Universal Workspace CLI installer
#
# Usage (no Rust required):
#   curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | bash
#
# Or with a specific version:
#   UWS_VERSION=v0.1.0 curl -fsSL .../install.sh | bash
#
# The installer:
#   1. Detects your OS and CPU architecture
#   2. Downloads the matching pre-built binary from GitHub Releases
#   3. Installs to /usr/local/bin (sudo) or ~/bin (no sudo)
#   4. Verifies the binary works

set -euo pipefail

REPO="splitmerge420/uws"
BINARY="uws"
INSTALL_DIR_SYSTEM="/usr/local/bin"
INSTALL_DIR_USER="${HOME}/.local/bin"

# ── Colors ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info()    { echo -e "${CYAN}[uws]${NC} $*"; }
success() { echo -e "${GREEN}[uws]${NC} $*"; }
warn()    { echo -e "${YELLOW}[uws]${NC} $*"; }
error()   { echo -e "${RED}[uws]${NC} ERROR: $*" >&2; exit 1; }

# ── Detect OS ────────────────────────────────────────────────────────────────
detect_os() {
  case "$(uname -s)" in
    Linux*)   echo "linux" ;;
    Darwin*)  echo "darwin" ;;
    MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
    *)        error "Unsupported OS: $(uname -s). Please build from source: https://github.com/${REPO}" ;;
  esac
}

# ── Detect CPU architecture ──────────────────────────────────────────────────
detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)   echo "x86_64" ;;
    aarch64|arm64)  echo "aarch64" ;;
    *)              error "Unsupported architecture: $(uname -m). Please build from source." ;;
  esac
}

# ── Map to Rust target triple ─────────────────────────────────────────────────
target_triple() {
  local os="$1" arch="$2"
  case "${os}-${arch}" in
    linux-x86_64)   echo "x86_64-unknown-linux-musl" ;;
    linux-aarch64)  echo "aarch64-unknown-linux-musl" ;;
    darwin-x86_64)  echo "x86_64-apple-darwin" ;;
    darwin-aarch64) echo "aarch64-apple-darwin" ;;
    windows-x86_64) echo "x86_64-pc-windows-msvc" ;;
    *)              error "No pre-built binary for ${os}-${arch}. Build from source." ;;
  esac
}

# ── Resolve version ───────────────────────────────────────────────────────────
resolve_version() {
  if [[ -n "${UWS_VERSION:-}" ]]; then
    echo "${UWS_VERSION}"
    return
  fi
  # Fetch the latest release tag from GitHub API
  local tag
  tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
  if [[ -z "$tag" ]]; then
    error "Could not determine latest release. Set UWS_VERSION=vX.Y.Z and retry."
  fi
  echo "$tag"
}

# ── Download and install ──────────────────────────────────────────────────────
main() {
  info "Detecting platform..."
  local os arch triple
  os=$(detect_os)
  arch=$(detect_arch)
  triple=$(target_triple "$os" "$arch")
  info "Platform: ${os}/${arch} → ${triple}"

  local version
  version=$(resolve_version)
  info "Version: ${version}"

  # Build download URL (cargo-dist archive naming)
  local ext="tar.gz"
  [[ "$os" == "windows" ]] && ext="zip"
  local archive="${BINARY}-${triple}.${ext}"
  local url="https://github.com/${REPO}/releases/download/${version}/${archive}"

  info "Downloading ${url}..."
  local tmpdir
  tmpdir=$(mktemp -d)
  trap 'rm -rf "${tmpdir}"' EXIT

  if command -v curl &>/dev/null; then
    curl -fsSL --progress-bar -o "${tmpdir}/${archive}" "${url}"
  elif command -v wget &>/dev/null; then
    wget -q --show-progress -O "${tmpdir}/${archive}" "${url}"
  else
    error "Neither curl nor wget found. Install one and retry."
  fi

  info "Extracting..."
  if [[ "$ext" == "tar.gz" ]]; then
    tar -xzf "${tmpdir}/${archive}" -C "${tmpdir}"
  else
    unzip -q "${tmpdir}/${archive}" -d "${tmpdir}"
  fi

  local bin_name="$BINARY"
  [[ "$os" == "windows" ]] && bin_name="${BINARY}.exe"
  local extracted_bin
  extracted_bin=$(find "${tmpdir}" -name "$bin_name" -type f | head -1)
  [[ -z "$extracted_bin" ]] && error "Could not find binary in archive. Please file an issue."
  chmod +x "$extracted_bin"

  # ── Choose install location ───────────────────────────────────────────────
  local install_dir
  if [[ -w "$INSTALL_DIR_SYSTEM" ]] || sudo -n true 2>/dev/null; then
    install_dir="$INSTALL_DIR_SYSTEM"
    info "Installing to ${install_dir} (requires sudo)..."
    sudo install -m 755 "$extracted_bin" "${install_dir}/${bin_name}"
  else
    install_dir="$INSTALL_DIR_USER"
    mkdir -p "$install_dir"
    info "Installing to ${install_dir} (no sudo required)..."
    install -m 755 "$extracted_bin" "${install_dir}/${bin_name}"

    # Remind user to add to PATH if needed
    if [[ ":${PATH}:" != *":${install_dir}:"* ]]; then
      warn "${install_dir} is not in your PATH."
      warn "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
      warn "  export PATH=\"\$PATH:${install_dir}\""
    fi
  fi

  # ── Verify ────────────────────────────────────────────────────────────────
  if command -v "$BINARY" &>/dev/null; then
    local installed_ver
    installed_ver=$("$BINARY" --version 2>/dev/null || echo "unknown")
    success "✓ uws installed successfully! (${installed_ver})"
    success ""
    success "Quick start:"
    success "  uws auth setup       # Set up Google Workspace auth"
    success "  uws ms-auth setup    # Set up Microsoft 365 auth"
    success "  uws --help           # Full command reference"
    success ""
    success "Docs: https://github.com/${REPO}"
  else
    warn "Binary installed to ${install_dir} but 'uws' is not in PATH yet."
    warn "Restart your shell or run: source ~/.bashrc"
  fi
}

main "$@"
