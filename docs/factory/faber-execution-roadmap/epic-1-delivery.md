# Epic 1 Delivery: Roadmap, Baseline Ledger, And `ab` Removal

**Created**: 2026-05-24
**Roadmap**: [`goal.md`](goal.md)
**Focused Goals**:
- [`../exempla-rust-e2e/goal.md`](../exempla-rust-e2e/goal.md)
- [`../remove-ab-dsl/goal.md`](../remove-ab-dsl/goal.md)

## Interpreted Problem

Epic 1 covers the first three roadmap steps: preserve the coordinating roadmap, create a durable Rust e2e exempla baseline ledger, and remove the retired `ab` collection DSL so it is no longer an active grammar/backend obligation.

## Normalized Spec

- Keep the existing execution roadmap as the umbrella control plane.
- Add a current ledger that classifies the full 138-file exempla baseline from the focused e2e goal.
- Resolve `examples/exempla/ab/ab.fab` through removal rather than backend repair.
- Remove active compiler support for `ab` pipelines across lexer, parser, AST, HIR, semantic passes, visitors, MIR unsupported lists, and backend codegen.
- Demote `ab`, `ubi`, `prima`, `ultima`, and `summa` from keyword syntax; `prima`, `ultima`, and `summa` remain ordinary stdlib method names.
- Update grammar/explain docs so they no longer teach `ab` as canonical collection syntax.
- Add/keep a guard that proves retired `ab` syntax is not accepted as executable Rust syntax.

## Repo-Aware Baseline

- `examples/exempla/` contained 138 `.fab` files at Epic 1 start.
- The focused e2e goal recorded a `71/138` Rust e2e pass baseline with 67 failures.
- `ab/ab.fab` was one of the 67 failures and existed only to exercise the retired DSL.
- `stdlib/norma/innatum/lista.fab` already exposes ordinary `prima`, `ultima`, and `summa` methods.

## Stage Graph

1. Classify the 138-file baseline in a durable ledger.
2. Remove the executable `ab` exemplar.
3. Remove active `ab` lexer/parser/AST/HIR/semantic/codegen surfaces.
4. Rewrite grammar and explain docs around ordinary collection methods.
5. Validate with focused compiler checks and a Rust e2e run.

## Checkpoints

- `docs/factory/exempla-rust-e2e/baseline-ledger.md` exists and accounts for all 138 original exempla.
- `docs/factory/remove-ab-dsl/ledger.md` records the removed DSL surfaces and replacement API direction.
- `rg` over compiler sources has no active `ExprKind::Ab`, `HirExprKind::Ab`, `TokenKind::Ab`, `HirCollection*`, or target-specific `ab` codegen branches.
- `cargo check -p radix` passes.
- Focused tests around retired `ab` syntax pass.
- Rust e2e no longer lists `ab/ab.fab`, because the stale exemplar was removed from the executable corpus.

## Gate Plan

Run, at minimum:

```bash
cargo check -p radix
cargo test -p radix ab
cargo test -p radix parser
cargo test -p radix codegen
cargo test -p radix exempla_rust_e2e -- --ignored --nocapture
```

Before closeout, run broader repo validation if time and existing failures permit.

## Validation Result

Completed on 2026-05-24:

- `cargo check -p radix` passed.
- `cargo test -p radix ab` passed.
- `cargo test -p radix parser` passed.
- `cargo test -p radix codegen` passed.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` passed as a Rust test and reported `71/137` exempla passing; no `ab/ab.fab` failure remains because the retired exemplar was removed.
- `./scripta/test` passed.
- `./scripta/lint` passed.
- `git diff --check` passed.
