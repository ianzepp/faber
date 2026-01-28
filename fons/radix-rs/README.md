# radix-rs

Rust implementation of the Faber compiler pipeline used by the `radix` crate.

## Design overview

- Frontend stages: lexing -> parsing -> semantic passes.
- Semantic passes: collect -> resolve -> lower -> typecheck -> borrow -> exhaustive -> lint.
- HIR is the stable, resolved AST used for codegen across targets.
- Types are interned in a `TypeTable` and referenced by `TypeId`.
- Name resolution uses a scoped symbol table keyed by `DefId`.

## Known TODO items

- Collect pass: add enum variants and class members to the symbol table.
- Resolve pass: validate type names with the interner, including primitives.
- Resolve pass: resolve variant paths in patterns and `finge` constructors.
- Type lowering: wire type alias resolution and generic arity checks.
- Type checking: implement bidirectional inference for expressions and statements.
- Borrow analysis: implement ownership tracking for Rust target diagnostics.
- Exhaustiveness: implement match exhaustiveness and unreachable pattern checks.
