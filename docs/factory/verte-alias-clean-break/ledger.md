# Verte Alias Clean-Break - Factory Ledger

**Phase Set Source**: `docs/factory/verte-alias-clean-break/plan.md`
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Delivery Spec Directory**: `docs/factory/verte-alias-clean-break/`
**Commit Policy**: Autocommit after each completed phase + validation per AGENTS.md
**Created**: 2026-05-21
**Mode**: clean-break / prerequisite to contextual-keyword-scope

## Current Phase
4 - Guardrail (search + test enforcement)

## Completed Phases
0 - Inventory (ledger + full classified inventory of postfix vs annotation uses)
1 - Front-end break (removed mappings + comments + test)
2 - Tests/examples (driver/mod_test.rs sources rewritten to ⇢ + negative rejection test added)
3 - Docs (EBNF.md, AGENTS.md, grammatica/{verba,structurae,typi}.md updated; prose/examples use only ⇢; @ innatum and paths preserved)

## Baseline (Phase 0 Start)

### Git Status
```
On branch main
Your branch is ahead of 'origin/main' by 39 commits.

nothing to commit, working tree clean
```
(From initial session git_status; subsequent `git log --oneline -1` showed "269c27ae Plan verte alias clean break")

### Pre-Break Acceptance Confirmed
- `crates/radix/src/lexer/scan.rs:724-726`: "qua", "innatum", "novum" all map to `TokenKind::Verte` (same as '⇢')
- `crates/radix/src/parser/expr.rs:636`: `check_keyword(TokenKind::Verte)` accepts any of them for cast/VerteExpr
- `crates/radix/src/driver/mod_test.rs`: ~15 embedded source snippets use old postfix forms and pass today
- `lexer/scan_test.rs:194`: dedicated test `lexes_verte_keyword_aliases`
- Docs and EBNF teach the aliases as valid expression syntax

`cargo test -p radix lexes_verte_keyword_aliases` : would PASS pre-break.

### Full Classified Inventory of `\b(qua|innatum|novum)\b` Matches

#### MUST CHANGE — Postfix Expression Alias Uses (Faber source syntax in tests)
- **Primary**: `crates/radix/src/driver/mod_test.rs` (lines ~639,640,825,879,1148,1193,1505,1536,1856,1887,1889,1890,1911,1912,2089 and surrounding):
  - Multiple `data qua textus`, `{} novum Counter`, `[] innatum lista<...>`, `{...} novum Type`, `finge ... qua Event`, `argv() qua lista`, `n qua fractus`
  - All embedded in `r#" ... "#` test program sources passed to driver/session.
  - Classification: **Rewrite to canonical `⇢` form** in Phase 2. Add negative tests proving old forms now rejected at parse with clear diagnostics.
- No other Rust test files embed old postfix source (parser/mod_test, etc. use canonical or avoid).

#### MUST CHANGE — Source Code Comments & Grammar Docs (implementation)
- `crates/radix/src/lexer/scan.rs:723-726`: keyword mappings + comment "Type operations (qua/innatum/novum are keyword aliases for ⇢)"
- `crates/radix/src/lexer/token.rs:204`: `Verte, // ⇢ / qua / innatum / novum — unified...`
- `crates/radix/src/parser/expr.rs:448,558,637`: EDGE comment, cast grammar, unified comment
- `crates/radix/src/syntax/ast.rs:701,906`: VerteExpr doc and FingeExpr cast comment
- `crates/radix/src/hir/nodes.rs:412`: "Subsumes qua (cast), innatum..., novum..."
- `crates/radix/src/hir/lower/expr.rs:299`: "Lower unified... (⇢ / qua / innatum / novum)"
- `crates/radix/src/lexer/scan_test.rs:195`: test name and loop over aliases
- Classification: **Remove mappings** (Phase 1), **update comments** to say "⇢ (Verte)" only, **repurpose test** to verify aliases now lex as Ident and postfix ⇢ still works.

#### MUST UPDATE — Grammar / User-Facing Documentation
- `EBNF.md`:
  - 355: `cast := ... ('⇢' | 'qua' | 'innatum' | 'novum') ...`
  - 367: "The keywords `qua`, `innatum`, and `novum` are permanent aliases for `⇢`."
  - 401-402: comment and fingeExpr `('⇢' | 'qua')`
  - 603: table row for Type Cast lists all four
  - Annotation `innatumAnnotation` and `§ innatum` references: **KEEP** (metadata)
- `docs/grammatica/typi.md`:
  - 105: "cast it explicitly (`qua`)"
  - 113: `iace novum Error { ... }` (prefix form, likely never valid postfix)
  - 288-295: `innatum` for empty collections, explanatory prose
  - 394: `novum capsa<...> { ... }` (prefix construction)
  - 450-469: entire "Type Casting" section using `qua` in prose + 6+ examples
  - Classification: **Rewrite all syntax examples to `⇢`**, update prose to present `⇢` as the (only) spelling; repair prefix examples to valid current syntax (e.g. `iace Error { ... }` or `{...} ⇢ Error`).
- `docs/grammatica/structurae.md`:
  - ~10+ examples and entire subsection "Creating Instances with novum" (240-358): titles, prose, code blocks using `novum`
  - 175,329 etc. scattered postfix `novum`
  - @ innatum section (358+): **KEEP**
  - Classification: **Mass rewrite of construction section** to "Postfix construction with ⇢", examples use `⇢`.
- `docs/grammatica/verba.md`:
  - 131: `novum` in Functions/Instantiation table
  - 281: `qua` in Prepositions table "as (type)"
  - Classification: **Remove or mark as historical**; point to `⇢` operator instead.
- `AGENTS.md:61`: "Empty collections need explicit types: `[] innatum lista<T>`, ... " (in critical rules)
  - Classification: **Update to `[] ⇢ lista<T>`** (canonical per grammar).
- `docs/faber-language-critique.md:330,349`: mentions in historical critique
  - Classification: **Contextualize as past spellings** or leave (analysis of prior design); no active teaching.
- Other `docs/factory/*.md` (contextual-keyword-scope/plan.md, faber-explain-coverage-completion/plan.md, this plan.md): historical/planning references — **leave** (they document the decision to remove).

#### KEEP — Annotation Metadata, Sections, Paths, Ordinary Identifiers (explicitly out of scope)
- All `@ innatum ...` (examples/annotatio/*.fab, stdlib/norma/innatum/*.fab, mathesis.fab, etc.) — 14+ lines
- `§ innatum` section references in EBNF/docs
- `stdlib/norma/innatum/` directory + file names
- `examples/exempla/innatum/`, `novum/` directories (already migrated content)
- Param name `novum` in `stdlib/norma/innatum/textus.fab:128` (`muta(..., textus novum)`)
- "quaeret", "quaerent" etc. in HAL (full-word identifiers, never were the bare keyword)
- `explain/verte.md`: already canonical `⇢`, aliases=["verte","cast"] (no old Latin ones) — good

#### IGNORE — Generated / Build / Non-Source
- `target/...` (explain_entries.rs contains old strings from prior builds; excluded by `-g '!target'`)
- Cargo lock, etc.

#### Residue After Planned Changes
Post-edit `rg` (with exclusions) must only hit:
- @ innatum annotations
- § innatum / innatumAnnotation grammar
- directory names and file paths
- ordinary idents like `novum` param, `quaeret*`
- historical plan/factory docs and critique (with context)
- this ledger + updated plan

Any bare postfix `qua|innatum|novum` in examples, grammatica/*.md, EBNF cast grammar, or test sources = failure.

## Validation Targets (for later phases)
- `cargo test -p radix verte` (if exists; else parser + driver + lexer tests mentioning Verte)
- `cargo test -p radix parser`
- `cargo test -p radix driver`
- `./scripta/test`
- `rg -n "\\b(qua|innatum|novum)\\b" EBNF.md docs examples stdlib crates/radix/src -g '!target'` — every hit classified above

## Final Validation (All Phases)

- `cargo test -p radix` (verte/parser/driver/lexer/scan + new rejection test): **PASS** (all 100+ driver tests, repurposed alias lex test, negative rejection test)
- `./scripta/test`: **PASS** (full suite including 268+ radix tests + doc-tests)
- `cargo fmt --all -- --check`: **PASS** (after auto-fmt on added test)
- `cargo clippy -p radix -- -D warnings`: **PASS** (clean)
- Residue `rg ...`: **PASS** — 0 unclassified bare postfix aliases in active source/docs. All hits are:
  - @ innatum / innatumAnnotation / § innatum (KEEP)
  - stdlib/norma/innatum/ paths + param `novum` (KEEP)
  - "formerly ..." comments in exempla/innatum & novum .fab (historical note, OK)
  - our plan/ledger + test strings + removal comments (intentional)
  - historical prose in critique/other plans (contextual)
- No reintroduction of expression aliases in grammar or examples.
- `⇢` glyph path remains fully functional and is the only accepted spelling.

### Post-Implementation Git
Changes committed in slices (phase 0-1, then docs+tests, then fmt). Working tree will be clean after final autocommit.

**All phases complete. Clean break delivered.**

- Only `⇢` accepted for postfix Verte in normal lexer mode.
- `qua`, `innatum`, `novum` now ordinary identifiers (usable as params, etc.).
- @ innatum metadata and paths untouched.
- Docs, tests, grammar, and negative tests enforce the boundary.
- Prerequisite satisfied for contextual-keyword-scope work.

*Opus perfectum est.*
