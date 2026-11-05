#!/bin/bash
# Synopsis & Usage:
# backup.sh - Snapshot local database and object store
# 
# Usage:
#   backup.sh [flags]
# 
# Flags:
#   --target <dir>     Backup destination directory (default: /var/backups)
#   --compress         Enable gzip compression
#   -h, --help         Show this help message

set -euo pipefail

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    cat <<'EOF'
backup.sh - Snapshot local database and object store

Usage:
  backup.sh [flags]

Flags:
  --target <dir>     Backup destination directory (default: /var/backups)
  --compress         Enable gzip compression
  -h, --help         Show this help message
EOF
    exit 0
fi

echo "Running backup task..."
