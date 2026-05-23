+++
term = "imperium"
kind = "annotation"
category = "cli"
canonical = true
summary = "Marks a function as a command handler."
syntax = "@ imperium \"<name>\""
related = ["cli", "optio", "operandus", "argumenta"]
+++

Marks a function as a named command handler in a CLI program.

```fab
@ imperium "emit"
@ optio target longum "target" typus textus vel "rust"
functio emit() argumenta args {}
```
