---
term: "sin"
kind: "keyword"
category: "control-flow"
canonical: true
summary: "Adds an else-if branch after a previous si branch."
syntax: "sin <condition> <block>"
examples:
  - "examples/exempla/si/sin.fab"
aliases:
  - "else if"
related:
  - "si"
  - "secus"
---

Use `sin` after `si` when another condition should be tested before the final fallback.

```fab
sin score ≥ 80 {
scribe "B"
}
```
