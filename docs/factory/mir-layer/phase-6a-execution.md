# Phase 6A Execution Spec

## Target

Implement `docs/factory/mir-layer/phase-6a-delivery.md`: define the shared target-neutral MIR contract for aggregates, projections, option/null operations, runtime intrinsics, and provider identity.

## Repo Baseline

- `MirStmtKind::Construct` currently carries `MirAggregate` plus positional `Vec<MirOperand>` fields.
- `MirProjection::Index` currently carries `MirValueId`, which does not match HIR lowering inputs.
- `MirValueKind` has no explicit option/null operation payloads beyond primitive unary/binop placeholders.
- `MirIntrinsic` uses vague variants such as `Print`, `FormatString`, `CollectionPush`, `Convert`, and `Provider(Symbol)`.
- Broad HIR lowering for aggregates, option chains, conversions, diagnostics, and provider calls is still rejected before MIR.

## Implementation Shape

- Move aggregate construction payloads into `MirAggregate`.
- Represent aggregate payloads as:
  - ordered items for tuples, lists, sets, and positional variants, including spread items when needed,
  - named operands for structs and named variants,
  - keyed operands for maps.
- Change index projections to carry `MirOperand` so Phase 6B can lower index reads and assignable index places without fabricating value IDs.
- Add explicit option/null MIR value operations for wrapping, nil checks, unwrap assertions, coalesce, and optional chain links.
- Replace vague runtime intrinsics with structured operation identity for diagnostics, string formatting, conversions, collection operations, panic, and provider calls.
- Preserve the existing `mori` lowering by mapping it to the structured panic runtime intrinsic.
- Keep broad aggregate/option/runtime HIR lowering fail-closed until Phase 6B/7.

## Validation Gates

- Focused node/dump tests for named struct construction and keyed map construction.
- Focused node/dump tests for operand-backed index projections.
- Focused node/dump tests for option operations.
- Focused node/dump tests for diagnostics, formatting, conversions, collections, and provider identity.
- `cargo test -p radix mir`
- `cargo test -p radix`
- `./scripta/ci`
