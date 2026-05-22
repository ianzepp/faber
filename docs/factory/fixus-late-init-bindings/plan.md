# Fixus Late-Initialized Bindings

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/fixus-late-init-bindings/`
**Mode**: language design / follow-up feature

## Interpreted Problem

Faber currently has `fixum` for immutable bindings initialized at declaration time and `varia` for mutable bindings. Some code wants a third shape: a binding that is declared before its value is known, assigned exactly once later, and then fixed permanently.

Today, writers must either:

```fab
varia textus nomen
# compute value
nomen ← value
```

which overstates mutability, or restructure code to force immediate initialization even when the value naturally comes from branches or staged setup.

The new `fixus` declaration form would reuse the same concept introduced by post-name field markers: fixed after initialization. This makes `fixus` pay rent as a general lifecycle word rather than a one-off field marker.

## Proposed Design

### Syntax

```fab
fixus textus nomen

si user.hasNickname {
    nomen ← user.nickname
}
secus {
    nomen ← "Anonymous"
}

nota nomen
```

Meaning:

- `nomen` is declared but uninitialized.
- It must be assigned exactly once before any read.
- Once assigned, it cannot be reassigned.
- Assignment type must match the declared type.

This gives a consistent lifecycle family:

```fab
fixum textus nomen ← "Marcus"  # initialized now, immutable
varia textus nomen ← "Marcus"  # initialized now, mutable
fixus textus nomen             # initialized later, then immutable
```

The `fixus` keyword is also used as a post-name marker on fields/properties:

```fab
genus User {
    textus id fixus
    textus nickname sponte fixus : "Anonymous"
}
```

The shared meaning is: fixed after initialization.

## Design Rules

- `fixus` binding declarations must include an explicit type.
- `fixus name` with inferred type is out of scope for the first implementation.
- A `fixus` binding has no usable value until definitely assigned.
- Reading a `fixus` binding before assignment is an error.
- Assigning a `fixus` binding more than once on any execution path is an error.
- Failing to assign a `fixus` binding before it is read or before scope exit, when it is required later, is an error.
- `fixus` assignment uses the normal assignment operator (`←`).
- `fixus` does not imply async/await behavior and does not resurrect the removed `figendum` shorthand.

## Break Boundary

### In Scope

- Add `fixus` as a local binding declaration starter.
- Parse `fixus <type> <name>` declarations.
- Add AST/HIR representation for late-initialized immutable bindings.
- Add definite-assignment analysis for `fixus` locals.
- Reject reads before assignment and second assignments.
- Add focused tests for branch initialization, duplicate assignment, and uninitialized reads.
- Document the relationship between `fixum`, `varia`, and `fixus`.

### Out of Scope

- Inferred-type `fixus name` declarations.
- Async sugar such as old `figendum name ← asyncCall()`.
- Destructuring `fixus` declarations.
- Cross-function or object-field definite assignment beyond local bindings.
- Changing the existing `fixum` and `varia` forms.
- Changing post-name field `fixus` semantics from the sponte/fixus declaration marker plan.

## Current Evidence

- `fixus` already exists as a keyword from `docs/factory/sponte-fixus-declaration-markers/`.
- `fixum` and `varia` are already parsed as declaration starters.
- Borrow/typecheck passes already reason about reassignment and mutation, but not this exact single-assignment uninitialized state.
- Historical `figendum` async declaration sugar is no longer present and should not be reused for this concept.

## Implementation Notes

- Prefer typed-only syntax first: `fixus textus nomen`.
- Treat the parser work as small, but do not underestimate analysis. The hard part is path-sensitive definite assignment.
- Existing assignment analysis can likely be extended with a binding state:
  - `Unassigned`
  - `Assigned`
  - `MaybeAssigned`
  - `Error`
- Branch joins must merge states conservatively.
- Loops require conservative treatment unless a loop is provably executed exactly once; first version should reject ambiguous loop-based initialization.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
|-------|------|------|------------|
| 0 | Design confirmation | Confirm `fixus` binding syntax and typed-only first cut. | Plan approved. |
| 1 | Inventory | Inspect current var declaration AST/HIR, assignment analysis, borrow/typecheck passes. | Ledger identifies exact edit sites. |
| 2 | Front-end | Parse `fixus <type> <name>` and represent it in AST/HIR. | Syntax parses; no semantics promised yet. |
| 3 | Definite assignment | Enforce assigned-before-read and single-assignment path rules. | Positive/negative semantic tests pass. |
| 4 | Codegen | Emit target-local declarations compatible with late assignment. | Rust/TS/Go outputs compile for supported shapes. |
| 5 | Docs & examples | Document `fixum` / `varia` / `fixus` and add examples. | Grammar docs teach the three binding forms. |
| 6 | Validation | Full test/lint/build pass. | `./scripta/ci` passes. |

## Open Questions

- Should unassigned `fixus` at scope exit be an error only when later read, or always an error?
- Should `fixus` be allowed inside `cura`, `itera`, and destructuring contexts in the first cut?
- Should branch initialization require every branch to assign exactly once before the join, or can assignment remain deferred after the join?
- Should `fixus` be permitted for top-level declarations, or only local/function scope?
- How should diagnostics phrase the distinction between `fixum` and `fixus` clearly?

## Validation

- Positive:
  - `fixus textus name` assigned in both `si` and `secus` branches, then read.
  - `fixus numerus value` assigned once after a guard block, then read.
- Negative:
  - read before assignment.
  - second assignment after first assignment.
  - only one branch assigns before later read.
  - assignment with wrong type.
  - inferred `fixus name` rejected.

---

*This plan intentionally keeps `fixus` late-init bindings separate from the ongoing `sponte` / field-marker migration. The keyword concept is shared; the implementation risk is not.*
