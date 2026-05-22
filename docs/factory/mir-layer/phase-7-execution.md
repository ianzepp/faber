# Phase 7 Execution Spec

## Target

Implement `docs/factory/mir-layer/phase-7-delivery.md`: lower supported runtime-backed HIR operations into Phase 6A target-neutral MIR runtime/provider calls.

## Repo Baseline

- Phase 6A already defines `MirRuntimeCall`, `MirIntrinsic`, diagnostic, conversion, collection, panic, and provider identity payloads.
- Phase 6B already lowers aggregate and option/null construction and projection.
- `mir/lower.rs` lowers `mori` to a panic runtime call but still rejected diagnostics, `scriptum`, `conversio`, method calls, and provider-backed imports.
- Existing target backends lower those runtime operations directly from HIR and remain out of scope for this phase.

## Implementation Shape

- Lower diagnostic verbs (`nota`, `vide`, `mone`, `scribe`) to `MirIntrinsic::Diagnostic`.
- Lower string-template application to `MirIntrinsic::FormatString` with the template symbol and evaluated arguments.
- Lower runtime conversion (`⇒`) to `MirIntrinsic::Convert` with runtime flavor, target type, hint symbols, source value, and optional fallback.
- Lower selected type-known collection methods to `MirIntrinsic::Collection`: append, immutable append, index/read, length, and contains.
- Lower calls through imported module/provider symbols to `MirIntrinsic::Provider` using Faber import identity and source method/function symbols.
- Keep unsupported method shapes and collection pipelines fail-closed.
- Keep backend consumption, ABI/linkage, WASM/native import names, Cargo dependency selection, and target translation strings out of MIR.

## Validation Gates

- Focused MIR tests for diagnostic runtime calls.
- Focused MIR tests for format-string runtime calls.
- Focused MIR tests for runtime conversion with fallback.
- Focused MIR tests for selected collection runtime calls.
- Focused MIR tests for imported provider identity.
- Negative MIR tests for unsupported runtime method shapes.
- `cargo test -p radix mir`
- `cargo test -p radix`
- `./scripta/ci`
