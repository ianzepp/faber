+++
term = "∨"
kind = "operator"
category = "bitwise"
canonical = true
summary = "Combines two expressions with logical or."
syntax = "<expression> ∨ <expression>"
aliases = ["bor"]
related = ["∧", "⊻", "¬"]
+++

Combines two expressions with logical or.

```fab
fixum _ either = falsum ∨ verum
```
