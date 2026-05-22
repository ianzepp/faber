+++
term = "iace"
kind = "keyword"
category = "errors"
canonical = true
summary = "Throws a recoverable error."
syntax = "iace <expression>"
aliases = ["throw"]
related = ["⇥", "cape", "mori", "redde"]
+++

Throws a recoverable error through the enclosing `⇥` alternate-exit path, or through a local `cape` handler attached to a structured statement.

```fab
functio fail() → numerus ⇥ textus {
    iace "err"
}
```

`iace` is not fatal; `mori` is the fatal path. Without a local handler or an enclosing function with a `⇥` type, `iace` is rejected.
