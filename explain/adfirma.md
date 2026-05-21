---
term: "adfirma"
kind: "keyword"
category: "testing"
canonical: true
summary: "Asserts that an expression is true at runtime or inside a test."
syntax: "adfirma <expression>"
examples:
  - "examples/exempla/adfirma/adfirma.fab"
aliases:
  - "assert"
related:
  - "proba"
  - "≡"
  - "≠"
---

Use `adfirma` to fail execution when a required condition is false.

```fab
adfirma result ≡ "hello world"
```
