+++
term = "numerus"
kind = "type"
category = "type"
canonical = true
summary = "Primitive integer number type."
syntax = "numerus"
aliases = ["integer", "int"]
related = ["fractus", "⇒"]
+++

Use `numerus` for integer values and indexes.

```fab
fixum numerus count ← 42
fixum _ parsed ← "42" ⇒ numerus
```
