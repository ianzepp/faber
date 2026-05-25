+++
term = "⇥"
kind = "operator"
category = "function"
canonical = true
summary = "Declares a recoverable alternate-exit type for failable functions."
syntax = "functio <name>(<params>) [→ <success-type>] ⇥ <error-type>"
examples = ["examples/exempla/functio/exitus.fab"]
aliases = ["exit arrow", "alternate exit arrow", "error arrow", "failable return"]
related = ["→", "functio", "iace", "cape", "mori"]
+++

Use `⇥` to declare the type carried by recoverable alternate exits. It may follow a normal `→` return type, or it may appear alone on an effect-only function whose normal success type is `vacuum`.

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b ≡ 0 ∴ iace "division by zero"
    redde a / b
}

functio logOrFail(textus line) ⇥ textus {
    si line ≡ "" ∴ iace "empty"
    nota line
}
```

`redde` exits through the normal `→` path. `iace` exits through the recoverable `⇥` path. `mori` remains fatal and is not part of the typed recoverable path.
