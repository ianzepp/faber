+++
term = "objectum"
kind = "type"
category = "type"
canonical = true
summary = "Structural object value type for open object-shaped data."
syntax = "objectum"
aliases = ["object"]
related = ["genus", "⇢", "ignotum"]
+++

Use `objectum` for open object-shaped data at dynamic boundaries, then cast or construct into a more specific shape when possible.

```fab
functio response() → objectum {
    redde { status: 200 }
}
```
