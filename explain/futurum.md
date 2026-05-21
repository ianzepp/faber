---
term: "futurum"
kind: "modifier"
category: "testing"
canonical: true
summary: "Marks a test case as pending future work with a reason."
syntax: "proba futurum <reason> <name> <block>"
examples:
  - "examples/exempla/proba/modificatores.fab"
aliases:
  - "todo"
  - "pending"
related:
  - "proba"
  - "omitte"
---

Use `futurum` after `proba` for a test case that documents planned behavior.

```fab
proba futurum "needs API" "creates users" {
adfirma verum
}
```
