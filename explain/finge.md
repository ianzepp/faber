+++
term = "finge"
kind = "keyword"
category = "object"
canonical = true
summary = "Constructs a tagged union variant."
syntax = "finge <variant> [{ <field> = <expr>, ... }] [∷ <type>]"
related = ["discretio", "verte"]
+++

Constructs a tagged union variant.

```fab
fixum _ event ← finge Click { x = 1, y = 2 } ∷ Eventus
```
