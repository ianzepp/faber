+++
term = "si"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Starts a conditional branch that runs when its condition is true."
syntax = "si <condition> <block>"
examples = ["examples/exempla/si/si.fab"]
aliases = ["if"]
related = ["sin", "secus", "reddit"]
+++

Use `si` for ordinary conditional control flow.

```fab
si age ≥ 18 {
scribe "Adult"
}
```
