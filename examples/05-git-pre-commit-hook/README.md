# Example 05: Live Git-Backed Pre-Commit Hook

This example demonstrates how to integrate **HeaderCraft** with [**`hk`**](https://github.com/jdx/hk) and [**`mise`**](https://github.com/jdx/mise) as an ultra-fast Git pre-commit hook in a real-world repository with full Git history.

It clones [**`fatih/color`**](https://github.com/fatih/color) (a popular Go repository with commits spanning from 2014 to present) and automatically formats license headers using **real Git commit history**.

---

## What This Demonstrates

1. **Dynamic Git Timestamp Resolution**:
   - `color.go` was first committed in **2014** &rarr; renders `Copyright (c) 2014-2026`
   - `color_windows.go` was first committed in **2023** &rarr; renders `Copyright (c) 2023-2026`
   - Zero hardcoded dates; timestamps are extracted directly from Git commits.
2. **`hk check` Failure & GCC Diagnostics**:
   - Files without valid headers fail `hk check` with standard GCC diagnostic output:
     `color.go:1:1: error: Missing or outdated file header [headercraft]`
3. **`hdr diff` / `hk diff`**:
   - Preview unified diffs before applying any changes.
4. **`hk fix` Automatic In-Place Update**:
   - Automatically writes the formatted headers into all files.
5. **Enforced Git Pre-Commit Hook**:
   - Pre-commit hook runs on every `git commit` to guarantee non-compliant headers never enter the repository.

---

## Quickstart

### Option A: One-Command Automated Demo

Run the automated walkthrough script from this directory:

```bash
cd examples/05-git-pre-commit-hook
./demo.sh
```

This will clone the repository, install the hook, demonstrate the failure, preview the diff, run the fix, and perform a verified git commit!

---

### Option B: Interactive / Manual Setup

```bash
cd examples/05-git-pre-commit-hook

# 1. Clone target repo and copy configs
./clone-and-setup.sh https://github.com/fatih/color.git /tmp/headercraft-color-demo

cd /tmp/headercraft-color-demo

# 2. Run hk check across all files (fails with GCC diagnostics)
hk check --all

# 3. Preview the Git-timestamped diff
hdr diff color.go color_windows.go doc.go

# 4. Auto-fix all files
hk fix --all

# 5. Verify all files now pass
hk check --all

# 6. Commit the changes (pre-commit hook automatically validates)
git add -A
git commit -m "chore: add license headers with headercraft and hk"
```

---

## Configuration Breakdown

### 1. `headercraft.toml`

Uses MiniJinja templating with `git.created_year` and `now.year`:

```toml
[header]
template = """
{% set start = git.created_year | default(now.year) -%}
Copyright (c) {{ start }}{% if start != now.year %}-{{ now.year }}{% endif %} {{ props.owner }}.
SPDX-License-Identifier: {{ props.license }}
"""

[props]
owner = "Fatih Arslan and Color Contributors"
license = "MIT"

[git]
enabled = true

[files]
includes = ["**/*.go"]
excludes = ["vendor/**"]

[[rules]]
name = "go"
extensions = ["go"]
style = "line"
comment_prefix = "// "
```

### 2. `hk.pkl`

Configures the `headers` hook in `hk`:

```pkl
amends "package://github.com/jdx/hk/releases/download/v2.11.0/hk@2.11.0#/Config.pkl"

steps {
  ["headers"] = new Step {
    glob = "**/*.go"
    exclude = List("vendor/**")
    check = "headercraft check --format gcc {{files}}"
    fix = "headercraft fix {{files}}"
    check_diff = "headercraft diff {{files}}"
    diagnostic_format = "gcc"
    diagnostic_tool = "headercraft"
  }
}
```

### 3. `mise.toml`

Defines developer tasks:

```toml
[tools]
hk = "2.11.0"

[tasks.check]
run = "hk check --all"

[tasks.fix]
run = "hk fix --all"

[tasks.diff]
run = "hdr diff $(git ls-files '*.go')"
```
