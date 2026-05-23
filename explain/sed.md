+++
term = "sed"
kind = "keyword"
category = "pattern"
canonical = true
summary = "Introduces a regex literal."
syntax = "sed <pattern> [<flags>]"
related = ["praefixum"]
+++

Introduces a regex literal.

```fab
fixum _ re ← sed "\d+" g
```
