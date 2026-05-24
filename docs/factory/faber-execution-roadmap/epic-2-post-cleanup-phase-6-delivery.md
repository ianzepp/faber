# Epic 2 Post-Cleanup Phase 6 Delivery: Rust Generated Prelude Extraction

## Interpreted Problem

The Epic 2 audit still identifies `crates/radix/src/codegen/rust/mod.rs` as
owning too many responsibilities, including generated helper prelude emission,
import discovery, backend state collection, and final output assembly. Previous
phases split expression emitters; this phase should extract generated prelude
and helper emission while preserving behavior.

## Normalized Spec

- Move generated prelude/import/helper logic out of `codegen/rust/mod.rs`.
- Keep `RustCodegen` orchestration and HIR state collection in `mod.rs`.
- Preserve the existing generated Rust surface, including:
  - file header and crate-level allows;
  - source/HIR import normalization;
  - scanned imports for `HashMap`, `HashSet`, `Future`, and `FaberValue`;
  - `FaberValue` helper emission;
  - async entry `__faber_block_on` helper emission.
- Do not change generated Rust behavior.

## Repo-Aware Baseline

- `mod.rs` currently contains `generate_prelude`, `normalize_import_path`,
  `collect_prelude_imports`, `generate_faber_value_helper`,
  `generate_block_on_helper`, and `collect_hir_imports`.
- `generate_output` uses these helpers while assembling file and module output.
- Existing Rust backend tests cover dynamic `FaberValue`, async entry block-on,
  usage-driven imports, and module generation.

## Stage Graph

1. Add `crates/radix/src/codegen/rust/prelude.rs`.
2. Move prelude/import/helper functions into the new module.
3. Update `mod.rs` to call the module functions and remove the method-only
   prelude helper from `RustCodegen`.
4. Run focused Rust backend tests around imports, dynamic values, async entry,
   and full output assembly.
5. Run full radix tests, lint, and diff checks.

## Checkpoints

- `mod.rs` no longer owns generated helper prelude bodies.
- `prelude.rs` has a narrow surface for output assembly.
- Existing generated Rust tests pass unchanged.

## Gate Plan

- `cargo test -p radix rust_dynamic -- --nocapture`
- `cargo test -p radix emits_async_futura_functions_and_entry_block_on -- --nocapture`
- `cargo test -p radix emits_usage_driven_and_importa_use_statements -- --nocapture`
- `cargo test -p radix emits_rust_function_and_entry_via_codegen_dispatch -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- `git diff --check`

## Open Questions

- None. If source import collection is more tightly coupled to `RustCodegen`
  than expected, keep only that small function in `mod.rs` and extract the
  generated helper bodies first.

## Closeout Update

Implemented on 2026-05-24. Generated Rust prelude and helper emission now lives
in `crates/radix/src/codegen/rust/prelude.rs`.

The new module owns:

- generated file header and crate-level allow attributes;
- HIR import normalization and source import collection;
- generated-body import scanning for `HashMap`, `HashSet`, `Future`, and
  `FaberValue`;
- `FaberValue` helper emission;
- async entry `__faber_block_on` helper emission and entry-await detection.

`crates/radix/src/codegen/rust/mod.rs` is reduced from roughly 1,020 lines to
roughly 780 lines and now keeps orchestration, backend state, and HIR collection
responsibilities.

Validation:

- `cargo test -p radix rust_dynamic -- --nocapture`
- `cargo test -p radix emits_async_futura_functions_and_entry_block_on -- --nocapture`
- `cargo test -p radix emits_usage_driven_and_importa_use_statements -- --nocapture`
- `cargo test -p radix emits_rust_function_and_entry_via_codegen_dispatch -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
