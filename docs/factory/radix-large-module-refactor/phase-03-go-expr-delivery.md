# Phase 3 Delivery Spec: Split Go Expression Codegen

**Phase**: 3 of 6 in the now-deleted `radix-large-module-refactor-factory-plan.md`
**Goal**: Split `codegen/go/expr.rs` into expression helper submodules while preserving `expr::generate_expr(...)` and Go output.
**Inputs**: Current Go expression codegen, Go codegen tests, existing `decl.rs`/`stmt.rs` call sites.
**Outputs**: `codegen/go/expr/mod.rs` plus helper modules for literals, ops, collection, access, option, call, convert, and variants.
**Out of Scope**: Go output behavior changes, type rendering changes, broad Go backend rewrites.

## Method-Anchored Extraction Plan

- `mod.rs`: `generate_expr`, `generate_expr_for_go_type`, and shared helpers used by many submodules.
- `literal.rs`: `generate_literal`, `render_scriptum_template`.
- `ops.rs`: unary, binary, assignment operator rendering.
- `collection.rs`: map/array literals, typed array literals, `ab` pipelines and filters.
- `access.rs`: map member expression and asserted map value typing.
- `option.rs`: option wrapping, optional chains, coalesce, option field helpers.
- `call.rs`: intrinsics, spread recovery, translated method calls, receiver handling.
- `convert.rs`: `verte` array conversion, boolean conversion, value conversion.
- `variants.rs`: variant constructors and variant value detection.

## Execution Rules

- Keep `expr::generate_expr` and `expr::generate_expr_for_go_type` stable for `decl.rs` and `stmt.rs`.
- Move complete free functions with attached comments.
- Re-export helper modules inside `expr/mod.rs` only at `pub(super)` visibility.
- Run formatter, `cargo check`, and the full validation gate before committing.

**Status**: Ready for implementation.
