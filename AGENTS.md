# AGENTS.md

Technical context, architectural conventions, and instructions for AI coding agents working on **HeaderCraft (`hdr`)**.

---

## 1. Project Overview

HeaderCraft is a sub-5ms, pluggable file preamble validator, metadata linter, and header formatter written in pure Rust. It is designed for native integration with `hk` and `mise`.

### Core Capabilities
- Validates and formats top-of-file headers (SPDX, copyright, generated code notices, script usage).
- Extracts metadata (`start_year`, `owner`, `author`) from existing files using regex named capture groups and reuses them in new headers.
- Evaluates multi-source variables: Git history (`git.*`), filesystem attributes (`fs.*`), clock (`now.*`), static config (`props.*`), dynamic external commands (`cmd.*`), and extracted variables (`vars.*`).
- Respects leading file directives: shebangs (`#!`), XML declarations (`<?xml`), HTML/SVG doctypes (`<!DOCTYPE`), Go build tags (`//go:build`), and Python/Ruby cookies (`# -*- coding: ...`, `# frozen_string_literal: true`).
- Supports check-only rules (`fixable = false`) with keyword matching without overwriting files.

---

## 2. Environment & Toolchains

The repository uses `mise` for toolchain management:

- **Rust**: `stable` (MSRV 1.75+)
- **Hooks & Task Runner**: `hk` (v2.11+)
- **Package Manager**: `cargo`

```bash
# Ensure shims and toolchains are available
export PATH="$HOME/.cargo/bin:$HOME/.local/share/mise/shims:$PATH"
mise install
```

---

## 3. Development Workflow & Commands

All standard development tasks are mapped in `mise.toml` and `Cargo.toml`:

```bash
# Build
cargo build                      # Debug build
cargo build --release            # Optimized binary (target/release/headercraft)

# Formatting & Linting
cargo fmt -- --check             # Check formatting
cargo fmt                        # Auto-format Rust code
cargo clippy --all-targets -- -D warnings   # Strict clippy linting (zero warnings policy)

# Running Pre-commit Hooks (Dogfooding)
hk check --all                   # Run all hooks: rustfmt, clippy, and headercraft self-check
mise run check                   # Runs fmt:check + clippy + test
```

---

## 4. Testing Instructions

Always run the full test suite before proposing or committing changes:

```bash
# Run all tests (27 unit tests + 7 CLI integration tests)
cargo test

# Run a specific unit or integration test
cargo test test_shebang_preservation
cargo test test_cli_diff_and_fix_workflow

# Run tests with full backtrace and output
RUST_BACKTRACE=1 cargo test -- --nocapture
```

### Test Organization
- `src/lib.rs` / `src/*.rs`: Unit tests inline in `mod tests` for classifier, config, extractor, and formatter.
- `tests/cli_tests.rs`: Integration tests executing the compiled CLI binary via `assert_cmd` and `tempfile`.

---

## 5. Architectural Invariants & Code Conventions

1. **Sub-5ms Execution Budget:**
   - Avoid heavyweight runtimes or network calls during scanning.
   - Use `ignore::WalkBuilder` for parallel, `.gitignore`-aware directory traversal.
   - Thread-safe command caching: static commands evaluate once per run; per-file commands evaluate once per distinct file.

2. **Preamble Directive Ordering:**
   When formatting files, never place comments above:
   1. Shebang (`#!`)
   2. HTML/SVG `<!DOCTYPE ...>`
   3. XML declaration (`<?xml ...?>`)
   4. Language/Compiler cookies (`# -*- coding: ...`, `# frozen_string_literal: true`)
   5. Go build constraints (`//go:build` and `// +build`)

3. **Keyword-Guarded Replacement:**
   - Scanning occurs only in the top 40 lines (`max_search`).
   - Never replace arbitrary top-of-file comments (e.g. docstrings or package notes). A comment block is only treated as a replaceable header if it contains one of the configured `keywords` (default: `copyright`, `license`, `spdx`, `@generated`).
   - If executable code appears before any header comment, the parser immediately switches to *insertion* mode.

4. **Line-Ending (CRLF) Idempotence:**
   - Detect source line endings (`\r\n` vs `\n`) and preserve them throughout formatting and diff generation.

5. **Exit Code & Diagnostic Contracts:**
   - `hdr check`: Exit `0` on clean; Exit `1` on violation.
   - `hdr check --format gcc`: Output `path:line:col: error: message [rule]` for seamless `hk` and IDE parsing.
   - `hdr check --format sarif`: Emit valid SARIF 2.1.0 JSON.
   - `hdr diff`: Exit `1` if diff exists (for `hk`'s `check_diff`), `0` if clean.
   - `hdr fix`: Modifies files in-place; returns Exit `1` only if non-fixable rules required manual intervention.

---

## 6. PR & Commit Conventions

- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `ci:`.
- Ensure `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass with zero warnings before pushing.
