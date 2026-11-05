# Example 4: Dual-Licensing (MIT OR Apache-2.0)

Configuring standard dual-licensing notices common in open source ecosystems (such as Rust crates).

## What this demonstrates
1. **Preserving Initial Author & Inception Year:** File `lib.rs` preserves `Initial Author` and inception year `2020` -> `2020-2026`.
2. **Dual SPDX identifiers:** Valid `SPDX-License-Identifier: MIT OR Apache-2.0` syntax.
3. **Cross-language rules:** Applied seamlessly across Rust, C, and Python.

## Try it
```bash
headercraft diff --config headercraft.toml before/
headercraft fix --config headercraft.toml before/
```
