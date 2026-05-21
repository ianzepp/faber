+++
term = "omitte"
kind = "modifier"
category = "testing"
canonical = true
summary = "Marks a test case as skipped with a reason."
syntax = "proba <name> omitte <reason> <block>"
examples = ["examples/exempla/proba/modificatores.fab"]
aliases = ["skip"]
related = ["proba", "futurum"]
+++

Use `omitte` after `proba` when the test should be recorded but not run.

```fab
proba "database connection" omitte "blocked" {
adfirma falsum
}
```
