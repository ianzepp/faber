+++
term = "="
kind = "operator"
category = "declaration"
canonical = true
summary = "Defines a compile-time declaration equivalence, such as a type alias or enum value."
syntax = "<declaration-name> = <compile-time-value>"
aliases = ["compile-time assignment", "definition"]
related = ["←", "typus", "ordo"]
+++

Use `=` where the grammar defines compile-time identity or declaration metadata. Use `←` for runtime value binding.

```fab
typus UserId = numerus

ordo Status {
    Ok = 0,
    Failed = 1,
}
```
