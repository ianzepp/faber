# Phase 1 Delivery Spec: Keyword Metadata Registry

**Parent Plan**: `docs/factory/contextual-keyword-scope/plan.md`
**Phase**: 1 - Metadata registry
**Status**: ready for validation
**Created**: 2026-05-21

## Interpreted Phase Problem

The compiler needs a central, text-first keyword registry before parser helpers or lexer migrations are introduced. The registry must describe current lexer spellings and known contextual vocabulary without changing user-visible syntax.

## Normalized Phase Spec

Add a metadata module near the lexer that provides:

- `KeywordSpec`
- `KeywordScope`
- `KeywordOwner`
- `KeywordCategory`
- `keyword_specs()`
- `lookup_keyword_spec(text)`

The registry must include:

- every current normal-mode spelling in `keyword_or_ident()` except `_`
- test-owned words marked as `TestOwned`
- contextual `page` even though it lexes as an identifier today
- annotation-owned vocabulary such as `futura` and `cursor`
- alias metadata for current alias/redirect vocabulary

The registry must not:

- change `keyword_or_ident()` behavior
- remove any `TokenKind`
- alter parser dispatch or recovery

## Repo-Aware Baseline

Relevant files:

- `crates/radix/src/lexer/scan.rs`: current keyword table
- `crates/radix/src/lexer/token.rs`: keyword token variants
- `crates/radix/src/parser/stmt.rs`: `cura`, `incipit`, `ad`, destructuring
- `crates/radix/src/parser/decl.rs`: declarations, modifiers, import, class/interface, annotations
- `crates/radix/src/parser/expr.rs`: operators/forms, collection DSL, rest/spread expressions

## Stage Graph

| Step | Task | Verification |
| ---- | ---- | ------------ |
| 1 | Add `lexer/keywords.rs` metadata types and static registry | compiles |
| 2 | Re-export metadata from `lexer/mod.rs` | downstream tests can inspect registry |
| 3 | Add tests proving lexer table spellings have specs | `cargo test -p radix keyword` |
| 4 | Add tests for contextual `page` and alias text preservation | `cargo test -p radix keyword` |
| 5 | Run full repo script | `./scripta/test` |

## Checkpoint And Gate

Checkpoint:

- registry and lexer agree on current active normal-mode spellings
- contextual `page` has a registry entry
- alias entries are represented as text-bearing specs
- no user-visible syntax changes

Verification commands:

```bash
cargo test -p radix keyword
./scripta/test
```

## Completion Notes

Phase 1 is complete only if the tests prove registry coverage from current source, not from a manually duplicated list.

