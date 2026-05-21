# Explain Corpus

`faber explain` is the installed lookup surface for Faber glyphs, keywords, and grammar terms. It is owned by the Faber project tool, not by the Radix compiler front end.

Use it when a term needs a short self-contained explanation:

```bash
faber explain ≡
faber explain proba
faber explain --search equality
faber explain --json custodi
faber explain --list
faber explain --category testing
```

The source entries live in the repository root under `explain/*.md`. The `crates/faber` build script embeds those Markdown files into the binary at compile time, so the installed tool does not need runtime access to this source tree.

Coverage is governed by `explain/coverage.toml`. The manifest lists the required canonical terms, legacy redirects, and deliberate exclusions that the `crates/faber` coverage test checks against the embedded corpus.

Canonical entries use stable ASCII slugs in the filename while keeping the exact Faber term in frontmatter. Legacy redirects use the adjacent `<slug>.legacy.md` filename pattern.

Exact lookups render as compact reference pages with `NAME`, `KIND`, `SYNTAX`, `DESCRIPTION`, `EXAMPLE`, and `RELATED` sections. Legacy lookups render a correction page with `NAME`, `STATUS`, `USE INSTEAD`, `DESCRIPTION`, `EXAMPLE`, and `SEE ALSO`.

The `examples` frontmatter field remains available in JSON and for corpus validation, but the plain installed CLI output does not print repository example paths.

`faber explain --search <query>` is a separate discovery mode. It returns a ranked list of matching entries instead of choosing one fuzzy match automatically.

Each entry uses frontmatter followed by a short Markdown body with one Faber code example:

````markdown
---
term: "proba"
kind: "keyword"
category: "testing"
canonical: true
summary: "Defines a single test case."
syntax: "proba <name> <block>"
aliases:
  - "test"
related:
  - "adfirma"
---

`proba` introduces one test case.

```fab
proba "arithmetic passes" {
    adfirma 1 + 1 ≡ 2
}
```
````

Allowed fields are `term`, `kind`, `category`, `canonical`, `summary`, `syntax`, `examples`, `aliases`, `legacy`, `canonical_term`, and `related`. Unknown fields fail validation. Non-canonical legacy entries must set `canonical_term`.
