# Structural Equals Syntax Factory Plan

**Status**: planned, pending implementation review
**Created**: 2026-05-23
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/structural-equals-syntax/`
**Mode**: language surface consistency / parser, lowering, docs, and examples follow-up
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber has settled on a mechanical distinction between runtime value flow and structural value definition:

- `←` assigns or moves a value at runtime.
- `=` defines a value in a structural, declaration-time, or compile-time-shaped position.
- `ut` renames or aliases a binding in a pattern-like position.
- `:` should not carry assignment meaning.

Current syntax still violates that model in several places:

```fab
genus Point {
    numerus x : 0
    numerus y : 0
}

fixum _ p ← { x: 10, y: 20 } ⇢ Point
fixum _ event ← finge Click { x: 10, y: 20 } ⇢ Event
fixum { nomen: n } ← persona
```

Those colons are not type annotations, and they are not ternary separators. They define field values or rename a destructured binding, so the same glyph carries unrelated meanings.

The design should remove that ambiguity without making Faber feel more JavaScript-like and without forcing every empty or structural literal through a trailing conversion arrow.

## Design Law

### Runtime Assignment Uses `←`

Use `←` when a value is assigned, rebound, or mutated during execution:

```fab
fixum Point p ← Point { x = 10, y = 20 }
varia numerus count ← 0
count ← count + 1
p.x ← 30
```

This includes ordinary binding initialization because the expression on the right is evaluated and bound at runtime.

### Structural Definition Uses `=`

Use `=` when defining a value inside a structural form:

```fab
genus Point {
    numerus x = 0
    numerus y = 0
}

fixum _ p ← Point { x = 10, y = 20 }
fixum _ raw ← { x = 10, y = 20 }
fixum _ event ← finge Click { x = 10, y = 20 } ⇢ Event
```

The important distinction is not whether the value expression is constant. It is whether the surrounding syntax is a structure definition rather than a runtime assignment statement.

### Renaming Uses `ut`

Use `ut` when the source name and local binding name differ:

```fab
fixum { nomen ut n } ← persona
```

`ut` is not assignment. It is an aliasing relationship inside a pattern.

### Colon Is Not Assignment

After this plan, active colon usage should be limited to:

```fab
fixum _ result ← cond ? left : right
```

Legacy closure syntax may keep `:` only until the legacy closure cleanup removes it:

```fab
clausura User user: non user.activus
```

No new colon-based field-value or rename syntax should be introduced.

## Proposed Surface Syntax

### Genus Field Defaults

Replace colon defaults with equals defaults:

```fab
genus Point {
    numerus x = 0
    numerus y = 0
}
```

Fields without defaults stay unchanged:

```fab
genus Point {
    numerus x
    numerus y
}
```

Post-name markers remain post-name markers:

```fab
genus User {
    textus nickname sponte = "Anonymous"
    textus id fixus
}
```

### Genus Construction

Prefer typed constructor form over untyped object plus trailing conversion:

```fab
fixum _ p ← Point { x = 10, y = 20 }
```

This should become the canonical spelling for constructing a known genus value.

The current shape:

```fab
fixum _ p ← { x: 10, y: 20 } ⇢ Point
```

should be replaced by the new structural equals form and, if needed during migration, parsed only long enough to produce a direct diagnostic and fix-it.

### Object Literals

Anonymous structural literals use the same equals field syntax:

```fab
fixum _ pointish ← { x = 10, y = 20 }
```

Whether this remains a general anonymous object value or only an intermediate structural literal should be confirmed during implementation inventory. The field separator law is the same either way.

### Variant Payloads

`finge` payloads also use equals because variant payload fields are structural definitions:

```fab
fixum _ event ← finge Click { x = 10, y = 20 } ⇢ Event
```

This plan does not require redesigning `finge` or event/variant typing. It only changes the payload field separator.

### Destructuring Rename

Destructuring keeps runtime binding on the outside and aliasing on the inside:

```fab
fixum { nomen ut n } ← persona
```

The colon form is retired:

```fab
fixum { nomen: n } ← persona
```

If the current parser supports nested destructuring, nested rename positions should follow the same rule. If it does not, nested support is out of scope; the syntax law still reserves `ut` for the rename operation.

### Empty Values

Known empty collection values should put the type on the left and the absence of value content on the right:

```fab
fixum lista<numerus> emptyNumbers ← vacua
fixum tabula<textus, numerus> emptyCounts ← vacua
```

This avoids both trailing conversion arrows and JavaScript-like generic constructor literals:

```fab
fixum _ emptyNumbers ← [] ⇢ lista<numerus>
fixum _ emptyMap ← tabula<textus, numerus>{}
```

The first implementation should require an explicit declared type on the left. This should be rejected unless a later design explicitly gives `vacua` a default type or broadens contextual inference:

```fab
fixum _ xs ← vacua
```

Inferred non-empty literals remain ordinary runtime expressions:

```fab
fixum _ xs ← [1, 2, 3]
```

## Break Boundary

### In Scope

- Replace genus field default `:` with `=`.
- Replace object and structural literal field `:` with `=`.
- Replace `finge` payload field `:` with `=`.
- Replace destructuring rename `:` with `ut`.
- Add or enable typed constructor syntax: `Point { x = 10, y = 20 }`.
- Add `vacua` as the canonical empty value expression when an explicit declared type is available.
- Update parser, AST/HIR representation, lowering, type checking, codegen, diagnostics, tests, examples, EBNF, and `explain` entries.
- Add migration diagnostics for old colon forms if the implementation elects a short parse-both phase.

### Out of Scope

- Changing ternary syntax.
- Reworking closure syntax beyond the existing legacy cleanup plan.
- Deciding the final replacement for `⇢` in explicit cast/ascription positions.
- Removing every use of `⇢`; this plan only removes cases where it was compensating for missing typed construction or typed empty values.
- Redesigning `finge` variant typing.
- Adding nested destructuring if it is not already supported.
- Changing callable type syntax, function declarations, or annotation syntax.

## Compatibility Stance

Prefer a clean break for source syntax once the new forms are implemented. Faber is still young enough that carrying two meanings for field separators is more expensive than a short migration.

Implementation may still use a temporary parse-both phase if it substantially improves diagnostics:

```text
old: { x: 10 }
new: { x = 10 }
```

Any temporary support must be deliberately staged for removal and must not be documented as active syntax.

## Current Colon Sites To Audit

| Surface | Current Meaning | Decision |
|---------|-----------------|----------|
| Ternary `cond ? a : b` | symbolic branch separator | Keep |
| Legacy closure `clausura x: expr` | legacy body separator | Remove with legacy closure cleanup |
| Genus field default `field : expr` | structural field default | Change to `=` |
| Object field `{ x: expr }` | structural field value | Change to `=` |
| `finge` payload `{ x: expr }` | structural variant payload value | Change to `=` |
| Destructuring rename `{ nomen: n }` | local alias | Change to `ut` |

## Stage Graph

| Phase | Name | Goal | Checkpoint |
|-------|------|------|------------|
| 0 | Design confirmation | Confirm the four-symbol law: `←`, `=`, `ut`, and limited `:`. | Plan approved. |
| 1 | Inventory | Find parser, lexer, AST/HIR, lowering, codegen, tests, examples, EBNF, and explain sites for colon field values, destructuring rename, empty collections, and `⇢` structural construction. | Ledger identifies exact edit sites and old syntax tests. |
| 2 | Structural equals parsing | Parse `=` in genus defaults, object literals, and `finge` payloads. | New field-value forms parse and preserve existing semantics. |
| 3 | Typed construction | Add or enable `Type { field = expr }` construction for genus values. | `Point { x = 10, y = 20 }` typechecks and emits correctly. |
| 4 | Destructuring rename | Parse and lower `{ source ut local }` destructuring aliases. | Positive and negative destructuring tests pass. |
| 5 | Empty value expression | Add `vacua` for explicitly typed empty declarations. | Typed empty list/map cases pass; context-free `vacua` is rejected. |
| 6 | Diagnostics and cleanup | Add migration diagnostics for old colon forms, then remove compatibility if clean-break timing allows. | Old forms fail with actionable diagnostics or are fully removed. |
| 7 | Docs and examples | Update EBNF, explain docs, AGENTS examples, and `.fab` examples. | No active docs teach colon field values or `[] ⇢ lista<T>` as canonical empty syntax. |
| 8 | Validation | Run formatting, tests, lint, marker checks, and representative emits. | `./scripta/ci` or equivalent full gate passes. |

## Open Questions

- Should `Point { x = 10 }` fully replace `{ x = 10 } ⇢ Point`, or should the latter remain as an explicit structural conversion form for unusual cases?
- Should `finge Click { x = 10 } ⇢ Event` keep the trailing event type as the canonical spelling, or should a later variant plan introduce a qualified constructor form?
- Should `vacua` work in return positions when the function return type is known?
- Should `vacua` work as a call argument when the parameter type is known?
- Should explicit ascription/cast keep `⇢`, move to `∷`, or split into two operators later?
- Should old colon field syntax be parsed for one release with diagnostics, or removed in the same implementation phase?

## Validation Matrix

Positive syntax and semantics:

```fab
genus Point {
    numerus x = 0
    numerus y = 0
}

functio main() → numerus {
    fixum _ p ← Point { x = 10, y = 20 }
    redde p.x
}
```

```fab
functio names(Persona persona) → textus {
    fixum { nomen ut n } ← persona
    redde n
}
```

```fab
functio empty() → lista<numerus> {
    fixum lista<numerus> xs ← vacua
    redde xs
}
```

Negative syntax and diagnostics:

```fab
fixum _ p ← { x: 10 }
fixum { nomen: n } ← persona
fixum _ xs ← vacua
genus Point { numerus x : 0 }
```

The negative cases should either fail outright or, during a temporary migration phase, produce diagnostics that name the canonical replacement.

## Documentation Rules

- EBNF is the source of grammar truth.
- `explain` is the user-facing reference text.
- Examples should show typed construction and `vacua`, not trailing conversion arrows for ordinary empty values.
- Do not document old colon field syntax except in a legacy or diagnostic note while migration support exists.

---

*The design target is mechanical consistency: a reader should know what a glyph does before reading the surrounding AST. `:` can remain a separator; it should stop pretending to assign, define, or rename.*
