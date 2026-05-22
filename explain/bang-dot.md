+++
term = "!."
kind = "operator"
category = "non-null-chain"
canonical = true
summary = "Performs a non-null member access."
syntax = "<expression>!.<name>"
related = ["![", "!("]
+++

Performs a non-null member access.

```fab
fixum _ name = person!.nomen
```
