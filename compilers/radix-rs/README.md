# radix-rs

Rust implementation of the Faber compiler pipeline used by the `radix` crate.

## CLI surface

The `radix` binary now has a split surface:

- product-facing commands: `build`, `targets`, `check`
- inspection commands: `lex`, `parse`, `hir`, `emit`

The legacy `emit-package` alias still exists as a hidden compatibility path, but
it is no longer part of the main help surface.

## Design overview

- Frontend stages: lexing -> parsing -> semantic passes.
- Semantic passes: collect -> resolve -> lower -> typecheck -> borrow -> exhaustive -> lint.
- HIR is the stable, resolved AST used for codegen across targets.
- Types are interned in a `TypeTable` and referenced by `TypeId`.
- Name resolution uses a scoped symbol table keyed by `DefId`.

## Known TODO items

- Resolve pass: add namespaced lookup for enum variants and class members.
- Type lowering: wire type alias resolution and generic arity checks.
- Type checking: implement bidirectional inference for expressions and statements.
- Borrow analysis: implement ownership tracking for Rust target diagnostics.
- Exhaustiveness: implement match exhaustiveness and unreachable pattern checks.
