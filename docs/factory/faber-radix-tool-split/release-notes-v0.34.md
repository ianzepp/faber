# Faber v0.34.0

## Continuity

v0.33.0 remains a valid release. It shipped the first renewed `faber` binary with both project commands and compiler inspection commands in one tool.

## What Changed

- **Faber** (`faber` binary from `crates/faber`) is now the user-facing project and package tool.
- **Radix** (`radix` binary from `crates/radix`) is the compiler-developer tool for `lex`, `parse`, `hir`, `emit`, `check`, `targets`, and `cli-ir`.
- The Cargo workspace moved from `radix/` to the repository root (`Cargo.toml`, `crates/`, `stdlib/`).
- Package discovery and local graph orchestration moved from the Radix driver into the Faber crate.
- Bun/Node are no longer required for normal development; use Cargo and `scripta/ci`.

## Compatibility

These commands remain on `faber` as aliases to the same behavior:

- `faber lex`, `faber parse`, `faber hir`, `faber emit`, `faber cli-ir`

Use `faber check`, `faber build`, and `faber targets` for project workflows. Use `radix` for direct compiler inspection.

## Stubs

`faber init`, `faber run`, and `faber test` print explicit not-implemented messages in v0.34.
