# Faber Romanus

**The Roman Craftsman** is a Latin programming language compiler centered on a Rust workspace at the repository root.

The active surface is intentionally narrow: the compiler library, two CLI binaries, Rust runtime support, Faber standard library definitions, examples, and docs. Older bootstrap and self-hosting implementations live in the sibling archive repository.

## Current Shape

| Path | Purpose |
| ---- | ------- |
| [`Cargo.toml`](Cargo.toml) | Cargo workspace manifest |
| [`crates/faber`](crates/faber) | User-facing project and package tool (`faber` binary) |
| [`crates/radix`](crates/radix) | Compiler library and developer tool (`radix` binary) |
| [`crates/norma`](crates/norma) | Rust runtime support crate for stdlib and HAL features |
| [`stdlib/norma`](stdlib/norma) | Faber standard library definitions with `@ verte` translation metadata |
| [`examples/exempla`](examples/exempla) | Example Faber programs |
| [`docs/grammatica`](docs/grammatica) | Language documentation |
| [`scripta`](scripta) | Shell helper scripts |

## Quick Start

```bash
./scripta/ci

cargo run -p faber -- targets
cargo run -p faber -- check examples/exempla/salve-munde.fab
cargo run -p faber -- build examples/exempla/salve-munde.fab
cargo run -p faber -- emit -t rust examples/exempla/salve-munde.fab

cargo run -p radix --bin radix -- targets
cargo run -p radix --bin radix -- emit -t rust examples/exempla/salve-munde.fab
```

Equivalent raw Cargo commands:

```bash
cargo fmt --all -- --check
cargo test --all
cargo build --release -p faber
cargo build --release -p radix --bin radix
```

## Package Manifest

Faber packages use `faber.toml` for project metadata and build configuration:

```toml
[package]
name = "salve"
version = "0.1.0"
edition = "2026"

[paths]
source = "src"
entry = "main.fab"

[build]
target = "rust"
kind = "bin"
```

Run `cargo run -p faber -- init hello` to create a starter package. See
[`docs/grammatica/manifest.md`](docs/grammatica/manifest.md) for the current manifest contract.

## CLI Roles (v0.34)

**Faber** is the user-facing project and package tool:

- `faber check`, `faber build`, `faber targets`
- `faber init`
- `faber run`; `faber test` (planned)
- Compatibility aliases: `faber lex`, `faber parse`, `faber hir`, `faber emit`, `faber cli-ir`

**Radix** is the compiler-developer tool:

- `radix lex`, `radix parse`, `radix hir`, `radix emit`, `radix check`, `radix targets`, `radix cli-ir`
- Package policy lives in Faber; use `faber check --package` and `faber build` for local packages.

v0.33 shipped a single `faber` binary that combined both layers. v0.34 splits the binaries while keeping Faber compatibility aliases for compiler inspection commands.

## Standard Library

Faber stdlib definitions live in [`stdlib/norma`](stdlib/norma):

```fab
@ verte ts "push"
@ verte rs "push"
functio adde(T elem) -> vacuum
```

Runtime-backed Rust support lives in [`crates/norma`](crates/norma).

## Language Notes

Faber uses type-first syntax:

```fab
textus nomen
numerus aetas
functio salve(textus name) -> textus
```

Use [`EBNF.md`](EBNF.md) as the formal grammar source, and [`docs/grammatica`](docs/grammatica) for prose explanations.

## Archive Note

Historical bootstrap compilers, self-hosting sources, old reference code, and old test harnesses are preserved in `../faber-archivum`. They are useful for archaeology, not for current main-repo commands or CI.

Opus perfectum est when `./scripta/ci` passes.
