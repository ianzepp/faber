---
term: "functio"
kind: "keyword"
category: "function"
canonical: true
summary: "Declares a named function or method."
syntax: "functio <name>(<params>) [→ <type>] <block>"
examples:
  - "examples/exempla/functio/functio.fab"
aliases:
  - "function"
related:
  - "→"
  - "incipit"
---

Use `functio` for reusable named behavior with optional parameters and return type.

```fab
functio duplica(numerus n) → numerus {
redde n * 2
}
```
