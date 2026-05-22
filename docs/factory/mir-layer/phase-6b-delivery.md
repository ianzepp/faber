# Phase 6B Delivery: Aggregate And Option Lowering

## Interpreted Problem

After Phase 6A defines the shared MIR contract, Phase 6B should lower Faber aggregate and option/null semantics into that contract. This phase should make MIR represent user-defined data, collection literals, projections, optional chain, non-null assertion, and `vel` without falling back to Rust syntax or target-specific runtime behavior.

Phase 6B is deliberately downstream of Phase 6A. If Phase 6B needs a new MIR node shape, the work should first update the Phase 6A contract rather than smuggling semantics through ad hoc runtime calls.

## Normalized Spec

- Lower tuple, list, map, set, struct, and enum-variant construction into Phase 6A aggregate payloads.
- Lower field access and assignable field places where the HIR and semantic types make them addressable.
- Lower index reads and assignable index places only according to the Phase 6A projection/addressability contract.
- Lower optional chain into explicit option/null control flow or option operations.
- Lower non-null assertion into explicit checked/unchecked option operation as defined by Phase 6A.
- Lower `vel` / coalesce according to Faber nullable semantics, not target-language truthiness.
- Keep collection methods, diagnostics, formatting, conversions, and provider-backed stdlib calls deferred to Phase 7 unless Phase 6B needs a Phase 6A-defined primitive operation.
- Keep unsupported aggregate shapes fail-closed with clear diagnostics.

## Repo-Aware Baseline

- HIR aggregate and option nodes are already typed by semantic analysis.
- MIR already lowers primitives, locals, calls, returns, control flow, alternate exits, and structured local handlers.
- Current MIR lowering explicitly rejects field/index access, aggregate literals, optional chains, and non-null assertions.
- Existing Rust/Go/TypeScript codegen contains target-specific aggregate and option lowering logic; Phase 6B should use it as evidence, not as MIR semantics.
- Phase 6A should have updated `MirStmtKind`, `MirValueKind`, `MirProjection`, `MirAggregate`, `MirIntrinsic`, and MIR dump rendering as needed.

## Stage Graph

1. Lower tuple literals into MIR aggregate construction.
2. Lower list/array literals, including empty typed literals and spread behavior only where type information is explicit.
3. Lower map and set literals through keyed aggregate payloads or Phase 6A runtime operations.
4. Lower struct construction with named fields and defaults only where semantic analysis already provides enough information.
5. Lower enum variant construction for variants whose payload shape is represented by Phase 6A.
6. Lower field reads to MIR places/values.
7. Lower index reads according to the Phase 6A addressability rule.
8. Extend assignment lowering for supported field/index places.
9. Lower optional chains to explicit MIR option operations/control flow.
10. Lower non-null assertions.
11. Lower `vel` / coalesce for nullable values.
12. Add negative diagnostics for unsupported aggregate or option shapes.

## Checkpoints

- `radix mir` can dump a simple struct construction and field read.
- `radix mir` can dump tuple and list construction.
- `radix mir` can dump map/set construction or reject unsupported keyed shapes explicitly.
- `radix mir` can dump optional chain without leaving `HirExprKind::OptionalChain` semantics implicit.
- `radix mir` can dump non-null assertion and nullable coalesce.
- Field/index assignment is either supported for clear addressable cases or rejected with precise diagnostics.
- Existing Phase 3-5C MIR tests continue to pass.
- No target backend consumes MIR.

## Fixture Candidates

Struct construction and field read:

```fab
genus Persona {
    textus nomen
    numerus aetas
}

functio nomen() → textus {
    fixum Persona p ← {
        nomen: "Ada",
        aetas: 36
    } ⇢ Persona
    redde p.nomen
}
```

List construction:

```fab
functio summa() → numerus {
    fixum lista<numerus> xs ← [1, 2, 3]
    redde xs[0]
}
```

Nullable chain:

```fab
functio nomen(Persona ∪ nihil p) → textus {
    redde p?.nomen vel "ignotus"
}
```

Non-null assertion:

```fab
functio certum(Persona ∪ nihil p) → textus {
    redde p!.nomen
}
```

## Out Of Scope

- Runtime-backed collection methods such as `adde`, `longitudo`, and provider calls.
- String formatting and diagnostic output.
- Runtime conversion lowering.
- Pattern-match lowering beyond variant construction inputs.
- MIR validation beyond construction-time expectations.
- Rust backend support.
- WASM or native output.

## Validation

- Focused MIR tests for aggregate construction and projection.
- Focused MIR tests for nullable chain/non-null/coalesce lowering.
- Negative MIR tests for unsupported aggregate shapes.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 6B complete.

## Completion Gate

Phase 6B is complete when supported aggregate and option/null HIR constructs lower into the Phase 6A MIR contract, high-level aggregate/optional-chain nodes no longer leak into MIR for the supported subset, unsupported shapes fail clearly, and existing MIR/backend behavior remains unchanged.
