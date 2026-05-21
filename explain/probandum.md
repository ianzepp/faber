+++
term = "probandum"
kind = "keyword"
category = "testing"
canonical = true
summary = "Groups related test cases into a named suite."
syntax = "probandum <name> <block>"
aliases = ["describe", "suite"]
related = ["proba", "adfirma"]
+++

`probandum` wraps related `proba` cases so a test file can describe the behavior under test.

```fab
probandum "math" {
proba "addition" {
    adfirma 1 + 1 ≡ 2
}
}
```
