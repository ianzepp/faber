# Phase 9 Delivery: MIR Rust Probe

**Status**: planned.

## Interpreted Problem

Phase 8 makes successful MIR lowering pass through validation before any backend can consume it. The next useful question is not yet "what is the permanent Rust backend architecture?" The next question is smaller and sharper: can validated MIR be consumed by an output path at all?

Phase 9 should answer that by adding a deliberately temporary MIR-to-Rust probe. The probe exists to test the compiler boundary from validated MIR into runnable target code. It should not establish the final module layout, backend trait, migration strategy, or long-term Rust codegen surface.

## Normalized Spec

- Add a MIR Rust probe emitter under the MIR experiment surface.
- Treat the probe artifact as temporary and replaceable.
- Consume only validated `MirProgram` values.
- Emit minimal Rust source for a narrow primitive/control-flow subset.
- Keep the existing HIR-to-Rust backend unchanged and default.
- Expose the probe only through explicit internal tests or an explicit experimental command/path.
- Compile generated Rust in tests.
- Compare behavior against the existing HIR-to-Rust backend for selected fixtures where practical.
- Fail closed for every unsupported MIR node with clear `MIR-to-Rust unsupported` diagnostics.
- Do not require a `main` entry point; emitted Rust should work as library-style functions.

## Probe Boundary

The long-lived architectural boundary is:

```text
validated MIR -> target code
```

The Phase 9 artifact that proves the boundary is intentionally not long-lived. Prefer a name and location that communicate probe status, for example:

```text
crates/radix/src/mir/rust_probe.rs
```

or:

```text
crates/radix/src/mir/probe_rust.rs
```

Do not create `crates/radix/src/codegen/rust_mir.rs` in Phase 9 unless the implementation explicitly chooses to promote the probe into a real backend after the delivery goal changes. That filename reads like durable backend architecture, and Phase 9 is not making that decision.

## Supported Subset

The first probe should support only the smallest subset needed to prove execution:

- MIR functions with parameters and local declarations,
- primitive return types and `vacuum`,
- integer, float, boolean, string, unit, and nil constants where already represented in validated MIR,
- local and temp operands,
- assignments to locals and temps,
- unary and binary primitive operations already lowered by Phases 3 and 4,
- direct calls to functions in the same MIR program when signatures are known,
- `return`,
- `goto`,
- `branch`,
- `unreachable`,
- simple `si` and `dum` only as their normalized MIR block/terminator shape.

The probe does not need to make emitted Rust beautiful. Correctness and explicit diagnostics matter more than readability.

## Fail-Closed Scope

Phase 9 should reject unsupported MIR explicitly rather than falling back silently or guessing:

- `return_error`,
- `try_call`,
- structured `cape` recovery edges,
- aggregate construction and projection unless a tiny case is consciously included,
- option/null operations beyond primitive constants,
- runtime intrinsics,
- provider calls,
- collection operations,
- indirect calls,
- switches,
- closures, async/cursor behavior, and deferred source constructs.

Every rejection should identify the unsupported MIR shape. The diagnostic should make clear that the limitation belongs to the probe, not to MIR validation.

## Rust Emission Strategy

The probe should avoid implementing a full control-flow structurer. A block-state loop is acceptable for Phase 9 because it proves executable target consumption without pretending to be final Rust codegen:

```rust
enum __FaberBlock {
    Bb0,
    Bb1,
}

let mut __faber_block = __FaberBlock::Bb0;
loop {
    match __faber_block {
        __FaberBlock::Bb0 => {
            __faber_block = __FaberBlock::Bb1;
        }
        __FaberBlock::Bb1 => {
            return value;
        }
    }
}
```

The exact emitted shape can be adjusted during implementation, but Phase 9 should not spend effort on structured Rust reconstruction unless it is simpler than the state-machine approach for the selected subset.

## Repo-Aware Baseline

- `crates/radix/src/mir/nodes.rs` defines the MIR model.
- `crates/radix/src/mir/validate.rs` validates successful MIR before it leaves `lower_analyzed_unit`.
- `crates/radix/src/codegen/rust/` is the existing HIR-to-Rust backend.
- `crates/radix/src/codegen/mod.rs` dispatches target codegen from HIR.
- No current file named `crates/radix/src/codegen/rust_mir.rs` exists in this checkout.
- No backend currently consumes MIR.

## Stage Graph

1. Choose the temporary probe module path under `crates/radix/src/mir/`.
2. Add a small public/internal API that accepts a validated `MirProgram` and emits Rust source.
3. Add a probe error type or reuse `CodegenError` only if doing so does not imply permanent codegen integration.
4. Implement Rust type spelling for the primitive supported subset.
5. Implement operand, place, value, and assignment emission for locals and temps.
6. Implement direct local function calls.
7. Implement `return`, `goto`, `branch`, and `unreachable`.
8. Use a block-state loop or equivalent simple CFG-preserving lowering.
9. Add tests that compile emitted Rust for primitive functions and simple control flow.
10. Add parity checks against existing HIR-to-Rust behavior for selected fixtures where practical.
11. Add fail-closed tests for unsupported MIR shapes.
12. Keep existing target backend dispatch unchanged unless an explicit experimental path is added.

## Checkpoints

- A simple validated MIR function emits Rust source.
- A primitive arithmetic function emitted from MIR compiles.
- A function call between two MIR functions emitted from MIR compiles.
- A simple conditional emitted from normalized MIR compiles and returns the expected value.
- A simple loop emitted from normalized MIR compiles and returns the expected value.
- Unsupported MIR shapes fail with `MIR-to-Rust unsupported` diagnostics.
- The existing HIR-to-Rust backend remains default.
- No package build or normal `emit -t rust` path silently switches to MIR.
- The probe can be deleted or relocated in a later phase without preserving its file path as public architecture.

## Out Of Scope

- Permanent Rust backend architecture.
- Replacing HIR-to-Rust codegen.
- Adding `crates/radix/src/codegen/rust_mir.rs` as a durable backend surface.
- Full Rust code quality or pretty structured Rust reconstruction.
- Cargo package generation changes except test harness support needed to compile emitted Rust.
- Runtime intrinsic lowering to Rust.
- Alternate-exit lowering to Rust.
- `cape` handler execution.
- Aggregate, option, collection, provider, async, closure, or iterator support.
- WASM, native, LLVM, or Cranelift work.
- ABI/layout decisions.
- Optimization.

## Validation

- Focused unit tests for Rust source emission from hand-built or lowered validated MIR.
- Compile tests for emitted Rust snippets using Rust-only tooling.
- Runtime behavior checks for selected primitive/control-flow fixtures where practical.
- Negative tests for unsupported MIR nodes.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `cargo fmt --all --check`.
- `./scripta/ci` before marking Phase 9 complete.

## Completion Gate

Phase 9 is complete when a deliberately temporary MIR Rust probe consumes validated MIR for a narrow primitive/control-flow subset, emits Rust that compiles and behaves correctly for selected fixtures, rejects unsupported MIR clearly, leaves the existing HIR-to-Rust backend as the default path, and avoids presenting the probe file as the permanent Rust backend architecture.
