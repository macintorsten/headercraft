#!/usr/bin/env bash
# HeaderCraft + hk End-to-End Pre-Commit Demo
# Runs a complete automated walkthrough of HeaderCraft + hk hook on a cloned repo.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_DIR="/tmp/headercraft-demo-repo"

export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"
if command -v mise >/dev/null 2>&1; then
    eval "$(mise activate bash)"
fi

echo "================================================================"
echo "  HeaderCraft + hk Pre-Commit Hook Live Demo"
echo "================================================================"
echo ""

# 1. Setup repository
"${SCRIPT_DIR}/clone-and-setup.sh" "https://github.com/fatih/color.git" "${DEMO_DIR}"

cd "${DEMO_DIR}"

echo ""
echo ">>> STEP 1: Running 'hk check --all' (Expecting FAIL)"
echo "----------------------------------------------------"
if hk check --all --no-progress; then
    echo "ERROR: Expected check to fail on unformatted files!"
    exit 1
else
    echo ""
    echo "--> hk check failed as expected! GCC diagnostics displayed above."
fi

echo ""
echo ">>> STEP 2: Running 'hdr diff' to preview git-timestamped changes"
echo "-----------------------------------------------------------------"
hdr diff color.go color_windows.go doc.go || true

echo ""
echo ">>> STEP 3: Running 'hk fix --all' to automatically apply headers"
echo "-----------------------------------------------------------------"
hk fix --all --no-progress

echo ""
echo ">>> STEP 4: Running 'hk check --all' again (Expecting PASS)"
echo "----------------------------------------------------------"
hk check --all --no-progress
echo "--> All headers successfully validated!"

echo ""
echo ">>> STEP 5: Testing Git Pre-Commit Hook on a commit"
echo "----------------------------------------------------"
git add headercraft.toml hk.pkl mise.toml color.go color_test.go color_windows.go doc.go
git commit -m "chore: format license headers with headercraft and hk"

echo ""
echo ">>> STEP 6: Verifying Git log"
echo "------------------------------"
git log -1 --stat

echo ""
echo "================================================================"
echo "  Demo finished successfully!"
echo "  You can inspect the repository at: ${DEMO_DIR}"
echo "================================================================"
