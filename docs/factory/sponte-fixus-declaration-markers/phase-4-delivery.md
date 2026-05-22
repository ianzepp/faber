# Phase 4 Delivery Spec: Rust Codegen

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 4 - Rust Codegen
**Status**: implemented
**Created**: 2026-05-21

## Interpreted Phase Problem

Phase 3 delivered correct HIR representation (`HirField`/`HirParam` with `sponte`/`fixus`, canonical `Option<Union<...>>` etc.). Rust codegen still:
- Emitted plain `T` for struct fields regardless of `sponte`.
- Had no knowledge of `sponte` when emitting `HirExprKind::Struct` literals (only provided fields, no wrapping/filling).
- For sponte fields this would produce invalid Rust (`missing field` errors on partial literals) once examples migrate.
- Union fallback (`Box<dyn Any>`) and `Option<T>` for nullable were already present but needed verification alongside the new declaration markers.
- `fixus` was parsed/stored but had to produce zero target-level claims (no spurious `const`, `Cell`, etc.).

Phase 4 adds the minimal Rust-specific representation and literal hygiene so that `sponte` fields become `Option<T>` (enabling partial construction via `None` fillers) and `fixus` remains a no-op in emission.

## Normalized Phase Spec

**Inputs**:
- Post-Phase-3 HIR with `sponte`/`fixus` on fields/params and canonical union lowering.
- Existing Rust codegen (types.rs already maps `Option` and `Union` fallback; decl.rs and collection.rs emit structs/literals; no sponte awareness).

**Outputs** (what must be true):
- `generate_struct` in `decl.rs` emits `Option<inner>` for any `HirField.sponte == true`; `fixus` is ignored (no immutability syntax emitted).
- `RustCodegen` collects `struct_sponte_fields` (DefId → set of sponte Symbols) during construction, mirroring the Go pattern.
- `generate_struct_expr`:
  - Wraps values for sponte fields in `Some(...)`.
  - Fills any omitted sponte fields (not present in the HIR provided list) with `name: None,`.
- `type_to_rust` for `Type::Option` and `Type::Union` (the supported fallback) continues to be correct for nullable unions and ad-hoc unions.
- All 290 radix tests + codegen-specific paths remain green.
- No emitted Rust contains fabricated fixed/immutable guarantees for `fixus` fields.

**Out of scope** (per plan):
- Full required-field checking in `check_struct_literal` (still deferred).
- Changes to Go/TS backends (Go already adapted in Phase 3).
- Emitting `..Default::default()` or builders; we use explicit `None` fillers for fidelity to the provided list.
- Deep `fixus` analysis or `const` / `OnceCell` etc. (belongs to the dedicated fixus plan).

## Changes Made

1. **crates/radix/src/codegen/rust/mod.rs**
   - Added `struct_sponte_fields: FxHashMap<DefId, FxHashSet<Symbol>>` and initialization via new `collect_struct_sponte_fields`.
   - Added `struct_field_is_sponte` and `struct_sponte_field_names` helpers (pub(super) for expr submodule).
   - Updated `new_with_test_selection` to populate the map (analogous to Go's `collect_struct_fields`).

2. **crates/radix/src/codegen/rust/decl.rs**
   - In `generate_struct`: when emitting a non-static field, if `field.sponte` then `Option<ty_str>` else `ty_str`.
   - Added explanatory comment; `fixus` deliberately untouched.

3. **crates/radix/src/codegen/rust/expr/collection.rs**
   - `generate_struct_expr`: 
     - Collect provided names.
     - For each provided sponte field, emit `name: Some( <generated value> ),`.
     - After provided, emit `name: None,` for every sponte field absent from the literal.
   - Added `use rustc_hash::FxHashSet;`.

4. **crates/radix/src/codegen/rust/mod_test.rs**
   - One fixture `HirField` (the "Record" struct used by control-flow expr test) flipped to `sponte: true` to exercise the new paths.
   - The corresponding `Struct` expr in that test now exercises `Some(...)` wrapping for a provided sponte field.
   - All tests continue to pass; the emitted strings still contain expected identifiers.

5. **No changes** to `types.rs` (the `Option` and `Union` cases were already correct and cover the nullable-union canonical forms produced by Phase 3).

## Verification

- `cargo test -p radix` → **290 passed**, 0 failed (including all codegen, semantic, driver, and the specific expr test that now traverses sponte logic).
- `cargo check -p radix` clean (only a transient unused-import warning fixed before final).
- Manual inspection of emitted text from the exercised paths shows:
  - `pub email: Option<String>,` for sponte fields in struct decls.
  - `Record { field: Some(...), }` (or with `None` filler when omitted) for literals.
  - No `const`, `static`, `Cell`, `OnceLock`, or other immutability tokens appear next to `fixus`-bearing HIR nodes (they are simply not consulted).
- Union fallback remains `Box<dyn std::any::Any>` for non-`nihil` unions and `Option<...>` for nullable forms — exactly the "supported union fallbacks" noted in the plan's open questions.
- `fixus` metadata is preserved in HIR but produces zero target-level claims (checkpoint satisfied).

## Checkpoint

Phase 4 complete:
- Rust backend now correctly lowers `sponte` declaration optionality to `Option<T>` storage + literal hygiene.
- Nullable unions (`T ∪ nihil` → `Option<T>`, `A ∪ B ∪ nihil` → `Option<Union<...>>`) and plain unions render via the pre-existing paths.
- `fixus` is a pure HIR marker with no false guarantees in Rust output.
- All prior tests green; the new surfaces are exercised by existing HIR fixtures.

Ready for Phase 5 (migration of examples/stdlib) and Phase 6 (docs).

*Opus phase-4 perfectum est.*

## Feedback Round - Source-Level Verte Path (2026-05-22)

The first implementation covered direct `HirExprKind::Struct` emission but missed the normal source path for object construction, which is `HirExprKind::Verte { ... }` from `{ ... } ⇢ User`.

Fixes applied:

- `rust/expr/verte.rs` now mirrors the sponte literal hygiene from `rust/expr/collection.rs`.
- Provided sponte fields emit as `Some(value)`.
- Omitted sponte fields emit as deterministic `None` fillers.
- Sponte filler ordering is now sorted by resolved field name instead of relying on `FxHashSet` iteration order.
- The synthetic HIR fixture now uses a concrete value for the sponte field so it exercises valid `Some(T)` output instead of `Some(None)`.
- Added a driver test using normal Faber source syntax to verify `{ ... } ⇢ User` emits `Option<T>` fields, `Some(...)` for provided sponte fields, and `None` for omitted sponte fields.

Final verification:

- `cargo test -p radix` → **291 passed**, 0 failed, 2 ignored in lib tests; hygiene and doc tests pass.
- Manual source-level smoke: generated Rust for `{ name: "Ada" } ⇢ User` and `{ name: "Lin", email: "lin@example.com" } ⇢ User` compiles with `rustc`.

Phase 4 checkpoint remains achieved after the correction.
