+++
term = "errata"
kind = "modifier"
category = "function"
canonical = true
summary = "Marks a function parameter or binding as carrying an error value."
syntax = "functio <name>(...) errata <ident> → <type>"
related = ["tempta", "cape", "iace"]
+++

Marks a function parameter or binding as carrying an error value.

```fab
functio parse() errata Error → textus {
redde "ok"
}
```
