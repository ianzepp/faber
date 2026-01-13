# Archived Faber Codegen Targets

These codegen implementations were removed from faber as part of the compiler role separation (see `consilia/compiler-roles.md`).

## Why archived?

- **Faber** is now the TypeScript-only reference compiler
- **Rivus** will handle multi-target codegen (TS, Python, Rust, Zig, C++)

## Contents

| Directory | Target | Lines | Notes |
|-----------|--------|-------|-------|
| `zig/` | Zig | ~3,700 | Systems programming target |
| `py/` | Python | ~3,100 | Popular scripting target |
| `rs/` | Rust | ~2,500 | Memory-safe systems target |
| `cpp/` | C++ | ~2,700 | Traditional systems target |
| `fab/` | Faber | ~1,900 | Self-compilation target |

## For rivus development

These implementations serve as reference for rivus codegen:
- Expression handlers in `*/expressions/`
- Statement handlers in `*/statements/`
- Preamble generation in `*/preamble/`
- Target-specific patterns and idioms

## Status

Archived 2026-01-12. Will be removed once rivus reaches feature parity.
