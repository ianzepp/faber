# Faber Explain Command Implementation Notes

Created: 2026-05-21

## Grammar And Documentation Drift

The explain corpus intentionally teaches the glyph-first spelling requested by the factory plan:

- `≡` instead of `==` or `===`
- `≠` instead of `!=` or `!==`
- `≤` instead of `<=`
- `≥` instead of `>=`
- `→` instead of `->`

Current Radix lexer behavior still accepts the legacy spellings, and parts of the older grammar and stdlib documentation still contain `->` and `=` examples. The `faber explain` entries make the product-facing rule explicit by treating those spellings as legacy redirects rather than canonical source.

Radix does not load, parse, or depend on the explain corpus. The corpus is embedded by `crates/faber/build.rs` into the Faber binary at compile time.
