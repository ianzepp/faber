+++
term = "fixum"
kind = "keyword"
category = "binding"
canonical = true
summary = "Declares an immutable binding."
syntax = "fixum [<type>] <pattern> ← <expression>"
examples = ["examples/exempla/fixum/fixum.fab"]
aliases = ["const", "immutable"]
related = ["varia", "←"]
+++

Use `fixum` when the binding should not be reassigned after initialization.

```fab
fixum numerus answer ← 42
```
