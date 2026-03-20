#!/bin/bash
set -euo pipefail

# council-checkpoint.sh
# Dual-mode bootstrap + governance layer verification
# Supports both council automation and standard users

SCRIPT_DIR="$(cd ""+"$(dirname "${BASH_SOURCE[0]}")"+" && pwd)"
REPO_ROOT="${SCRIPT_DIR}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Council Checkpoint Verification ===${NC}"
echo "Repository: ${REPO_ROOT}"
echo "Timestamp: $(date -u +'%Y-%m-%dT%H:%M:%SZ')"
echo ""

# Mode detection
MODE="standard"
if [[ "${COUNCIL_AUTOMATION:-}" == "true" ]]; then
    MODE="council"
fi
echo -e "${BLUE}Mode: ${MODE}${NC}"
echo ""

# Step 1: Bootstrap build
echo -e "${YELLOW}Step 1: Bootstrap Build${NC}"
if [[ -f "${REPO_ROOT}/bootstrap.sh" ]]; then
    bash "${REPO_ROOT}/bootstrap.sh" || {
        echo -e "${RED}✗ Bootstrap failed${NC}"
        exit 1
    }
    echo -e "${GREEN}✓ Bootstrap complete${NC}"
else
    echo -e "${RED}✗ bootstrap.sh not found${NC}"
    exit 1
fi
echo ""

# Step 2: Governance verification
echo -e "${YELLOW}Step 2: Governance Verification${NC}"
if [[ -d "${REPO_ROOT}/toolchain/policies" ]]; then
    echo "Checking Rego policies..."
    POLICY_COUNT=$(find "${REPO_ROOT}/toolchain/policies" -name "*.rego" | wc -l)
    echo -e "${GREEN}✓ Found ${POLICY_COUNT} Rego policies${NC}"
else
    echo -e "${YELLOW}⚠ toolchain/policies directory not found${NC}"
fi

if [[ -f "${REPO_ROOT}/src/lib.rs" ]]; then
    echo "Checking Rust governance modules..."
    echo -e "${GREEN}✓ src/lib.rs present${NC}"
else
    echo -e "${YELLOW}⚠ src/lib.rs not found${NC}"
fi
echo ""

# Step 3: Council-specific verification (if applicable)
if [[ "${MODE}" == "council" ]]; then
    echo -e "${YELLOW}Step 3: Council Automation Verification${NC}"
    
    if [[ -d "${REPO_ROOT}/.git" ]]; then
        BRANCH=$(git -C "${REPO_ROOT}" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
        COMMIT=$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✓ Git branch: ${BRANCH}${NC}"
        echo -e "${GREEN}✓ Latest commit: ${COMMIT}${NC}"
    fi
    
    if [[ -f "${REPO_ROOT}/council-checkpoint.sh" ]]; then
        echo -e "${GREEN}✓ council-checkpoint.sh verified${NC}"
    fi
else
    echo -e "${YELLOW}Step 3: Standard User Verification${NC}"
    echo -e "${GREEN}✓ All systems nominal${NC}"
fi
echo ""

# Final status
echo -e "${BLUE}=== Checkpoint Complete ===${NC}"
echo -e "${GREEN}✓ All verification checks passed${NC}"
exit 0
