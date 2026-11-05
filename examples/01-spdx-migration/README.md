# Example 1: SPDX Migration

Migrating legacy verbose copyright boilerplate (e.g. 15-line Apache 2.0 notices) to compact, machine-readable [REUSE / SPDX specification](https://spdx.dev) tags.

## What this demonstrates
1. **Preserving History:** `extract_regex` preserves the original inception year (`2019`, `2021`) and author (`Custom Dev`).
2. **Fallback Defaults:** Unheadered files (`retry.ts`) fall back to the current year (`2026`) and default owner (`Acme Corp.`).
3. **Format Standardization:** Converts multiline comment blocks and line comments into standard `SPDX-FileCopyrightText` and `SPDX-License-Identifier` tags.

## Try it
```bash
headercraft diff --config headercraft.toml before/
headercraft fix --config headercraft.toml before/
```
