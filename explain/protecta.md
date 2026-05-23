+++
term = "protecta"
kind = "keyword"
category = "modifier"
canonical = true
summary = "Parsed visibility annotation without a stable active member-visibility contract."
syntax = "@ protecta"
aliases = ["protected"]
related = ["publica", "privata"]
+++

`@ protecta` is parsed as annotation metadata, but the active compiler does not yet expose a stable protected-member visibility model.

```fab
@ protecta
functio tune() {}
```
