---
term: "tempta"
kind: "keyword"
category: "errors"
canonical: true
summary: "Starts a try block."
syntax: "tempta <block> cape <name> <block> [demum <block>]"
aliases:
  - "try"
related:
  - "cape"
  - "demum"
---

Starts a try block.

```fab
tempta {
    scribe "ok"
} cape err {
    scribe err
}
```
