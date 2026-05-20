# Phase 4 Delivery Spec: Split Rust Expression Codegen

**Phase**: 4 of 6  
**Source Plan**: `docs/radix-large-module-refactor-factory-plan.md`  
**Owned Area**: `radix/crates/radix/src/codegen/rust/expr.rs` and `radix/crates/radix/src/codegen/rust/expr/**`

## Objective

Split Rust expression codegen into cohesive submodules without changing emitted Rust output.

## Target Shape

```text
radix/crates/radix/src/codegen/rust/expr/
├── mod.rs
├── literal.rs
├── ops.rs
├── block.rs
├── pattern.rs
├── collection.rs
├── control.rs
├── call.rs
├── option.rs
└── format.rs
```

## Responsibility Map

- `mod.rs`: public `generate_expr`, `generate_expr_unwrapped`, dispatch, and conversion-heavy shared helpers.
- `literal.rs`: literal rendering, string escaping, regex flag handling.
- `ops.rs`: unary and binary operator rendering and expression generation.
- `block.rs`: expression block generation.
- `pattern.rs`: pattern rendering.
- `collection.rs`: collection pipeline, arrays, structs, tuples, and map-key helpers.
- `control.rs`: `tempta`, `if`, match, loop, while, for, and interval expression forms.
- `call.rs`: function and method calls, including failable call propagation.
- `option.rs`: optional chain and non-null rendering.
- `format.rs`: `scriptum`, `scribe` format selection, and format-template helpers.

## Execution Rules

- Move complete Rust items or complete match-arm bodies into handlers; do not slice through attributes, comments, signatures, or braces.
- Preserve `expr::generate_expr(...)` and `expr::generate_expr_unwrapped(...)` call sites.
- Keep Rust type rendering in `types.rs` and failable analysis in `failable.rs`.
- Keep behavior unchanged; this phase is structural only.
- Run `cargo fmt --manifest-path radix/Cargo.toml --all` and `cargo check --manifest-path radix/Cargo.toml -p radix` during extraction.
- Run the full validation gate before commit.

## Checkpoint

- Rust expression codegen compiles from the new `expr/` module tree.
- All target modules contain live code aligned with the responsibility map.
- Full validation gate passes.
