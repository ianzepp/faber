---
term: "prae"
kind: "modifier"
category: "type"
canonical: true
summary: "Introduces a compile-time type parameter."
syntax: "prae typus <name>"
aliases:
  - "before"
related:
  - "typus"
  - "functio"
---

Introduces a compile-time type parameter.

```fab
functio max(prae typus T, T a, T b) → T {
redde a
}
```
