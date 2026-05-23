+++
term = "fixum"
kind = "keyword"
category = "binding"
canonical = true
summary = "Declares an immutable binding."
syntax = "fixum <type|_> <pattern> [← <expression>]"
examples = ["examples/exempla/fixum/fixum.fab"]
aliases = ["const", "immutable"]
related = ["varia", "←"]
+++

Use `fixum` when the binding should not be reassigned after initialization. Runtime initialization uses `←`; `_` asks the compiler to infer the type from the initializer.

```fab
fixum numerus answer ← 42
fixum _ nomen ← "Marcus"
```
