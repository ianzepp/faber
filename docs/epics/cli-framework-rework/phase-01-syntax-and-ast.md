# Phase 01: Syntax and AST

## Goal

Make the planned CLI annotation syntax parse into explicit AST structures instead of generic `AnnotationStmt` metadata.

## Scope

- Split `@ cli` and `@ imperium` into distinguishable annotation variants with payloads.
- Extend `OptioAnnotation` to represent the planned grammar, including binding name, optional explicit type, short flag, long flag, boolean flag marker, description, global marker, and default value.
- Extend `OperandusAnnotation` to represent rest operands, full type syntax, binding name, description, global marker, and default value.
- Decide whether `@ versio`, `@ descriptio`, `@ alias`, and `@ imperia` remain generic annotations in this phase or become structured variants immediately.
- Add parser tests that prove the intended syntax becomes the intended AST.

## Out Of Scope

- Command tree construction.
- Type checking CLI declarations.
- Code generation.
- Help text formatting.

## Design Questions

- Should the parser accept both historical option forms permanently, or should one become a compatibility-only path with a warning later?
- Should `vel` defaults be stored as raw tokens, literals, or expressions?
- Should `lista<textus>` and `lista<numerus>` be accepted by the annotation parser as full `TypeExpr` values?

## Acceptance

- CLI annotations no longer require downstream passes to parse raw token lists.
- Malformed structured CLI annotations produce parser diagnostics.
- Existing non-CLI annotations continue to round-trip as generic annotation statements.

