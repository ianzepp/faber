# Epic 2 Post-Cleanup Phase 5 Delivery: Rust Control Emitter Decomposition

## Interpreted Problem

The Epic 2 audit still identifies `crates/radix/src/codegen/rust/expr/control.rs`
as a production module mixing failable control, branch expressions, match
expressions, loops, iteration, ranges, closures, and await lowering. This phase
should reduce that module by construct family while keeping generated Rust
behavior stable.

## Normalized Spec

- Convert `expr/control.rs` into a directory-backed module.
- Preserve public entrypoints used by `expr/mod.rs`.
- Move branch and option-shaped `si` helpers into a `branch` submodule if the
  boundary is clean.
- Move `elige`/match helpers into a `match_expr` submodule.
- Move loop, while, for, and range helpers into an `iteration` submodule.
- Keep failable `tempta`, closures, and await in `mod.rs` unless they are
  mechanically separable without broad visibility churn.
- Do not change generated Rust behavior.

## Repo-Aware Baseline

- `expr/control.rs` is about 670 lines.
- Existing focused Rust backend tests cover iteration, collection methods,
  option-shaped branches, failable lowering, and broad expression codegen.
- The file's helper signatures already share the same broad expression-emission
  context used by the rest of the backend.

## Stage Graph

1. Convert `expr/control.rs` into `expr/control/mod.rs` and wire submodules.
2. Extract branch/option-shaped `si` emission into `expr/control/branch.rs`.
3. Extract match/`elige` emission into `expr/control/match_expr.rs`.
4. Extract loop/iteration/range emission into `expr/control/iteration.rs`.
5. Run focused Rust backend tests, full radix tests, lint, and diff checks.

## Checkpoints

- `expr/control/mod.rs` is materially smaller and owns only orchestration or
  leftovers that are not cleanly separable in this phase.
- Submodule names reflect construct families.
- Existing tests pass with unchanged generated behavior.

## Gate Plan

- `cargo test -p radix codegen::rust::tests::collections -- --nocapture`
- `cargo test -p radix codegen::rust::tests::optional -- --nocapture`
- `cargo test -p radix expr_codegen_handles_control_flow_and_operators -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- `git diff --check`

## Open Questions

- None. If a helper boundary becomes invasive, leave that helper in `mod.rs`
  and document the remaining follow-up.

## Closeout Update

Implemented on 2026-05-24. `expr/control.rs` is now the directory-backed
`expr/control/` module:

- `mod.rs` keeps `tempta`, closure, and await lowering.
- `branch.rs` owns `si` lowering and option-shaped branch helpers.
- `match_expr.rs` owns `elige`/match lowering and wildcard/exhaustiveness
  helpers.
- `iteration.rs` owns bare loops, `dum`, `itera`, borrowed array iteration,
  map-key iteration, and range lowering.

`expr/control/mod.rs` is now roughly 170 lines. The moved submodule entrypoints
use `pub(in crate::codegen::rust::expr)` so `expr/mod.rs` can continue importing
the same control functions while lower-level helpers remain private.

Validation:

- `cargo test -p radix codegen::rust::tests::collections -- --nocapture`
- `cargo test -p radix codegen::rust::tests::optional -- --nocapture`
- `cargo test -p radix expr_codegen_handles_control_flow_and_operators -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
