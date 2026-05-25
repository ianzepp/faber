+++
term = "clausura"
kind = "keyword"
category = "function"
canonical = true
summary = "Declares or explains an inline closure expression; compact ∴ syntax is preferred for new code."
syntax = "<type> <name> [→ <type>] [⇥ <error-type>] ∴ <expression|fac-block>"
aliases = ["closure", "lambda"]
related = ["∴", "functio", "fac", "cape", "cede"]
+++

Prefer compact closure syntax: type-first parameters followed by `∴`. Use `_` in the type slot when the surrounding callable supplies the parameter type.

Expression-bodied closures may infer their result from the expression after `∴`. Use a `fac` body for multi-statement closures. A closure `fac` body may attach `cape`, but it cannot use postfix `dum`.

If a closure body uses `redde`, the closure itself must declare an explicit normal return type with `→`. Context from the surrounding call does not create an implicit `redde` channel.

If a closure body uses an escaping `iace`, the closure itself must declare an explicit recoverable error type with `⇥`. A local `fac { ... } cape err { ... }` may catch `iace` inside the closure without `⇥`.

```fab
fixum _ active ← users.filtrata(_ user ∴ user.activus)
fixum _ total ← numeri.compone((numerus a, numerus b) ∴ a + b)
fixum _ failer ← numerus n ⇥ textus ∴ fac {
    iace "bad"
}
fixum _ parsed ← texts.mappata(textus s → numerus ⇥ textus ∴ fac {
    redde s ⇒ numerus
} cape err {
    redde 0
})
```
