# Example 04: Open Source Dual-Licensing (MIT OR Apache-2.0)

Standardize dual-licensing notices common in open source ecosystems (such as Rust crates and cross-language libraries) across Rust, C, and Python.

---

## Visual Comparison (Before & After)

### 1. Inception Year & Author Preservation (`lib.rs`)

```rust
// BEFORE (Minimal legacy header):
// Copyright 2020 Initial Author
pub fn parse_data(input: &[u8]) -> usize { input.len() }

// AFTER (Expanded to canonical dual-licensing notice):
/*
 * Copyright (c) 2020-2026 Initial Author
 * Licensed under either of Apache License 2.0 or MIT license at your option.
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */
pub fn parse_data(input: &[u8]) -> usize { input.len() }
```

### 2. C Block Comments (`codec.c`)

```c
// BEFORE (Unheadered native source):
#include <stddef.h>

size_t encode(const char *src) { return 0; }

// AFTER (Standardized C-style block comment with default owner):
/*
 * Copyright (c) 2026 Rust Ecosystem Contributors
 * Licensed under either of Apache License 2.0 or MIT license at your option.
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */
#include <stddef.h>

size_t encode(const char *src) { return 0; }
```

### 3. Python Line Comments (`helper.py`)

```python
# BEFORE:
def format_output(val):
    return str(val)

# AFTER:
# Copyright (c) 2026 Rust Ecosystem Contributors
# Licensed under either of Apache License 2.0 or MIT license at your option.
# SPDX-License-Identifier: MIT OR Apache-2.0

def format_output(val):
    return str(val)
```

---

## How It Works

### Configuration (`headercraft.toml`)
```toml
[header]
template = """
{% set start = vars.start_year | default(git.created_year) | default(now.year) -%}
Copyright (c) {{ start }}{% if start | string != now.year | string %}-{{ now.year }}{% endif %} {{ vars.owner | default(props.owner) | default('Project Contributors') }}
Licensed under either of Apache License 2.0 or MIT license at your option.
SPDX-License-Identifier: MIT OR Apache-2.0
"""
extract_regex = '(?i)copyright\s+(?:\(c\)\s+)?(?P<start_year>\d{4})(?:-(?P<end_year>\d{4}))?\s+(?P<owner>[^\n\.\,]+)'

[props]
owner = "Rust Ecosystem Contributors"
```

---

## Try It

### Option A: Via `hk` and `mise` (One-Liner)
```bash
# Check compliance across all files
mise exec hk@latest -- hk check --all

# Auto-fix in-place
mise run fix
```

### Option B: Standalone CLI
```bash
# 1. Preview changes as unified diff
headercraft diff --config headercraft.toml before/

# 2. Check violations in GCC diagnostic format
headercraft check --format gcc --config headercraft.toml before/

# 3. Apply changes in-place
headercraft fix --config headercraft.toml before/

# 4. Verify all files now pass
headercraft check --config headercraft.toml before/
```
