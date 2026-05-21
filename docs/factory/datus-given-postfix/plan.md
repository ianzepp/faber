# Datus Given Declaration Marker

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
- Declaration optionality is expressed with a **post-name** marker: `datus`.
- Nullable value types are expressed as a union with `nihil`: `T ∪ nihil`.
- Ownership and borrowing modes (`de`, `in`) remain **prefixes**, as they are prepositional/relational in nature and read more naturally in that position.

This creates a principled split rather than forcing all optionality through one syntax. `datus` marks obligation: the field, parameter, option, or other declared data slot is not required, but may be freely given by the caller/provider. `T ∪ nihil` marks value domain: the value may be either `T` or `nihil`.

**Word choice**: `datus` (Latin "given", past participle of *dare*) was selected after evaluating several candidates (`fortuitus`, `incertus`, `casus`, `fors`, etc.). It is not intended to mean "the value is currently present." It means the declared slot is not demanded; it is accepted when given.

## Break Boundary

### In Scope
- Replacement of `si` as a declaration-level optional marker with `datus` after the declared name.
- Replacement of `si T` nullable value types with `T ∪ nihil` in return clauses, type aliases, variable type annotations, casts/conversions, and other pure type positions.
- Grammar updates for `typeAnnotation`, field declarations, parameter declarations, return clauses, and inline union type expressions.
- Updates to the parser, `TypeExpr`, semantic types (`Option<T>` / `Union` lowering), and downstream phases (HIR, type checking, lowering, codegen).
- Migration of existing uses of `si` in type positions across the codebase, examples, stdlib, and documentation.
- New negative tests and guardrails to prevent reintroduction of `si` for optionality.
- Documentation and teaching material updates (grammar docs, AGENTS.md, explain entries if needed).

### Out of Scope (for this plan)
- Moving `de` or `in` to postfix position.
- Changes to `ignotum` (the top-level unknown type).
- Introduction of a dedicated "required" keyword (the absence of `datus` remains the signal).
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

# Optional declared slots ("given")
textus email datus
numerus score datus
de textus avatar datus
in numerus mutableField datus

# With default value
textus name datus vel "Anonymous"
```

In parameters:
```fab
functio invite(textus email datus) → vacuum
functio paginate(numerus pagina datus vel 1, numerus per_pagina datus vel 10) → textus
```

In return position and pure type annotations, use union-with-`nihil` value syntax:
```fab
functio find(...) → Person ∪ nihil
typus MaybePerson = Person ∪ nihil
fixum Person ∪ nihil maybe ← nihil
```

### Design Rules

- Absence of `datus` in a declaration → the declared slot is required.
- Presence of `datus` in a declaration → the declared slot is not required; it is supplied only when given.
- `datus` follows the declared name, not the type.
- `datus` may be combined with `vel` for default values.
- `datus` is for declaration contexts with a name slot: fields, parameters, and similar data/input declarations.
- Pure type positions do not use `datus`; they use `T ∪ nihil` when the value may be `nihil`.
- `T ∪ nihil` may lower to the existing option representation (`Option<T>`) rather than a generic ad-hoc union when exactly one non-`nihil` member is present.
- `de` and `in` continue to appear before the type.

## Stage Graph

| Phase | Name                        | Goal                                                                 | Checkpoint |
|-------|-----------------------------|----------------------------------------------------------------------|----------|
| 0     | Design & Planning           | Produce this plan and confirm scope and word choice                  | Plan approved |
| 1     | Inventory                   | Locate every use of `si` as a declaration optional marker or nullable type marker | Full classified inventory (ledger) |
| 2     | Grammar & Front-end         | Update parser, AST (`TypeExpr`), declaration parsing, union glyph lexing, and keyword handling | `datus` parses in declarations; `T ∪ nihil` parses in type positions; `si` no longer accepted for optionality/nullability |
| 3     | Semantic & Lowering         | Ensure `datus` and `T ∪ nihil` produce the intended option/union semantics | Declaration optionality and nullable value types typecheck and lower correctly |
| 4     | Codegen & Runtime           | Verify all targets correctly emit optional/nullable types and any supported non-null unions | All backends produce correct output or documented fallback output |
| 5     | Migration & Examples        | Update examples, stdlib, and internal tests                          | No remaining `si` used for optionality in source |
| 6     | Documentation & Teaching    | Update EBNF, grammatica docs, AGENTS.md, and explain entries         | Docs teach `datus` for declaration optionality and `T ∪ nihil` for nullable value types |
| 7     | Guardrails & Validation     | Add tests and searches that protect the new rule                     | Clean CI + residue search passes |

## Open Questions

- Should `∪` initially support all inline union type expressions (`A ∪ B ∪ C`), or should implementation begin with the narrow nullable form (`T ∪ nihil`) and leave broader unions for a follow-up?
- Should `T ∪ nihil` canonicalize to `Option<T>` at semantic lowering time, while other unions lower to `Type::Union(...)`?
- How should diagnostics distinguish the concepts: "optional" / "given" for declaration obligation, and "nullable" / "may be nihil" for value type domain?
- Should `datus` be a contextual keyword only in declaration-marker position, allowing ordinary identifiers named `datus` elsewhere?
- Is there a need for a short symbolic declaration form later (`textus name?`), or do we stay keyword-only?
- How will this interact with future work on contextual keywords or annotation forms?

## Validation

- All existing tests continue to pass after migration.
- `./scripta/ci` passes (fmt, test, clippy, release builds).
- Residue search for old `si` usage in type positions is clean (outside of `si` as the conditional keyword).
- New negative tests reject `si` when used for declaration optionality or nullable value types.
- New parser tests cover `textus email datus`, `functio find() → textus ∪ nihil`, and `typus MaybeText = textus ∪ nihil`.
- New lowering/typecheck tests prove `T ∪ nihil` can accept both `T` and `nihil`, and that the supported nullable representation emits correctly for each backend.
- Real `genus` and signature examples show improved readability.

---

*This plan is intentionally a first draft. It will be refined as we work through the details.*
