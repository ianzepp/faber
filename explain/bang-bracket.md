+++
term = "!["
kind = "operator"
category = "non-null-chain"
canonical = true
summary = "Performs a non-null indexed access."
syntax = "<expression>![<expression>]"
related = ["!.", "!("]
+++

Performs a non-null indexed access.

```fab
fixum _ first ← items![0]
```
