+++
term = "textus"
kind = "type"
category = "type"
canonical = true
summary = "Primitive string/text type."
syntax = "textus"
aliases = ["string"]
related = ["string", "scriptum", "⇒"]
+++

Use `textus` for Unicode text values.

```fab
fixum textus nomen ← "Marcus"
fixum _ greeting ← "Salve, §!"(nomen)
```
