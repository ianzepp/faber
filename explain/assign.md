+++
term = "←"
kind = "operator"
category = "assignment"
canonical = true
summary = "Assigns a value to a binding, field, or assignable expression."
syntax = "<place> ← <expression>"
examples = ["examples/exempla/fixum/fixum.fab"]
aliases = ["assignment", "assign"]
related = ["=", "fixum", "varia", "⊕"]
+++

Use `←` for runtime value binding and reassignment. Declarations use it when the right-hand expression is evaluated at runtime; assignable expressions use it to replace the current value.

```fab
varia numerus total ← 0
total ← total + 1
```
