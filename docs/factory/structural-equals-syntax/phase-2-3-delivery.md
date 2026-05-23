# Phase 2/3 Delivery: Structural Equals Parsing And Typed Construction

**Status**: complete
**Checkpoint**: structural `=` field-value forms parse and preserve existing semantics; `Point { x = 10 }` typechecks through the existing struct construction path.

## Scope

This slice implements phases 2 and 3 from the factory plan:

- Accept `=` for genus field defaults.
- Accept `=` for object literal field values.
- Accept `=` for `finge` payload field values.
- Add typed constructor parsing for named types with structural field bodies, such as `Point { x = 10, y = 20 }`.
- Update Faber self-codegen to emit `=` for structural fields.

## Compatibility

Colon field forms are still accepted temporarily in the parser so existing examples and tests continue to compile. They are no longer emitted by Faber self-codegen. Diagnostics and eventual clean-break removal remain phase 6 work.

Typed construction is intentionally gated to brace bodies that look like structural fields. This avoids misparsing existing control-flow forms such as `si ready { ... }`, `dum ready { ... }`, and `casu Active { ... }`.

## Validation

- `cargo check -p radix`
- `cargo test -p radix structural_equals`
- `cargo test -p radix`
- Representative CLI check for:

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

Result: the program checks successfully, with only the expected unused-function warning in the temporary verification file.
