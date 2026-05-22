+++
term = "sic"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Starts the true branch of a one-line conditional expression."
syntax = "si <condition> sic <expression> secus <expression>"
aliases = ["then"]
related = ["si", "secus"]
+++

Starts the true branch of a one-line conditional expression.

```fab
fixum _ max = a > b sic a secus b
```
