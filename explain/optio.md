+++
term = "optio"
kind = "annotation"
category = "cli"
canonical = true
summary = "Declares a CLI option annotation."
syntax = "@ optio <binding> [brevis \"x\"] [longum \"name\"] [typus <type>] [descriptio \"...\"] [ubique] [vel <default>]"
related = ["operandus", "cli", "imperium", "ubique", "vel"]
+++

Declares a CLI option bound to the following entry point or command. The type follows the `typus` marker; `typus bivalens` is a boolean flag.

```fab
@ optio verbose brevis "v" longum "verbose" typus bivalens descriptio "Verbose output"
@ optio limit longum "limit" typus numerus vel 20
```
