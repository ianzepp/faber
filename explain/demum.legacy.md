+++
term = "demum"
kind = "legacy"
category = "errors"
canonical = false
summary = "Deferred finally/cleanup spelling; resource cleanup semantics are not active grammar."
syntax = "demum <block>"
aliases = ["finally"]
related = ["fac", "cape"]
canonical_term = "fac"
+++

`demum` cleanup/finally semantics are deferred. Use structured resource constructs such as `cura` where supported, or a `fac`/`cape` boundary for local recovery.

```fab
fac {
    scribe "work"
} cape err {
    scribe err
}
```
