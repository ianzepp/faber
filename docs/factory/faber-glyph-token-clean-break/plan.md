# Faber Glyph Token Clean-Break Factory Plan

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-glyph-token-clean-break/`
**Mode**: clean-break / full edit
**Commit Policy**: Commit after each completed phase and validation gate pass

## Prerequisite Assumption

This plan now assumes the Faber explain coverage completion work has already landed before the glyph clean break starts.

Specifically:

- `faber explain` exists in the Faber build tool, not the Radix compiler.
- The explain corpus covers all active keywords and canonical glyph/operator tokens.
- Legacy redirect entries exist for old spellings such as `==`, `!=`, `<=`, `>=`, `->`, and old compound assignment forms where applicable.
- Explain corpus filenames use stable ASCII token slugs, with legacy redirects named as `<token-slug>.legacy.md`.

That means this clean-break plan does not need to create the user-facing explanation surface from scratch. Its job is to remove old source-token acceptance from the compiler and keep the already-complete explain corpus accurate as migration/help text.

## Interpreted Problem

Faber has accumulated compatibility acceptance for old ASCII multi-character operators even though the language should now use canonical single-glyph source tokens. This creates confusing docs, examples, and tests: old forms such as `==` keep passing, so non-canonical syntax continues to spread.

The clean break is:

- remove old multi-character ASCII operator acceptance from the Faber lexer/parser surface,
- update examples to canonical glyph syntax,
- update compiler tests to use canonical glyph syntax,
- add negative tests proving old ASCII forms are rejected,
- update grammar docs so they no longer teach old forms as valid Faber source,
- preserve `faber explain` legacy redirects so users and agents can ask what removed spellings mean.

The requested implementation order is parser/compiler break first, then fix what breaks:

1. Remove old token acceptance from the compiler front end.
2. Run tests and example checks to expose all old-style usage.
3. Update tests.
4. Update examples.
5. Update grammar docs, explain redirect copy if needed, and guardrails.

## Break Boundary

The authoritative contract is Faber source syntax.

This cleanup applies to:

- `.fab` examples and fixtures,
- `.fab` snippets embedded in Rust compiler tests,
- grammar docs that teach Faber source syntax,
- lexer/parser tests that currently assert old token acceptance,
- generated Faber codegen round-trip expectations,
- `explain/*.md` legacy redirect entries, but only to ensure they describe old forms as removed/non-canonical.

This cleanup does **not** apply to:

- generated Rust, Go, or TypeScript output assertions,
- shell command examples such as `--help`, `--release`, or `cargo run`,
- target translation template strings such as `@ verte ts "... == ..."`,
- prose references that explicitly compare Faber to target-language syntax,
- `faber explain` legacy redirect entries that intentionally document removed spellings as invalid Faber source,
- Rust implementation code that naturally uses `==`, `!=`, `=>`, `->`, `&&`, or `||`.

Implementation sessions must avoid broad regex replacement across Rust source.

## Canonical Source Token Set

Initial canonical mapping:

| Old accepted Faber source token | Canonical Faber source token | Notes |
| ------------------------------- | ---------------------------- | ----- |
| `==` | `≡` | Equality |
| `!=` | `≠` | Inequality |
| `<=` | `≤` | Less-than-or-equal |
| `>=` | `≥` | Greater-than-or-equal |
| `->` | `→` | Return type arrow and endpoint binding arrow |
| `+=` | `⊕` | Add assignment |
| `-=` | `⊖` | Subtract assignment |
| `*=` | `⊛` | Multiply assignment |
| `/=` | `⊘` | Divide assignment |

Potentially in-scope after inventory:

| Old token | Canonical token | Decision |
| --------- | --------------- | -------- |
| `===` | `≡` or `est` | Remove as old strict-equality compatibility; choose documented replacement during implementation. |
| `!==` | `≠` or `non est` | Remove as old strict-inequality compatibility; choose documented replacement during implementation. |
| `&&` | `et` | Remove if currently accepted; prefer Latin keyword. |
| `||` | `aut` | Remove if currently accepted; prefer Latin keyword. |
| `!` | `non` or `¬` | Requires care because `!.`, `![`, and `!(` are distinct postfix/non-null forms. Do not remove casually. |
| `? :` | `sic ... secus` | Requires a separate language-design decision; do not remove in the first cut unless explicitly approved. |

Out of scope for this plan unless the user expands the break:

- arithmetic `+`, `-`, `*`, `/`, `%`,
- simple relational `<` and `>`,
- ordinary punctuation such as `(`, `)`, `{`, `}`, `[`, `]`, `.`, `,`, `:`,
- CLI dashes and shell syntax in docs.

## Repo-Aware Baseline

Confirmed current compiler behavior:

- `crates/radix/src/lexer/scan.rs` tokenizes both ASCII and glyph forms:
  - `==` and `≡` both become `TokenKind::EqEq`,
  - `!=` and `≠` both become `TokenKind::BangEq`,
  - `<=` and `≤` both become `TokenKind::LtEq`,
  - `>=` and `≥` both become `TokenKind::GtEq`,
  - `->` and `→` both become `TokenKind::Arrow`,
  - `+=` and `⊕` both become `TokenKind::PlusEq`, etc.
- `crates/radix/src/lexer/scan_test.rs` explicitly tests mixed old and canonical operator acceptance.
- `crates/radix/src/parser/expr.rs` still documents old precedence forms such as `==`, `!=`, `<=`, and `>=`.
- `docs/grammatica/operatores.md` still teaches `==`, `===`, `!=`, `!==`, `&&`, `||`, `? :`, and `<=`/`>=` examples.
- Faber codegen already prints equality and inequality back as canonical glyphs in `crates/radix/src/codegen/faber/ops.rs`.
- Examples already contain some canonical glyph usage, but docs and tests still contain old forms.

Expected explain baseline before this plan starts:

- `faber explain ≡`, `faber explain →`, and other canonical glyph lookups work from the installed/build tool.
- `faber explain ==`, `faber explain ->`, and related legacy lookups return guidance toward canonical syntax instead of implying those spellings are valid source.
- The explain corpus has already been renamed away from glyph/punctuation filenames.
- Coverage validation for explain entries already exists, so this plan should update explain entries only when the clean break changes canonical/legacy wording.

Important files:

- `crates/radix/src/lexer/scan.rs`
- `crates/radix/src/lexer/token.rs`
- `crates/radix/src/lexer/scan_test.rs`
- `crates/radix/src/parser/expr.rs`
- `crates/radix/src/parser/decl.rs`
- `crates/radix/src/parser/types.rs`
- `crates/radix/src/parser/stmt.rs`
- `crates/radix/src/parser/mod_test.rs`
- `crates/radix/src/driver/mod_test.rs`
- `crates/radix/src/codegen/faber/ops.rs`
- `examples/**/*.fab`
- `explain/*.md`
- `docs/grammatica/*.md`
- `docs/grammatica/explain.md`
- `docs/factory/faber-test-runner-evolution/plan.md`
- `docs/factory/faber-explain-coverage-completion/plan.md`

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Inventory and baseline | Capture old token inventory, current acceptance evidence, and completed explain baseline. | Ledger records exact old tokens, explain redirect status, and initial failing/passing state. |
| 1 | Front-end break | Remove old ASCII multi-character token acceptance from lexer/parser surface. | Tests fail only because callers still use removed source tokens. |
| 2 | Compiler tests migration | Update lexer/parser/driver/codegen tests to canonical glyph source syntax and add rejection tests. | Compiler tests pass and old tokens are rejected. |
| 3 | Examples migration | Update `.fab` examples and fixtures to canonical glyph syntax. | Example checks pass; no old tokens remain in Faber examples. |
| 4 | Grammar and explain docs migration | Update grammatica docs to teach canonical glyph syntax and align explain legacy wording. | Grammar docs no longer advertise removed source tokens; explain redirects still document removed spellings. |
| 5 | Guardrails and validation | Add searches/tests that prevent reintroducing old source tokens where practical. | Full validation passes and residue search is clean. |

## Phase Details

### Phase 0: Inventory and Baseline

Steps:

- Inspect `git status --short`.
- Verify the explain coverage completion plan has landed:
  - canonical glyph entries exist,
  - legacy redirect entries exist,
  - legacy redirect filenames use `<token-slug>.legacy.md`,
  - explain coverage validation passes.
- Capture current lexer acceptance for old and glyph tokens.
- Search separately in:
  - `examples/**/*.fab`,
  - `crates/radix/src/**/*_test.rs`,
  - `docs/grammatica/*.md`,
  - `explain/*.md`,
  - factory plans that embed Faber snippets.
- Create a ledger in this artifact directory.
- Classify each match as:
  - Faber source to change,
  - generated target-language assertion to leave alone,
  - shell/CLI syntax to leave alone,
  - intentional explain legacy redirect to preserve,
  - prose comparison to rewrite or clarify.

Checkpoint:

- Inventory names the old token set to remove in Phase 1.
- Inventory confirms explain coverage and legacy redirects are already in place.
- No behavior changed.

### Phase 1: Front-End Break

Steps:

- Remove old ASCII multi-character source-token branches from `crates/radix/src/lexer/scan.rs`:
  - `==`,
  - `!=`,
  - `<=`,
  - `>=`,
  - `->`,
  - `+=`,
  - `-=`,
  - `*=`,
  - `/=`.
- Remove `===` and `!==` acceptance.
- Keep canonical glyph branches:
  - `≡`,
  - `≠`,
  - `≤`,
  - `≥`,
  - `→`,
  - `⊕`,
  - `⊖`,
  - `⊛`,
  - `⊘`.
- Update parser comments and error messages that specifically name removed source tokens.
- Do not remove token enum variants yet if they still represent canonical glyph tokens internally.
- Run focused lexer/parser tests and expect failures from stale fixtures.

Checkpoint:

- Old ASCII operators are no longer accepted as valid Faber source.
- Canonical glyph operators still lex and parse.
- Breakage list is understood.

### Phase 2: Compiler Tests Migration

Steps:

- Update Faber source snippets inside Rust tests:
  - `==` -> `≡`,
  - `!=` -> `≠`,
  - `<=` -> `≤`,
  - `>=` -> `≥`,
  - `->` -> `→`,
  - compound assignments to `⊕`, `⊖`, `⊛`, `⊘`.
- Update lexer tests:
  - remove assertions that old ASCII forms produce normal tokens,
  - add negative assertions that old forms produce lex errors or parse failures,
  - keep positive assertions for glyph forms.
- Leave generated target-language output assertions alone.
- Leave Rust syntax and CLI flag strings alone.

Checkpoint:

- `cargo test -p radix lexer parser` or equivalent focused tests pass.
- `cargo test -p radix` passes after compiler-test migration.
- Tests explicitly prove at least `==`, `!=`, `<=`, `>=`, and `->` are rejected in Faber source.

### Phase 3: Examples Migration

Steps:

- Update `.fab` examples and fixtures under `examples/`.
- Include package fixtures introduced by test-runner planning once they exist.
- Avoid changing comments that show shell commands or target-language examples unless they incorrectly teach Faber source syntax.
- Run example checks:

```bash
cargo run -p faber -- check examples/exempla/salve-munde.fab
cargo test -p radix exempla_faber_roundtrip_e2e -- --ignored
```

Run broader example gates if practical.

Checkpoint:

- No old ASCII multi-character source tokens remain in `.fab` examples.
- Example check/roundtrip gates pass or documented blockers are recorded.

### Phase 4: Grammar and Explain Docs Migration

Steps:

- Update `docs/grammatica/operatores.md`.
- Update function/type docs from `->` to `→` where the snippet is Faber source.
- Update test-runner plan snippets if needed.
- Update `docs/grammatica/verba.md` so it no longer maps canonical Faber operators to old source tokens as though they are accepted syntax.
- Review `explain/*.legacy.md` entries for removed tokens and make sure they say the spelling is non-canonical or removed, not merely an alternate accepted spelling.
- Do not delete legacy explain entries solely because lexer support was removed; they are the migration/help interface.
- Keep target-language comparison tables only when clearly labeled as target-language output, not Faber input.

Checkpoint:

- Docs teach glyph-first Faber source syntax.
- Removed old tokens are not presented as valid Faber source.
- `faber explain <old-token>` still gives a useful canonical replacement.

### Phase 5: Guardrails and Validation

Steps:

- Add a focused hygiene test or script if feasible that scans `.fab` examples for removed old source tokens.
- Add lexer/parser negative tests for removed forms.
- Run:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
cargo build --release -p radix
```

- Run targeted residue search and classify any remaining old-token strings:

```bash
rg -n '==|!=|===|!==|<=|>=|->|\+=|-=|\*=|/=' examples docs/grammatica explain crates/radix/src crates/faber/src
```

Checkpoint:

- Validation passes.
- Residue search contains only allowed target-language, shell, Rust implementation, or intentional explain legacy contexts.
- Work is committed.

## Epic Candidates And Scopable Issues

### Issue A: Lexer Clean Break

Remove old ASCII multi-character operator scanning.

Acceptance:

- old forms no longer become normal Faber tokens,
- glyph forms still work,
- tests prove both sides.

### Issue B: Parser and Diagnostics Cleanup

Update parser comments, grammar notes, and diagnostics to stop naming removed source forms.

Acceptance:

- no parser comments describe removed tokens as grammar,
- parse errors point users toward glyph forms where useful.

### Issue C: Compiler Test Migration

Migrate embedded Faber snippets in Rust tests.

Acceptance:

- source snippets use glyph forms,
- generated target-language assertions are untouched,
- `cargo test -p radix` passes.

### Issue D: Examples Migration

Migrate `.fab` examples.

Acceptance:

- examples use canonical glyph tokens,
- example validation passes.

### Issue E: Docs and Guardrails

Update grammar docs, align explain legacy redirect wording, and add residue guardrails.

Acceptance:

- grammar docs no longer teach removed forms,
- explain legacy redirects remain available and describe removed forms as non-canonical,
- hygiene checks or negative tests prevent easy regression.

## Checkpoints

Commit after each completed phase:

- after inventory ledger,
- after front-end break plus expected failing state if useful,
- after compiler tests pass,
- after examples pass,
- after docs and final validation.

Do not combine parser removal, all test migration, all examples, and all docs in one giant commit. The break should be auditable.

## Companion Skill Plan

- Use `clean-break` throughout implementation; this is intentionally compatibility removal.
- Use `poker-face` after Phase 2 and Phase 5 to verify old syntax is actually gone.
- Use `zombie-docs` after Phase 4 to catch stale grammar examples.
- Use `carmack-linus` only if the canonical token set becomes controversial or if removing `? :` / `!` is reconsidered.

## Gate Plan

Required final gate:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
cargo build --release -p radix
```

Required semantic proof:

- `adfirma 1 + 1 ≡ 2` parses.
- `adfirma 1 + 1 == 2` fails.
- `functio f() → vacuum {}` parses.
- `functio f() -> vacuum {}` fails.
- `x ⊕ 1` parses as add assignment where assignment syntax is valid.
- `x += 1` fails where it previously parsed.
- `faber explain ==` points to `≡` and says `==` is non-canonical or removed.
- `faber explain ->` points to `→` and says `->` is non-canonical or removed.
- `.fab` examples no longer contain removed old source tokens.

## Open Questions

- Should `? :` ternary be removed in this clean break, or deferred because canonical `sic ... secus` is not a single glyph?
- Should logical ASCII `&&` and `||` be explicitly rejected if any support remains, or is this already keyword-only?
- Should `!` be removed as logical-not syntax in favor of `non`/`¬`, while preserving `!.`, `![`, and `!(`?
- Should simple ASCII `<` and `>` stay because they are single-character mathematical operators, or should the language eventually require Latin/range-style alternatives?
- Should grammar docs preserve a historical migration note listing removed ASCII forms, or should legacy history live only in `faber explain <old-token>` redirects?
