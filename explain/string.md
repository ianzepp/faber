+++
term = "string"
kind = "literal"
category = "string"
canonical = true
summary = "Creates a short textus string literal."
syntax = "\"text\""
aliases = ["\"...\""]
related = ["block-string", "scriptum"]
+++

Creates a short `textus` string literal. Use string-template application for formatting.

```fab
fixum _ greeting ← "Salve"
fixum _ message ← "Salve, §!"(name)
```
