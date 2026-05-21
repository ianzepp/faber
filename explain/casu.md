+++
term = "casu"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Introduces a branch inside an elige or discerne statement."
syntax = "casu <expression> <block>"
aliases = ["case"]
related = ["elige", "discerne"]
+++

Introduces a branch inside an elige or discerne statement.

```fab
casu 200 {
    scribe "OK"
}
```
