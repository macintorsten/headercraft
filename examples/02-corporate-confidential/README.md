# Example 02: Corporate Confidentiality & Security Banners

Enforce proprietary legal notices, security classification levels, and legal entity contact information across closed-source Java, XML, and SQL codebases.

---

## Visual Comparison (Before & After)

### 1. XML Prolog Preservation (`beans.xml`)

```xml
<!-- BEFORE (Single-line legacy copyright): -->
<?xml version="1.0" encoding="UTF-8"?>
<!-- Copyright (c) 2017 Globex Financial GmbH -->
<beans xmlns="http://www.springframework.org/schema/beans">
</beans>

<!-- AFTER (Prolog preserved on line 1, multiline banner inserted safely on line 2): -->
<?xml version="1.0" encoding="UTF-8"?>
<!--
  CONFIDENTIAL - PROPRIETARY INTERNAL USE ONLY
  Copyright (c) 2017-2026 Globex Financial GmbH. All rights reserved.
  Unauthorized copying or distribution is strictly prohibited.
  Contact: security@globex.example
-->
<beans xmlns="http://www.springframework.org/schema/beans">
</beans>
```

### 2. Java Block Comments (`BillingService.java`)

```java
// BEFORE (Unheadered service class):
package com.globex.billing;

public class BillingService {
    public void processPayment() {}
}

// AFTER (Prepended with standard C-style block banner):
/*
 * CONFIDENTIAL - PROPRIETARY INTERNAL USE ONLY
 * Copyright (c) 2026 Globex Financial GmbH. All rights reserved.
 * Unauthorized copying or distribution is strictly prohibited.
 * Contact: security@globex.example
 */
package com.globex.billing;

public class BillingService {
    public void processPayment() {}
}
```

### 3. SQL Double-Dash Line Comments (`accounts.sql`)

```sql
-- BEFORE:
SELECT account_id, balance, iban FROM customer_accounts WHERE active = true;

-- AFTER (Prefixed with SQL '-- ' line comment banner):
-- CONFIDENTIAL - PROPRIETARY INTERNAL USE ONLY
-- Copyright (c) 2026 Globex Financial GmbH. All rights reserved.
-- Unauthorized copying or distribution is strictly prohibited.
-- Contact: security@globex.example

SELECT account_id, balance, iban FROM customer_accounts WHERE active = true;
```

---

## How It Works

### Configuration (`headercraft.toml`)
```toml
[header]
template = """
{% set start = vars.start_year | default(git.created_year) | default(now.year) -%}
{{ props.classification }}
Copyright (c) {{ start }}{% if start | string != now.year | string %}-{{ now.year }}{% endif %} {{ props.company }}. All rights reserved.
Unauthorized copying or distribution is strictly prohibited.
Contact: {{ props.contact }}
"""
extract_regex = '(?i)copyright\s+(?:\(c\)\s+)?(?P<start_year>\d{4})(?:-(?P<end_year>\d{4}))?'

[props]
company = "Globex Financial GmbH"
classification = "CONFIDENTIAL - PROPRIETARY INTERNAL USE ONLY"
contact = "security@globex.example"
```

- **Cross-language styles**: HeaderCraft applies `/* ... */` to Java, `<!-- ... -->` to XML, and `-- ` to SQL automatically.
- **Preamble protection**: The XML parser recognises `<?xml ...?>` and inserts the banner directly below it, maintaining XML validity.
- **Centralized company properties**: Changing `props.classification` updates banners enterprise-wide.

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
