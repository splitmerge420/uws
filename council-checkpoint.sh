#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd ""$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Council Checkpoint (Dual-Mode) ==="
echo "Mode: bootstrap + toolchain/policies"

# Stage 1: Bootstrap
echo ""
echo "[1/2] Running bootstrap.sh..."
if [ -f bootstrap.sh ]; then
  bash bootstrap.sh
else
  echo "ERROR: bootstrap.sh not found"
  exit 1
fi

# Stage 2: Toolchain Policies
echo ""
echo "[2/2] Running toolchain/policies verification..."
if [ -f toolchain/policies/orchestrate.sh ]; then
  bash toolchain/policies/orchestrate.sh
else
  echo "WARN: toolchain/policies/orchestrate.sh not found (optional)"
fi

echo ""
echo "✓ Council checkpoint complete"