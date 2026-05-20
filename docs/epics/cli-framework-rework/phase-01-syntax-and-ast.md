# Phase 01: Syntax and AST

**Status**: Completed on 2026-05-20.

## Completion Notes

- `@ cli` and `@ imperium` now parse into distinct structured annotation variants with string payloads.
- `@ optio` uses the canonical binding-first grammar and rejects the historical type-first form and bare `bivalens` modifier.
- `@ optio typus <type>` and `@ operandus <type>` store full `TypeExpr` values.
- `vel` defaults are stored as `Expr` values; annotation-mode `verum`, `falsum`, and `nihil` defaults normalize to literal expressions.
- `@ versio`, `@ descriptio`, `@ alias`, and `@ imperia` remain generic annotations for this phase.
- Parser coverage includes positive AST-shape tests and syntax-shape error tests for malformed structured CLI annotations.

## Goal

Make the planned CLI annotation syntax parse into explicit AST structures instead of generic `AnnotationStmt` metadata.

## Scope

- Split `@ cli` and `@ imperium` into distinguishable annotation variants with payloads.
- Replace the historical two-form option grammar with one canonical binding-first form:

    ```fab
    @ optio <ident> [brevis <string>] [longum <string>] [typus <type>] [descriptio <string>] [ubique] [vel <value>]
    ```

- Extend `OptioAnnotation` to represent the canonical grammar, including binding name, optional explicit `typus`, short flag, long flag, description, global marker, and default value.
- Extend `OperandusAnnotation` to represent rest operands, full type syntax, binding name, description, global marker, and default value.
- Decide whether `@ versio`, `@ descriptio`, `@ alias`, and `@ imperia` remain generic annotations in this phase or become structured variants immediately.
- Add parser tests that prove the intended syntax becomes the intended AST.

## Phase Decisions

- The active implementation should not support the old type-first option syntax (`@ optio textus output ...`).
- The active implementation should not support bare `bivalens` as an option modifier.
- Omitted `typus` means `typus textus`.
- `typus bivalens` declares a boolean flag and consumes no argument value.
- Boolean flags default to `falsum`.
- Historical option syntaxes may remain documented as archive material, but they are not part of this implementation phase unless deliberately reintroduced later.

## Out Of Scope

- Command tree construction.
- Type checking CLI declarations.
- Code generation.
- Help text formatting.

## Design Questions

- Resolved for Phase 01: `vel` defaults are stored as expressions.
- Resolved for Phase 01: full type syntax is accepted by the annotation parser as `TypeExpr`, including `lista<textus>` and `lista<numerus>`.
- Resolved for Phase 01: `typus bivalens vel verum` parses as a normal default expression; default type compatibility remains a later validation concern.

## Acceptance

- CLI annotations no longer require downstream passes to parse raw token lists.
- Malformed structured CLI annotations produce parser diagnostics.
- Existing non-CLI annotations continue to round-trip as generic annotation statements.
- Parser tests reject historical option forms unless a later design pass explicitly reinstates them.
