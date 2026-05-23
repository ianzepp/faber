+++
term = "⇒"
kind = "operator"
category = "conversion"
canonical = true
summary = "Converts a value with explicit runtime parsing or coercion semantics."
syntax = "<expression> ⇒ <type> [vel <fallback>]"
aliases = ["convert"]
related = ["⇢"]
+++

`⇒` performs runtime conversion. Parsing may fail; provide `vel` when failure should fall back to a value instead of propagating the backend's failure behavior.

```fab
fixum _ n ← "42" ⇒ numerus
fixum _ safe ← userInput ⇒ numerus vel 0
fixum _ label ← 42 ⇒ textus
```
