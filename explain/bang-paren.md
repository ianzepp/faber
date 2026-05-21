---
term: "!("
kind: "operator"
category: "non-null-chain"
canonical: true
summary: "Performs a non-null call."
syntax: "<expression>!(<args>)"
related:
  - "!."
  - "!["
---

Performs a non-null call.

```fab
fixum value = maybeFn!(1, 2)
```
