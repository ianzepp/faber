+++
term = "custodi"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Groups early-exit guard checks before the main body of a function."
syntax = "custodi <block>"
examples = ["examples/exempla/custodi/custodi.fab"]
aliases = ["guard"]
related = ["si", "∴", "redde"]
+++

Use `custodi` to place validation and early returns at the top of a function.

```fab
custodi {
si b ≡ 0 {
    redde 0
}
}
```
