---
term: "varia"
kind: "keyword"
category: "binding"
canonical: true
summary: "Declares a mutable binding."
syntax: "varia [<type>] <pattern> ← <expression>"
examples:
  - "examples/exempla/varia/varia.fab"
aliases:
  - "let"
  - "mutable"
related:
  - "fixum"
  - "←"
  - "⊕"
---

Use `varia` when a binding will be reassigned or updated later.

```fab
varia numerus counter ← 0
```
