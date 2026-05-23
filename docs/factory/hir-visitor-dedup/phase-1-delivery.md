# Phase 1 Delivery: Read-Only HIR Visitor Adoption

## Status

Complete after focused verification.

## Objective

Use the existing `hir::visit::HirVisitor` infrastructure to remove duplicated read-only recursive HIR traversal from compiler analyses and codegen helpers.

## Scope

- Port simple HIR collection and predicate helpers to `HirVisitor`.
- Prefer local visitor structs over large recursive `match HirExprKind` walkers.
- Keep target emission logic unchanged.
- Preserve fail-closed behavior and diagnostics.
- Run focused tests after each coherent port.

## Initial Targets

- TypeScript await detection.
- Go expression use counting and `ad` detection.
- Faber name collection.
- Rust failable dependency analysis if it can be ported cleanly without changing suppression semantics.

## Out Of Scope

- Mutable HIR traversal.
- Typechecking mutation/finalization.
- Large semantic-pass rewrites that require control-flow-specific state.
- MIR changes.

## Validation

- `cargo fmt --all --check` passed.
- `cargo test -p radix codegen::` passed: 71 tests passed.
- `rg` confirmed the targeted duplicated codegen helpers were removed:
  `contains_await_in_expr`, `expr_contains_ad`, `collect_expr_use_counts`,
  `collect_block_use_counts`, `collect_item_use_counts`,
  `collect_pattern_use_counts`, `collect_expr_names`,
  `collect_block_names`, and `collect_pattern_names`.

## Result

The phase removed the large read-only codegen traversal walkers from:

- `crates/radix/src/codegen/ts/expr.rs`
- `crates/radix/src/codegen/go/mod.rs`
- `crates/radix/src/codegen/faber/names.rs`
- `crates/radix/src/codegen/rust/failable.rs`

The net code change was roughly 1,000 fewer lines of duplicated recursive HIR
walking in codegen surfaces, with target emission behavior unchanged.
