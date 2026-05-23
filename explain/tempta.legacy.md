+++
term = "tempta"
kind = "legacy"
category = "errors"
canonical = false
summary = "Legacy try/catch spelling; use fac with cape for local recovery."
syntax = "fac <block> cape <name> <block>"
aliases = ["try"]
related = ["fac", "cape"]
canonical_term = "fac"
+++

`tempta` is not canonical Faber source. Use `fac { ... } cape err { ... }` for a one-shot recoverable-error boundary.

```fab
fac {
    scribe "ok"
} cape err {
    scribe err
}
```
