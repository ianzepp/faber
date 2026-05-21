# Faber Explain Coverage Completion Factory Plan

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-explain-coverage-completion/`
**Primary Owner**: `crates/faber` build/project tool and root `explain/` corpus
**Compiler Reference**: `crates/radix/src/lexer`
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

The first `faber explain` implementation established the command, the embedded Markdown/frontmatter corpus, and a useful seed set of entries. It did not complete coverage for the language surface.

Current seed coverage includes:

- core binding/function/testing keywords such as `fixum`, `varia`, `functio`, `proba`, `probandum`, and `adfirma`,
- core conditional keywords such as `si`, `sin`, `secus`, `custodi`, and `reddit`,
- comparison and assignment glyphs such as `≡`, `≠`, `≤`, `≥`, `←`, `→`, `⊕`, `⊖`, `⊛`, and `⊘`,
- legacy redirect entries for `==`, `!=`, `<=`, `>=`, and `->`.

That is not enough for the intended use case. If `faber explain <term>` is the installed grammar interface for humans and future agents, every active Faber keyword and every meaningful glyph/operator token should have a self-contained entry.

The follow-on work is to move from seed coverage to complete audited coverage.

## Boundary

This plan owns:

- expanding the root `explain/` corpus,
- renaming corpus files to stable ASCII slugs,
- adding coverage validation so missing explain entries fail tests,
- documenting the coverage policy,
- keeping entries embedded into the `faber` binary,
- preserving redirects for legacy spellings that users or agents may ask about.

This plan does not own:

- changing language syntax,
- removing old lexer compatibility branches,
- implementing the glyph clean-break,
- changing `radix` command behavior,
- making the compiler depend on the explain corpus.

The `radix` lexer is used as the implementation inventory for active tokens. The compiled `faber explain` corpus remains the public interface; entries must stand alone and must not require installed users to have this source tree.

## Corpus File Naming

Explain lookup must be driven by frontmatter, not by the filename.

Use stable ASCII filenames derived from the compiler token or AST-style name, converted to kebab case:

| Term | Token-style name | Corpus filename |
| ---- | ---------------- | --------------- |
| `≡` | `EqEq` | `eq-eq.md` |
| `≠` | `BangEq` | `bang-eq.md` |
| `≤` | `LtEq` | `lt-eq.md` |
| `≥` | `GtEq` | `gt-eq.md` |
| `→` | `Arrow` | `arrow.md` |
| `←` | `Assign` or `LeftArrow` | `assign.md` or `left-arrow.md` |
| `⊕` | `PlusEq` | `plus-eq.md` |
| `⇢` | `Verte` | `verte.md` |
| `⇒` | `Conversio` | `conversio.md` |

The frontmatter remains the source of the user-facing term:

```markdown
---
term: "≡"
kind: "operator"
category: "comparison"
canonical: true
summary: "Compares two values for equality and returns bivalens."
syntax: "<expression> ≡ <expression>"
---
```

This avoids filesystem and tooling problems with glyph filenames while preserving exact `faber explain ≡` lookup behavior.

Keywords may keep their literal keyword names as filenames because they are already stable ASCII spellings, for example `functio.md` and `proba.md`.

Legacy redirect files should use the canonical token slug with a `.legacy.md` suffix so they sort next to the canonical entry. For example:

| Legacy term | Corpus filename |
| ----------- | --------------- |
| `==` | `eq-eq.legacy.md` |
| `!=` | `bang-eq.legacy.md` |
| `<=` | `lt-eq.legacy.md` |
| `>=` | `gt-eq.legacy.md` |
| `->` | `arrow.legacy.md` |

Coverage validation should fail if a required entry uses a non-ASCII or punctuation-heavy filename when an ASCII token slug is available.

## Human Output Format

The current plain renderer exposes the entry data model directly:

```text
functio
Kind: keyword
Category: function
Meaning: Declares a named function or method.
```

That is readable, but it is not the desired long-term human interface. The default renderer should become a compact man-page-style language reference view, while `--json` remains the stable machine interface.

Target shape for canonical entries:

```text
FUNCTIO(7)                 Faber Language Reference                 FUNCTIO(7)

NAME
    functio - declares a named function or method

SYNTAX
    functio <name>(<params>) [→ <type>] <block>

DESCRIPTION
    `functio` introduces a named callable block. Parameters are declared using
    Faber's type-first form, and the optional return type follows `→`.

EXAMPLE
    functio duplica(numerus n) → numerus {
        redde n * 2
    }

RELATED
    →, incipit

EXAMPLES
    examples/exempla/functio/functio.fab
```

Target shape for legacy entries:

```text
==(7)                    Faber Language Reference                     ==(7)

NAME
    == - legacy spelling for ≡

STATUS
    Legacy. Not canonical Faber source.

USE INSTEAD
    ≡

EXAMPLE
    adfirma left ≡ right

SEE ALSO
    ≡
```

Renderer requirements:

- Default output is human-first and structured like a small Unix reference page.
- Use ANSI styling only when stdout is a TTY.
- Honor `NO_COLOR`.
- Add a `--plain` or equivalent escape hatch if styled output is introduced.
- Keep `--json` unchanged.
- Avoid complex Markdown parsing in the first pass; extracting the first Faber code block is enough.
- Add snapshot-style tests for no-color/plain output.

## Search Behavior

Current lookup behavior is exact, not search:

```bash
faber explain functio
faber explain ≡
faber explain ==
faber explain --list
faber explain --category testing
faber explain --json proba
```

The registry resolves exact terms, aliases, and legacy spellings. It does not currently provide fuzzy search, substring search, or full-text search.

Add a follow-on search surface after coverage is complete:

```bash
faber explain --search function
faber explain --search equality
faber explain --search "return type"
```

Search should inspect:

- `term`,
- `summary`,
- `syntax`,
- `aliases`,
- `legacy`,
- `related`,
- body text.

Search output should be concise and deterministic:

```text
Search: equality

≡      operator / comparison   Compares two values for equality and returns bivalens.
≠      operator / comparison   Compares two values for inequality and returns bivalens.
est    keyword  / logic        Checks identity rather than value equality.
```

Search is separate from lookup. `faber explain <term>` should remain exact and predictable; suggestions can be offered on failed lookup, but fuzzy matching should not silently choose an entry.

## Coverage Definition

Coverage should be explicit by tier.

| Tier | Required? | Source | Examples |
| ---- | --------- | ------ | -------- |
| Active keywords | yes | `keyword_or_ident` and annotation keyword tables | `functio`, `si`, `proba`, `praepara` |
| Canonical glyph/operator tokens | yes | scanner operator branches and `TokenKind` | `≡`, `∧`, `⇢`, `‥` |
| Testing keywords | yes | lexer keyword table and test-runner docs | `proba`, `omitte`, `solum_in` |
| Legacy redirects | yes, when recognized or recently removed | compatibility tokens and clean-break plan | `==`, `->`, `+=` |
| Punctuation | selective | parser/scanner behavior | `@`, `§`, `?`, chaining forms |
| Internal token names | no | Rust enum names only | `TokenKind::EqEq` |

Punctuation should not become noisy encyclopedia filler. Add entries only when the token carries Faber-specific meaning or is likely to confuse users:

- include `@` because it introduces compiler/tool metadata,
- include `§` because it participates in `scriptum()` template formatting,
- include optional/non-null chaining forms such as `?.`, `?[`, `?(`, `!.`, `![`, and `!(` if they are active language syntax,
- exclude ordinary grouping punctuation such as `(`, `)`, `{`, `}`, `[`, `]`, `,`, `:`, `;`, and `.` unless a future phase decides users need them.

## Current Gap Snapshot

The current corpus has 30 entries:

```text
!=
->
<=
==
>=
adfirma
custodi
fixum
functio
futurum
importa
incipit
omitte
proba
probandum
reddit
secus
si
sin
varia
←
→
≠
≡
≤
≥
⊕
⊖
⊘
⊛
```

Known missing keyword families include:

- declarations and types: `genus`, `pactum`, `typus`, `ordo`, `discretio`,
- modifiers: `abstractus`, `generis`, `nexum`, `publica`, `privata`, `protecta`, `prae`, `ceteri`, `immutata`, `iacit`, `curata`, `errata`, `exitus`, `optiones`,
- control flow: `sic`, `dum`, `itera`, `elige`, `casu`, `ceterum`, `discerne`, `fac`, `ergo`,
- transfer and errors: `redde`, `rumpe`, `perge`, `tacet`, `tempta`, `cape`, `demum`, `iace`, `mori`, `moritor`,
- async and closure: `futura`, `cursor`, `cede`, `clausura`,
- boolean, null, and logic: `verum`, `falsum`, `nihil`, `et`, `aut`, `non`, `vel`, `est`,
- object and type operations: `ego`, `finge`, `sub`, `implet`, `qua`, `innatum`, `novum`, `numeratum`, `fractatum`, `textatum`, `bivalentum`,
- output and entry surfaces: `scribe`, `vide`, `mone`, `incipiet`, `argumenta`, `cura`, `arena`, `ad`,
- miscellaneous language forms: `ex`, `de`, `in`, `ut`, `pro`, `omnia`, `sparge`, `praefixum`, `scriptum`, `lege`, `lineam`, `sed`,
- ranges: `ante`, `usque`, `per`, `intra`, `inter`,
- collection DSL: `ab`, `ubi`, `prima`, `ultima`, `summa`,
- testing: `praepara`, `praeparabit`, `postpara`, `postparabit`, `solum`, `tag`, `temporis`, `metior`, `repete`, `fragilis`, `requirit`, `solum_in`,
- nullability and numeric constraints: `nulla`, `nonnulla`, `nonnihil`, `negativum`, `positivum`.

Known missing glyph/operator entries include:

- logical glyphs: `∧`, `∨`, `⊻`, `¬`,
- shift glyphs: `≪`, `≫`,
- remaining compound assignment glyphs: `⊜`, `⊚`,
- conversion glyphs: `⇢`, `⇒`,
- range glyphs: `‥`, `…`,
- metadata and formatting glyphs: `@`, `§`,
- optional and non-null chaining forms: `?.`, `?[`, `?(`, `!.`, `![`, `!(`.

The implementation session must refresh this snapshot from the live lexer before editing because token coverage may have changed by then.

## Entry Quality Bar

Each new entry should answer four questions quickly:

1. What is this term?
2. Where does it appear syntactically?
3. What does it mean at runtime or compile time?
4. What is the smallest correct example?

Entries should be short but not vague. A useful entry is usually:

- one precise summary sentence in frontmatter,
- one compact syntax form,
- one short explanation paragraph,
- one valid Faber snippet,
- related terms that help navigation.

Do not use invented syntax in examples. Check existing examples, grammar docs, and parser behavior when unsure.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Inventory refresh | Generate the current explain coverage ledger from the live lexer and corpus. | Ledger lists covered, missing, excluded, and redirect terms. |
| 1 | Filename normalization | Rename existing punctuation/glyph corpus files to ASCII token slugs. | Existing lookups still work after rename. |
| 2 | Coverage validator | Add automated validation for required explain entries and filename policy. | Tests fail when required entries are missing or misnamed. |
| 3 | Keyword corpus expansion | Add entries for all active keyword spellings. | Keyword coverage test passes. |
| 4 | Glyph/operator corpus expansion | Add entries for all canonical glyph/operator terms and useful punctuation forms. | Operator coverage test passes. |
| 5 | Redirect and alias pass | Add or normalize legacy redirects and search aliases. | Legacy lookups return canonical guidance. |
| 6 | Human renderer and search | Replace raw field output with a man-page-style renderer and add explicit search. | Plain output reads like a reference page; search returns ranked summaries. |
| 7 | Docs and UX pass | Document the coverage contract, renderer, search, and list/category behavior. | User-facing docs match behavior. |
| 8 | Full validation | Run formatting, tests, clippy, and release builds. | Repo passes validation and coverage gates. |

## Phase Details

### Phase 0: Inventory Refresh

Steps:

- Inspect `git status --short`.
- List current `explain/*.md` terms.
- Record current filenames separately from frontmatter terms.
- Extract keyword spellings from the normal keyword table and any annotation keyword table.
- Extract canonical glyph/operator terms from scanner branches and `TokenKind`.
- Classify each term as:
  - required canonical entry,
  - required legacy redirect,
  - optional punctuation entry,
  - excluded ordinary punctuation,
  - implementation-only token.
- Write `docs/factory/faber-explain-coverage-completion/inventory.md`.

Checkpoint:

- The implementation session has a concrete missing-entry checklist before adding corpus files.

### Phase 1: Filename Normalization

Steps:

- Rename existing glyph filenames to ASCII token-style slugs.
- Rename existing legacy punctuation filenames to the adjacent-sort `<token-slug>.legacy.md` pattern.
- Keep each file's `term` frontmatter unchanged.
- Prefer token names from `TokenKind` when obvious.
- Record any ambiguous slug choice in the inventory, especially for `←`.
- Run focused lookup checks for renamed entries:

```bash
cargo run -p faber -- explain ≡
cargo run -p faber -- explain →
cargo run -p faber -- explain ==
```

Checkpoint:

- No required entry uses a glyph filename or punctuation filename where an ASCII token slug is available.
- Existing `faber explain <term>` behavior survives the rename.

### Phase 2: Coverage Validator

Steps:

- Add a validation path under `crates/faber` tests or build support that can compare required terms against embedded explain entries.
- Prefer a maintainable manifest file if Rust source parsing would be too brittle.
- If using a manifest, keep it near the corpus, for example `explain/coverage.toml`.
- Validate:
  - every required keyword has an entry,
  - every required glyph/operator has an entry,
  - every legacy redirect has `canonical = false` and `canonical_term`,
  - every `canonical_term` points to an existing canonical entry,
  - every `related` term points to an existing entry unless explicitly marked external,
  - canonical filenames follow the ASCII token-slug policy,
  - legacy filenames follow the adjacent-sort `<token-slug>.legacy.md` policy,
  - unknown frontmatter fields fail validation.

Checkpoint:

- Focused coverage tests fail before corpus expansion and pass after missing entries are added.

### Phase 3: Keyword Corpus Expansion

Steps:

- Add entries for missing declaration/type keywords.
- Add entries for modifiers.
- Add entries for control flow.
- Add entries for transfer and error handling.
- Add entries for async, closure, boolean, null, logical, object, type operation, output, entry, miscellaneous, range, collection, testing, and nullability families.
- Use existing example files where accurate examples already exist.
- Create minimal snippets in entry bodies where repo examples do not yet cover the term.

Checkpoint:

- `faber explain <keyword>` works for every active keyword spelling.
- Keyword coverage validation passes.

### Phase 4: Glyph/Operator Corpus Expansion

Steps:

- Add canonical entries for missing logical glyphs: `∧`, `∨`, `⊻`, `¬`.
- Add canonical entries for shift glyphs: `≪`, `≫`.
- Add canonical entries for remaining compound assignment glyphs: `⊜`, `⊚`.
- Add canonical entries for conversion glyphs: `⇢`, `⇒`.
- Add canonical entries for range glyphs: `‥`, `…`.
- Add useful punctuation/form entries: `@`, `§`, optional chaining, and non-null chaining.
- Decide whether ASCII arithmetic operators `+`, `-`, `*`, `/`, and `%` should be included as operator entries. If included, classify them as ordinary arithmetic operators, not glyph-specific terms.

Checkpoint:

- `faber explain <glyph>` works for every canonical glyph/operator form.
- Operator coverage validation passes.

### Phase 5: Redirect and Alias Pass

Steps:

- Normalize existing legacy entries for `==`, `!=`, `<=`, `>=`, and `->`.
- Add redirects for old compound assignment spellings if they remain recognized or are part of the clean-break migration story:
  - `+=` -> `⊕`,
  - `-=` -> `⊖`,
  - `*=` -> `⊛`,
  - `/=` -> `⊘`,
  - `%=` -> `⊜` if applicable.
- Add redirects for strict equality spellings if relevant:
  - `===` -> `est` or `≡`, depending on the active language decision,
  - `!==` -> `non est` or `≠`, depending on the active language decision.
- Add aliases that are likely user queries:
  - `equals`, `equality`, `not equals`,
  - `return arrow`, `assignment`,
  - `test`, `skip`, `todo`, `async test`,
  - `range`, `inclusive range`, `exclusive range`.

Checkpoint:

- Legacy and natural-language lookups guide users to canonical Faber syntax.

### Phase 6: Human Renderer and Search

Steps:

- Refactor plain output into a dedicated renderer layer.
- Render canonical entries with `NAME`, `SYNTAX`, `DESCRIPTION`, `EXAMPLE`, `RELATED`, and `EXAMPLES` sections.
- Render legacy entries with `NAME`, `STATUS`, `USE INSTEAD`, `EXAMPLE`, and `SEE ALSO` sections.
- Add TTY-aware ANSI styling for section headings and dim metadata if the project wants styled output.
- Honor `NO_COLOR` and provide a plain/no-style path for tests and scripts.
- Add `faber explain --search <query>`.
- Keep exact lookup behavior unchanged for `faber explain <term>`.
- Add snapshot-style tests for canonical, legacy, and search output.

Checkpoint:

- `faber explain functio` reads like a compact language reference page.
- `faber explain ≡` and `faber explain ==` have distinct canonical and legacy layouts.
- `faber explain --search equality` returns multiple deterministic candidates.
- `faber explain --json <term>` remains unchanged.

### Phase 7: Docs and UX Pass

Steps:

- Update `docs/grammatica/explain.md` with the coverage policy.
- Update README command examples if renderer, search, list, or category output changes.
- Ensure `faber explain --list` and `faber explain --category <name>` remain useful with the expanded corpus.
- If list output becomes too large, group by category and keep plain output scannable.
- Make sure generated build artifacts remain out of source control.

Checkpoint:

- Docs tell users what coverage to expect.
- Large-corpus list output is still readable.

### Phase 8: Full Validation

Run:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
cargo build --release -p radix
```

Also run focused manual checks:

```bash
cargo run -p faber -- explain proba
cargo run -p faber -- explain solum_in
cargo run -p faber -- explain ≡
cargo run -p faber -- explain ∧
cargo run -p faber -- explain ⇢
cargo run -p faber -- explain --search equality
cargo run -p faber -- explain --list
```

Checkpoint:

- All validation passes.
- The coverage ledger has no required missing terms.

## Acceptance Criteria

- Every active keyword spelling accepted by the lexer has an `explain/` entry or a deliberate exclusion recorded in the inventory.
- Every canonical glyph/operator token accepted by the lexer has an `explain/` entry or a deliberate exclusion recorded in the inventory.
- Canonical corpus filenames use stable ASCII token slugs, while frontmatter `term` stores the exact keyword or glyph.
- Legacy redirect filenames use `<token-slug>.legacy.md`, while frontmatter `term` stores the exact legacy spelling.
- Legacy spellings that users are likely to ask about redirect to canonical entries.
- Coverage validation is automated.
- `faber explain` remains implemented in the Faber build tool, not the Radix compiler.
- Entries are self-contained and do not require source-tree-only grammar files.
- Full validation passes.

## Risks and Decisions

- Some lexer tokens may be accepted but not fully supported by parser or semantic analysis. Do not overclaim support; entry copy should describe the current documented language behavior.
- Annotation-mode keywords may not appear in the normal keyword table. Inventory must check all lexer keyword tables, not just one function.
- The glyph clean-break plan may remove some legacy ASCII acceptance. Redirect entries can still be useful after removal because users and agents may ask what the old spelling means.
- Complete coverage can create a lot of files. Use consistent frontmatter and compact prose so the corpus remains maintainable.
