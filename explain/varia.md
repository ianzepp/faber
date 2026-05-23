+++
term = "varia"
kind = "keyword"
category = "binding"
canonical = true
summary = "Declares a mutable binding."
syntax = "varia <type|_> <pattern> [← <expression>]"
examples = ["examples/exempla/varia/varia.fab"]
aliases = ["let", "mutable"]
related = ["fixum", "←", "⊕"]
+++

Use `varia` when a binding will be reassigned or updated later. Runtime initialization and later reassignment both use `←`.

```fab
varia numerus counter ← 0
counter ← counter + 1
```
