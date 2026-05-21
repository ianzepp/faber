+++
term = "→"
kind = "operator"
category = "function"
canonical = true
summary = "Introduces an explicit function, method, or closure return type."
syntax = "functio <name>(<params>) → <type> <block>"
examples = ["examples/exempla/functio/functio.fab"]
aliases = ["return arrow", "arrow"]
legacy = ["->"]
related = ["functio", "redde"]
+++

Use `→` after a parameter list when a declaration or closure spells out its return type.

```fab
functio duplica(numerus n) → numerus {
redde n * 2
}
```
