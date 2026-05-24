# Epic 2 Post-Cleanup Phase 7 Delivery

## Interpreted Problem

The Rust backend test root still carries hand-built type-rendering assertions after the broader backend test split. Those tests are cohesive and can move into a dedicated child test module without changing behavior.

## Normalized Spec

- Add a `types_test` child module under `crates/radix/src/codegen/rust/tests`.
- Move the existing Rust type rendering tests out of `mod_test.rs`.
- Preserve exact test bodies with mechanical movement where possible.
- Fix only module glue and ancestor-private paths needed after the move.
- Validate the focused tests, hygiene test, and workspace Rust checks used by the epic cleanup.

## Repo-Aware Baseline

- `crates/radix/src/codegen/rust/mod_test.rs` still owns shared helpers and imports used by child backend test modules.
- Existing child test modules use `#[path = "tests/*_test.rs"]` and can import shared fixtures through `use super::*;`.
- Hygiene checks expect split backend files to end in `_test.rs`.

## Stage Graph

1. Add the child module declaration and destination file.
2. Move the two type-rendering tests mechanically.
3. Patch path qualifiers from the root test module to the moved child module context.
4. Run focused tests, hygiene, full radix tests, lint, and diff checks.
5. Update the audit progress ledger and commit the phase.

## Checkpoints

- The moved tests keep their names and assertions.
- No generated Rust behavior changes.
- `mod_test.rs` shrinks and remains the shared root for common fixtures.

## Gate Plan

- `cargo test -p radix type_to_rust_covers_composite_and_special_cases -- --nocapture`
- `cargo test -p radix valor_type_renders_to_norma_datum_valor_and_supports_si_valor -- --nocapture`
- `cargo test -p radix --test hygiene`
- `cargo test -p radix`
- `./scripta/lint`
- `git diff --check`
