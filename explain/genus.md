+++
term = "genus"
kind = "keyword"
category = "type"
canonical = true
summary = "Declares a concrete type with fields and methods."
syntax = "genus <name> [<type-params>] <block>"
aliases = ["class", "struct"]
related = ["pactum", "abstractus", "implet", "sub"]
+++

Declares a concrete type with type-first fields and methods.

```fab
genus Persona {
    textus nomen
    numerus aetas: 0

    functio salve() → textus {
        redde "Salve, §!"(ego.nomen)
    }
}
```
