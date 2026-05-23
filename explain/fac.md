+++
term = "fac"
kind = "keyword"
category = "control-flow"
canonical = true
summary = "Starts a scoped do-while style loop."
syntax = "fac <block> [cape <name> <block>] [dum <condition>]"
aliases = ["do"]
related = ["dum", "cape", "clausura", "perge"]
+++

`fac` executes a scoped block once. Add `cape` for a local recoverable-error boundary, or add postfix `dum` to repeat the same `fac` block after each execution while the condition remains true.

Inside compact closure syntax, `fac` supplies a multi-statement body after `∴`; that closure-body form may use `cape` but not postfix `dum`.

```fab
fac {
    iace "bad"
} cape err {
    scribe err
}

fac {
    scribe "tick"
} dum verum
```
