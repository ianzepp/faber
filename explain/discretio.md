+++
term = "discretio"
kind = "keyword"
category = "type"
canonical = true
summary = "Declares a tagged union with variant payloads."
syntax = "discretio <name> [<type-params>] <block>"
aliases = ["union", "sum type"]
related = ["ordo", "discerne", "finge"]
+++

Declares a tagged union with variant payloads.

```fab
discretio Resultatus<T> {
Ok { T valor },
Err { textus nuntius }
}
```
