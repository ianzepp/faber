# Phase 1 Delivery: Optional Parameters And Nullable Returns

## Interpreted Problem

The truthful Go e2e gate records two failures in the optional/nullable cluster:

- `examples/exempla/functio/optionalis.fab`
- `examples/exempla/si/ergo-redde.fab`

Both compile through Radix but fail in `go run`. The generated Go does not
represent `sponte` parameters without defaults as pointer optionals, does not
fill omitted optional/default arguments at direct call sites, and emits explicit
`redde` statements without the surrounding nullable return type context.

## Normalized Spec

Implement a focused HIR-backed Go codegen slice that:

- Emits `sponte` parameters without defaults as `*T`.
- Fills omitted direct-call arguments from HIR defaults, or `nil` for
  no-default optional parameters.
- Wraps supplied arguments for no-default optional parameters as `*T`, while
  preserving already-optional values.
- Threads the current function/method return type through statement emission so
  explicit `redde` in nested guard blocks wraps non-`nihil` values for
  `T ∪ nihil` returns.
- Keeps the Rust backend and exemplar corpus unchanged.

Out of scope:

- Cursor/generator lowering.
- `inter` lowering.
- Expression-valued block `any` conversion fixes outside this nullable return
  path.
- Harness metadata changes except removing expected-failure entries if the
  fixed exempla pass.

## Repo-Aware Baseline

Current gated Go e2e baseline:

```text
Go e2e exempla: 91/101 exempla files pass end-to-end
Expected-output checks enabled for 1 exempla files
```

Relevant Go backend surfaces:

- `crates/radix/src/codegen/go/mod.rs`: shared backend metadata catalogs.
- `crates/radix/src/codegen/go/decl.rs`: function/method parameter and body
  emission.
- `crates/radix/src/codegen/go/expr/call.rs`: direct call argument emission.
- `crates/radix/src/codegen/go/expr/option.rs`: pointer optional wrapper.
- `crates/radix/src/codegen/go/stmt.rs`: explicit `redde` statement emission.

## Stage Graph

1. Add Go-side direct function parameter metadata derived from HIR declarations.
2. Emit no-default optional parameters as pointer optionals.
3. Use metadata to recover omitted optional/default direct-call arguments.
4. Thread return type context through statement block emission and wrap
   explicit nullable returns through `generate_expr_for_go_type`.
5. Add focused Go codegen tests.
6. Run focused tests, full radix tests, and the strict Go e2e gate.
7. Remove expected-failure metadata for exempla that now pass and update the
   ledger pass count.

## Checkpoints

Focused:

```bash
cargo test -p radix codegen::go -- --nocapture
```

Broad:

```bash
cargo test -p radix
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
```

The Go e2e gate must either improve by exactly the fixed exempla and remove
their metadata, or fail if metadata and reality diverge.

## Gate Plan

The phase passes when:

- `functio/optionalis.fab` passes Go e2e or any remaining failure is outside the
  phase and documented.
- `si/ergo-redde.fab` passes Go e2e or any remaining failure is outside the
  phase and documented.
- No unexpected Go e2e failures are introduced.
- `cargo test -p radix` passes.

## Open Questions

None.

## Completion Evidence

Implemented in this phase:

- No-default `sponte` parameters now emit as pointer-shaped Go parameters.
- Direct calls now use HIR function parameter metadata to fill omitted defaults
  or `nil`, and to wrap supplied values for optional pointer parameters.
- Go statement lowering now carries current function/method return type context
  so nullable explicit returns wrap non-`nihil` values.
- Go e2e harness output now reports `gofmt`, `go run`, and non-gating `go vet`
  findings as distinct categories.
- Expected-failure metadata was removed for the two exempla fixed by this
  phase.

Verification:

```bash
cargo test -p radix codegen::go -- --nocapture
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
cargo test -p radix
```

Final gated Go e2e result:

```text
Go e2e exempla: 93/101 exempla files pass end-to-end
Expected-output checks enabled for 1 exempla files
```

The remaining eight Go e2e failures are still recorded as expected failures in
the harness. `go vet` findings are printed as `[vet]` records but are not yet a
hard gate.
