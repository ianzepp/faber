# Phase 5B Delivery: Alternate-Exit MIR Lowering

## Interpreted Problem

After Phase 5A, Faber can express a typed alternate exit path in function signatures. Phase 5B should make MIR represent that path explicitly without lowering it to Rust `Result`, exceptions, or target-specific runtime behavior.

The core invariant is:

```text
redde exits through →
iace exits through ⇥
mori exits fatally and is not recoverable
```

## Normalized Spec

- Add `error_ty: Option<MirType>` to `MirFunction`.
- Preserve the normal `return_ty: MirType` unchanged.
- Render alternate-exit types deterministically in `radix mir`.
- Lower `iace expr` to `MirTerminatorKind::ReturnError`.
- Materialize `iace` operands with type information where needed, matching the Phase 3 `redde` operand discipline.
- Reject `iace` in MIR lowering if the enclosing MIR function has no alternate-exit type.
- Lower `mori expr` as fatal flow: target-neutral panic operation followed by `Unreachable`.
- Keep `tempta`, `cape`, `demum`, and failable-call handling deferred.
- Keep no backend consuming MIR.

## MIR Contract

Representative dump shape:

```text
function f0 -> ty#1 ⇥ ty#0 {
  temps:
    %0: ty#0
  bb0:
    %0 = const string sym#N: ty#0
    return_error %0
}
```

`return_error` is recoverable failure flow. It is not fatal, and it is not Rust syntax.

`return_error` is also not an exception throw. It exits the current MIR function through that function's declared alternate-exit type. If the caller wants to propagate that failure, a later caller-side phase must lower an explicit propagation construct into its own `return_error`.

Fatal failure should use the existing target-neutral runtime-call shape:

```text
runtime_call panic(...)
unreachable
```

The panic runtime call should use `numquam` as its `return_ty` when available, then seal the block with `Unreachable`. If `MirIntrinsic::Panic` is not sufficiently expressive for typed operands, Phase 5B may tighten the runtime-call payload only as narrowly as needed for `mori`.

## Repo-Aware Baseline

- `MirTerminatorKind::ReturnError(MirOperand)` already exists.
- `MirIntrinsic::Panic` already exists.
- `MirStmtKind::RuntimeCall` already exists.
- `dump.rs` already renders `return_error`.
- `mir::lower` currently rejects `HirExprKind::Throw` and `HirExprKind::Panic` as unsupported.
- Phase 5A added `HirFunction::err_ty` and semantic `FuncSig::err`.
- Phase 5A already rejects ordinary source-level failable calls until caller-side handling or propagation syntax exists.
- Phase 5A allows `iace` inside a local `tempta` / `cape` handler by typechecking the handled body with a local unknown error sink.
- No backend consumes MIR.

## Interaction With Phase 5A

Phase 5B should assume only semantically accepted source reaches `radix mir`, but MIR unit tests may still construct malformed HIR defensively.

- A normal source-level `iace` without `⇥ Error` should already fail in Phase 5A semantic analysis.
- MIR lowering should still reject fabricated HIR where `HirExprKind::Throw` appears in a function with no `err_ty`.
- A failable source-level call in ordinary expression position should already fail in Phase 5A semantic analysis.
- Phase 5B should not add a `TryCall` / success-error edge form yet.
- `HirExprKind::Tempta` must remain fail-closed as a whole. Do not descend into its body to lower inner `iace` statements during Phase 5B.

## Stage Graph

1. Add `error_ty: Option<MirType>` to `MirFunction`.
2. Update MIR construction tests and deterministic dump formatting.
3. Thread Phase 5A's HIR function error type into MIR function lowering.
4. Add function-builder access to the current error type.
5. Lower `HirExprKind::Throw` through an `iace` helper.
6. Materialize the thrown operand into a typed temp when necessary.
7. Seal the current block with `ReturnError`.
8. Lower `HirExprKind::Panic` through a `mori` helper.
9. Emit a target-neutral panic runtime call and seal with `Unreachable`.
10. Add defensive unit tests for fabricated HIR with `iace` but no alternate-exit type.
11. Keep `Tempta` fail-closed with an explicit local-handler diagnostic.
12. Keep failable direct calls out of MIR lowering until caller propagation syntax is defined; Phase 5A should reject them before MIR.

## Checkpoints

- `radix mir` shows both normal and alternate-exit types for failable functions.
- `iace` in a failable function emits `return_error`.
- `iace` in a non-failable function fails clearly.
- `mori` emits fatal unreachable flow and does not require `⇥`.
- `tempta` / `cape` / `demum` remain unsupported with explicit diagnostics.
- Source-level failable calls remain rejected before MIR unless and until a later caller-side phase defines propagation or local handling.
- No `TryCall` or failable-call control-flow terminator is introduced in Phase 5B.
- Existing Phase 3 and Phase 4 MIR tests continue to pass.
- No target backend consumes MIR.

## Fixture Candidates

Uncaught recoverable error:

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b = 0 ergo iace "division by zero"
    redde a / b
}
```

Fatal failure:

```fab
functio impossible() → vacuum {
    mori "impossible state"
}
```

Deferred local handling:

```fab
functio handled() → numerus ⇥ textus {
    tempta {
        iace "later"
    }
    cape err {
        redde 0
    }
    redde 1
}
```

## Out Of Scope

- `tempta`, `cape`, and `demum` lowering.
- Caller-side propagation syntax.
- Caller-side local handling syntax.
- Failable call lowering.
- Cleanup/finally semantics.
- Rust backend support.
- WASM or native output.

## Validation

- Focused MIR tests for `error_ty` dump formatting.
- Focused MIR tests for `iace` to `return_error`.
- Focused MIR tests for `mori` to panic plus `unreachable`.
- Negative MIR tests for deferred `tempta`.
- Defensive MIR unit tests for fabricated undeclared `iace`.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 5B complete.

## Completion Gate

Phase 5B is complete when MIR explicitly represents the typed alternate exit path for failable functions, `iace` lowers to `return_error`, `mori` lowers to fatal unreachable flow, local handling remains fail-closed, and existing target codegen behavior is unchanged.
