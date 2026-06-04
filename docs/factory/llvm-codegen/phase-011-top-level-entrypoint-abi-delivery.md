# Phase 011: Top-Level Initialization And Entrypoint ABI Delivery

## Interpreted Problem

MIR already lowers explicit `incipit` and implicit top-level executable code as a
synthetic function with no source definition and no name. LLVM currently emits
that function as an internal-looking generated name such as `@f1`, so the text
probe has no stable entrypoint surface even though Wasm exports the same MIR
function as `incipit`.

## Normalized Spec

- Define the LLVM text-probe entrypoint symbol for synthetic MIR entry
  functions.
- Keep this as a text/ABI naming policy, not a native execution promise.
- Avoid collisions with user-authored functions named `incipit`.
- Keep top-level constants and source-order initialization policy deferred
  unless current MIR already models them as ordinary entry statements.
- Preserve MIR and Wasm behavior.
- Update the LLVM baseline ledger with measured counts and residual gaps.

## Repo-Aware Baseline

- MIR synthetic entry functions have `source: None` and `name: None`.
- Wasm exports synthetic entry functions as `incipit`.
- LLVM currently names synthetic entry functions with generated function IDs.
- Phase 010 e2e baseline: 59/102 LLVM emitted, 0/102 verifier-valid, 15
  unsupported LLVM diagnostics.

## Stage Graph

1. Add an LLVM entry-function detector matching the MIR/Wasm synthetic entry
   convention.
2. Emit synthetic entry functions as `@incipit`.
3. Keep user-authored `functio incipit` callable by suffixing it when the
   synthetic entry consumes the canonical symbol.
4. Add focused tests for ordinary entry emission and collision behavior.
5. Update the baseline ledger and record validation evidence.

## ABI Decision

The LLVM text probe reserves `@incipit` for the synthetic Faber entry function:

```llvm
define void @incipit() {
  ...
}
```

This is only a stable symbol in emitted text. This phase does not define a C
`main`, process argument ABI, native executable linking, global constructors, or
runtime startup.

## Checkpoints

- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- Completion audit against this spec before commit.

## Wasm Follow-Up

No MIR shape or Wasm import naming changes are expected. This phase aligns the
LLVM symbol policy with the existing Wasm entry export policy.

## Completion Evidence

Phase 011 reserves the LLVM text-probe `@incipit` symbol for MIR synthetic entry
functions. The detector is intentionally narrow: the function must be sourceless,
nameless, parameterless, and `vacuum`-returning. This preserves hand-built MIR
helpers and user functions while matching the current real entry lowering shape.

Implemented support:

- Synthetic entry functions emit as `define void @incipit()`.
- User-authored functions whose sanitized base name collides with `incipit` get
  a deterministic function-id suffix.
- Direct calls continue to resolve through the same function-name helper, so
  collision suffixing remains call-consistent.

Still deferred:

- C `main` or native executable startup.
- Process arguments and `incipit argumenta` ABI.
- Top-level constants and source-order global initialization outside current
  entry-block MIR.
- Runtime startup and linker policy.

Focused tests added:

- `llvm_text_target_emits_incipit_entry_symbol`
- `llvm_text_target_keeps_user_incipit_name_from_colliding_with_entry_symbol`

Measured e2e baseline after implementation:

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

cargo test -p radix mir -- --nocapture
result: 144 passed, 0 failed

cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored

cargo test -p radix
result: 561 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored

./scripta/lint
result: passed

rustfmt --edition 2021 --check crates/radix/src/mir/llvm_text.rs crates/radix/src/mir/llvm_text_test.rs
result: passed

git diff --check
result: passed
```

## Completion Audit

- Stable LLVM text entry symbol is implemented for current MIR synthetic entry
  functions.
- User-symbol collision handling is deterministic and tested.
- No native execution, C ABI, process argument, or startup claim was introduced.
- No MIR or Wasm behavior was changed.
