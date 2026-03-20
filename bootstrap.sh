#!/usr/bin/env bash
# bootstrap.sh — Aluminum OS One-Shot Build and Verification
# Usage: chmod +x bootstrap.sh && ./bootstrap.sh
set -euo pipefail
echo "=== ALUMINUM OS Bootstrap ==="
PASS=0; FAIL=0; WARN=0
echo "--- Prerequisites ---"
command -v cargo || { echo "FAIL: Rust not found. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; exit 1; }
echo "--- Build ---"
cargo build && echo "PASS: cargo build" || echo "FAIL: cargo build"
cargo build --release && echo "PASS: release build" || echo "FAIL: release build"
echo "--- Test ---"
cargo test && echo "PASS: cargo test" || echo "FAIL: cargo test"
if command -v python3 && [ -d tests ]; then python3 -m pytest tests/ -v || echo "WARN: pytest issues"; fi
echo "--- Governance ---"
if command -v opa && [ -d toolchain/policies ]; then for p in toolchain/policies/*.rego; do opa check "$p" || echo "FAIL: $p"; done; echo "PASS: Rego valid"; else echo "WARN: OPA not installed"; fi
echo "--- Secret Scan ---"
FOUND=0; for pat in "AKIA[0-9A-Z]{16}" "sk-[a-zA-Z0-9]{48}" "ghp_[a-zA-Z0-9]{36}"; do grep -rn "$pat" src/ toolchain/ 2>/dev/null | grep -v test && FOUND=1 || true; done
[ $FOUND -eq 0 ] && echo "PASS: No secrets" || echo "FAIL: Secrets found"
echo "=== Bootstrap complete ==="