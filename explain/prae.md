+++
term = "prae"
kind = "modifier"
category = "type"
canonical = true
summary = "Introduces a compile-time type parameter before ordinary parameters."
syntax = "prae typus <name>"
aliases = ["before"]
related = ["typus", "functio"]
+++

Introduces a compile-time type parameter. `prae typus` entries come first in the function parameter list.

```fab
functio max(prae typus T, T a, T b) → T {
    redde a
}
```
