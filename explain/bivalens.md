+++
term = "bivalens"
kind = "type"
category = "type"
canonical = true
summary = "Primitive boolean type whose values are verum and falsum."
syntax = "bivalens"
aliases = ["boolean", "bool"]
related = ["verum", "falsum", "et", "aut", "non"]
+++

Use `bivalens` for truth values.

```fab
fixum bivalens enabled ← verum
fixum _ ready ← enabled et non falsum
```
