+++
term = "dum"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Repeats a block while a condition remains true."
syntax = "dum <condition> <block|∴ statement>"
aliases = ["while"]
related = ["si", "∴", "fac"]
+++

Prefix `dum` is a pre-test loop: the condition is checked before each execution. Postfix `dum` belongs to `fac`, forming a do-while loop after the block has executed once.

```fab
dum counter < 5 {
    scribe counter
}

fac {
    counter ⊕ 1
} dum counter < 5
```
