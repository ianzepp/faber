# Phase 020: Genus Method MIR Lowering

## Interpreted Problem

The live Wasm exemplar harness has a large frontend-analysis stop cluster at
`unsupported MIR lowering: method call before runtime/provider MIR lowering`.
Some of those calls are ordinary genus methods, not host/provider or collection
operations. They should lower through target-neutral MIR as ordinary direct
calls with an explicit receiver value.

## Normalized Spec

- Lower bodyful genus methods into MIR functions.
- Bind `ego` in method bodies to an explicit receiver parameter.
- Lower `receiver.method(args...)` to a direct `MirCallee::Definition` call
  when the receiver type names a struct method in the current unit.
- Preserve existing provider and collection method lowering behavior.
- Do not add Wasm-specific method nodes or ABI policy to MIR.
- Keep runtime/host method surfaces such as providers and collection methods
  separate from genus methods.
- Handle the Wasm compile-valid shapes newly exposed by lowering method bodies:
  duplicate same-spelled method names, aggregate projection assignments, and
  mixed numerus/fractus arithmetic.

## Repo-Aware Baseline

Current harness counts after Phase 019:

```text
frontend analyzed: 101/101
MIR lowered: 64/101
Wasm emitted: 63/101
compile-valid: 63/101
```

Representative genus method failures include:

- `examples/exempla/abstractus/abstractus.fab`
- `examples/exempla/ego/ego.fab`
- `examples/exempla/genus/creo.fab`
- `examples/exempla/genus/methodi.fab`

## Stage Graph

1. Extend MIR lowering context with a struct-method lookup map.
2. Register method signatures in MIR validation with receiver as parameter zero.
3. Lower methods from struct declarations as MIR functions with an explicit
   receiver binding for `ego`.
4. Resolve genus method calls before collection/runtime fallback and lower them
   to direct definition calls.
5. Add focused MIR tests for receiver binding and method-call lowering.
6. Run the Wasm e2e harness and update the baseline ledger.

## Checkpoints

- `ego.field` lowers as a field projection from the receiver parameter.
- Method calls become ordinary MIR calls; Wasm sees no method-specific node.
- Collection/provider methods still go through `MirIntrinsic`.
- Exemplar tier changes are recorded honestly.

## Gate Plan

- `cargo test -p radix genus_method -- --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Result

Genus methods now lower through target-neutral MIR as ordinary functions with
the receiver as parameter zero. `ego` resolves to that receiver local, and calls
to genus methods are direct definition calls before runtime/provider fallback.

Wasm emission also learned the narrow compile-valid adjuncts exposed by those
method bodies:

- duplicate function-name disambiguation for same-spelled methods;
- host-imported aggregate projection setters for field, variant-field, and
  index assignment places;
- opaque `ignotum` handle carriers and handle equality;
- explicit `i64` to `f64` operand coercion for mixed numeric binary ops.

The phase raised compile-valid Wasm exemplar coverage from 63/101 to 71/101.
Instantiate, run, and behavior tiers remain unclaimed because no local
`wasmtime` host is available.

## Open Questions

No user decision needed for this phase. Receiver mutability remains represented
as the existing aggregate-handle model at MIR/Wasm compile-valid tier; full
runtime object mutation semantics remain a later host/runtime concern.
