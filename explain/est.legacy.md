+++
term = "==="
kind = "legacy"
category = "logic"
canonical = false
summary = "Legacy equality spelling; use est in Faber source."
syntax = "<expression> est <expression>"
related = ["est"]
canonical_term = "est"
+++

`===` is not canonical Faber source. Use `est`.

```fab
fixum _ isSame ← left est right
```
