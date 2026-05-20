# Phase 2 Delivery Spec: Split Typecheck Pass

**Phase**: 2 of 6 in `radix-large-module-refactor-factory-plan.md`
**Goal**: Split `semantic/passes/typecheck.rs` into cohesive submodules while preserving `semantic::passes::typecheck::typecheck(...)` and diagnostics behavior.
**Inputs**: Current typecheck pass, `typecheck_test.rs`, semantic pass module wiring.
**Outputs**: `semantic/passes/typecheck/mod.rs` plus responsibility modules from the master plan.
**Out of Scope**: Inference behavior changes, diagnostic wording changes, type system changes, broad test rewrites.

## Method-Anchored Extraction Plan

- `mod.rs`: shared structs/state, `typecheck`, `TypeChecker::new`, pass orchestration (`check_program`), module declarations, test path.
- `collect.rs`: `collect_items`, `collect_struct`, `function_signature`, `param_mode_from_hir`.
- `finalize.rs`: `finalize_hir`, `finalize_item`, `finalize_function`, `finalize_block`, `finalize_stmt`, `finalize_expr`, `finalize_object_field`.
- `item.rs`: `check_item`, `check_const`, `check_function`.
- `stmt.rs`: `check_block`, `check_stmt`, `check_local`, `check_return`, scope helpers if needed.
- `expr.rs`: `check_expr`, `check_expr_with_expected`, `check_ab`, closure dispatch where needed.
- `ops.rs`: `check_binary`, `check_unary`, `check_assign`, `check_assign_op`, `numeric_bin`, `common_type`.
- `call.rs`: `check_call`, `check_method_call`, `check_call_args`, `check_spread_array_compat`, `check_call_from_type`, `function_signature_from_type`, `build_call_signature`.
- `access.rs`: `check_path`, `check_field`, `check_index`, optional/non-null chains, field/index lookup, `check_lvalue`.
- `aggregate.rs`: arrays, tuples, struct literals, struct field validation, closures if their structural aggregation dependencies fit better here.
- `control.rs`: `check_if`, `check_condition`, `check_match`.
- `pattern.rs`: `check_pattern`.
- `convert.rs`: `check_verte`, `check_conversio`, `check_deref`.
- `infer.rs`: inference variables, resolution, unification, occurs check.
- `lookup.rs`: type lookup helpers, method signature lookup, primitive helpers, bindings, error helper.

## Execution Rules

- Move complete methods with attached attributes and comments.
- Convert moved `fn` methods to `pub(super) fn` unless private within the same submodule is clearly enough.
- Keep module visibility internal to `typecheck`.
- After each cohesive batch: `cargo fmt --manifest-path radix/Cargo.toml --all` and `cargo check --manifest-path radix/Cargo.toml -p radix`.
- Before commit: run the full phase gate from the master plan.

**Status**: Ready for implementation.
