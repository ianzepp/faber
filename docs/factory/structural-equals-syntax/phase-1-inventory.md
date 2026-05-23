# Phase 1 Inventory: Structural Equals Syntax

**Status**: complete
**Checkpoint**: exact edit sites for colon field values, typed construction, destructuring rename, empty values, docs, examples, and tests are identified.

## Interpreted Problem

Faber currently still uses `:` for several structural value definitions, even though the language model reserves `←` for runtime value assignment, `=` for structural value definition, and `ut` for aliases. Phase 1 inventories the active implementation and documentation surfaces before changing behavior.

## Parser Sites

| Surface | File | Current State | Phase |
| --- | --- | --- | --- |
| Genus field defaults | `crates/radix/src/parser/decl.rs` | `parse_class_member` accepts `TokenKind::Colon` before the initializer expression. | 2 |
| Object literal field values | `crates/radix/src/parser/expr.rs` | `parse_object_fields` accepts `:` for string, computed, and identifier keys. Identifier shorthand has no separator. | 2 |
| `finge` payload fields | `crates/radix/src/parser/expr.rs` | `parse_variant_expr` accepts `:` after each payload field name. | 2 |
| Typed construction | `crates/radix/src/parser/expr.rs` | No `Type { ... }` constructor parse path; `{ ... } ∷ Type` is handled by existing `Verte` lowering. | 3 |
| Destructuring rename | `crates/radix/src/parser/decl.rs`, `crates/radix/src/parser/stmt.rs` | Array binding destructuring exists. Object binding destructuring is not implemented. `ex` destructuring already uses `ut`. | 4 |
| Empty value expression | `crates/radix/src/lexer/scan.rs`, parser/lowering/typecheck | `vacua` is not a keyword or expression. Empty collections currently use `[] ∷ lista<T>` / `{ } ∷ tabula<K,V>`. | 5 |

## Representation And Semantics Sites

| Surface | File | Current State |
| --- | --- | --- |
| Object entries | `crates/radix/src/syntax/ast.rs`, `crates/radix/src/hir/nodes.rs`, `crates/radix/src/hir/lower/expr.rs` | Existing AST/HIR object entries carry keys and optional values. No separator is stored, so `:` to `=` is parser/codegen only. |
| Struct construction | `crates/radix/src/hir/lower/expr.rs`, `crates/radix/src/semantic/passes/typecheck/convert.rs` | Existing `Verte` lowering extracts object entries and typechecking validates them against a struct target. Typed construction can reuse this path. |
| `finge` payloads | `crates/radix/src/hir/lower/expr.rs` | Payload fields lower to ordered call arguments; field names are not preserved beyond parsing. The separator change is parser/docs only for target codegen. |
| Faber emitter | `crates/radix/src/codegen/faber/decl.rs`, `crates/radix/src/codegen/faber/literal.rs`, `crates/radix/src/codegen/faber/expr.rs` | Self-codegen emits `:` for genus defaults, object fields, and struct literal fields. |

## Test And Example Sites

| Surface | Files |
| --- | --- |
| Parser tests | `crates/radix/src/parser/mod_test.rs` |
| Driver/compiler tests | `crates/radix/src/driver/mod_test.rs`, codegen target tests under `crates/radix/src/codegen/**/mod_test.rs` |
| Faber self-codegen tests | `crates/radix/src/codegen/faber/mod_test.rs` |
| Canonical examples | `examples/exempla/genus/*.fab`, `examples/exempla/finge/finge.fab`, plus scattered `{} ∷ Type` and `[] ∷ lista<T>` examples |

## Documentation Sites

| Surface | Files |
| --- | --- |
| Grammar | `EBNF.md` |
| Explain corpus | `explain/` |
| Repo instructions | `AGENTS.md` |
| Release/design history | `docs/`, especially docs that mention `{ x: 10 } ∷ Type` as active syntax |

## Phase 2/3 Implementation Strategy

Accept `=` directly in genus defaults, object fields, and `finge` payloads. Add typed construction by parsing `Ident { ... }` as a synthetic `Verte` expression whose source is an object literal and whose target is the named type, reusing the current lowering/typechecking/codegen behavior for `{ ... } ∷ Type`.

Colon forms should move to diagnostics/cleanup in phase 6 rather than remain canonical. Faber self-codegen must emit `=` so roundtrip tests stop teaching the retired form.
