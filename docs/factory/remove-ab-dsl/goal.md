# Goal: Remove The `ab` Collection DSL

**Status**: problem defined, not started
**Created**: 2026-05-24
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/remove-ab-dsl/`
**Mode**: language simplification and corpus/docs cleanup
**Commit Policy**: Commit after each completed phase and validation gate pass

## Summary

Remove the grammar-level `ab` collection pipeline DSL and migrate examples, docs, tests, and backend code toward ordinary collection library calls such as list filtering, mapping, slicing, and reduction methods. The language should not carry a separate chaining grammar for collection operations that can be expressed through regular `lista`/`tabula` APIs.

## Problem

- `ab` is currently a special collection DSL with dedicated keywords and grammar (`ab`, `ubi`, `prima`, `ultima`, `summa`).
- The DSL creates parser, HIR, typecheck, codegen, explain, and e2e obligations distinct from ordinary method calls.
- The same behavior is better represented as normal list/map library calls and closures, keeping collection behavior in the stdlib/API layer rather than in syntax.
- `examples/exempla/ab/ab.fab` currently fails Rust e2e; that failure should not be fixed by hardening the DSL if the DSL is being removed.

## Goals

- Remove `ab` as canonical active Faber syntax.
- Remove or retire `ubi`, `prima`, `ultima`, and `summa` as collection-DSL-only keywords.
- Replace executable `ab` exempla with ordinary collection method examples.
- Update grammar, explain corpus, and docs so they no longer teach `ab` as a canonical collection pipeline.
- Remove backend-specific `HirExprKind::Ab` lowering/codegen obligations once examples and tests have migrated.
- Preserve ordinary list/map library functionality, including filtering, mapping, first/last/take, and sum/reduce behavior, through regular APIs.

## Non-Goals

- Do not design a replacement DSL.
- Do not remove list/map collection functionality itself.
- Do not add temporary compatibility code unless needed for a short, explicitly bounded migration phase.
- Do not keep `prima`, `ultima`, or `summa` globally reserved if their only remaining purpose was the retired DSL.
- Do not weaken closure or stdlib method typechecking to make migrated examples pass.

## Ground Truth Researched

- `EBNF.md`: defines `abExpr`, collection filter syntax, and transforms `prima`, `ultima`, `summa`.
- `explain/ab.md`, `explain/ubi.md`, `explain/prima.md`, `explain/ultima.md`, `explain/summa.md`: teach the DSL as canonical explain terms.
- `examples/exempla/ab/ab.fab`: executable corpus example focused entirely on `ab` filtering and transforms.
- `crates/radix/src/parser/expr.rs`: parses `ab` as a primary expression and has dedicated collection DSL parsing.
- `crates/radix/src/syntax/ast.rs`: has `ExprKind::Ab`, `AbExpr`, filters, and transforms.
- `crates/radix/src/hir/nodes.rs`, `crates/radix/src/hir/lower/expr.rs`: preserve `Ab` in HIR.
- `crates/radix/src/semantic/passes/resolve.rs` and `crates/radix/src/semantic/passes/typecheck/expr.rs`: resolve and typecheck the DSL.
- `crates/radix/src/codegen/rust/expr/collection.rs`, `crates/radix/src/codegen/go/expr/collection.rs`, `crates/radix/src/codegen/ts/expr.rs`, `crates/radix/src/codegen/faber/expr.rs`: emit target-specific forms for `ab`.
- `docs/factory/contextual-keyword-scope/plan.md`: already notes `ab`, `ubi`, `prima`, `ultima`, and `summa` as candidates if the collection DSL is demoted to stdlib methods.

## Reference Packet

Before editing, inspect:

- `EBNF.md`: remove active `ab` grammar and adjust keyword notes.
- `crates/radix/src/lexer/scan.rs`, `crates/radix/src/lexer/token.rs`, `crates/radix/src/lexer/keywords.rs`: decide whether DSL keywords become identifiers or legacy diagnostics.
- `crates/radix/src/parser/expr.rs`: remove `ab` primary parsing or convert it to a clear retired-syntax diagnostic.
- `crates/radix/src/syntax/ast.rs`: remove `AbExpr` and related collection DSL syntax nodes after parser users are gone.
- `crates/radix/src/hir/nodes.rs`, `crates/radix/src/hir/lower/expr.rs`, `crates/radix/src/hir/visit.rs`: remove HIR `Ab` shape and visitor branches.
- `crates/radix/src/semantic/passes/resolve.rs`, `crates/radix/src/semantic/passes/typecheck/expr.rs`: remove dedicated `ab` handling.
- `crates/radix/src/codegen/**`: remove target-specific `ab` emission once HIR no longer contains it.
- `stdlib/norma/innatum/lista.fab`: verify ordinary list methods can express migrated examples; add library methods only if they are truly missing.
- `examples/exempla/ab/ab.fab`: migrate, move, or remove the DSL example.
- `explain/`: retire or rewrite explain terms for `ab`, `ubi`, `prima`, `ultima`, and `summa`.

## Constraints And Invariants

- Collection behavior should live in library APIs, not grammar-specific pipeline syntax.
- Type-first syntax and existing closure syntax remain canonical.
- Do not invent new chaining grammar during removal.
- Parser diagnostics should be truthful: either `ab` is legacy/retired syntax with a replacement suggestion, or it is just an identifier after keyword demotion.
- If keywords are demoted, avoid breaking unrelated uses of `prima`, `ultima`, or `summa` as ordinary identifiers.
- Rust e2e should not count `ab/ab.fab` as a backend semantic bug once this removal goal is active.

## Implementation Shape

### Phase 0: Inventory And Replacement API Check

Create a short ledger of every active `ab` DSL surface and the library-call replacement expected for each DSL feature: boolean-property filter, `ubi` predicate filter, `prima`, `ultima`, `summa`, and chained transforms. Verify whether current `lista`/`tabula` stdlib APIs can express the replacements.

### Phase 1: Migrate Exempla And Tests

Rewrite executable examples and tests that currently depend on `ab` to use ordinary list/map calls and closures. If a test exists only to prove the DSL, delete or replace it with parser diagnostics proving the syntax is retired.

### Phase 2: Retire Parser And Keyword Surface

Remove `ab` primary parsing and either demote DSL-only keywords to identifiers or produce explicit legacy diagnostics with replacement guidance. Keep this phase tight so parser behavior is easy to review.

### Phase 3: Remove HIR, Semantic, And Backend Branches

Delete `Ab` AST/HIR shapes, visitors, resolver/typechecker branches, and Rust/Go/TS/Faber codegen branches after no active parser path can construct them. This is the structural cleanup phase.

### Phase 4: Documentation And Explain Cleanup

Update `EBNF.md`, explain corpus, contextual-keyword docs, release/history notes where they describe active syntax, and any corpus references. If keeping legacy explain entries, mark them as retired and point to library methods.

### Phase 5: Validation And Drift Guard

Run focused parser, semantic, codegen, and e2e validation. Add a guard test that `ab` is no longer accepted as active collection DSL syntax, or that it produces the intended retired-syntax diagnostic.

## Acceptance Criteria

- No active Faber grammar path accepts `ab` as a collection pipeline expression.
- `ab`, `ubi`, `prima`, `ultima`, and `summa` are no longer documented as canonical collection DSL syntax.
- Executable collection examples use ordinary list/map library calls.
- No backend has target-specific `ab` codegen obligations.
- Rust e2e no longer reports `examples/exempla/ab/ab.fab` as a DSL backend failure; the example is migrated, moved, or removed.
- Existing collection library behavior remains available and tested.

## Validation

- `cargo test -p radix ab` should either disappear because focused tests were removed or pass because only legacy-diagnostic/replacement tests remain.
- `cargo test -p radix parser` should pass after parser removal.
- `cargo test -p radix codegen` should pass after backend branch removal.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` should no longer include an `ab` DSL failure.
- `./scripta/test` should pass before closeout.
- `./scripta/lint` should pass before closeout.

## Open Questions

- Should `ab` become a hard parse error with a replacement hint, or should it demote fully to an ordinary identifier?
- Should `ubi`, `prima`, `ultima`, and `summa` demote to identifiers immediately, or remain legacy-reserved long enough to produce better diagnostics?
- Which exact `lista` methods should be the canonical replacements for first, last, take, and sum behavior if current stdlib naming is incomplete?
- Should migrated examples preserve one file named `ab.fab` as a legacy-negative fixture, or should the file be removed/renamed entirely?

## Stop Conditions

- Stop if ordinary collection APIs cannot yet express the migrated executable examples without inventing new syntax.
- Stop if removing keywords would silently change parse behavior in a way that hides a better legacy diagnostic.
- Stop if backend branch removal would require broad unrelated refactors.
- Stop before deleting historical documentation that should instead be marked as retired design history.
