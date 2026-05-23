+++
term = "cape"
kind = "keyword"
category = "errors"
canonical = true
summary = "Starts a catch block."
syntax = "<structured-statement> cape <name> <block>"
aliases = ["catch"]
related = ["fac", "iace", "⇥"]
+++

`cape` attaches to structured statements that can own a local error boundary, including `fac`, conditional arms, loops, `elige`, `cura`, and `ad`. It does not attach to arbitrary bare blocks.

```fab
fac {
    iace "bad"
} cape err {
    scribe err
}
```
