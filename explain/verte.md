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

`⇢` is a compile-time type-directed inhabitation operator. The target type determines the mode: primitive or alias casts and explicit type inhabitation. Prefer `vacua` for ordinary typed empty collection values and typed construction for ordinary `genus` values.

```fab
fixum lista<numerus> emptyNumbers ← vacua
fixum _ point ← Point { x = 10, y = 20 }
```
