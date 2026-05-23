+++
term = "nihil"
kind = "keyword"
category = "boolean"
canonical = true
summary = "Represents the null value and can prefix a null check."
syntax = "nihil [<expression>]"
related = ["nonnihil", "nonnulla", "∪", "sponte"]
+++

Represents the null value and can prefix a null check.

```fab
fixum _ nothing ← nihil
```

In type positions, the canonical nullable form is the union `T ∪ nihil` (see `∪`). Declaration-level optionality uses the `sponte` marker after the name (`textus email sponte`), which is distinct from value-domain nullability.
