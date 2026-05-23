+++
term = "sponte"
kind = "keyword"
category = "declaration"
canonical = true
summary = "Marks a named declaration slot (parameter or genus field) as voluntary — the caller or provider may omit the value."
syntax = "<type> <name> sponte [fixus] [= default | vel default]"
examples = ["examples/exempla/functio/optionalis.fab", "examples/exempla/optionalis/optionalis.fab"]
aliases = ["optional", "voluntary"]
related = ["fixus", "vel", "nihil", "∪", "functio", "genus"]
+++

Marks a declaration slot (parameter or genus field) as voluntary: the provider may omit it without error.

`sponte` (Latin "of one's own accord") signals that the slot is not required; it is accepted when supplied.

```fab
# Optional parameter without default (receives nihil when omitted)
functio greet(textus nomen, textus titulus sponte) → textus

# Optional parameter with default
functio paginate(numerus pagina sponte vel 1, numerus per_pagina sponte vel 10) → textus

# Genus field that may be absent
genus User {
    textus name
    textus email sponte
    numerus score sponte fixus = 0
}
```

`sponte` is a **declaration marker**, not a type modifier. It appears after the declared name.

- Order: `type name [sponte] [fixus] [default]`
- `fixus` may appear without `sponte` for required slots that become immutable after first assignment.
- When a `sponte` slot is omitted and has no default, the runtime representation is absence (Rust `None`).
- With a default, the default supplies the value for omitted construction.

**Important distinction**: `sponte` governs *provision obligation*. It is unrelated to whether the *value domain* admits `nihil`. Use `T ∪ nihil` in pure type positions (returns, aliases, casts, annotations) when a value may legitimately be null.

See also `fixus` for post-initialization immutability and `∪` for inline union (nullable) value types.
