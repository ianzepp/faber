---
term: "≠"
kind: "operator"
category: "comparison"
canonical: true
summary: "Compares two values for inequality and returns bivalens."
syntax: "<expression> ≠ <expression>"
examples:
  - "examples/exempla/binarius/binarius.fab"
aliases:
  - "not equals"
  - "inequality"
legacy:
  - "!="
related:
  - "≡"
  - "adfirma"
---

Use `≠` when an expression should be true only when the two sides do not compare equal.

```fab
adfirma 10 ≠ 5
```
