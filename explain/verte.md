+++
term = "⇢"
kind = "operator"
category = "conversion"
canonical = true
summary = "Makes a value inhabit a target type shape through type-directed cast, construction, or native realization."
syntax = "<expression> ⇢ <type>"
aliases = ["verte", "cast"]
related = ["⇒"]
+++

`⇢` is a compile-time type-directed inhabitation operator. The target type determines the mode: primitive or alias casts, empty native collection construction, or `genus` instantiation from object-shaped data.

```fab
fixum _ emptyNumbers ← [] ⇢ lista<numerus>
fixum _ point ← { x: 10, y: 20 } ⇢ Point
```
