+++
term = "fractus"
kind = "type"
category = "type"
canonical = true
summary = "Primitive floating-point number type."
syntax = "fractus"
aliases = ["float"]
related = ["numerus", "⇒"]
+++

Use `fractus` for fractional numeric values.

```fab
fixum fractus ratio ← 3.14
fixum _ parsed ← "3.14" ⇒ fractus
```
