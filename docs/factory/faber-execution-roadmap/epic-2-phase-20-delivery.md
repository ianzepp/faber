# Epic 2 Phase 20 Delivery: Empty Typed Constructors

## Interpreted Problem

`Counter {}` and `Calculator {}` are valid typed constructors according to `EBNF.md`, but the parser only recognizes typed constructors when the brace body begins with a field assignment. Empty constructor syntax is currently split into a type path expression followed by an empty block, so Rust emits invalid `Counter; { }`-style code.

## Normalized Spec

- Parse `Type {}` as a typed constructor with an empty field list.
- Preserve existing typed constructor parsing for `Type { field = value }`.
- Do not introduce new constructor-call semantics or `creo` execution in this phase.
- Rely on existing Rust omitted-field/default emission once HIR receives an empty struct literal.
- Add focused parser/codegen coverage.

## Repo-Aware Baseline

- `EBNF.md` defines `typedConstructor := typeAnnotation '{' fieldList? '}'`.
- `parser/expr.rs::looks_like_typed_constructor_fields` currently requires the first two tokens after `{` to be a field key and `=`.
- Rust struct literal codegen already has `generate_omitted_struct_fields`, which can fill defaulted and `sponte` fields when the HIR struct literal has no provided fields.

## Stage Graph

1. Adjust parser typed-constructor lookahead so an immediate `}` is accepted.
2. Add a parser test or codegen test proving empty typed constructors survive as constructors.
3. Run focused validation against `genus/methodi.fab`.
4. Run the ignored Rust exempla e2e suite.

## Epic Candidates And Scopable Issues

This phase only fixes the parse boundary for empty typed constructors. Method chaining that returns `ego`, spread calls, and `creo` constructor hooks remain separate blockers.

## Checkpoints

- Generated Rust for `Counter {}` includes `Counter { count: 0, }`.
- `genus/methodi.fab` no longer fails on `expected value, found struct Counter`.
- The e2e pass count is recorded after validation.

## Companion Skill Plan

- `factory`: preserve phase boundary, validation, and commit discipline.
- `delivery`: this saved implementation artifact.

## Gate Plan

- Focused unit test for empty typed constructors.
- `cargo run -p faber -- emit -t rust examples/exempla/genus/methodi.fab`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`

## Open Questions

- `creo` execution and owned-return method chaining remain open for later phases.
