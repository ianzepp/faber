+++
term = "dum"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Repeats a block while a condition remains true."
syntax = "dum <condition> <block>"
aliases = ["while"]
related = ["si", "ergo"]
+++

Repeats a block while a condition remains true.

```fab
dum counter < 5 {
    scribe counter
}
```
