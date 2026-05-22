+++
term = "⇥"
kind = "operator"
category = "function"
canonical = true
summary = "Declares a recoverable alternate-exit type after a normal return type."
syntax = "functio <name>(<params>) → <success-type> ⇥ <error-type>"
examples = ["examples/exempla/functio/exitus.fab"]
aliases = ["exit arrow", "alternate exit arrow", "error arrow", "failable return"]
related = ["→", "functio", "iace", "cape", "mori"]
+++

Use `⇥` after a normal `→` return type to declare the type carried by recoverable alternate exits.

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b ≡ 0 ergo iace "division by zero"
    redde a / b
}
```

`redde` exits through the normal `→` path. `iace` exits through the recoverable `⇥` path. `mori` remains fatal and is not part of the typed recoverable path.
