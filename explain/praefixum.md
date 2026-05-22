+++
term = "praefixum"
kind = "keyword"
category = "comptime"
canonical = true
summary = "Forces compile-time evaluation of an expression."
syntax = "praefixum(<expression>)"
related = []
+++

Forces compile-time evaluation of an expression.

```fab
fixum _ value = praefixum(1 + 2)
```
