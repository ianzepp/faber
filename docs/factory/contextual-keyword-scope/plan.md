# Contextual Keyword Scope Factory Plan

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/contextual-keyword-scope/`
**Mode**: staged language-surface refactor
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber has too many words reserved in the global lexer keyword table. Some words are true language-wide syntax (`si`, `redde`, `functio`, `nihil`). Others are only meaningful inside a parent construct (`arena` under `cura`, `argumenta` under `incipit`, function modifiers after a function signature, member modifiers inside `genus`). Treating all of them as global `TokenKind` keywords creates avoidable pressure:

- user identifiers collide with words that are only meaningful in narrow grammar positions,
- parser recovery and diagnostics need to know about a larger statement restart surface,
- docs and explain coverage overstate which words are globally reserved,
- each new feature is tempted to add another global keyword instead of owning its local grammar,
- contextual exceptions are currently added by one-off parser allowlists.

The goal is not to remove Latin vocabulary. The goal is to stop pretending every vocabulary word is globally reserved.

## Current Reality

There is one reliable contextual framework today:

- `LexerMode::Annotation` after `@` turns words into identifiers for the rest of the line.
- `LexerMode::Section` after `§` turns words into identifiers for the rest of the line.

Outside those lexer modes, contextualization is ad hoc:

- `parse_member_ident()` allows a hard-coded subset of keyword tokens as member names.
- `parse_annotation_name()` has its own hard-coded keyword exceptions.
- annotation modifier parsing uses string checks such as `eat_annotation_ident("vel")`.
- `cura page` is already contextual by accident: `arena` is a real global keyword, while `page` is matched as an identifier string.
- `parse_ident()` has a small special case for `tag`.

This plan turns that ad hoc behavior into an explicit keyword-scope model and then migrates low-risk keyword families out of the global keyword table.

## Non-Goals

This plan does not cover test-related keywords. Test syntax is being reworked separately.

Out of scope for this plan:

- `proba`
- `probandum`
- `praepara`
- `praeparabit`
- `postpara`
- `postparabit`
- `omitte`
- `futurum`
- `solum`
- `tag`
- `temporis`
- `metior`
- `repete`
- `fragilis`
- `requirit`
- `solum_in`

Do not spend implementation energy normalizing or contextualizing those words in this plan except where mechanical compiler changes require keeping tests green.

## Design Principle

The lexer should globally reserve only words that are meaningful as general syntax starts, expression operators, literals, or broadly valid statement forms.

Context-only words should lex as identifiers in normal mode and be consumed by parser helpers at the grammar sites that own them.

In other words:

```text
global keyword:        si, redde, functio, nihil
contextual keyword:    arena after cura
contextual modifier:   curata after function signature
ordinary identifier:   arena everywhere else
```

## Proposed Vocabulary Model

Introduce a keyword metadata registry before changing lexer behavior.

Suggested names:

```rust
pub enum KeywordScope {
    Global,
    Contextual(&'static [KeywordContext]),
    Annotation,
    Section,
}

pub enum KeywordContext {
    InConstruct(&'static str),
    After(&'static str),
    InPosition(&'static str),
}

pub struct KeywordSpec {
    pub text: &'static str,
    pub kind: Option<TokenKind>,
    pub scope: KeywordScope,
    pub category: KeywordCategory,
}
```

`KeywordScope`, not `TokenScope`, is the better first name because the immediate issue is reserved words, not every token kind. Punctuation, literals, EOF, and identifiers should not need fake scope metadata.

The registry should support:

- inventory and audit output,
- docs and `faber explain` coverage checks,
- parser helpers for contextual matching,
- negative tests proving contextual words remain legal identifiers outside their owning construct.

The registry does not need to drive lexing in Phase 1.

## Parser Helper Target

Add shared helpers instead of local string checks:

```rust
eat_contextual("arena")
expect_contextual("arena", "expected curator kind")
eat_contextual_one_of(&["arena", "page"])
parse_contextual_ident(&["cape", "inter", "nota"])
```

The helpers should work when the current token is either:

- `TokenKind::Ident(sym)` with matching text, or
- an old global `TokenKind` during migration, so phases can be sliced safely.

This compatibility shape lets one keyword family migrate at a time without requiring a flag day.

## Keyword Classification Draft

### Keep Global

These words currently earn global reservation:

- declarations: `fixum`, `varia`, `functio`, `typus`, `genus`, `pactum`, `ordo`, `discretio`, `importa`
- control flow: `si`, `sin`, `secus`, `ergo`, `dum`, `itera`, `elige`, `casu`, `ceterum`, `discerne`, `custodi`, `fac`
- transfer/error flow: `redde`, `rumpe`, `perge`, `tacet`, `tempta`, `cape`, `demum`, `iace`, `mori`, `adfirma`
- literals/operators: `verum`, `falsum`, `nihil`, `et`, `aut`, `non`, `vel`, `est`
- core object/type references: `ego`, `finge`
- expression forms that are intentionally syntax: `cede`, `clausura`

This list should still be audited during Phase 0, but it is not the first target.

### First Contextual Candidates

Low-risk first family:

- `arena` under `cura`
- `page` should be handled by the same helper even though it currently is not a global keyword

Next candidates:

- `argumenta` under `incipit`, `incipiet`, and function modifier positions
- `exitus` under entry/function modifier positions
- `curata`, `errata`, `optiones`, `immutata`, `iacit` after function signatures
- `generis`, `nexum` inside `genus` members
- `sub`, `implet`, `abstractus` around `genus` declarations
- `ceteri` in parameter lists, destructuring rest patterns, import rest, and extraction rest
- `sparge` in argument and literal spread positions

Potential cleanup candidates after context support exists:

- `scribe` as a legacy diagnostic alias
- `vide` and `mone` if diagnostic levels move to stdlib/HAL
- `lege` and `lineam` if input moves to stdlib/HAL
- `scriptum` if formatting becomes stdlib or string interpolation syntax
- `sed` if regex literals become a typed construction form
- `ab`, `ubi`, `prima`, `ultima`, `summa` if the collection DSL is demoted to stdlib methods
- `ante`, `usque`, `intra`, `inter` if glyph range/membership forms become canonical
- `qua`, `innatum`, `novum` if `⇢` becomes the only compile-time cast spelling
- `nulla`, `nonnulla`, `nonnihil`, `negativum`, `positivum` if convenience predicates are cut

Those are not all Phase 1 work. They are the backlog unlocked by a real contextual mechanism.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline inventory | Classify active non-test keywords by scope and parser ownership. | Ledger lists each keyword as global, contextual, alias, candidate cut, or test-owned. |
| 1 | Metadata registry | Add `KeywordSpec` / `KeywordScope` metadata without behavior changes. | Registry agrees with current lexer table; validation test prevents unclassified non-test keywords. |
| 2 | Contextual parser helpers | Add shared helpers for matching identifier-backed contextual words. | Existing parser behavior unchanged; helper tests cover ident and old-token migration forms. |
| 3 | First migration: `cura` kind | Move `arena` out of global keyword handling and parse it contextually with `page`. | `cura arena ...` and `cura page ...` still parse; `fixum arena ← 1` is legal. |
| 4 | Function and entry modifiers | Contextualize `argumenta`, `exitus`, `curata`, `errata`, `optiones`, `immutata`, `iacit`. | Modifier syntax still parses; those words become legal identifiers outside modifier positions. |
| 5 | Genus/member modifiers | Contextualize `generis`, `nexum`, `sub`, `implet`, and possibly `abstractus`. | Genus syntax still parses; those words become legal identifiers outside genus contexts. |
| 6 | Rest/spread/context operators | Contextualize `ceteri` and `sparge` where feasible. | Rest/spread syntax still parses; non-context usage is identifier-safe. |
| 7 | Alias and low-value cuts | Decide whether to delete or demote low-value convenience keywords. | Removed aliases have explain legacy entries or migration notes; canonical docs are clean. |
| 8 | Guardrails | Add tests/searches/docs checks preventing accidental global keyword additions. | New global keywords require explicit scope metadata and reviewer-visible justification. |

## Phase Details

### Phase 0: Baseline Inventory

Steps:

- Inspect `crates/radix/src/lexer/scan.rs` and `crates/radix/src/lexer/token.rs`.
- Exclude test-related keywords from this plan.
- For every remaining keyword, record:
  - source text,
  - `TokenKind`,
  - current parser owner,
  - whether it can start a statement,
  - whether it can appear as an expression operator/form,
  - whether it is only meaningful under a parent construct,
  - whether it has glyph or other aliases,
  - whether examples/docs teach it as canonical.
- Capture existing ad hoc contextual sites:
  - `LexerMode::Annotation`,
  - `LexerMode::Section`,
  - `parse_member_ident`,
  - `parse_annotation_name`,
  - annotation modifier string checks,
  - `cura page` identifier check.

Deliverable:

- `docs/factory/contextual-keyword-scope/ledger.md` with a keyword inventory table.

Checkpoint:

- No compiler behavior changed.
- Inventory names the first migration family.

### Phase 1: Metadata Registry

Steps:

- Add a central registry near the lexer or a new language metadata module.
- Start with non-test keywords only.
- Include enough category and scope metadata to answer:
  - is this globally reserved?
  - if contextual, which grammar construct owns it?
  - if legacy/alias, what is the canonical spelling?
- Add tests that every non-test keyword in `keyword_or_ident` has a registry entry.
- Do not change `keyword_or_ident` yet.

Checkpoint:

- `cargo test -p radix keyword` or equivalent focused test passes.
- `./scripta/test` passes.
- No user-visible syntax change.

### Phase 2: Contextual Parser Helpers

Steps:

- Add parser helpers for contextual matching.
- Make helpers accept both `Ident("word")` and current legacy `TokenKind::Word` forms during migration.
- Replace local string checks where obvious, starting with `cura page`.
- Do not remove any keyword from the lexer table yet.

Checkpoint:

- Parser code has a single obvious path for contextual words.
- Existing behavior is unchanged.
- Tests cover:
  - successful contextual match from identifier,
  - successful contextual match from existing keyword token,
  - failure with a useful diagnostic.

### Phase 3: First Migration: `cura`

Steps:

- Remove `"arena" => TokenKind::Arena` from normal lexer keyword mapping.
- Keep `TokenKind::Arena` only if needed temporarily for compatibility tests; otherwise delete it.
- Parse `cura arena` and `cura page` via contextual helper.
- Add negative/identifier tests:
  - `fixum arena ← 1` parses as a variable declaration,
  - `fixum page ← 1` continues to parse,
  - `cura bogus {}` produces a curator-kind diagnostic.
- Update docs and explain entries to say `arena` is contextual under `cura`.

Checkpoint:

- `cargo test -p radix cura`
- `./scripta/test`
- No stale docs claim `arena` is globally reserved.

### Phase 4: Function and Entry Modifiers

Steps:

- Migrate one family at a time:
  - `argumenta`
  - `exitus`
  - `curata`
  - `errata`
  - `optiones`
  - `immutata`
  - `iacit`
- Parser ownership:
  - `incipit` / `incipiet` own entry modifiers,
  - function signature parser owns function modifiers.
- Add identifier-safety tests for each migrated word.
- Keep diagnostics precise: when a modifier is misspelled in modifier position, report modifier syntax, not generic expression parse failure.

Checkpoint:

- Modifier examples still parse.
- Migrated words can be used as ordinary identifiers outside modifier positions.
- Full test suite passes after each small family.

### Phase 5: Genus and Member Modifiers

Steps:

- Migrate:
  - `generis`
  - `nexum`
  - `sub`
  - `implet`
  - possibly `abstractus`
- Keep `genus` and `pactum` global.
- Parser ownership:
  - `genus` declaration owns inheritance and implementation words,
  - genus member parser owns member modifiers.
- Add tests that ordinary functions/locals can use migrated words outside genus contexts.

Checkpoint:

- Genus examples and interface tests pass.
- Ordinary identifiers no longer collide with migrated words.

### Phase 6: Rest and Spread Contexts

Steps:

- Evaluate whether `ceteri` and `sparge` are safe to contextualize.
- `ceteri` appears in several grammar positions:
  - function rest parameters,
  - destructuring rest fields,
  - import rest,
  - extraction rest.
- `sparge` appears in:
  - call arguments,
  - array literals,
  - object literals.
- Migrate only if parser helper coverage can keep diagnostics clear.

Checkpoint:

- Rest/spread examples parse.
- `fixum ceteri ← 1` and `fixum sparge ← 1` are legal if migrated.

### Phase 7: Alias and Convenience Keyword Decisions

This phase is design work plus implementation slices. Candidates:

- Delete `scribe` or make it an explain legacy redirect to `nota`.
- Collapse `qua` / `innatum` / `novum` aliases if `⇢` is canonical.
- Cut unary convenience predicates:
  - `nulla`
  - `nonnulla`
  - `nonnihil`
  - `negativum`
  - `positivum`
- Move `lege` / `lineam` to HAL if input should not be syntax.
- Move `vide` / `mone` to HAL/std diagnostics if only `nota` should remain syntax.
- Decide whether `scriptum`, `sed`, and the collection DSL words should remain language syntax.

Checkpoint:

- Each deletion has:
  - a concrete replacement,
  - migration docs,
  - negative tests,
  - explain legacy/canonical updates if applicable.

### Phase 8: Guardrails

Steps:

- Add a test that fails when a new non-test keyword is added without a `KeywordSpec`.
- Add a test or script that reports `KeywordScope::Global` additions in a high-signal way.
- Update contributor docs:
  - new words should default to contextual,
  - global reservation requires justification,
  - aliases require an explicit clean-break/migration plan.
- Keep `faber explain` aligned with keyword scope:
  - global keyword,
  - contextual keyword,
  - legacy/removed keyword,
  - stdlib concept.

Checkpoint:

- `./scripta/test`
- `./scripta/lint`
- keyword-scope audit passes.

## Validation Matrix

Every behavior-changing phase should run:

```bash
cargo check -p radix
cargo test -p radix <focused-filter>
./scripta/test
```

When examples or codegen are touched, also run:

```bash
cargo test -p radix exempla_faber_roundtrip_e2e -- --ignored
```

When docs or explain entries are touched, ensure:

```bash
./scripta/test
rg -n "<removed-keyword>" . -g '!target' -g '!Cargo.lock'
```

## Risks

- Parser recovery can get worse if contextual words become plain identifiers without replacement recovery boundaries.
- Diagnostics can degrade from “expected curator kind” to generic expression errors.
- Syntax highlighters and explain entries may lag behind the compiler.
- Moving too many words in one phase will make failures hard to attribute.
- Contextual words in type positions may collide with type names in surprising ways if helper boundaries are loose.

## Implementation Rule

Do not migrate broad families first.

Start with `cura arena/page` because it already exposes the problem cleanly:

- `arena` is global,
- `page` is contextual,
- both are curator kinds,
- neither should be globally reserved.

If that migration is not clean, the framework is not ready for function modifiers or member modifiers.

