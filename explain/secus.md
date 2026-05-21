+++
term = "secus"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Runs the fallback branch when preceding conditional branches do not match."
syntax = "secus <block>"
examples = ["examples/exempla/si/secus.fab"]
aliases = ["else", "otherwise"]
related = ["si", "sin"]
+++

Use `secus` as the default branch in an `si` chain or as the alternate side of a Latin ternary expression.

```fab
secus {
scribe "Minor"
}
```
