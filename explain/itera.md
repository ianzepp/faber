+++
term = "itera"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Starts a for-each iteration statement."
syntax = "itera <mode> <expression> <binding> <block>"
aliases = ["iterate"]
related = ["ex", "de", "ab"]
+++

Starts a for-each iteration statement.

```fab
itera ex numbers fixum n {
    scribe n
}

itera ab 0‥3 fixum i {
    scribe i
}
```
