# Structural Equals Syntax Phase 6 Delivery

**Status**: complete
**Phase**: 6 - Diagnostics and cleanup
**Date**: 2026-05-23

## Target

Remove temporary colon compatibility for structural field values after the replacement syntax is implemented.

## Scope

- Genus field defaults must use `=`.
- Object literal fields must use `=`.
- Typed constructor fields must use `=`.
- `finge` payload fields must use `=`.
- Destructuring renames must use `ut`.

## Validation

- Parser negative tests cover retired colon forms.
- Radix tests pass after updating stale syntax fixtures.

Commands run:

```bash
cargo test -p radix
```
