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
- `cura "page"` is already contextual by accident: `arena` is a real global keyword, while `page` is matched as an identifier string.
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
    Contextual(&'static [KeywordOwner]),
    Annotation,
    Section,
    TestOwned,
    Alias { canonical: &'static str },
}

pub enum KeywordOwner {
    CuraKind,
    EntryModifier,
    FunctionModifier,
    GenusHeader,
    GenusMember,
    RestPattern,
    SpreadExpression,
    AnnotationName,
    AnnotationModifier,
    MemberIdentifier,
}

pub struct KeywordSpec {
    pub text: &'static str,
    pub token_kind: Option<TokenKind>,
    pub scope: KeywordScope,
    pub category: KeywordCategory,
}
```

`KeywordScope`, not `TokenScope`, is the better first name because the immediate issue is reserved words, not every token kind. Punctuation, literals, EOF, and identifiers should not need fake scope metadata.

`KeywordOwner` should be a closed enum, not descriptive strings. A string such as `"after function signature"` documents intent but does not stop a helper from being called in the wrong grammar position. Closed owners make tests and audits precise.

The registry should support:

- inventory and audit output,
- docs and `faber explain` coverage checks,
- parser helpers for contextual matching,
- negative tests proving contextual words remain legal identifiers outside their owning construct.

The registry must cover both:

- words currently returned by `keyword_or_ident()`,
- contextual vocabulary that already exists without a global keyword token, such as `page` under `cura`.

Validation must be text-first, not token-first, because several spellings can map to the same `TokenKind` today (`qua`, `innatum`, and `novum` all map to `TokenKind::Verte`).

`token_kind` is a neutral current-state bridge: the `TokenKind` emitted for this word in normal lexer mode today, if any. It is `None` for contextual words that already lex as identifiers, such as `page`.

The registry does not need to drive lexing in Phase 1.

## Parser Helper Target

Add shared helpers instead of local string checks:

```rust
eat_contextual(KeywordOwner::CuraKind, "arena")
expect_contextual(KeywordOwner::CuraKind, "arena", "expected curator kind")
eat_contextual_one_of(KeywordOwner::CuraKind, &["arena", "page"])
parse_contextual_ident(KeywordOwner::MemberIdentifier, &["cape", "inter", "nota"])
```

Use two related but separate helper families:

- contextual grammar helpers consume grammar words such as `arena` in `cura "arena"`,
- contextual identifier helpers parse identifier positions that intentionally allow old reserved words, such as member names.

Those jobs should not collapse into one helper. One consumes a local grammar terminal; the other returns an `Ident`.

The helper contract should be explicit:

- `eat_contextual(...) -> Option<Span>` advances only on match and returns the matched span,
- `expect_contextual(...) -> Result<Span, ParseError>` emits the owning context's diagnostic on failure,
- `eat_contextual_one_of(...) -> Option<(&'static str, Span)>` advances only on match,
- `parse_contextual_ident(...) -> Result<Ident, ParseError>` returns an interned identifier and preserves the source span,
- all helpers must accept only words registered for the requested `KeywordOwner`.

During migration, helpers should work when the current token is either:

- `TokenKind::Ident(sym)` with matching text, or
- an old global `TokenKind` during migration, so phases can be sliced safely.

This compatibility shape lets one keyword family migrate at a time without requiring a flag day.

Compatibility `TokenKind` acceptance is temporary per migrated family. Once a word leaves `keyword_or_ident()`, follow-up cleanup should either remove the unused token variant or prove another grammar site still owns it.

Contextual recovery also needs an executable rule. If a migrated word previously participated in parser restart boundaries, the owning parser code must recover on the parent construct rather than relying on a global token kind.

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

Potential cleanup backlog after context support exists:

- `scribe` as a diagnostic alias redirect
- `vide` and `mone` if diagnostic levels move to stdlib/HAL
- `lege` and `lineam` if input moves to stdlib/HAL
- `scriptum` if formatting becomes stdlib or string interpolation syntax
- `sed` if regex literals become a typed construction form
- `ab`, `ubi`, `prima`, `ultima`, `summa` if the collection DSL is demoted to stdlib methods
- `ante`, `usque`, `intra`, `inter` if glyph range/membership forms become canonical
- `qua`, `innatum`, `novum` if `⇢` becomes the only compile-time cast spelling
- `nulla`, `nonnulla`, `nonnihil`, `negativum`, `positivum` if convenience predicates are cut

Those are not part of this implementation factory. They are a follow-up language-design backlog unlocked by a real contextual mechanism.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline inventory | Classify active non-test keywords by scope and parser ownership. | Ledger lists each keyword as global, contextual, alias, backlog candidate, or test-owned. |
| 1 | Metadata registry | Add `KeywordSpec` / `KeywordScope` metadata without behavior changes. | Registry agrees with current lexer table and contextual vocabulary; validation prevents unclassified active spellings. |
| 2 | Contextual parser helpers | Add shared grammar-word and contextual-identifier helpers. | Existing parser behavior unchanged; helper tests cover ident and old-token migration forms. |
| 3 | First migration: `cura` kind | Move `arena` out of global keyword handling and parse it contextually with `page`. | `cura "arena" ...` and `cura "page" ...` still parse; `fixum _ arena ← 1` is legal. |
| 4 | Function and entry modifiers | Contextualize `argumenta`, `exitus`, `curata`, `errata`, `optiones`, `immutata`, `iacit`. | Modifier syntax still parses; those words become legal identifiers outside modifier positions. |
| 5 | Genus/member modifiers | Contextualize `generis`, `nexum`, `sub`, `implet`, and possibly `abstractus`. | Genus syntax still parses; those words become legal identifiers outside genus contexts. |
| 6 | Rest/spread/context operators | Contextualize `ceteri` and `sparge` where feasible. | Rest/spread syntax still parses; non-context usage is identifier-safe. |
| 7 | Guardrails | Add tests/searches/docs checks preventing accidental global keyword additions. | New global keywords require explicit scope metadata and reviewer-visible justification. |

## Phase Details

### Phase 0: Baseline Inventory

Steps:

- Inspect `crates/radix/src/lexer/scan.rs` and `crates/radix/src/lexer/token.rs`.
- Exclude test-related keywords from this plan.
- For every remaining keyword, record:
  - source text,
  - `TokenKind`,
  - `KeywordOwner` if contextual,
  - current parser owner,
  - whether it can start a statement,
  - whether it is currently a parser recovery boundary,
  - whether it can appear as an expression operator/form,
  - whether it is only meaningful under a parent construct,
  - whether it has glyph or other aliases,
  - whether examples/docs teach it as canonical.
- Record contextual vocabulary that is not currently in `keyword_or_ident()`, starting with `page`.
- Capture existing ad hoc contextual sites:
  - `LexerMode::Annotation`,
  - `LexerMode::Section`,
  - `parse_member_ident`,
  - `parse_annotation_name`,
  - annotation modifier string checks,
  - `cura "page"` identifier check.

Deliverable:

- `docs/factory/contextual-keyword-scope/ledger.md` with a keyword inventory table.
- A short identifier-safety matrix naming which syntactic positions each migrated word must remain legal in outside its owner context.

Checkpoint:

- No compiler behavior changed.
- Inventory names the first migration family.
- Every planned migration has an explicit owner, parser call site, and recovery note.

### Phase 1: Metadata Registry

Steps:

- Add a central registry near the lexer or a new language metadata module.
- Start with active language keywords and known contextual vocabulary. Test-owned words can be marked `TestOwned` but should not drive this migration.
- Include enough category and scope metadata to answer:
  - is this globally reserved?
  - if contextual, which grammar construct owns it?
  - if alias/redirect, what is the canonical spelling?
- Add tests that every active spelling in `keyword_or_ident()` has a registry entry.
- Add tests that contextual non-lexer vocabulary such as `page` has a registry entry.
- Add tests that alias spellings are represented by text, not collapsed by shared `TokenKind`.
- Do not change `keyword_or_ident` yet.
- Do not let the registry become a passive duplicate forever: Phase 2 helpers or Phase 7 guardrails must consume it.

Checkpoint:

- `cargo test -p radix keyword` or equivalent focused test passes.
- `./scripta/test` passes.
- No user-visible syntax change.
- Registry and lexer agree on current active spellings, including spellings that share one token kind.

### Phase 2: Contextual Parser Helpers

Steps:

- Add parser helpers for contextual grammar matching.
- Add separate parser helpers for identifier positions that allow registered contextual words or current keyword-token words.
- Make helpers accept both `Ident("word")` and current `TokenKind::Word` forms during migration.
- Replace local string checks where obvious, starting with `cura "page"`.
- Do not remove any keyword from the lexer table yet.
- Wire helper validation to `KeywordOwner` so a word registered for one owner cannot be silently accepted in another owner.
- Add or preserve recovery behavior at the owning parser site before removing any globally reserved token kind.

Checkpoint:

- Parser code has one obvious path for contextual grammar words and one obvious path for contextual identifier exceptions.
- Existing behavior is unchanged.
- Tests cover:
  - successful contextual match from identifier,
  - successful contextual match from existing keyword token,
  - failure with a useful diagnostic.
  - rejection when the same spelling is requested under the wrong `KeywordOwner`.

### Phase 3: First Migration: `cura`

Steps:

- Remove `"arena" => TokenKind::Arena` from normal lexer keyword mapping.
- Keep `TokenKind::Arena` only if needed temporarily for compatibility tests; otherwise delete it.
- Parse `cura "arena"` and `cura "page"` via contextual helper.
- Add negative/identifier tests:
  - `fixum _ arena ← 1` parses as a variable declaration,
  - `fixum _ page ← 1` continues to parse,
  - `cura "page" source fixum textus page {}` still distinguishes the `page` kind word from later type/binding identifiers,
  - `cura bogus {}` produces a curator-kind diagnostic.
- Update docs and explain entries to say `arena` is contextual under `cura`.
- Document the ambiguity rule: in the immediate `cura` kind slot, `arena` and `page` are consumed as kind words. Outside that slot they are ordinary identifiers.

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
- Treat `abstractus` as a harder case, not a default migration. If migrated, statement dispatch must explicitly parse `abstractus genus` by lookahead and preserve diagnostics for misspelled class declarations.
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
- `fixum _ ceteri ← 1` and `fixum _ sparge ← 1` are legal if migrated.

### Phase 7: Guardrails

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
  - removed alias,
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

Start with `cura "arena"/page` because it already exposes the problem cleanly:

- `arena` is global,
- `page` is contextual,
- both are curator kinds,
- neither should be globally reserved.

If that migration is not clean, the framework is not ready for function modifiers or member modifiers.
