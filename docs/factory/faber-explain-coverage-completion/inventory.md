# Explain Coverage Inventory

Generated from the live lexer and parser inventory on 2026-05-21.

## Snapshot

The explain corpus is now driven by `explain/coverage.toml`, which lists the required canonical terms, legacy redirects, and deliberate exclusions that the coverage test enforces.

Canonical coverage currently includes the full live surface from the lexer tables and parser entry points, including:

- declarations and types
- modifiers
- control flow
- transfer and errors
- async and closures
- boolean, logic, and nullability
- objects, type operations, and conversions
- entry, I/O, and miscellaneous forms
- ranges and collections
- testing
- punctuation and metadata entries
- canonical glyph and operator entries

Legacy redirects currently exist for:

- `==` -> `≡`
- `!=` -> `≠`
- `<=` -> `≤`
- `>=` -> `≥`
- `->` -> `→`
- `===` -> `est`
- `!==` -> `non est`

Deliberately excluded ordinary arithmetic operators:

- `+`
- `-`
- `*`
- `/`
- `%`

## Filename Policy

Canonical glyph and punctuation terms use ASCII slugs in the filename while keeping the exact token or phrase in frontmatter.

Preferred canonical slugs:

- `≡` -> `eq-eq.md`
- `≠` -> `bang-eq.md`
- `≤` -> `lt-eq.md`
- `≥` -> `gt-eq.md`
- `→` -> `arrow.md`
- `←` -> `assign.md`
- `⊕` -> `plus-eq.md`
- `⊖` -> `minus-eq.md`
- `⊛` -> `star-eq.md`
- `⊘` -> `slash-eq.md`
- `⇢` -> `verte.md`
- `⇒` -> `conversio.md`
- `∧` -> `amp.md`
- `∨` -> `pipe.md`
- `⊻` -> `caret.md`
- `¬` -> `tilde.md`
- `≪` -> `sinistratum.md`
- `≫` -> `dextratum.md`
- `‥` -> `dot-dot.md`
- `…` -> `ellipsis.md`
- `?.` -> `question-dot.md`
- `?[` -> `question-bracket.md`
- `?(` -> `question-paren.md`
- `!.` -> `bang-dot.md`
- `![` -> `bang-bracket.md`
- `!(` -> `bang-paren.md`
- `⊜` -> `amp-eq.md`
- `⊚` -> `pipe-eq.md`
- `@` -> `at.md`
- `§` -> `section.md`
- `non est` -> `non-est.md`
- `solum_in` -> `solum-in.md`

Preferred legacy slugs:

- `==` -> `eq-eq.legacy.md`
- `!=` -> `bang-eq.legacy.md`
- `<=` -> `lt-eq.legacy.md`
- `>=` -> `gt-eq.legacy.md`
- `->` -> `arrow.legacy.md`
- `===` -> `est.legacy.md`
- `!==` -> `non-est.legacy.md`

## Coverage Contract

The explain corpus is complete only when:

- every active keyword, annotation marker, and meaningful glyph/operator token has a matching entry or redirect,
- required legacy spellings resolve to canonical guidance,
- canonical glyph entries use ASCII slug filenames where a stable slug exists,
- legacy redirect filenames use `<token-slug>.legacy.md`,
- deliberate exclusions remain recorded in `explain/coverage.toml`,
- the embedded corpus continues to load without validation errors.
