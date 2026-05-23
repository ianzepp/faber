+++
term = "∷"
kind = "operator"
category = "conversion"
canonical = true
summary = "Explicitly ascribes a static target type to an expression."
syntax = "<expression> ∷ <type>"
aliases = ["verte", "cast", "ascription"]
related = ["⇒"]
+++

`∷` is a compile-time type-ascription operator. It gives an existing expression an explicit static target type. Prefer `vacua` for ordinary typed empty collection values and typed construction for ordinary `genus` values.

```fab
fixum lista<numerus> emptyNumbers ← vacua
fixum _ point ← Point { x = 10, y = 20 }
fixum _ text ← data ∷ textus
```
