+++
term = "block-string"
kind = "literal"
category = "string"
canonical = true
summary = "Creates a block textus string literal."
syntax = "❝text❞"
aliases = ["❝...❞"]
related = ["string"]
+++

Creates a block `textus` string literal. Block strings may span lines and can contain ordinary double quotes without escaping.

```fab
fixum _ quote = ❝he said "salve"❞
```
