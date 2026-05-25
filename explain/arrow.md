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

For bodyful named functions, omitting `→` means effect-only `vacuum`; that body cannot use `redde`. Statement-bodied closures follow the same `redde` rule. Expression-bodied closures may omit `→` and infer their result from the expression after `∴`.

```fab
functio duplica(numerus n) → numerus ⇥ textus {
    si n < 0 ∴ iace "negative"
    redde n * 2
}
```
