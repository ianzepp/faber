# Epic 2 Phase 13 Delivery: Clone Owned Call Arguments

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, Rust ownership lowering for function-call arguments

## Interpreted Problem

`examples/exempla/itera/in-functione.fab` passes the same array binding to several functions. Rust codegen currently emits each call with the `Vec<T>` path directly, so the first call consumes the vector and later calls fail with moved-value errors.

## Normalized Spec

When an ordinary Rust call argument is a path to a semantically owned value, clone it at the call boundary. Keep primitive copy values, literals, and constructed temporary expressions unchanged.

## Checkpoints

- Repeated calls with the same array binding emit `.clone()` on the argument.
- Inline array literals still pass as constructed temporaries.
- `itera/in-functione.fab` no longer fails on moved `numbers`.
- Rust e2e pass count improves or any newly exposed blocker is recorded.

## Validation

- `cargo test -p radix clones_owned_path_arguments_for_function_calls -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `74/100` exempla files passing end-to-end.
- `examples/exempla/itera/in-functione.fab` no longer appears in the Rust e2e failure list.
