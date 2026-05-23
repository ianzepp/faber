+++
term = "aut"
kind = "keyword"
category = "logic"
canonical = true
summary = "Combines boolean expressions with logical or."
syntax = "<expression> aut <expression>"
aliases = ["or"]
related = ["et", "vel", "⊻"]
+++

Combines boolean expressions with logical inclusive OR. Use `⊻` for exclusive OR.

```fab
fixum _ either ← falsum aut verum
```
