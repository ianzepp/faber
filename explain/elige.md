+++
term = "elige"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Starts a value-based branch statement."
syntax = "elige <expression> <block>"
aliases = ["switch", "choose"]
related = ["casu", "ceterum"]
+++

Starts a value-based branch statement.

```fab
elige status {
    casu "active" {
        scribe "Running"
    }
}
```
