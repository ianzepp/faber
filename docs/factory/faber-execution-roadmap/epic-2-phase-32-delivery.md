# Epic 2 Phase 32 Delivery: Rust Dynamic Object Values

## Interpreted Problem

The remaining Rust e2e failures include a shared dynamic object cluster. Object literals with heterogeneous values lower to `HashMap<String, Box<dyn std::any::Any>>`, but Rust codegen inserts raw primitives and strings, emits map dot access as Rust field access, and then cannot print or clone dynamic values.

## Normalized Spec

- Replace the Rust backend's generated representation for `ignotum` and anonymous union value positions with a local dynamic value type in emitted Rust.
- Convert values inserted into dynamic map literals through that generated dynamic value type.
- Lower string-key map dot access through keyed lookup instead of Rust struct-field syntax.
- Keep the phase local to Rust codegen and focused tests.
- Do not solve `ad/ad.fab`; that remains Epic 3.

## Repo-Aware Baseline

- Dynamic types currently render in `crates/radix/src/codegen/rust/types.rs`.
- `verte` map construction lives in `crates/radix/src/codegen/rust/expr/verte.rs`.
- Field/index access lives in `crates/radix/src/codegen/rust/expr/access.rs`.
- The e2e harness compiles generated Rust directly with `rustc --edition=2021`, so the dynamic helper must be emitted into generated source rather than depend on the external `norma` crate.

## Stage Graph

1. Add a generated `FaberValue` helper to the Rust prelude only when generated code uses it.
2. Render `ignotum` and anonymous unions as `FaberValue`.
3. Wrap dynamic map inserted values as `FaberValue::from(...)`.
4. Emit map field access as `.get(...).cloned().unwrap_or_default()`.
5. Add focused Rust codegen tests, then run focused and full validation.

## Checkpoints

- Focused tests prove dynamic maps use `FaberValue`, insert converted values, and dot-access maps by key.
- Direct exempla checks show at least one previously blocked dynamic object exemplar advances.
- Full ignored Rust e2e count is recorded after the phase.

## Gate Plan

- `cargo test -p radix rust_dynamic_object_values`
- `cargo test -p radix`
- `cargo test -p radix exempla_rust_e2e -- --ignored`
- Poker-face audit before commit.

## Open Questions

- Object spread type inference may still need a later semantic/codegen phase if a dynamic base is spread into a statically inferred map.
