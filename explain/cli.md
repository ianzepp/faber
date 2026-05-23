+++
term = "cli"
kind = "annotation"
category = "cli"
canonical = true
summary = "Marks the root incipit as a CLI program."
syntax = "@ cli <name>"
related = ["imperium", "optio", "operandus", "argumenta"]
+++

Marks the root `incipit` as a CLI program. Command options and operands are declared with annotations before the entry point or command function.

```fab
@ cli "faber"
@ optio verbose longum "verbose" typus bivalens
incipit argumenta args {}
```
