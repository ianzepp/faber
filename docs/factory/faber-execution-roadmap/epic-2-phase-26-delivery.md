# Epic 2 Phase 26 Delivery: Rust Optional Parameters

Timestamp: 2026-05-24 13:34:00 EDT

## Objective

Make Rust output for Faber optional function parameters compile and run for the standalone `functio/optionalis.fab` exemplar.

## Baseline

Latest Epic 2 Rust e2e result: 87/100 exempla pass.

`examples/exempla/functio/optionalis.fab` currently emits invalid Rust because:

- `sponte` parameters are emitted as concrete Rust values, but their bodies compare them with `None`.
- `vel` defaults are parsed in the AST but not preserved in HIR, so Rust codegen cannot fill omitted trailing arguments.
- Call sites with omitted optional arguments are emitted as under-arity Rust calls.

## Implementation Plan

1. Preserve parameter default expressions on `HirParam`.
2. Lower optional parameter bindings as option-shaped values only when the parameter has no default.
3. Emit Rust signatures with:
   - no-default `sponte` parameters as `Option<T>`;
   - `sponte vel` parameters as concrete `T`.
4. Fill omitted direct-call arguments from preserved defaults or `None`.
5. Wrap supplied arguments for no-default optional slots in `Some(...)`.
6. Add focused Rust codegen coverage and run the regular plus ignored Rust e2e gates.

## Non-Goals

- Do not implement full control-flow null narrowing.
- Do not change parser grammar.
- Do not touch Epic 3 `ad` capability calls.
- Do not solve the dynamic `ignotum`/object cluster.

## Validation

- `cargo test -p radix optional_parameters_with_defaults_at_direct_call_sites --lib`
- `cargo test -p radix optional_params_no_longer_require_all_arguments --lib`
- `cargo test -p radix`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`

Result: Rust e2e exempla moved from 87/100 to 88/100. `examples/exempla/functio/optionalis.fab` now passes end-to-end; the remaining failures are outside this phase's optional-parameter scope.
