+++
term = "!=="
kind = "legacy"
category = "logic"
canonical = false
summary = "Legacy inequality spelling; use non est in Faber source."
syntax = "<expression> non est <expression>"
related = ["non est"]
canonical_term = "non est"
+++

`!==` is not canonical Faber source. Use `non est`.

```fab
fixum _ isDifferent ← left non est right
```
