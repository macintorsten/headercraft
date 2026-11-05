# Example 6: Synchronizing Script Headers with CLI `--help` Output

Automatically maintain and synchronize in-code usage comments with the actual runtime output of `bash <file> --help` or `-h`.

## The Problem
Shell scripts, CLI utilities, and automation scripts often contain comment headers describing how to invoke them. Over time, developers add flags or alter parameters inside the script's `--help` implementation but forget to update the top-of-file comments. This causes documentation drift and confuses users reading the source.

## The HeaderCraft Solution
With HeaderCraft's `[commands]` engine and `{file}` placeholder:
1. **Dynamic Execution:** HeaderCraft executes `bash {file} --help` against each target script and captures its output into `cmd.help`.
2. **Comment Formatting:** Wraps the multi-line help output into clean `# ` comments placed directly under the `#!/bin/bash` shebang.
3. **Drift Detection:** `hdr check` detects if in-code comments no longer match the real `--help` output.
4. **Auto-Fix:** `hdr fix` updates or inserts the latest usage instructions in milliseconds.

## Configuration (`headercraft.toml`)
```toml
[commands]
# Runs the script with --help to obtain the canonical synopsis
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

## Running the Demo
Run the automated end-to-end walkthrough script:
```bash
./run-demo.sh
```

Or run individual commands:
```bash
# 1. Preview how backup.sh and deploy.sh will be updated
headercraft diff --config headercraft.toml before/

# 2. Fix scripts in-place
headercraft fix --config headercraft.toml before/

# 3. Verify all scripts pass check
headercraft check --config headercraft.toml before/
```
