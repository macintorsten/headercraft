# Example 01: SPDX Specification & License Migration

Migrate legacy, multiline copyright boilerplate (such as 15-line Apache 2.0 notices) into compact, machine-readable [SPDX / REUSE specification](https://spdx.dev) tags across Go, Python, and TypeScript.

---

## Visual Comparison (Before & After)

### 1. Preserving Inception Year & Owner (`queue.go`)

```go
// BEFORE (Verbose boilerplate from 2019):
/*
 * Copyright (c) 2019 Acme Corp.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
package queue

// AFTER (Standardized SPDX tags with year range 2019-2026):
/*
 * SPDX-FileCopyrightText: 2019-2026 Acme Corp.
 * SPDX-License-Identifier: Apache-2.0
 */
package queue
```

### 2. Preserving Shebang & Custom Author (`worker.py`)

```python
# BEFORE (Line comments by custom contributor in 2021):
#!/usr/bin/env python3
# Copyright (c) 2021 Custom Dev
# Licensed under Apache 2.0
import sys

# AFTER (Shebang preserved on Line 1, custom author retained):
#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2021-2026 Custom Dev
# SPDX-License-Identifier: Apache-2.0
import sys
```

### 3. Adding Missing Headers with Fallback Defaults (`retry.ts`)

```typescript
// BEFORE:
export const retry = async <T>(fn: () => Promise<T>): Promise<T> => fn();

// AFTER (Defaults to current year and default repository owner):
/*
 * SPDX-FileCopyrightText: 2026 Acme Corp.
 * SPDX-License-Identifier: Apache-2.0
 */
export const retry = async <T>(fn: () => Promise<T>): Promise<T> => fn();
```

---

## How It Works

### Configuration (`headercraft.toml`)
```toml
[header]
template = """
{% set start = vars.start_year | default(git.created_year) | default(now.year) -%}
SPDX-FileCopyrightText: {{ start }}{% if start | string != now.year | string %}-{{ now.year }}{% endif %} {{ vars.owner | default(props.owner) | default('Acme Corp.') }}
SPDX-License-Identifier: {{ props.spdx_id }}
"""

# Regex extracts inception year and original author from legacy comments
extract_regex = '(?i)(?:SPDX-FileCopyrightText:|copyright\s+(?:\(c\)\s+)?)(?P<start_year>\d{4})(?:-(?P<end_year>\d{4}))?\s+(?P<owner>[^\n\.\,]+)'

[props]
owner = "Acme Corp."
spdx_id = "Apache-2.0"
```

1. **`extract_regex`** captures `start_year` (`2019`, `2021`) and `owner` (`Custom Dev`) from the existing header comments.
2. **Template logic** checks if `start != now.year` to format `2019-2026`. If unheadered, it falls back to `2026 Acme Corp.`
3. **Format standardization** replaces arbitrary legacy text with uniform SPDX identifiers.

---

## Try It

### Option A: Via `hk` and `mise` (One-Liner)
```bash
# Check headers using hk
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
