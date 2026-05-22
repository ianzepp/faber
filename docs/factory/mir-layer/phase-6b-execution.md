# Phase 6B Execution Spec

## Target

Implement `docs/factory/mir-layer/phase-6b-delivery.md`: lower supported aggregate and option/null HIR constructs into the Phase 6A MIR contract.

## Repo Baseline

- Phase 6A defines ordered, named, and keyed aggregate payloads, operand-backed index projections, option operations, and structured runtime identity.
- `mir/lower.rs` still rejects field/index access, aggregate literals, optional chains, and non-null assertions.
- Object literals are represented in HIR as `Verte { entries: Some(...) }` for struct/map construction.
- Variant construction appears as calls to variant `DefId`s.
- Tuple syntax is not currently a source surface, but `HirExprKind::Tuple` exists and can be lowered when present.

## Implementation Shape

- Refine ordered aggregate payloads to preserve array/set spread elements.
- Lower arrays/lists and tuple HIR nodes to ordered aggregate construction.
- Lower object-to-struct and object-to-map `Verte` entries to named/keyed aggregate construction, including HIR-backed struct field defaults where present.
- Lower set construction from array-like `Verte` sources to ordered set aggregate construction.
- Lower enum variant calls to enum-variant aggregate construction using HIR enum metadata.
- Lower field and index reads as MIR place projections.
- Extend assignment lowering for local field/index places.
- Lower optional chain to `MirOptionOp::Chain`.
- Lower non-null member/index access through `MirOptionOp::Unwrap { mode: Assert }` followed by a projection.
- Lower `vel`/coalesce to `MirOptionOp::Coalesce`.
- Keep runtime-backed collection methods, diagnostics, string formatting, conversions, and provider calls deferred to Phase 7.

## Validation Gates

- Focused MIR tests for struct construction, field read, and field defaults.
- Focused MIR tests for list construction, list spread, and index read.
- Focused MIR tests for map and set construction.
- Focused MIR tests for field/index assignment.
- Focused MIR tests for optional chain, non-null assertion, and coalesce.
- Focused MIR tests for enum variant construction where the supported HIR shape is available.
- Negative MIR tests for unsupported aggregate shapes.
- `cargo test -p radix mir`
- `cargo test -p radix`
- `./scripta/ci`
