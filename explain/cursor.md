---
term: "cursor"
kind: "annotation"
category: "async"
canonical: true
summary: "Marks a function as a generator."
syntax: "@ cursor"
related:
  - "futura"
  - "cede"
---

Marks a function as a generator.

```fab
@ cursor
functio range(numerus n) → numerus {
cede n
}
```
