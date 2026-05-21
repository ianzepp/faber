+++
term = "solum_in"
kind = "keyword"
category = "testing"
canonical = true
summary = "Restricts a test to a named environment or target."
syntax = "solum_in <string> <string>"
related = ["requirit"]
+++

Restricts a test to a named environment or target.

```fab
proba "case" solum_in "ci" {}
```
