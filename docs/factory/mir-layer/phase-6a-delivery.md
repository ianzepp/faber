# Phase 6A Delivery: Aggregate, Option, And Runtime MIR Contract

**Status**: complete.

## Interpreted Problem

Phase 6 and Phase 7 both need the same MIR surface. Aggregate and option lowering need named construction, projection, optional-flow, and collection shapes. Runtime intrinsic lowering needs target-neutral operation identity for printing, string formatting, conversions, collection methods, and provider-backed stdlib calls.

If those phases implement independently, they will invent overlapping node shapes and make later validation/backend work harder. Phase 6A is the shared contract phase: it should settle the MIR node vocabulary before either aggregate/option lowering or runtime/provider lowering expands behavior.

## Normalized Spec

- Define the MIR contract for aggregate construction payloads.
- Define named struct fields and keyed map entries without relying on positional operands alone.
- Define projection payloads for fields, variant fields, and indexes using operand-friendly indexes.
- Define explicit option/null operations or control-flow forms needed by optional chain, non-null assertion, and `vel`.
- Define runtime intrinsic payloads for diagnostics, string formatting, conversions, collection operations, and provider calls.
- Keep the contract target-neutral: no Rust paths, Cargo metadata, WASM imports, native symbols, or target-specific stdlib snippets.
- Preserve deterministic `radix mir` rendering for every new MIR shape.
- Do not lower broad new HIR constructs in this phase except tiny construction-only fixtures needed to test rendering.

## Contract Areas

### Aggregates

Current `MirStmtKind::Construct { aggregate, fields: Vec<MirOperand> }` is not enough for Phase 6B because it loses names and keys.

Phase 6A should define one of these equivalent target-neutral shapes:

- distinct aggregate payload variants for tuple/list/set/struct/map/variant,
- or a single `MirAggregateFields` enum with named and keyed cases.

Required data:

- tuple/list/set elements in stable order, including explicit spread items where later lowering needs them,
- struct fields as `Symbol -> MirOperand` or ordered named fields,
- map entries as key/value operand pairs,
- enum variants as variant `DefId` plus named or positional payloads.

### Projections

Current `MirProjection::Index(MirValueId)` should be tightened before broader lowering. Phase 6B lowering usually has an index `MirOperand`, not a stable value ID.

Phase 6A should either:

- allow `MirProjection::Index(MirOperand)`, if projections may carry operands,
- or define index access as an explicit MIR value/runtime operation instead of a place projection.

The chosen rule must say when an index is addressable as a place and when it is a value-only read.

### Options

Phase 6A should decide the MIR vocabulary for:

- nil literal versus typed none,
- wrapping `T` into `T ∪ nihil`,
- testing nil/non-nil,
- unwrapping or asserting non-null,
- `vel` / coalesce,
- optional member/index/call chains.

This should not bake in Rust `Option<T>`. It should expose Faber's semantics as MIR.

### Runtime Intrinsics

Current `MirIntrinsic::{Print, FormatString, CollectionPush, Convert, Provider(Symbol)}` is too vague for Phase 7.

Phase 6A should define enough structured payload to distinguish:

- diagnostic verb/severity: `nota`, `vide`, `mone`, `scribe`,
- string template symbol plus arguments,
- conversion flavor: construction/cast versus runtime parse/conversion, with fallback where relevant,
- collection operation identity: append, immutable append, index, length, contains, etc.,
- provider identity separate from target linkage.

Provider calls should preserve Faber/stdlib identity, not target translation strings from `@ verte`.

## Repo-Aware Baseline

- `crates/radix/src/mir/nodes.rs` already has placeholder aggregate, projection, runtime-call, and intrinsic types.
- `crates/radix/src/mir/dump.rs` already renders aggregate and runtime-call placeholders deterministically.
- `crates/radix/src/mir/lower.rs` currently rejects field/index access, aggregate literals, optional chains, non-null assertions, string templates, diagnostics, conversions, and provider-backed calls with explicit unsupported diagnostics.
- HIR already has `Struct`, `Tuple`, `Array`, `Field`, `Index`, `OptionalChain`, `NonNull`, `Scribe`, `Scriptum`, `Verte`, `Conversio`, and `MethodCall`.
- The semantic `TypeTable` already represents `Array`, `Map`, `Set`, `Option`, `Struct`, `Enum`, `Interface`, `Record`, `Union`, and `Func`.
- Stdlib metadata lives in `stdlib/norma` and includes `@ verte` target translations, but MIR should not consume target translation strings as semantics.

## Stage Graph

1. Inventory current MIR placeholder shapes and decide which remain stable.
2. Replace positional-only aggregate construction with explicit aggregate payload data.
3. Tighten projection/index representation and document addressable versus value-only access.
4. Add option operation or option control-flow node shapes.
5. Replace vague runtime intrinsic variants with structured intrinsic/provider payloads.
6. Update deterministic MIR dump rendering for all new shapes.
7. Add MIR construction/dump tests for aggregate payloads, projections, option operations, runtime intrinsics, and provider identity.
8. Keep broad HIR lowering unchanged and fail-closed.
9. Update Phase 6B and Phase 7 docs if the final contract differs from their assumptions.

## Checkpoints

- MIR can represent named struct construction without losing field names.
- MIR can represent map construction without losing keys.
- MIR can represent index reads in a way Phase 6B can lower from HIR without fabricating unstable value IDs.
- MIR can represent optional-chain/nullability operations without Rust `Option<T>` syntax.
- MIR can represent diagnostic, formatting, conversion, collection, and provider calls with structured target-neutral identity.
- `radix mir` dump output is deterministic for each new shape.
- Existing Phase 3-5C MIR lowering tests still pass.
- No backend consumes the new MIR shapes.

## Out Of Scope

- Broad lowering of struct/enum/list/map/option HIR expressions.
- Runtime/provider call lowering from real stdlib metadata.
- MIR validation.
- Rust backend support.
- WASM or native output.

## Validation

- Focused MIR node/dump tests for the new contract.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 6A complete.

## Completion Gate

Phase 6A is complete when MIR has a stable target-neutral contract for aggregate payloads, projections, option/null operations, and runtime/provider operation identity, and Phase 6B and Phase 7 can implement lowering against that shared contract without inventing additional node vocabulary.
