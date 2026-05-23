+++
term = "?["
kind = "operator"
category = "optional-chain"
canonical = true
summary = "Performs an optional indexed access."
syntax = "<expression>?[<expression>]"
related = ["?.", "?("]
+++

Performs an optional indexed access.

```fab
fixum _ first ← items?[0]
```
