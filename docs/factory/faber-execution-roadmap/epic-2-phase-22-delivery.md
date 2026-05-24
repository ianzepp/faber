# Epic 2 Phase 22 Delivery: Direct Call Spread Arguments

## Interpreted Problem

`vocatio/vocatio.fab` still fails because `add(sparge numbers)` lowers to a normal one-argument call. The parser records `Argument.spread`, but HIR call lowering discards that flag, so typecheck and Rust codegen cannot distinguish spread calls from ordinary array arguments.

## Normalized Spec

- Preserve the `sparge` marker for HIR call and method-call arguments.
- Require the spread marker before applying spread-array compatibility in call typechecking.
- For Rust direct function calls with one spread array argument, expand the argument into positional indexed arguments using the callee's known parameter count.
- Keep method spread expansion and general variadic behavior out of this phase.
- Add focused coverage for `add(sparge numbers)`.

## Repo-Aware Baseline

- Parser AST already has `Argument { spread, value, span }`.
- HIR currently stores `Call(..., Vec<HirExpr>)` and `MethodCall(..., Vec<HirExpr>)`, losing spread.
- Typecheck has `check_spread_array_compat`, but currently applies it to any single array argument because HIR lacks a spread flag.
- Rust codegen emits direct calls from HIR arguments without expansion.

## Stage Graph

1. Add a HIR call-argument node with `spread` plus `expr`.
2. Thread call arguments through lowering, visitors, typecheck, failable scans, and Rust call codegen.
3. Precompute direct function arity for Rust call expansion.
4. Expand supported direct spread calls in Rust.
5. Validate with focused tests and the ignored e2e corpus.

## Epic Candidates And Scopable Issues

This phase targets direct function calls such as `add(sparge numbers)`. Method spread calls, object spreads into calls, and richer tuple/variadic lowering remain later work if exempla require them.

## Checkpoints

- `add(sparge numbers)` emits a two-argument Rust call for `add(numerus, numerus)`.
- Ordinary one-array calls no longer receive spread compatibility merely because their type is an array.
- `vocatio/vocatio.fab` clears its final spread-call blocker if no unrelated errors remain.

## Companion Skill Plan

- `factory`: preserve phase boundary, validation, and commit discipline.
- `delivery`: this saved implementation artifact.

## Gate Plan

- Focused Rust codegen test for direct spread call expansion.
- Direct Rust emission for `examples/exempla/vocatio/vocatio.fab`.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`.

## Open Questions

- Whether spread arguments should lower to MIR as array expansion or as a first-class call argument shape remains outside this Rust backend phase.
