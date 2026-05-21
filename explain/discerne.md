---
term: "discerne"
kind: "keyword"
category: "control-flow"
canonical: true
summary: "Starts an exhaustive pattern match over a value or values."
syntax: "discerne <subject> <block>"
aliases:
  - "match"
related:
  - "discretio"
  - "casu"
---

Starts an exhaustive pattern match over a value or values.

```fab
discerne value {
    casu Ok {
        scribe "ok"
    }
}
```
