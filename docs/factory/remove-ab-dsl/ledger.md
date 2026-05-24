# `ab` Removal Ledger

**Created**: 2026-05-24
**Status**: Epic 1 implementation ledger

## Break Boundary

The active Faber language no longer has a grammar-level `ab` collection pipeline. The words `ab`, `ubi`, `prima`, `ultima`, and `summa` are not collection-DSL tokens. `prima`, `ultima`, and `summa` remain available as ordinary method identifiers where `stdlib/norma` defines them.

## Surface Inventory

| Surface | Epic 1 action | Replacement |
| --- | --- | --- |
| `ab <source> <property>` | Removed parser/HIR/codegen support | Collection filtering APIs and closures |
| `ab <source> non <property>` | Removed parser/HIR/codegen support | Collection filtering APIs and closures |
| `ab <source> ubi <condition>` | Removed retired syntax from docs | Collection filtering APIs and closures |
| `prima` transform | Removed as DSL keyword | `<lista>.prima(n)` |
| `ultima` transform | Removed as DSL keyword | `<lista>.ultima(n)` |
| `summa` transform | Removed as DSL keyword | `<lista>.summa()` |
| Chained transforms | Removed target-specific backend lowering | Ordinary method-call composition |

## Implementation Evidence

- `examples/exempla/ab/ab.fab` was deleted because it existed only to teach and exercise the retired DSL.
- `crates/radix/src/lexer/scan.rs` no longer tokenizes the DSL words as collection keywords.
- `crates/radix/src/parser/expr.rs` no longer accepts `ab` as a primary expression.
- `crates/radix/src/syntax/ast.rs` and `crates/radix/src/hir/nodes.rs` no longer define `Ab` expression nodes or collection transform structs.
- Resolver, typecheck, visitors, MIR lowering diagnostics, and Rust/Go/TS/Faber codegen no longer carry target-specific `ab` branches.
- `EBNF.md` and `explain/` mark `ab`/`ubi` as retired and describe `prima`/`ultima`/`summa` as methods.

## Validation Result

Focused validation proved both sides of the break:

- `cargo test -p radix ab` includes `retired_ab_pipeline_is_not_active_rust_syntax`.
- `cargo test -p radix parser` and `cargo test -p radix codegen` passed after removing parser and backend branches.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reported `71/137` exempla passing and no `ab/ab.fab` failure.
- `./scripta/test` and `./scripta/lint` passed.
