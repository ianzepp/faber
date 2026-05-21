---
term: "intra"
kind: "keyword"
category: "range"
canonical: true
summary: "Checks whether a value lies within a range."
syntax: "<expression> intra <range>"
related:
  - "inter"
---

Checks whether a value lies within a range.

```fab
si age intra 18 usque 65 {
scribe "working age"
}
```
