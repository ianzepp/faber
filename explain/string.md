+++
term = "\"...\""
kind = "literal"
category = "string"
canonical = true
summary = "Creates a short textus string literal."
syntax = "\"text\""
related = ["block-string", "scriptum"]
+++

Creates a short `textus` string literal. Use string-template application for formatting.

```fab
fixum greeting = "Salve"
fixum message = "Salve, §!"(name)
```
