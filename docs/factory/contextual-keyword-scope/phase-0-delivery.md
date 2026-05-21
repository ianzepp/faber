# Phase 0 Delivery Spec: Baseline Keyword Inventory

**Parent Plan**: `docs/factory/contextual-keyword-scope/plan.md`
**Phase**: 0 - Baseline inventory
**Status**: implemented
**Created**: 2026-05-21

## Interpreted Phase Problem

The compiler currently reserves too many normal-mode words in `keyword_or_ident()`. Before moving any word out of the lexer table, this phase records the current vocabulary, the parser site that owns each contextual candidate, and the recovery consequence of later migrations.

No compiler behavior changes are allowed in Phase 0.

## Normalized Phase Spec

Inputs:

- `crates/radix/src/lexer/scan.rs`
- `crates/radix/src/lexer/token.rs`
- parser ownership sites under `crates/radix/src/parser/`
- current grammar/docs/examples as evidence for canonical spellings

Outputs:

- `docs/factory/contextual-keyword-scope/ledger.md`
- explicit first migration family: `cura` kind words `arena` and `page`
- explicit parser owner and recovery note for each planned migration family

Out of scope:

- changing `keyword_or_ident()`
- changing parser behavior
- normalizing test-owned words

## Repo-Aware Baseline

Current keyword resolution has three surfaces:

- normal mode: `keyword_or_ident()` in `crates/radix/src/lexer/scan.rs`
- annotation mode: all words after `@` lex as identifiers for the line
- section mode: all words after `§` lex as identifiers for the line

Current ad hoc contextual behavior:

- `parse_cura_stmt()` matches `arena` as `TokenKind::Arena` and `page` as identifier text.
- `parse_member_ident()` allows selected keyword tokens as member names.
- `parse_annotation_name()` allows selected keyword tokens as annotation names.
- annotation modifier parsing uses direct string checks.

## Stage Graph

| Step | Task | Evidence |
| ---- | ---- | -------- |
| 1 | Extract current normal-mode keyword table | `scan.rs::keyword_or_ident()` |
| 2 | Map parser ownership for contextual candidates | `parser/stmt.rs`, `parser/decl.rs`, `parser/expr.rs`, `parser/mod.rs` |
| 3 | Record recovery boundaries | `parser/mod.rs::is_recovery_boundary()` |
| 4 | Write inventory and identifier-safety matrix | `ledger.md` |

## Checkpoint And Gate

Checkpoint:

- no compiler behavior changed
- inventory names `cura arena/page` as the first migration family
- every planned migration has owner, parser call site, and recovery note

Verification:

```bash
git diff -- docs/factory/contextual-keyword-scope/ledger.md
```

No cargo command is required for this phase alone because it is documentation-only.

## Completion Notes

Phase 0 is complete when the ledger exists and can be used as Phase 1 metadata input.

