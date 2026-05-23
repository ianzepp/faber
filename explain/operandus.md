+++
term = "operandus"
kind = "annotation"
category = "cli"
canonical = true
summary = "Declares a CLI operand annotation."
syntax = "@ operandus [ceteri] <type> <binding> [descriptio \"...\"] [ubique] [vel <default>]"
related = ["optio", "cli", "imperium", "ceteri", "ubique"]
+++

Declares a positional CLI operand. Use `ceteri` to collect the remaining operands.

```fab
@ operandus ceteri textus files descriptio "Input files"
```
