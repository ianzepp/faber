+++
term = "∪"
kind = "operator"
category = "type"
canonical = true
summary = "Inline union type: a value is of type A or type B. T ∪ nihil is the canonical spelling of a nullable type."
syntax = "<type> ∪ <type> [ ∪ <type> ... ]"
examples = ["examples/exempla/si/ergo-redde.fab", "examples/exempla/typus/typus.fab", "examples/exempla/est/est.fab"]
aliases = ["cup", "inline union", "ad-hoc union", "nullable union"]
related = ["discretio", "nihil", "sponte", "fixus", "typus", "→"]
+++

Declares an ad-hoc union of value types at the type level.

The `∪` (cup) operator forms an inline union: a value of the union type may be any of the listed members.

```fab
# Nullable return — the idiomatic way to express "may be absent"
functio divide(numerus a, numerus b) → numerus ∪ nihil

# Type alias for a nullable
typus MaybeText = textus ∪ nihil

# Local binding with nullable type annotation
fixum numerus ∪ nihil maybeValue ← nihil
```

**Canonicalization rules** (applied during lowering):

- `T ∪ nihil` (or `nihil ∪ T`) lowers to the internal `Option<T>` representation.
- `A ∪ B ∪ nihil` lowers to an optional union representation.
- Duplicate members are removed: `T ∪ T ∪ nihil` → `Option<T>`.
- `nihil ∪ nihil` is invalid and produces a diagnostic (degenerate absence-only union).
- Non-null unions (`A ∪ B` where neither is nihil) lower to a backend-specific fallback today. The Rust target renders them as `Box<dyn std::any::Any>`.

**Distinction from `discretio`**:

- `discretio` defines a *tagged* / *sum* / *variant* union with named cases and payloads. It is a nominal, declared type used with `discerne` / `finge`.
- `∪` defines an *untagged structural union* of anonymous members. It is primarily used for the nullable case `T ∪ nihil`; broader ad-hoc unions are supported syntactically and lower according to backend capabilities.

**Declaration markers vs value unions**:

- Use `sponte` *after the name* on parameters and genus fields to mark voluntary provision (`textus email sponte`).
- Use `T ∪ nihil` *in type position* for return clauses, `typus` aliases, casts (`⇢`), and variable type annotations when the *value* may be `nihil`.
- These are separate concerns: `sponte` controls obligation at the call/construction site; `∪ nihil` controls the domain of values the slot may hold.

The old `si T` prefix and `T?` suffix forms are no longer accepted for nullability.
