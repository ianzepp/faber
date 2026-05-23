+++
term = "vacuum"
kind = "type"
category = "type"
canonical = true
summary = "Primitive no-value return type."
syntax = "vacuum"
aliases = ["void", "unit"]
related = ["functio", "redde"]
+++

Use `vacuum` when a function returns no useful value. It may be omitted from a function declaration when no return value is needed.

```fab
functio log(textus message) → vacuum {
    scribe message
    redde
}
```
