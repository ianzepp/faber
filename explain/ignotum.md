+++
term = "ignotum"
kind = "type"
category = "type"
canonical = true
summary = "Top-level unknown escape type; not a nullability marker."
syntax = "ignotum"
aliases = ["unknown", "any"]
related = ["∷", "est", "∪"]
+++

Use `ignotum` at an external boundary when the type is not known yet. Narrow or cast before using specific members.

```fab
functio require(textus path) → ignotum
fixum _ runtime ← require("runtime") ∷ Runtime
```
