# Phase 4 Delivery: Destructuring Rename

**Status**: complete
**Checkpoint**: `{ source ut local }` parses and lowers as object binding destructuring; old colon rename remains unimplemented and should fail.

## Scope

Implement object binding destructuring for local declarations:

```fab
fixum { nomen ut n } ← persona
varia { count ut c } ← stats
```

The outside binding still uses runtime assignment `←`; the inside alias uses `ut`. This phase does not add nested object destructuring.

## Implementation Plan

- Extend `BindingPattern` with an object-field form reusing the existing `ExField` shape.
- Parse `fixum { field [ut alias], ... } ← expr` and `varia { ... } ← expr` as untyped destructuring declarations.
- Lower object binding declarations by expanding each field to a local initialized from `source.field`, matching the existing `ex source fixum field ut alias` behavior.
- Teach resolver and AST visitors about the new binding variant.
- Add parser and driver tests for positive `ut` aliases and negative colon rename syntax.

## Validation

- `cargo check -p radix`
- `cargo test -p radix object_destruct`
- `cargo test -p radix`
