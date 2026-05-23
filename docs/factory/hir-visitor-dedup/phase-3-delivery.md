# Phase 3 Delivery: Shared Codegen HIR Error Scanner

## Status

Complete after focused verification.

## Objective

Remove the remaining shared codegen recursive HIR error-expression scanner and
route it through `HirVisitor`.

## Scope

- Replace `find_error_expr_in_*` recursive helpers in `codegen/mod.rs`.
- Preserve first-error fail-fast behavior.
- Keep backend-specific expression emitters unchanged.

## Validation

- `cargo fmt --all --check` passed.
- `cargo test -p radix codegen::rust::tests::codegen_rejects_hir_error_nodes_for_all_targets` passed.
