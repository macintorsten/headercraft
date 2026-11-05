# Example 2: Corporate Confidentiality & Security Banners

Enforcing proprietary legal notices, security classification levels, and legal entity contact information across closed-source codebases.

## What this demonstrates
1. **Multi-language support:** Correct syntax selection across SQL (`--`), XML (`<!-- -->`), and Java (`/* */`).
2. **Prolog Preservation:** XML declaration `<?xml version="1.0" ...?>` is retained on line 1, inserting the banner safely on line 2.
3. **Centralized Legal Compliance:** Modifying `props.company`, `props.classification`, or `props.contact` immediately rolls out across all files.

## Try it
```bash
headercraft diff --config headercraft.toml before/
headercraft fix --config headercraft.toml before/
```
