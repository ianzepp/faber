# Phase 018 Delivery Spec: Option Coalesce And Bitwise Wasm Emission

## Interpreted Problem

The current Wasm harness measures 61/101 compile-valid exemplars. After Phase
017, `examples/exempla/binarius/binarius.fab` reaches validated MIR and stops at
Wasm emission with `MIR-to-WASM unsupported: option value`. After coalesce
support, the same exemplar exposes integer bitwise Wasm emission as the next
blocker. Both are ordinary Wasm-emission gaps in the same `binarius`
compile-valid path.

## Normalized Spec

- Emit compile-valid Wasm for `MirOptionOp::Coalesce` when the option carrier
  and fallback lower to the same raw Wasm carrier class.
- Emit integer bitwise Wasm ops for MIR `BitAnd`, `BitOr`, `BitXor`, `Shl`, and
  `Shr` over `numerus`/`i64` values.
- Preserve `nil` as the existing zero handle representation.
- Keep runtime semantics honest: this is a compile-valid handle-level MVP, not a
  complete nullable runtime ABI.
- Keep other option operations explicitly unsupported unless this phase adds
  tests and implementation for them.
- Do not add Wasm-specific policy to MIR.

## Repo-Aware Baseline

- Current counts: 63/101 MIR lowered, 61/101 Wasm emitted, 61/101 compile-valid.
- Current Wasm-emission blockers:
  - `binarius/binarius.fab`: `option value`
  - `si/est.fab`: `type Primitive(Ignotum)`
- Existing Wasm value model maps `Type::Option(_)`, arrays, maps, structs,
  records, sets, and enums to `AggregateHandle` (`i32`).
- Existing `MirConstant::Nil` emits `(i32.const 0)`.

## Stage Graph

1. Add narrow `MirOptionOp::Coalesce` emission in `wasm_text.rs`.
2. Add integer bitwise op mappings in `wasm_text.rs`.
3. Add focused Wasm tests that validate emitted coalesce and bitwise WAT.
4. Run focused MIR/Wasm tests and the ignored exemplar harness.
5. Update the phase result artifact and baseline ledger with measured tier
   counts.
6. Run full validation and commit the phase.

## Epic Candidates And Scopable Issues

- This phase is one scoped issue: close the remaining Wasm-emission gaps in the
  `binarius` path without adding a runtime ABI.
- Full nullable ABI, `Some` construction, unwrap, and optional chains are not in
  scope.

## Checkpoints

- Focused Wasm test validates the generated WAT with `wasm-tools`.
- Harness proves whether `binarius.fab` moves to compile-valid.
- Full `cargo test -p radix` and `./scripta/lint` pass before commit.

## Companion Skill Plan

- Factory supervises this phase directly.
- No subagent is needed; the implementation surface is narrow.

## Gate Plan

- Commit only if there are no e2e regressions and full validation passes.
- Keep instantiate/run tiers at zero unless a real host/runtime tool appears.

## Open Questions

- None for this phase.
