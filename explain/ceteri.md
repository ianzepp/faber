+++
term = "ceteri"
kind = "modifier"
category = "function"
canonical = true
summary = "Collects remaining parameters, operands, or extracted fields."
syntax = "ceteri <type> <name>"
aliases = ["rest"]
related = ["functio", "operandus", "ex"]
+++

Collects remaining values at that position. In parameter and CLI operand lists it is the rest-style slot.

```fab
functio sum(ceteri numerus[] nums) → numerus {
    redde 0
}
```
