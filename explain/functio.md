+++
term = "functio"
kind = "keyword"
category = "function"
canonical = true
summary = "Declares a named function or method."
syntax = "functio <name>(<params>) [modifiers] [→ <type>] [⇥ <error-type>] <block>"
examples = ["examples/exempla/functio/functio.fab"]
aliases = ["function"]
related = ["→", "⇥", "redde", "sponte", "ceteri", "prae", "curata", "futura", "cursor"]
+++

Use `functio` for reusable named behavior. Parameters are type-first, type parameters use `prae typus`, and `redde` exits through the normal `→` return path.

If a bodyful function omits `→`, it is effect-only and its normal success type is `vacuum`. Such a body must not contain `redde`; add an explicit `→` type when the function returns a value.

Recoverable alternate exits are declared with `⇥`; values on that path are raised with `iace` and handled locally with `fac { ... } cape err { ... }`. A function may be effect-only and failable by using `⇥` without `→`.

```fab
functio divide(prae typus T, numerus a, numerus b sponte vel 1) → numerus ⇥ textus {
    si b ≡ 0 ∴ iace "division by zero"
    redde a / b
}

functio log(textus line) {
    nota line
}

functio failOnly(textus message) ⇥ textus {
    iace message
}
```
