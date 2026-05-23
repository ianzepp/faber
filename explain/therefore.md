+++
term = "∴"
kind = "operator"
category = "control-flow"
canonical = true
summary = "Introduces a compact consequent, statement body, or closure body."
syntax = "<head> ∴ <statement-or-expression>"
aliases = ["ergo", "therefore"]
related = ["si", "dum", "clausura", "redde", "tacet"]
+++

`∴` is the canonical therefore marker. In statement forms it introduces one following statement body; in compact closures it separates the typed parameter signature from the expression or `fac` body.

```fab
si value < 0 ∴ redde 0
fixum _ active ← users.filtrata(_ user ∴ user.activus)
fixum _ parsed ← textus raw → numerus ∴ fac {
    redde raw ⇒ numerus
}
```
