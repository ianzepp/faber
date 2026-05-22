+++
term = "cede"
kind = "keyword"
category = "async"
canonical = true
summary = "Awaits a promise or yields a value depending on context."
syntax = "cede <expression>"
aliases = ["await", "yield"]
related = ["futura", "cursor"]
+++

Awaits a promise or yields a value depending on context.

```fab
fixum _ data = cede fetchData()
```
