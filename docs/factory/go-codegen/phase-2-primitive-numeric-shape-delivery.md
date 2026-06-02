# Phase 2 Delivery: Primitive Numeric Shape

## Interpreted Problem

The live Go e2e gate records `examples/exempla/genus/creo.fab` as an expected
failure because generated Go emits a float literal into an integer-shaped
expression path:

```text
./main.go:35:16: 3.14159 (untyped float constant) truncated to int
```

The source field is declared as `fractus area`, so the fix should preserve the
field's `float64` shape and ensure assignments into that field use the expected
destination type. This is a type-context issue, not a reason to coerce blindly
at the final Go text layer.

## Normalized Spec

Implement a focused Go backend slice that:

- Emits assignments into known typed destinations through the same
  `generate_expr_for_go_type` path used by returns and declarations.
- Uses struct field metadata to recover the expected destination type for
  `ego.field` and other field lvalues when the HIR proves the field type.
- Preserves existing Rust backend behavior and does not reshape exempla.
- Removes `genus/creo.fab` from Go expected-failure metadata only if the strict
  e2e gate proves it now passes.

Out of scope:

- General cursor/generator lowering.
- `inter` lowering.
- Expression-valued block `any` conversions.
- Ad capability support for Go.

## Repo-Aware Baseline

Current gated Go e2e baseline before this phase:

```text
Go e2e exempla: 93/101 exempla files pass end-to-end
Expected-output checks enabled for 1 exempla files
```

Relevant surfaces:

- `crates/radix/src/codegen/go/mod.rs`: struct field metadata catalogs.
- `crates/radix/src/codegen/go/stmt.rs`: assignment lowering.
- `crates/radix/src/codegen/go/expr/mod.rs`: context-aware expression emission.
- `crates/radix/src/codegen/go/mod_test.rs`: focused Go backend assertions.
- `crates/radix/src/exempla_e2e_test.rs`: strict Go expected-failure metadata.

## Stage Graph

1. Reproduce or inspect generated Go for `genus/creo.fab`.
2. Trace assignment lowering for field destinations.
3. Add destination-type lookup for field lvalues where metadata is available.
4. Emit RHS expressions with `generate_expr_for_go_type` when an assignment
   destination type is known.
5. Add focused Go codegen coverage for `fractus` field assignment in `creo`.
6. Run focused tests and strict Go e2e.
7. Update expected-failure metadata and the baseline ledger if the exemplar
   starts passing.

## Checkpoints

Focused:

```bash
cargo test -p radix codegen::go -- --nocapture
```

Checkpoint:

```bash
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
cargo test -p radix
./scripta/lint
```

## Gate Plan

The phase passes when:

- `genus/creo.fab` passes Go e2e and is removed from expected-failure metadata,
  or the artifact records why the failure is outside this phase.
- No unexpected Go e2e failures are introduced.
- Focused Go codegen tests cover the field-assignment numeric context.
- `cargo test -p radix` and lint pass.

## Open Questions

None.

## Completion Evidence

Implemented in this phase:

- Assignment lowering now derives a destination type for path and field lvalues
  when HIR/type metadata proves one.
- RHS assignment emission now uses context-aware Go expression generation when
  a destination type is available.
- Expected-`fractus` arithmetic now promotes `numerus` operands to `float64`
  for `+`, `-`, `*`, and `/`.
- `genus/creo.fab` was removed from Go expected-failure metadata after the
  strict e2e gate proved it passes.

Verification:

```bash
cargo test -p radix codegen::go -- --nocapture
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
cargo test -p radix
./scripta/lint
```

Final gated Go e2e result:

```text
Go e2e exempla: 94/101 exempla files pass end-to-end
Expected-output checks enabled for 1 exempla files
```

The remaining seven Go e2e failures are still recorded as expected failures in
the harness. `go vet` findings remain separate `[vet]` records and are not yet
a hard gate.
