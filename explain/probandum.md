+++
term = "probandum"
kind = "keyword"
category = "testing"
canonical = true
summary = "Groups related test cases into a named suite."
syntax = "probandum <name> [modifiers] <block>"
aliases = ["describe", "suite"]
related = ["proba", "adfirma"]
+++

`probandum` wraps related `proba` cases so a test file can describe the behavior under test. Modifiers follow the suite name and are inherited by contained cases.

```fab
probandum "math" tag "math" {
proba "addition" {
    adfirma 1 + 1 ≡ 2
}
}
```
