# Faber v0.34.0

## Continuity

v0.33.0 remains a valid release. It shipped the first renewed `faber` binary with both project commands and compiler inspection commands in one tool.

## What Changed

- **Faber** (`faber` binary from `crates/faber`) is now the user-facing project and package tool.
- **Radix** (`radix` binary from `crates/radix`) is the compiler-developer tool for `lex`, `parse`, `hir`, `emit`, `check`, `targets`, and `cli-ir`.
- The Cargo workspace moved from `radix/` to the repository root (`Cargo.toml`, `crates/`, `stdlib/`).
- Package discovery and local graph orchestration moved from the Radix driver into the Faber crate.
- `faber build` emits a generated Rust crate under `target/faber/` and builds with Cargo artifacts under the package `target/` directory.
- `faber run` builds and executes package binaries.
- `faber test` is now a Cargo-backed package test runner with support for generated Faber tests, ignored/future tests, focused tests, and source-level selectors.
- `faber explain` embeds the glyph/keyword explanation corpus and supports direct lookup, search, listing, and JSON output.
- Bun/Node are no longer required for normal development; use Cargo and `scripta/ci`.

## Compatibility

These commands remain on `faber` as aliases to the same behavior:

- `faber lex`, `faber parse`, `faber hir`, `faber emit`, `faber cli-ir`

Use `faber check`, `faber build`, and `faber targets` for project workflows. Use `radix` for direct compiler inspection.

## Current Limits

- Package builds are Rust-backed in this release.
- `faber test` uses Cargo's standard Rust test harness, so generated Rust `proba_*` names may still appear in output.
- Some parsed test modifiers are preserved in metadata for future runner phases but are not enforced yet.
