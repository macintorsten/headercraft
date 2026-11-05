# HeaderCraft Roadmap & Issue Tracker

This document tracks identified bugs, edge cases, feature requests, and the forward roadmap for HeaderCraft (`hdr`).

---

## 1. Edge Cases & Bug Tracker

### Resolved in v0.1.0
- [x] **CRLF line endings preservation:** Detected `\r\n` line endings in source files and preserved them across all inserted comment lines and trailing newlines, ensuring idempotence on Windows.
- [x] **Go build tags (`//go:build` and `// +build`):** Kept build tags at the very top of Go files and positioned header comments below them, separated by a blank line to adhere to Go toolchain rules.
- [x] **Python PEP 263 encoding cookies:** Handled `# -*- coding: ... -*-` or `# coding=...` on lines 1 or 2, placing header comments below the cookie.
- [x] **Non-header comments protection:** Bounded header searching to leading file preambles; stopped scanning when non-comment code statements begin so that internal comments mentioning "license" or "copyright" are never deleted.
- [x] **Empty file trailing newline:** Ensured newly formatted empty files always end with a clean newline.

### Backlog
- [ ] **UTF-8 BOM preservation:** Preserve initial `0xEF, 0xBB, 0xBF` byte order marks in XML/C# files when inserting headers.
- [ ] **Explicit HTML `<!DOCTYPE>` policy:** Configurable placement policy for HTML doctypes.
- [ ] **Strict config loading:** Fail immediately with exit code 1 if a user passes `--config <file>` pointing to a nonexistent or malformed file, rather than falling back to defaults.

---

## 2. Feature Requests & Roadmap

### Priority 1: Multi-Rule Customization
- [ ] **Per-rule templates and regexes:** Allow `[[rules]]` in `headercraft.toml` to define their own `template` and `extract_regex`, enabling monorepos with different licenses per directory (e.g. `crates/` MIT/Apache, `server/` AGPL, `drivers/` GPL).
- [ ] **Rule action `skip`:** Support explicit `action = "skip"` for third-party or vendored directories (`vendor/**`, `third_party/**`).

### Priority 2: CLI & IDE Integration
- [ ] **`--dry-run` flag:** Add `--dry-run` as a dedicated CLI flag on `fix` for non-destructive dry-run execution.
- [ ] **Stdin / stdout piping:** Support `headercraft fix --stdin-path=src/App.java < input > output` for seamless integration into editor buffers (VS Code, Neovim, IntelliJ).
- [ ] **`--files-from <path>`:** Read NUL-delimited file lists from file or stdin (`-`), allowing `hk` and `git diff -z` to pipe file lists without argument limit ceilings.
- [ ] **`--explain` mode:** Print debugging breakdown showing matched language rule, extracted regex capture groups, Git metadata resolved, and insertion offsets.

### Priority 3: Large-Scale Performance
- [ ] **Batched Git metadata cache:** Batch `git status` and `git log` queries across all candidate files in a single process rather than launching Git subcommands per file.
- [ ] **Parallel file formatting:** Use `rayon` to parallelize file formatting across CPU cores for repos with 10,000+ files.
- [ ] **Atomic file writes:** Write files using temporary files and atomic rename to protect against unexpected termination or disk exhaustion.
