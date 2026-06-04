# Phase 2 Delivery: HIR Bridge Review

**Status**: complete  
**Date**: 2026-06-04

## Decision

`HirParamMode` preserves source ownership markers through HIR and remains
source-shaped. It is not the same enum as `semantic::ParamMode`, which models
compiler passing policy in English.

## Change

Aligned `HirParamMode` with syntax markers:

- `Ref` → `De`, `MutRef` → `In`, `Move` → `Ex`; `Owned` unchanged.
- `hir/lower/decl.rs` now lowers syntax modes 1:1 into HIR.
- `param_mode_from_hir` in typecheck maps HIR modes to `semantic::ParamMode`
  (`De` → `Ref`, etc.) at the compiler boundary.

## Rationale

HIR still carries spans and source contracts for borrow checking and Faber
round-trip codegen. English names on Latin semantics duplicated the Phase 1
problem without adding semantic normalization value.