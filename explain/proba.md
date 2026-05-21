+++
term = "proba"
kind = "keyword"
category = "testing"
canonical = true
summary = "Defines a single test case."
syntax = "proba <name> <block>"
examples = ["examples/exempla/proba/proba.fab"]
aliases = ["test", "it"]
related = ["probandum", "adfirma", "omitte", "futurum"]
+++

`proba` introduces one test case; the body is normal Faber code and usually contains assertions.

```fab
proba "arithmetic passes" {
adfirma 1 + 1 ≡ 2
}
```
