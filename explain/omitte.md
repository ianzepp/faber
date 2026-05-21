---
term: "omitte"
kind: "modifier"
category: "testing"
canonical: true
summary: "Marks a test case as skipped with a reason."
syntax: "proba omitte <reason> <name> <block>"
examples:
  - "examples/exempla/proba/modificatores.fab"
aliases:
  - "skip"
related:
  - "proba"
  - "futurum"
---

Use `omitte` after `proba` when the test should be recorded but not run.

```fab
proba omitte "blocked" "database connection" {
adfirma falsum
}
```
