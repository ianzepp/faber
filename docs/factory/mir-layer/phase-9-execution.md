# Phase 9 Execution Spec

## Target

Implement `docs/factory/mir-layer/phase-9-delivery.md`: add a deliberately temporary MIR-to-Rust probe that proves validated MIR can be consumed by an executable output path without changing the existing HIR-to-Rust backend.

## Repo Baseline

- MIR validation is already available through `validate_program` and successful `lower_analyzed_unit` results.
- Existing target dispatch in `crates/radix/src/codegen/mod.rs` still consumes HIR.
- No `crates/radix/src/codegen/rust_mir.rs` backend surface exists.

## Implementation Shape

- Add `crates/radix/src/mir/rust_probe.rs`.
- Export `emit_rust_probe` and `MirRustProbeError` from `crates/radix/src/mir/mod.rs`.
- Keep the probe under `mir`, not under durable target backend architecture.
- Use synthetic Rust names for functions, locals, temporaries, and block enum variants so the probe file path and naming policy remain replaceable.
- Emit library-style Rust functions; no `main` is required.
- Preserve explicit MIR control flow with a simple block-state loop and `match`.
- Support primitive Rust type spelling for `textus`, `numerus`, `fractus`, `bivalens`, `vacuum`, and `nihil`.
- Support local/temp operands, primitive constants, local/temp assignments, primitive unary/binary operations, direct function calls by MIR function id or in-program definition id, `return`, `goto`, `branch`, and `unreachable`.
- Fail closed with diagnostics beginning `MIR-to-Rust unsupported` for unsupported types, projections, option operations, runtime calls, aggregate construction, indirect calls, `return_error`, `try_call`, and `switch`.
- Leave normal `emit -t rust` and codegen target dispatch unchanged.

## Validation Gates

- Focused unit tests build validated MIR programs before probe emission.
- Generated Rust is compiled with `rustc`.
- Runtime checks execute generated Rust for primitive arithmetic, direct calls, conditionals, and loops.
- A lowered fixture is emitted through both the MIR probe and the existing HIR-to-Rust backend, and both generated programs are compiled and checked for matching behavior.
- Library-style emission compiles without a `main`.
- Negative tests check explicit unsupported diagnostics for `return_error`, runtime calls, aggregate construction, `try_call`, `switch`, projections, and indirect calls.
- `cargo test -p radix mir`
- `cargo test -p radix`
- `cargo fmt --all --check`
- `./scripta/ci`
