# Sponte Voluntary Declaration Marker

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/datus-given-postfix/`
**Mode**: language design / breaking change with migration

## Interpreted Problem

The current mechanism for expressing optional/nullable values (`si`) is a prefix modifier (`si textus`, `si numerus`, etc.). While functional, it creates three practical problems:

1. **Keyword overload**: `si` is already the global conditional keyword. Reusing it in type position makes `si` mean both "if" and "nullable/optional", which weakens the language's shape and makes diagnostics harder to phrase cleanly.

2. **Readability in data definitions**: In `genus` (and `pactum`) blocks, a mix of required and optional fields produces poor visual alignment because the `si` prefix pushes type names out of column:
   ```fab
   textus name
   numerus age
   si textus email
   si textus phone
   ```

3. **Modifier ordering friction**: When combining optionality with ownership modes (`de` / `in`), writers must choose an order (`si de textus` vs `de si textus`), creating unnecessary decision overhead and inconsistency.

The deeper issue is that "this declared slot is not required" and "this value's type includes `nihil`" are related but not identical concepts. The goal is to free `si` back to control flow, make declaration optionality visually subordinate to the type/name pair, and use value-type syntax for nullable return and alias forms.

## Rationale

After exploration, the chosen direction is:

- Required is the default (no keyword).
- Declaration optionality is expressed with a **post-name** marker: `sponte`.
- One-time post-initialization immutability is expressed with a second **post-name** marker: `fixus`.
- Nullable value types are expressed as a union with `nihil`: `T ∪ nihil`.
- Ownership and borrowing modes (`de`, `in`) remain **prefixes**, as they are prepositional/relational in nature and read more naturally in that position.

This creates a principled split rather than forcing all optionality through one syntax. `sponte` marks obligation: the field, parameter, option, or other declared data slot is voluntary, accepted when provided but not required from the caller/provider. `fixus` marks lifecycle: once the declared slot receives its initial value, either from the provider or from a `vel` default, it cannot be changed. `T ∪ nihil` marks value domain: the value may be either `T` or `nihil`.

**Word choice**: `sponte` (Latin "of one's own accord") was selected after evaluating several candidates (`datus`, `ultro`, `fortuitus`, `incertus`, `casus`, `fors`, etc.). It avoids the English-reader collision between `datus` and date/data terminology while preserving the intended idea: the slot is not demanded; it is supplied voluntarily. `fixus` mirrors the existing `fixum` immutability vocabulary while making the one-time-set field/property rule visible at the declaration site.

## Break Boundary

### In Scope
- Replacement of `si` as a declaration-level optional marker with `sponte` after the declared name.
- Introduction of `fixus` after the declared name as a post-initialization immutability marker.
- Replacement of `si T` nullable value types with `T ∪ nihil` in return clauses, type aliases, variable type annotations, casts/conversions, and other pure type positions.
- Grammar updates for `typeAnnotation`, field declarations, parameter declarations, return clauses, and inline union type expressions.
- Updates to the parser, declaration AST/HIR metadata, `TypeExpr`, semantic types (`Option<T>` / `Union` lowering), and downstream phases (HIR, type checking, lowering, codegen).
- Migration of existing uses of `si` in type positions across the codebase, examples, stdlib, and documentation.
- New negative tests and guardrails to prevent reintroduction of `si` for optionality.
- Documentation and teaching material updates (grammar docs, AGENTS.md, explain entries if needed).

### Out of Scope (for this plan)
- Moving `de` or `in` to postfix position.
- Changes to `ignotum` (the top-level unknown type).
- Introduction of a dedicated "required" keyword (the absence of `sponte` remains the signal).
- Broader changes to how optionality is handled in the runtime or target languages.

## Current Evidence

- `si` is currently recognized in `parse_type()` as a general nullable prefix and also appears in the parameter grammar.
- `TypeExpr` already carries a `nullable: bool` flag.
- Semantic layer already lowers nullable types to `Type::Option(...)` in many places.
- Semantic layer already has an internal `Type::Union(...)`, but source-level inline union type syntax (`T ∪ U`) is not currently implemented.
- Existing usage of `si` for optionality appears in examples, stdlib annotations, tests, and documentation.
- `de` and `in` are handled as ownership modes separate from the nullable flag.

## Proposed Design

### Syntax

```fab
# Required (default)
textus name
numerus count
de textus handle

# Optional declared slots ("voluntary")
textus email sponte
numerus score sponte
de textus avatar sponte
in numerus mutableField sponte

# Fixed after initialization
textus id fixus
textus email sponte fixus

# With default value
textus name sponte vel "Anonymous"
textus nickname sponte fixus vel "Anonymous"
```

In parameters:
```fab
functio invite(textus email sponte) → vacuum
functio paginate(numerus pagina sponte vel 1, numerus per_pagina sponte vel 10) → textus
```

In return position and pure type annotations, use union-with-`nihil` value syntax:
```fab
functio find(...) → Person ∪ nihil
typus MaybePerson = Person ∪ nihil
fixum Person ∪ nihil maybe ← nihil
```

### Design Rules

- Absence of `sponte` in a declaration → the declared slot is required from the caller/provider unless a default is supplied.
- Presence of `sponte` in a declaration → the declared slot is voluntary; it is accepted when provided but not required.
- `sponte` follows the declared name, not the type.
- `sponte` may be combined with `vel` for default values.
- `sponte` is for declaration contexts with a name slot: fields, parameters, and similar data/input declarations.
- `fixus` follows the declared name after any presence marker. Canonical order is `<type> <name> [sponte] [fixus] [vel default]`.
- `fixus` may appear without `sponte`: `textus id fixus` means a required slot that becomes immutable after initialization.
- `sponte fixus vel "Anonymous"` means the provider may omit the slot; if omitted, the default is used; after construction/defaulting the slot is fixed regardless of where the initial value came from.
- Pure type positions do not use `sponte`; they use `T ∪ nihil` when the value may be `nihil`.
- `T ∪ nihil` may lower to the existing option representation (`Option<T>`) rather than a generic ad-hoc union when exactly one non-`nihil` member is present.
- `de` and `in` continue to appear before the type.

## Stage Graph

| Phase | Name                        | Goal                                                                 | Checkpoint |
|-------|-----------------------------|----------------------------------------------------------------------|----------|
| 0     | Design & Planning           | Produce this plan and confirm scope and word choice                  | Plan approved |
| 1     | Inventory                   | Locate every use of `si` as a declaration optional marker or nullable type marker | Full classified inventory (ledger) |
| 2     | Grammar & Front-end         | Update parser, AST (`TypeExpr`), declaration parsing, union glyph lexing, and keyword handling | `sponte` and `fixus` parse in declarations; `T ∪ nihil` parses in type positions; `si` no longer accepted for optionality/nullability |
| 3     | Semantic & Lowering         | Ensure `sponte`, `fixus`, and `T ∪ nihil` produce the intended obligation/lifecycle/option/union semantics | Declaration optionality, fixed-after-initialization slots, and nullable value types typecheck and lower correctly |
| 4     | Codegen & Runtime           | Verify all targets correctly emit optional/nullable types and any supported non-null unions | All backends produce correct output or documented fallback output |
| 5     | Migration & Examples        | Update examples, stdlib, and internal tests                          | No remaining `si` used for optionality in source |
| 6     | Documentation & Teaching    | Update EBNF, grammatica docs, AGENTS.md, and explain entries         | Docs teach `sponte` for declaration optionality, `fixus` for post-initialization fixed slots, and `T ∪ nihil` for nullable value types |
| 7     | Guardrails & Validation     | Add tests and searches that protect the new rule                     | Clean CI + residue search passes |

## Open Questions

- Should `∪` initially support all inline union type expressions (`A ∪ B ∪ C`), or should implementation begin with the narrow nullable form (`T ∪ nihil`) and leave broader unions for a follow-up?
- Should `T ∪ nihil` canonicalize to `Option<T>` at semantic lowering time, while other unions lower to `Type::Union(...)`?
- How should diagnostics distinguish the concepts: "voluntary" / "optional" for declaration obligation, "fixed" for post-initialization lifecycle, and "nullable" / "may be nihil" for value type domain?
- Should `sponte` and `fixus` be contextual keywords only in declaration-marker position, allowing ordinary identifiers with those names elsewhere?
- Should reversed marker order (`textus email fixus sponte`) be rejected rather than normalized to preserve one canonical style?
- Is there a need for a short symbolic declaration form later (`textus name?`), or do we stay keyword-only?
- How will this interact with future work on contextual keywords or annotation forms?

## Validation

- All existing tests continue to pass after migration.
- `./scripta/ci` passes (fmt, test, clippy, release builds).
- Residue search for old `si` usage in type positions is clean (outside of `si` as the conditional keyword).
- New negative tests reject `si` when used for declaration optionality or nullable value types.
- New parser tests cover `textus email sponte`, `textus email sponte fixus`, `functio find() → textus ∪ nihil`, and `typus MaybeText = textus ∪ nihil`.
- New semantic tests cover `textus id fixus`, `textus email sponte fixus`, and `textus nickname sponte fixus vel "Anonymous"` so defaults are applied before fixed-state enforcement.
- New lowering/typecheck tests prove `T ∪ nihil` can accept both `T` and `nihil`, and that the supported nullable representation emits correctly for each backend.
- Real `genus` and signature examples show improved readability.

---

*This plan is intentionally a first draft. It will be refined as we work through the details.*
