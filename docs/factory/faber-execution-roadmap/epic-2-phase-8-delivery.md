# Epic 2 Phase 8 Delivery: Standalone Closure Example

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, stale exemplar source correction

## Interpreted Problem

`examples/exempla/clausa/clausa.fab` is a closure example, but it demonstrates collection processing with JS-style `.map` and `.filter` calls. Those names are not the canonical Faber collection method surface, and the broader stdlib method translation work belongs to a separate backend phase.

## Normalized Spec

Keep the file focused on closure syntax and closure calls. Rewrite the collection portion to use existing standalone Faber constructs (`itera`, `appende`, and explicit `vacua` list declarations) instead of stale JS-style method names.

## Checkpoints

- The example still demonstrates single-parameter, multi-parameter, typed-return, and reused closures.
- The source no longer calls `.map` or `.filter`.
- Rust e2e no longer reports `clausa/clausa.fab` as a failed exemplar.

## Validation

- `cargo run -p faber -- check examples/exempla/clausa/clausa.fab`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `68/100` exempla files passing end-to-end.
- `examples/exempla/clausa/clausa.fab` no longer appears in the Rust e2e failure list.
