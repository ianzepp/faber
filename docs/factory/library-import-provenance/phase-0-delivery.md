# Phase 0 Delivery: Baseline Ledger

## Objective

Record the current import syntax, resolver behavior, runtime bridge behavior,
and HTTP runtime type identity risks before changing source behavior.

## Scope

- Read-only evidence gathering.
- No compiler behavior changes.
- Persist a baseline ledger for later phase audits.

## Evidence Sources

- `git status --short`
- `EBNF.md`
- `crates/faber/src/library.rs`
- `crates/faber/src/package_test.rs`
- `crates/radix/src/codegen/rust/expr/call/runtime.rs`
- `crates/radix/src/codegen/rust/expr/call/mod.rs`
- `crates/radix/src/codegen/rust/mod.rs`
- `crates/radix/src/codegen/rust/tests/http_test.rs`

## Phase Result

Phase 0 is complete as a baseline-only phase. The detailed evidence is in
`ledger.md`.

## Checkpoint

The ledger captures:

- grammar examples already using provider syntax;
- active package tests still using slash-form built-in Norma imports;
- current resolver behavior selecting `norma` from slash path segments;
- current Rust call bridge keyed by receiver local name;
- current HTTP runtime interface mapping keyed by interface name and method
  lists;
- a focused Rust backend test that currently passes through the unsafe bridge.
