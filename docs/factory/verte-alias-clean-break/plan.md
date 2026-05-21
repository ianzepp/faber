# Verte Alias Clean-Break Plan

**Status**: completed
**Completed**: 2026-05-21 (phases 0-4 + validation in one session)
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/verte-alias-clean-break/`
**Mode**: clean-break / prerequisite planning
**Relationship**: prerequisite to `docs/factory/contextual-keyword-scope/plan.md`

## Interpreted Problem

Faber currently accepts four spellings for the same postfix type operation:

```fab
value ⇢ Target
value qua Target
value innatum Target
value novum Target
```

The compiler already treats these as one semantic operator. In normal lexer mode, `qua`, `innatum`, and `novum` all map to `TokenKind::Verte`, and Faber codegen emits the canonical glyph form `⇢`.

That behavior is acceptable if `⇢` is the intended super-operator that dispatches by source expression and resolved target type. What is not acceptable is continuing to expose three Latin expression aliases that imply separate semantics the compiler does not preserve.

The clean break is:

- make `⇢` the only accepted postfix type conversion/construction spelling,
- remove normal-mode expression aliases `qua`, `innatum`, and `novum`,
- keep `@ innatum` annotation metadata unless separately renamed,
- update docs, examples, tests, and explain entries so they do not teach alias syntax as valid source,
- make `qua`, `innatum`, and `novum` ordinary identifiers outside annotation contexts once no longer reserved.

## Break Boundary

Authoritative contract: Faber source syntax uses `⇢` for postfix type conversion and construction.

In scope:

- normal lexer keyword mapping for `qua`, `innatum`, and `novum`,
- parser comments and grammar productions for postfix `Verte`,
- Faber examples and Rust test snippets using postfix aliases,
- grammar docs that teach `qua`, `innatum`, or `novum` as expression syntax,
- explain entries and redirects for removed expression aliases,
- negative tests proving alias forms are rejected as postfix syntax.

Out of scope:

- `@ innatum` annotation metadata,
- `§ innatum` section names,
- directory names such as `stdlib/norma/innatum`,
- ordinary identifiers named `novum`, such as function parameters,
- generated target-language code,
- broader contextual keyword migration.

## Current Evidence

- `crates/radix/src/lexer/scan.rs` maps `qua`, `innatum`, and `novum` to `TokenKind::Verte`.
- `crates/radix/src/parser/expr.rs` documents `cast := ('⇢' | 'qua' | 'innatum' | 'novum') type`.
- `crates/radix/src/hir/nodes.rs` describes `Verte` as a unified conversion/construction expression.
- `crates/radix/src/codegen/faber/expr.rs` emits `⇢` for the unified HIR node.
- `EBNF.md`, `docs/grammatica/typi.md`, and `docs/grammatica/structurae.md` still teach the Latin alias forms.
- `stdlib/norma/*.fab` and annotation examples use `@ innatum`, which should not be removed by this plan.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Inventory | Separate postfix alias uses from annotation/prose/identifier uses. | Ledger classifies every `qua`, `innatum`, and `novum` match. |
| 1 | Front-end break | Remove normal-mode alias tokenization for postfix `Verte`. | `⇢` still parses; `qua` / `innatum` / `novum` no longer parse as postfix operators. |
| 2 | Tests and examples | Rewrite valid source snippets to `⇢` and add rejection tests. | Focused parser/driver/codegen tests pass. |
| 3 | Docs and explain | Update grammar/docs and explain legacy entries. | Docs teach only `⇢` for expression syntax and preserve `@ innatum` metadata wording. |
| 4 | Guardrail | Prevent accidental reintroduction of expression aliases. | Search/test gate distinguishes forbidden postfix aliases from allowed annotation uses. |

## Phase Notes

### Phase 0: Inventory

Search separately in:

- `crates/radix/src/**/*`,
- `examples/**/*.fab`,
- `stdlib/**/*.fab`,
- `docs/**/*.md`,
- `EBNF.md`.

Classify matches as:

- postfix expression alias to replace,
- annotation metadata to keep,
- section/path/prose reference to clarify,
- ordinary identifier to leave alone,
- test expecting old alias acceptance to invert or delete.

### Phase 1: Front-End Break

Remove these normal-mode keyword branches:

```rust
"qua" => TokenKind::Verte,
"innatum" => TokenKind::Verte,
"novum" => TokenKind::Verte,
```

Keep glyph tokenization:

```rust
'⇢' => TokenKind::Verte
```

After this phase, `qua`, `innatum`, and `novum` should lex as identifiers in normal mode. Do not remove `TokenKind::Verte`; it remains the internal token for `⇢`.

### Phase 2: Tests and Examples

Update snippets such as:

```fab
fixum asText ← data qua textus
fixum items ← [] innatum lista<textus>
fixum user ← { nomen: "Julia" } novum Persona
```

to:

```fab
fixum asText ← data ⇢ textus
fixum items ← [] ⇢ lista<textus>
fixum user ← { nomen: "Julia" } ⇢ Persona
```

Add negative tests that the old postfix forms fail with clear parser diagnostics.

### Phase 3: Docs and Explain

Update:

- `EBNF.md`,
- `docs/grammatica/typi.md`,
- `docs/grammatica/structurae.md`,
- `docs/grammatica/verba.md`,
- explain coverage docs or corpus entries that list active expression keywords.

Docs should say:

- `⇢` is the only expression spelling,
- `@ innatum` is annotation metadata and remains valid,
- removed expression aliases may appear in legacy material but are not valid Faber source.

### Phase 4: Guardrail

Add a focused test or scriptable search that catches reintroduced postfix alias syntax in examples and grammar docs while allowing:

- `@ innatum`,
- `§ innatum`,
- file paths containing `innatum`,
- ordinary identifiers where the parser now permits them.

## Validation

Run after implementation phases:

```bash
cargo test -p radix verte
cargo test -p radix parser
cargo test -p radix driver
./scripta/test
```

Run docs/source residue searches with exclusions for annotation metadata and paths:

```bash
rg -n "\\b(qua|innatum|novum)\\b" EBNF.md docs examples stdlib crates/radix/src -g '!target'
```

Each remaining match must be intentionally classified.
