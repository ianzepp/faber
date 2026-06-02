# Phase 2 Delivery Spec: TypeScript Genus Instance Methods

## Interpreted Problem

The baseline shows a broad TypeScript typecheck cluster around `genus`,
methods, `creo`-style construction, and `pactum`/`implet` contracts. The
largest immediately actionable root cause is that TypeScript emits ordinary
Faber methods as static members and lowers `ego` field access to class-name
field access.

## Normalized Spec

- Emit ordinary `genus` methods as instance methods for the TypeScript backend.
- While emitting a method body, lower the HIR path that represents `ego` to
  TypeScript `this`.
- Preserve explicit static behavior only when the HIR carries a receiver mode
  that truly means static; do not invent a new source syntax.
- Keep the change target-local in `crates/radix/src/codegen/ts/`.
- Add focused TypeScript backend coverage for instance method and `ego` output.
- Re-run the TypeScript exemplar harness and record updated tier counts.

## Repo-Aware Baseline

- HIR lowering currently assigns `HirReceiver::None` to genus methods, while
  Rust and Go already interpret these as instance receiver methods.
- HIR lowers `ego` to `HirExprKind::Path(<current struct DefId>)`.
- TypeScript currently emits `HirReceiver::None` methods with `static` and
  resolves the current struct path through the class name, producing code such
  as `static area()` and `Rectangle.width`.
- Baseline TypeScript counts are:

```text
frontend analyzed: 101/101
TypeScript emitted: 100/101
typecheck-valid: 64/101
runnable: 63/101
```

## Stage Graph

1. Add TypeScript codegen context for the active method receiver.
2. Emit genus methods as instance members and map active `ego` paths to `this`.
3. Add focused tests for generated instance methods and `this.field` access.
4. Run focused TS tests, direct exemplar emissions/typechecks, full radix tests,
   lint, and the ignored TS e2e harness.
5. Update `baseline-ledger.md` with the new phase result and remaining clusters.

## Checkpoints

- `cargo test -p radix codegen::ts -- --nocapture`
- Direct TypeScript emission/typecheck for `examples/exempla/genus/methodi.fab`
- Direct TypeScript emission/typecheck for `examples/exempla/implet/implet.fab`
- `cargo test -p radix exempla_ts_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Gate Plan

Commit when focused tests pass, representative method-heavy exemplars improve,
the corpus harness reports updated counts honestly, and no radix regression is
introduced.
