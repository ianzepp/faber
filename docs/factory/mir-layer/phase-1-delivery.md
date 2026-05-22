# Phase 1 Delivery: MIR Data Model

## Interpreted Problem

Introduce the MIR module as a model-only compiler layer. Phase 1 should define stable MIR node types, typed references, block structure, and deterministic formatting without adding HIR lowering, CLI inspection, validation, or backend behavior.

## Normalized Spec

- Add `crates/radix/src/mir`.
- Define MIR IDs for functions, blocks, locals, temporaries, and values.
- Wrap semantic `TypeId` in a MIR-owned type reference instead of exposing raw `TypeId` throughout MIR nodes.
- Leave room for later ABI/layout metadata without choosing runtime representation now.
- Define program, function, block, statement, terminator, value, place, intrinsic, runtime-call, and aggregate structures.
- Add deterministic debug/dump rendering suitable for test snapshots.
- Add focused unit tests proving basic construction and stable rendering.
- Avoid lowering and target behavior changes.

## Repo-Aware Baseline

- Semantic types live in `crate::semantic::{TypeId, TypeTable}`.
- HIR definitions and node IDs live in `crate::hir::{DefId, HirId}`.
- Dedicated test files use `#[cfg(test)] #[path = "..._test.rs"] mod tests;`.
- The crate root exports compiler modules in `crates/radix/src/lib.rs`.

## Stage Graph

1. Add MIR module files and crate export.
2. Implement MIR newtypes and node enums.
3. Implement deterministic textual dump rendering.
4. Add focused MIR construction/rendering tests.
5. Run focused tests, then broader radix tests.

## Checkpoints

- `cargo test -p radix mir` passes.
- MIR tests prove deterministic formatting and basic construction.
- Existing codegen, CLI, and driver behavior are untouched except for the new exported module.

## Gate Plan

- No call sites should consume MIR in phase 1.
- No CLI command should expose MIR until phase 2.
- No target backend should depend on MIR until later phases.

## Open Questions

- Exact source-span diagnostics for MIR validation are deferred to the validation phase.
