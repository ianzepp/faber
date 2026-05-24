# Epic 2 Post-Cleanup Phase 3 Delivery: Rust Backend Test Decomposition

## Interpreted Problem

The Epic 2 audit now has its main Rust backend correctness risks closed, but
`crates/radix/src/codegen/rust/mod_test.rs` remains a single broad test module
mixing optional/nullability, dynamic values, collections, runtime/failable
behavior, type rendering, and orchestration tests. This makes future Rust
backend phases harder to review and run narrowly.

## Normalized Spec

- Split focused clusters out of `mod_test.rs` into companion modules under
  `crates/radix/src/codegen/rust/tests/`.
- Preserve existing test names and behavior unless a tiny glue change is needed
  for module visibility.
- Keep shared test construction helpers in the root Rust codegen test module.
- Do not refactor production code as part of this phase unless needed to keep
  tests compiling.
- Keep the phase mechanical enough that `git diff` clearly shows moved tests
  rather than rewritten behavior.

## Repo-Aware Baseline

- Rust backend tests are currently wired by
  `#[cfg(test)] #[path = "mod_test.rs"] mod tests;` in
  `crates/radix/src/codegen/rust/mod.rs`.
- `mod_test.rs` is roughly 2,800 lines and contains about 50 `#[test]`
  functions.
- The existing test module has access to private Rust backend items through
  `use super::*`; child companion modules can inherit those imports from the
  test root with `use super::*`.

## Stage Graph

1. Add a `tests/` companion-module directory and wire it from `mod_test.rs`.
2. Move optional/nullability tests into `tests/optional_test.rs`.
3. Move dynamic value tests into `tests/dynamic_test.rs`.
4. Move collection/iteration tests into `tests/collections_test.rs`.
5. Move runtime/failable/type-rendering tests into smaller focused modules if
   the first splits compile cleanly.
6. Run focused tests after each meaningful batch, then run the full radix test
   and lint gates.

## Checkpoints

- `mod_test.rs` is materially smaller and no longer contains every Epic 2 Rust
  backend behavior cluster.
- Moved tests retain their original names, so targeted test invocations still
  work.
- No production Rust output behavior changes in this phase.

## Gate Plan

- `cargo test -p radix optional -- --nocapture`
- `cargo test -p radix dynamic -- --nocapture`
- `cargo test -p radix collections -- --nocapture` or direct moved-test names
- `cargo test -p radix`
- `./scripta/lint`
- `git diff --check`

## Open Questions

- None. If a cluster has surprising private helper coupling, leave it in
  `mod_test.rs` and continue with the cleanly separable clusters.

## Closeout Update

Implemented on 2026-05-24. The Rust backend tests now have focused companion
modules for optional/nullability, dynamic values, collections/iteration, calls,
declarations, and failable lowering under
`crates/radix/src/codegen/rust/tests/`. The files use `_test.rs` names so the
repo hygiene scanner treats them as companion tests rather than production
source.

`mod_test.rs` is reduced from roughly 2,800 lines to roughly 1,840 lines. The
large hand-built control-flow/codegen fixture and type-rendering tests remain in
the root test module for a later, more careful split.

Validation:

- `cargo test -p radix codegen::rust::tests::collections -- --nocapture`
- `cargo test -p radix codegen::rust::tests::optional -- --nocapture`
- `cargo test -p radix codegen::rust::tests::calls -- --nocapture`
- `cargo test -p radix codegen::rust::tests::decl -- --nocapture`
- `cargo test -p radix codegen::rust::tests::failable -- --nocapture`
- `cargo test -p radix --test hygiene`
- `cargo test -p radix`
- `./scripta/lint`
