+++
term = "functio"
kind = "keyword"
category = "function"
canonical = true
summary = "Declares a named function or method."
syntax = "functio <name>(<params>) [modifiers] [→ <type> [⇥ <error-type>]] <block>"
examples = ["examples/exempla/functio/functio.fab"]
aliases = ["function"]
related = ["→", "⇥", "redde", "sponte", "ceteri", "prae", "curata", "futura", "cursor"]
+++

Use `functio` for reusable named behavior. Parameters are type-first, type parameters use `prae typus`, and `redde` exits through the normal `→` return path.

Recoverable alternate exits are declared with `⇥`; values on that path are raised with `iace` and handled locally with `fac { ... } cape err { ... }`.

```fab
functio divide(prae typus T, numerus a, numerus b sponte vel 1) → numerus ⇥ textus {
    si b ≡ 0 ∴ iace "division by zero"
    redde a / b
}
```
