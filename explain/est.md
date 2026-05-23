+++
term = "est"
kind = "keyword"
category = "logic"
canonical = true
summary = "Tests whether a value is or matches a type-like target."
syntax = "<expression> est <expression>"
related = ["non est"]
+++

Tests whether a value is or matches a type-like target.

```fab
fixum _ isNull ← maybeValue est nihil
```
