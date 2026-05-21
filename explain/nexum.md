+++
term = "nexum"
kind = "modifier"
category = "type"
canonical = true
summary = "Marks a class member as bound to the current instance."
syntax = "nexum <type> <name> [: <expression>]"
related = ["genus", "ego"]
+++

Marks a class member as bound to the current instance.

```fab
genus Persona {
nexum numerus aetas: 3
}
```
