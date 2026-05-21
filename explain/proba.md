+++
term = "proba"
kind = "keyword"
category = "testing"
canonical = true
summary = "Defines a single test case."
syntax = "proba <name> [modifiers] <block>"
examples = ["examples/exempla/proba/proba.fab"]
aliases = ["test", "it"]
related = ["probandum", "adfirma", "omitte", "futurum"]
+++

`proba` introduces one test case; the body is normal Faber code and usually contains assertions. Modifiers follow the test name.

```fab
proba "arithmetic passes" tag "math" {
adfirma 1 + 1 ≡ 2
}
```
