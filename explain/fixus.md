+++
term = "fixus"
kind = "keyword"
category = "declaration"
canonical = true
summary = "Records that a declaration slot (field or parameter) is intended to be immutable after its initial value is set."
syntax = "<type> <name> [sponte] fixus [ : default | vel default ]"
examples = ["examples/exempla/functio/optionalis.fab"]
aliases = ["fixed", "late-init", "one-time"]
related = ["sponte", "fixum", "varia", "genus"]
+++

Records post-initialization immutability intent on a declared slot.

`fixus` (Latin "fixed") indicates that once the slot receives its first value — either from an explicit argument, a field default, or construction — it should be treated as fixed and not mutated thereafter.

```fab
# Required field that becomes fixed after init
textus id fixus

# Optional + fixed: may be omitted at construction; once set (by default or value) it is fixed
textus nickname sponte fixus : "Anonymous"

# Parameter that is fixed after the call receives it
functio register(textus token fixus) → vacuum
```

`fixus` is a **declaration marker** that follows the name (after any `sponte`).

- Canonical order: `<type> <name> [sponte] [fixus] [default]`
- `fixus` without `sponte` means the slot is required but immutable after assignment.
- `sponte fixus` with default: provider may omit; default fills; result is fixed.
- Full compile-time or runtime enforcement of "no mutation after init" is provided by dedicated fixed-field / late-initialization work (see `docs/factory/fixus-late-init-bindings/`).

In current Rust codegen, `fixus` is preserved as metadata on HIR but does not emit target-level `const` or immutable wrappers (no false guarantees).

Use `fixum` for local bindings that are immutable by language rule; `fixus` for declared slots with one-time-set lifecycle intent.
