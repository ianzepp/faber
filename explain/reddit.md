+++
term = "reddit"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Returns an expression from a single-line conditional consequence."
syntax = "si <condition> reddit <expression>"
examples = ["examples/exempla/si/reddit.fab"]
aliases = ["then return"]
related = ["si", "custodi", "→"]
+++

Use `reddit` for compact conditional returns where a full block would add noise.

```fab
si value < 0 reddit 0
```
