+++
term = "imperia"
kind = "annotation"
category = "cli"
canonical = true
summary = "Marks a module as a mounted command tree."
syntax = "@ imperia <path>"
related = ["cli", "imperium"]
+++

Generic metadata for a mounted command tree. The active parser keeps `@ imperia` as a generic annotation; command lowering currently centers on structured `@ cli`, `@ imperium`, `@ optio`, and `@ operandus`.

```fab
@ imperia "jobs" ex jobsModulum
```
