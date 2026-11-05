# Example 06: Synchronizing Script Headers with CLI `--help` Output

Automatically maintain and synchronize in-code usage comments with the actual runtime output of `bash <file> --help` or `-h`.

---

## Visual Comparison (Before & After)

### 1. Bootstrapping Missing Usage (`backup.sh`)

```bash
# BEFORE (No usage documentation):
#!/bin/bash
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

# AFTER (HeaderCraft ran `bash backup.sh --help` and inserted the synopsis):
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
```

### 2. Auto-Fixing Drifted Flags (`deploy.sh`)

```bash
# BEFORE (Header was missing newer flags --dry-run and --tag):
#!/bin/bash
# Synopsis & Usage:
# deploy.sh - Deploy artifact to cloud environments
# Usage:
#   deploy.sh --env <staging|prod>

# AFTER (Updated to match current CLI flags):
#!/bin/bash
# Synopsis & Usage:
# deploy.sh - Deploy artifact to cloud environments
# 
# Usage:
#   deploy.sh [flags]
# 
# Flags:
#   --env <name>       Target environment (staging or production)
#   --dry-run          Preview changes without deploying
#   --tag <version>    Release tag to deploy
#   -h, --help         Show this help message
```

---

## How It Works

### Configuration (`headercraft.toml`)
```toml
[commands]
# Runs the script with --help to obtain the canonical synopsis
# Uses caching so static commands run once, while {file} commands run per script
help = "bash {file} --help"

[[rules]]
name = "script_help_sync"
extensions = ["sh", "bash"]
style = "line"
comment_prefix = "# "
template = """
Synopsis & Usage:
{{ cmd.help }}
"""
keywords = ["Synopsis & Usage:"]
```

### Strict Sync vs. Drift-Tolerant Check
- **Strict Sync (Default)**: Any difference between the comment and `--help` triggers `hdr check` failure and is auto-corrected by `hdr fix`.
- **Drift-Tolerant Check**: To enforce presence without overwriting custom notes, remove the `template` and use `fixable = false`:
  ```toml
  [[rules]]
  name = "require_usage"
  fixable = false
  keywords = ["Usage:"]
  ```

---

## Try It

### Option A: Via `hk` and `mise` (One-Liner)
```bash
# Check script header compliance
mise exec hk@latest -- hk check --all

# Auto-fix in-place
mise run fix
```

### Option B: Standalone Automated Walkthrough
```bash
./run-demo.sh
```

### Option C: Standalone CLI
```bash
# 1. Preview changes as unified diff
headercraft diff --config headercraft.toml before/

# 2. Fix scripts in-place
headercraft fix --config headercraft.toml before/

# 3. Verify all scripts now pass check
headercraft check --config headercraft.toml before/
```
