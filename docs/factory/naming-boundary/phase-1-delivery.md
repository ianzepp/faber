# Phase 1 Delivery: Source Mode Enum Cleanup

**Status**: complete  
**Date**: 2026-06-04

## Scope

Renamed parser-level mode enums that model source markers:

- `syntax::ParamMode`: `Ref` → `De`, `MutRef` → `In`, `Move` → `Ex`; `Owned` unchanged (no written marker).
- `syntax::TypeMode`: `Ref` → `De`, `MutRef` → `In`.

## Files

- `crates/radix/src/syntax/ast.rs`
- `crates/radix/src/parser/{decl,types,mod_test}.rs`
- `crates/radix/src/semantic/passes/resolve.rs` (syntax comparisons only)
- `crates/radix/src/hir/lower/{decl,types}.rs` (syntax→HIR bridge arms)

## Out of scope (Phase 2)

- `HirParamMode` and `semantic::ParamMode` remain English compiler passing modes.

## Validation

See plan validation gates; focused parser + full radix test run required before commit.