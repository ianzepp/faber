# Epic 2 Post-Cleanup Phase 4 Delivery: Rust Call Emitter Decomposition

## Interpreted Problem

The Epic 2 audit still identifies `crates/radix/src/codegen/rust/expr/call.rs`
as a large production module that mixes direct calls, argument planning, variant
constructors, method calls, stdlib collection translations, and runtime module
bridges. The previous phase split tests; this phase should reduce production
module size while keeping behavior stable.

## Normalized Spec

- Split `expr/call.rs` into a directory-backed module with focused companion
  files.
- Preserve public entrypoints used by `expr/mod.rs`:
  `generate_call_expr` and `generate_method_call_expr`.
- Move direct-call argument planning and spread handling into an `args`
  submodule.
- Move stdlib collection/text method lowering into a `stdlib` submodule.
- Move runtime `norma` module bridge helpers into a `runtime` submodule if the
  boundary is clean.
- Keep variant constructor lowering near direct-call emission unless extracting
  it is mechanical.
- Do not change generated Rust behavior in this phase.

## Repo-Aware Baseline

- `expr/call.rs` is about 1,000 lines.
- Focused Rust backend companion tests now exist for calls, collections,
  dynamic values, declarations, failable lowering, and optional behavior.
- The strongest focused gates for this phase are:
  `codegen::rust::tests::calls`, `collections`, `dynamic`, and `failable`.

## Stage Graph

1. Convert `expr/call.rs` into `expr/call/mod.rs` and wire submodules.
2. Extract argument-planning helpers into `expr/call/args.rs`.
3. Extract stdlib collection/text method lowering into `expr/call/stdlib.rs`.
4. Extract runtime module bridge helpers into `expr/call/runtime.rs` if the move
   stays mechanical.
5. Run focused Rust backend tests, full radix tests, lint, and diff checks.

## Checkpoints

- `expr/call/mod.rs` is materially smaller and owns orchestration rather than
  all helper implementations.
- Submodule boundaries are named by responsibility, not by arbitrary line
  count.
- Existing focused Rust backend tests pass with unchanged test names.

## Gate Plan

- `cargo test -p radix codegen::rust::tests::calls -- --nocapture`
- `cargo test -p radix codegen::rust::tests::collections -- --nocapture`
- `cargo test -p radix codegen::rust::tests::dynamic -- --nocapture`
- `cargo test -p radix codegen::rust::tests::failable -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- `git diff --check`

## Open Questions

- None. If a helper boundary requires broad visibility churn, leave that helper
  in `mod.rs` and keep the phase focused on the clean splits.

## Closeout Update

Implemented on 2026-05-24. `expr/call.rs` is now the directory-backed
`expr/call/` module:

- `mod.rs` keeps direct call/method orchestration and variant constructor
  lowering.
- `args.rs` owns target-typed argument emission, optional-parameter defaults,
  spread-call expansion, and owned path cloning.
- `stdlib.rs` owns stdlib `lista`, `tabula`, and `textus` method translations.
- `runtime.rs` owns norma runtime module bridge naming.

`expr/call/mod.rs` is now roughly 280 lines. The stdlib translation file remains
larger because all collection/text method translations still share one receiver
type dispatch boundary; splitting `lista` and `tabula` further is optional
follow-up work.

Validation:

- `cargo test -p radix codegen::rust::tests::calls -- --nocapture`
- `cargo test -p radix codegen::rust::tests::collections -- --nocapture`
- `cargo test -p radix codegen::rust::tests::dynamic -- --nocapture`
- `cargo test -p radix codegen::rust::tests::failable -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
