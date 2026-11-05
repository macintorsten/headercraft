#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HDR_BIN="${HDR_BIN:-$SCRIPT_DIR/../../target/debug/headercraft}"

if [ ! -x "$HDR_BIN" ]; then
    echo "Building headercraft binary..."
    cargo build --manifest-path "$SCRIPT_DIR/../../Cargo.toml"
fi

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "=== Example 06: Synchronizing Script Headers with CLI --help ==="
echo "Copying sample scripts into temporary sandbox..."
cp -r "$SCRIPT_DIR/before" "$WORKDIR/scripts"

echo ""
echo "--- Step 1: Check scripts against headercraft.toml (fails on missing/drifted usage) ---"
if "$HDR_BIN" check --config "$SCRIPT_DIR/headercraft.toml" "$WORKDIR/scripts" 2>&1; then
    echo "ERROR: Expected check to fail, but it succeeded."
    exit 1
else
    echo "✓ Detected missing usage in backup.sh and drifted flags in deploy.sh"
fi

echo ""
echo "--- Step 2: Show unified diff (inferred from running 'bash <file> --help') ---"
"$HDR_BIN" diff --config "$SCRIPT_DIR/headercraft.toml" "$WORKDIR/scripts" || true

echo ""
echo "--- Step 3: Run 'headercraft fix' to auto-update in-code usage comments ---"
"$HDR_BIN" fix --config "$SCRIPT_DIR/headercraft.toml" "$WORKDIR/scripts"

echo ""
echo "--- Step 4: Verify all scripts now pass check ---"
"$HDR_BIN" check --config "$SCRIPT_DIR/headercraft.toml" "$WORKDIR/scripts"

echo ""
echo "✓ Success! Script usage comments are now in 100% sync with their --help output."
