# Epic 2 Phase 28 Delivery: Rust Innatum Collection Literals

Timestamp: 2026-05-24 14:48:00 EDT

## Objective

Make Rust output for the standalone `innatum/innatum.fab` exemplar compile and run by honoring typed native collection literals and list first-element translation.

## Baseline

Latest Epic 2 Rust e2e result: 89/100 exempla pass.

`examples/exempla/innatum/innatum.fab` currently emits invalid Rust for:

- `fixum tabula<textus, numerus> scores ← { alice = 95, bob = 87 }`, because the object literal backend emits `HashMap<String, Box<dyn Any>>` instead of the expected `HashMap<String, i64>`.
- `nums.primus()`, because the Rust backend leaves the Faber collection method name in generated Rust.

## Implementation Plan

1. Use expression type information to emit object-map literals as typed `HashMap<K, V>` when the HIR expression type is a map.
2. Convert identifier/string object keys to Rust map keys using the declared map key type.
3. Emit map values with the declared value type rather than `Box<dyn Any>`.
4. Add Rust stdlib translation for `primus()` on arrays.
5. Add focused Rust codegen coverage and run regular plus ignored Rust e2e validation.

## Non-Goals

- Do not solve the general dynamic `ignotum` object cluster.
- Do not change source grammar or example syntax.
- Do not touch Epic 3 `ad` capability calls.

## Validation

- `cargo run -p faber -- emit -t rust examples/exempla/innatum/innatum.fab > /tmp/faber-phase28-innatum.rs`
- `rustc --edition=2021 /tmp/faber-phase28-innatum.rs -o /tmp/faber-phase28-innatum`
- `/tmp/faber-phase28-innatum`
  - Output: `42`, `2`, `95`, `Some(1)`
- `cargo test -p radix typed_map_literals_assignments_and_primus --lib`
  - Result: 1 passed.
- `cargo test -p radix`
  - Result: 451 passed, 0 failed, 3 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`
  - Result: `Rust e2e exempla: 90/100 exempla files pass end-to-end`.
  - `innatum/innatum.fab` no longer appears in the failing exemplar list.

Remaining Rust e2e failures after this phase:

- `ad/ad.fab`
- `destructura/objectum.fab`
- `genus/creo.fab`
- `incipiet/incipiet.fab`
- `itera/cursor-iteratio.fab`
- `itera/de.fab`
- `membrum/membrum.fab`
- `objectum/objectum.fab`
- `praefixum/praefixum.fab`
- `si/est.fab`
