# Structural Equals Syntax Phase 7 Delivery

**Status**: complete
**Phase**: 7 - Docs and examples
**Date**: 2026-05-23

## Target

Update active documentation and examples so canonical source uses:

- `=` for structural field defaults and field values.
- `ut` for destructuring aliases.
- typed construction for ordinary genus values.
- `vacua` for explicitly typed empty collection values.

## Validation

- Active docs no longer teach colon field values as current syntax.
- Example programs parse with the current clean-break parser.

Commands run:

```bash
cargo run -q -p radix --bin radix -- parse <changed-example>
cargo test -p radix
```

The full example-tree parse sweep still reports pre-existing legacy `tempta` examples; changed examples parse cleanly.
