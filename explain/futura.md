+++
term = "futura"
kind = "annotation"
category = "async"
canonical = true
summary = "Marks a function as asynchronous."
syntax = "@ futura"
related = ["cursor", "cede"]
+++

Marks a function as asynchronous. `@ futura` is an annotation on the declaration, not a post-function modifier.

```fab
@ futura
functio fetchData() → textus {
    fixum _ data ← cede loadData()
    redde data
}
```
