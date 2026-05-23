# Phase 5 Delivery: Typed Empty Value Expression

**Status**: complete
**Checkpoint**: `vacua` works where an explicit expected type is available and is rejected without context.

## Scope

Implement the canonical empty value expression:

```fab
fixum lista<numerus> xs ← vacua
fixum tabula<textus, numerus> counts ← vacua
```

The first implementation is intentionally context-required. `fixum _ xs ← vacua` should fail because there is no declared type to inhabit.

## Implementation Plan

- Parse standalone `vacua` as a dedicated expression while leaving member names such as `.vacua()` unaffected.
- Lower to a dedicated HIR expression so typechecking can require an expected type.
- In typechecking, accept `vacua` only when the expected type resolves to a supported empty collection shape.
- Emit target-specific empty values for Rust, TypeScript, Go, and Faber roundtrip code.
- Add positive list/map tests and a negative context-free test.

## Validation

- `cargo check -p radix`
- `cargo test -p radix vacua`
- `cargo test -p radix`
