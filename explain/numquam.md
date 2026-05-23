+++
term = "numquam"
kind = "type"
category = "type"
canonical = true
summary = "Primitive never type for code paths that do not return normally."
syntax = "numquam"
aliases = ["never"]
related = ["mori", "exitus"]
+++

Use `numquam` for functions or expressions that cannot produce a normal value.

```fab
functio fail(textus message) → numquam {
    mori message
}
```
