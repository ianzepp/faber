# Phase 0 Delivery Spec: Baseline Ledger

## Scope

Record the current HTTP HAL baseline before changing interface, runtime, or
compiler behavior.

## Required Evidence

- Current `git status --short`.
- `cargo run -p faber -- check stdlib/norma/hal/http.fab`.
- `cargo check -p norma`.
- Current `crates/norma/hal/mod.rs` exports.
- Current Rust Norma runtime bridge entries in
  `crates/radix/src/codegen/rust/expr/call/runtime.rs`.
- Current package behavior for a package importing `norma/hal/http`.

## Output

Write `docs/factory/norma-http-hal/ledger.md` with command results and concise
source observations.

## Gate

The phase is complete when the ledger exists, reflects the current worktree,
and no source behavior changes have been made.
