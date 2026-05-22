+++
term = "→"
kind = "operator"
category = "function"
canonical = true
summary = "Introduces the normal return type for a function, method, closure, or function type."
syntax = "functio <name>(<params>) → <type> [⇥ <error-type>] <block>"
examples = ["examples/exempla/functio/functio.fab"]
aliases = ["return arrow", "arrow"]
legacy = ["->"]
related = ["⇥", "functio", "redde"]
+++

Use `→` after a parameter list when a declaration or closure spells out its normal return type. A recoverable alternate-exit type may follow with `⇥`.

```fab
functio duplica(numerus n) → numerus ⇥ textus {
    si n < 0 ergo iace "negative"
    redde n * 2
}
```
