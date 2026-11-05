#!/usr/bin/env bash
# HeaderCraft + hk Pre-Commit Hook Setup Script
# Clones an open source repo, injects headercraft configuration, and installs the hk pre-commit hook.

set -euo pipefail

REPO_URL="${1:-https://github.com/fatih/color.git}"
TARGET_DIR="${2:-/tmp/headercraft-color-demo}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== 1. Cloning Repository ==="
echo "Source: ${REPO_URL}"
echo "Target: ${TARGET_DIR}"

if [ -d "${TARGET_DIR}" ]; then
    echo "Cleaning up existing directory ${TARGET_DIR}..."
    rm -rf "${TARGET_DIR}"
fi

git clone --quiet "${REPO_URL}" "${TARGET_DIR}"
cd "${TARGET_DIR}"

echo "=== 2. Copying HeaderCraft & hk Configurations ==="
cp "${SCRIPT_DIR}/headercraft.toml" .
cp "${SCRIPT_DIR}/hk.pkl" .
cp "${SCRIPT_DIR}/mise.toml" .

# Ensure mise and headercraft are on PATH
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"
if command -v mise >/dev/null 2>&1; then
    eval "$(mise activate bash)"
fi

echo "=== 3. Installing hk Pre-Commit Hook ==="
# Install pre-commit hook directly in the repository
hk install --force-local --legacy

echo ""
echo "=========================================================="
echo "Setup Complete in: ${TARGET_DIR}"
echo "=========================================================="
echo "Try running the following commands:"
echo "  cd ${TARGET_DIR}"
echo ""
echo "  1. Test check (fails because headers are missing/outdated):"
echo "     hk check --all"
echo ""
echo "  2. Preview the diff using git commit timestamps:"
echo "     hdr diff \$(git ls-files '*.go')"
echo ""
echo "  3. Auto-fix headers in all files:"
echo "     hk fix --all"
echo ""
echo "  4. Verify all files now pass:"
echo "     hk check --all"
echo ""
echo "  5. Test git commit (pre-commit hook will run):"
echo "     git add -A && git commit -m 'chore: add license headers with headercraft'"
echo "=========================================================="
