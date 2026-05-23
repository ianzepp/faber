+++
term = "pactum"
kind = "keyword"
category = "type"
canonical = true
summary = "Declares a behavioral contract with method signatures."
syntax = "pactum <name> [<type-params>] <block>"
aliases = ["interface"]
related = ["genus", "implet"]
+++

Declares a behavioral contract with method signatures but no method bodies.

```fab
pactum Drawable {
    functio draw() → vacuum
}
```
