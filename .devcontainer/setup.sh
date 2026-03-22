#!/usr/bin/env bash
# .devcontainer/setup.sh — Codespace post-create setup for uws
# Runs automatically after the container is created.
set -euo pipefail

echo "=== uws Codespace Setup ==="

# ── Rust toolchain ──────────────────────────────────────────────
echo "→ Installing Rust components..."
rustup component add clippy rustfmt 2>/dev/null || true

# ── Build ───────────────────────────────────────────────────────
echo "→ Building uws..."
cargo build 2>/dev/null || echo "  Build warnings (non-fatal)"

# ── Python deps for toolchain/ ──────────────────────────────────
echo "→ Installing Python dependencies..."
pip install --quiet pyyaml pytest 2>/dev/null || true

# ── gh CLI: install uws extension ───────────────────────────────
echo "→ Registering uws as gh extension..."
if command -v gh >/dev/null 2>&1; then
  # Link the local checkout as a gh extension for development
  GH_EXT_DIR="${HOME}/.local/share/gh/extensions/gh-uws"
  mkdir -p "$(dirname "$GH_EXT_DIR")"
  if [ ! -e "$GH_EXT_DIR" ]; then
    ln -s "$(pwd)/gh-extension" "$GH_EXT_DIR" 2>/dev/null || true
    echo "  gh uws extension linked at $GH_EXT_DIR"
  fi
fi

# ── Add cargo bin to PATH ────────────────────────────────────────
if ! echo "$PATH" | grep -q "${HOME}/.cargo/bin"; then
  echo 'export PATH="${HOME}/.cargo/bin:${PATH}"' >> "${HOME}/.bashrc"
fi

# Symlink the dev build to PATH
BIN_DIR="${HOME}/.local/bin"
mkdir -p "$BIN_DIR"
if [ -f "$(pwd)/target/debug/uws" ]; then
  ln -sf "$(pwd)/target/debug/uws" "$BIN_DIR/uws"
  echo "  uws dev binary linked at $BIN_DIR/uws"
fi

echo ""
echo "=== Setup complete ==="
echo ""
echo "  Quick start:"
echo "    uws --help"
echo "    gh uws providers"
echo "    python toolchain/janus_runner.py boot"
echo "    python mcp_server/server.py --transport stdio"
echo ""
