+++
term = "promissum"
kind = "type"
category = "async"
canonical = true
summary = "Promise-like result type associated with futura functions."
syntax = "promissum<T>"
aliases = ["promise"]
related = ["futura", "cede"]
+++

`@ futura` functions represent work whose result becomes available later; target backends may lower that shape through `promissum<T>` or the target's native promise/future type.

```fab
@ futura
functio load() → textus {
    redde "data"
}
```
