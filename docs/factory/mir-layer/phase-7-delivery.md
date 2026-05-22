# Phase 7 Delivery: Runtime Intrinsic Boundary

## Interpreted Problem

After Phase 6A defines the shared MIR contract, Phase 7 should lower runtime-backed Faber operations into target-neutral MIR intrinsics and provider calls. The goal is to move semantic runtime operation identity out of target code generators without importing Rust, Go, TypeScript, WASM, or native linkage details into MIR.

Phase 7 should not redefine aggregate or option MIR shapes. It consumes the Phase 6A contract and leaves aggregate/option lowering itself to Phase 6B.

## Normalized Spec

- Lower diagnostic output verbs into structured MIR runtime operations.
- Lower string-template application into a target-neutral formatting operation.
- Lower runtime conversions (`⇒`) into structured conversion operations, including fallback behavior where represented by HIR.
- Lower collection methods that are runtime-backed operations into target-neutral collection/provider calls.
- Lower stdlib/provider-backed calls using Faber/stdlib identity rather than target translation strings.
- Keep provider identity separate from target linkage.
- Preserve deterministic `radix mir` output for every lowered runtime operation.
- Keep target backends unchanged; no backend consumes MIR in this phase.

## Runtime Contract

Phase 7 should use the Phase 6A runtime/provider payloads to represent operation identity.

Required operation classes:

- diagnostic output: `nota`, `vide`, `mone`, `scribe`,
- string formatting: template symbol plus evaluated arguments,
- runtime conversion: source, target type, conversion hints, and optional fallback,
- collection operations: append, immutable append, index/read, length, contains, and other small direct methods selected for the first slice,
- provider/stdin/stdlib calls: stable Faber or norma identity plus arguments.

The MIR operation must not contain:

- Rust module paths,
- Cargo dependencies,
- TypeScript/Go/Python method names,
- WASM import module names,
- native symbols or linker names,
- raw `@ verte` translation strings as the semantic operation.

## Repo-Aware Baseline

- `stdlib/norma` contains target translations and method metadata, including `@ verte` entries.
- Existing target codegens already lower diagnostics, string formatting, conversions, and many collection methods directly from HIR.
- `crates/radix/src/mir/nodes.rs` has placeholder `MirIntrinsic` and `MirRuntimeCall` shapes that Phase 6A should refine.
- `crates/radix/src/mir/lower.rs` currently rejects diagnostics, string templates, conversions, method calls, and collection pipelines before runtime MIR lowering.
- Phase 6B owns aggregate and nullable construction/projection semantics, but some collection operations may be represented as Phase 6A runtime operations.

## Stage Graph

1. Confirm Phase 6A contract is present and sufficient for runtime/provider operation identity.
2. Lower `HirExprKind::Scribe` / diagnostic verbs into structured runtime calls.
3. Lower `HirExprKind::Scriptum` into a structured string-format operation.
4. Lower `HirExprKind::Conversio` into a runtime conversion operation.
5. Lower selected collection method calls whose semantics are runtime-backed and type-known.
6. Lower provider-backed stdlib calls using stable Faber/stdlib identity.
7. Preserve unsupported diagnostics for operations that need closure, async, collection pipeline, or provider ABI decisions not covered by this phase.
8. Add deterministic MIR dump tests for each runtime operation class.
9. Keep existing HIR-to-target codegen behavior unchanged.

## Checkpoints

- `radix mir` can dump `nota`, `vide`, and `mone` as distinct target-neutral runtime operations.
- `radix mir` can dump string-template formatting without Rust `format!` or target-specific syntax.
- `radix mir` can dump a runtime conversion without encoding a target parse function.
- `radix mir` can dump selected collection operations without using target method names such as `push`, `len`, or `includes`.
- `radix mir` can dump provider/stdin/stdlib operation identity without target linkage strings.
- Unsupported runtime-backed constructs fail clearly.
- Existing Phase 3-6B MIR tests continue to pass.
- No target backend consumes MIR.

## Fixture Candidates

Diagnostics:

```fab
functio log(textus name) → vacuum {
    nota "salve"
    vide name
    mone "cave"
}
```

String formatting:

```fab
functio greet(textus name) → textus {
    redde "Salve, §!"(name)
}
```

Runtime conversion:

```fab
functio parse(textus raw) → numerus {
    redde raw ⇒ numerus vel 0
}
```

Collection method:

```fab
functio count(lista<numerus> xs) → numerus {
    redde xs.longitudo()
}
```

Provider-backed call:

```fab
importa ex "norma:hal/consolum" privata consolum

functio read() → textus {
    redde consolum.lege()
}
```

## Out Of Scope

- Aggregate/option construction and projection lowering owned by Phase 6B.
- Closure lowering and collection pipelines requiring closures.
- Runtime ABI, native linkage, WASM imports, or Cargo dependency generation.
- MIR validation beyond construction-time checks.
- Rust backend support.
- WASM or native output.

## Validation

- Focused MIR tests for diagnostics, formatting, conversions, collection operations, and provider identity.
- Negative MIR tests for unsupported runtime-backed shapes.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 7 complete.

## Completion Gate

Phase 7 is complete when selected runtime-backed HIR operations lower into the Phase 6A target-neutral runtime/provider MIR contract, MIR dumps preserve stable semantic operation identity without target strings, unsupported runtime-backed shapes fail clearly, and existing target codegen behavior remains unchanged.
