# Phase 012: Provider, Async, Closures, And Deferred Surfaces Delivery

## Interpreted Problem

The LLVM probe now covers a useful scalar, runtime-helper, handle, nullable,
switch, and entry-symbol subset. The remaining provider, async/cursor, closure,
callable-value, and native-execution surfaces cross runtime and calling
convention boundaries that should not be guessed inside the text emitter.

## Normalized Spec

- Name the deferred surfaces that should stay fail-closed.
- Preserve current explicit diagnostics for provider runtime calls and callable
  value callees.
- Keep async/cursor, closures, higher-order collection methods, provider blocks,
  and native executable startup outside this LLVM continuation.
- Split future implementation into separate delivery plans rather than adding a
  broad target-specific runtime model here.
- Preserve MIR and Wasm behavior.
- Update the LLVM baseline ledger with measured counts and the next planning
  handoff.

## Repo-Aware Baseline

- LLVM provider runtime calls fail with
  `MIR-to-LLVM unsupported: provider runtime call`.
- LLVM callable-value calls fail with `MIR-to-LLVM unsupported: value callee`.
- Async/cursor constructs, provider blocks, closures, and collection
  higher-order methods fail before LLVM emission in MIR lowering.
- Phase 011 e2e baseline: 59/102 LLVM emitted, 0/102 verifier-valid, 15
  unsupported LLVM diagnostics.

## Deferred Surface Map

- Provider/HAL effects: needs a host ABI, provider identity lowering, capability
  policy, and runtime dispatch/linkage plan.
- Async/cursor lowering: needs suspension representation, yield/await ABI,
  scheduling/runtime policy, and interaction with `cede`.
- Closures/callable values: needs environment capture layout, function pointer
  or trampoline ABI, lifetime/ownership policy, and call dispatch lowering.
- Higher-order collection methods: depends on callable values and collection
  runtime iteration ABI.
- Native execution: needs verifier/toolchain availability, runtime library
  linkage, `main`/startup, process args, and diagnostics/runtime symbols.
- Global initialization: needs top-level constant representation and
  source-order initialization policy.

## Stage Graph

1. Record the deferred-surface map and current diagnostics.
2. Keep implementation unchanged for Phase 012.
3. Re-measure focused LLVM and e2e counts to prove no accidental movement.
4. Update the baseline ledger so the next worker starts from the completed
   continuation, not stale Phase 001-era failure clusters.
5. Commit the classification handoff.

## Future Plan Split

Recommended follow-up plans:

- Provider/HAL runtime ABI.
- Callable values and closures.
- Async/cursor lowering.
- LLVM native execution and runtime linking.
- Global initialization and top-level constants.

## Checkpoints

- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- Completion audit against this spec before commit.

## Wasm Follow-Up

No MIR shape or Wasm import naming changes are expected. Future provider,
async, closure, and runtime plans must compare against existing Wasm behavior
where source semantics overlap.

## Completion Evidence

Phase 012 is a classification and handoff phase. It intentionally makes no code
changes: current fail-closed behavior remains the correct LLVM probe behavior
until separate runtime, callable-value, async, and native-execution plans define
their ABIs.

Current evidence:

- Provider runtime calls remain covered by
  `llvm_text_target_rejects_provider_runtime_calls`.
- Callable-value calls remain covered by `llvm_text_target_rejects_value_callee`.
- Async/cursor, provider blocks, closures, and higher-order collection methods
  remain visible in the e2e harness as MIR-lowering failures rather than LLVM
  crashes.
- E2E counts are unchanged from Phase 011.

Measured e2e baseline after classification:

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
frontend analyzed: 102/102
MIR lowered: 74/102
LLVM emitted: 59/102
verifier-valid: 0/102
unsupported diagnostic: 15
result: passed
```

Final validation:

```text
cargo test -p radix llvm -- --nocapture
result: 29 passed, 0 failed, 1 ignored

cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed

cargo test -p radix
result: 561 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored

./scripta/lint
result: passed
```

## Completion Audit

- Deferred surfaces are named and split into future planning tracks.
- No provider, async, closure, callable-value, native startup, or global
  initialization semantics were guessed inside LLVM text lowering.
- Existing explicit unsupported diagnostics remain the contract.
- MIR and Wasm behavior are unchanged.
