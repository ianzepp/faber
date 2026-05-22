+++
term = "vel"
kind = "keyword"
category = "logic"
canonical = true
summary = "Provides a default when the left side is null."
syntax = "<expression> vel <expression>"
aliases = ["??"]
related = ["nihil", "nonnihil"]
+++

Provides a default when the left side is null.

```fab
fixum _ name = maybeName vel "default"
```
