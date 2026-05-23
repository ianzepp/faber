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
| [`explain`](explain) | Embedded explanation corpus for `faber explain` |
| [`stdlib/norma`](stdlib/norma) | Faber standard library definitions with `@ verte` translation metadata |
| [`hosts/macos-arm64`](hosts/macos-arm64) | Placeholder host runtime for future Faber-produced Wasm components on macOS arm64 |
| [`examples/exempla`](examples/exempla) | Example Faber programs |
| [`EBNF.md`](EBNF.md) | Formal grammar and spec commentary |
| [`docs`](docs) | Delivery plans, release notes, and design history |
| [`scripta`](scripta) | Shell helper scripts |

## Quick Start

Install the current released CLI with Homebrew:

```bash
brew install ianzepp/tap/faber
faber --version
```

The `faber` crate name on crates.io is not this project; use the Homebrew tap
or build from this repository.

From a checkout:

```bash
./scripta/ci

cargo run -p faber -- targets
cargo run -p faber -- explain ≡
cargo run -p faber -- explain ⇥
cargo run -p faber -- explain --search equality
cargo run -p faber -- explain --json proba
cargo run -p faber -- check examples/exempla/salve-munde.fab
cargo run -p faber -- build -o /tmp/faber-out examples/exempla/salve-munde.fab
cargo run -p faber -- test examples/exempla/proba/packages/passing
cargo run -p faber -- emit -t rust examples/exempla/salve-munde.fab

cargo run -p radix --bin radix -- targets
cargo run -p radix --bin radix -- mir examples/exempla/salve-munde.fab
cargo run -p radix --bin radix -- emit -t rust examples/exempla/salve-munde.fab
```

The CI wrapper expands to:

```bash
cargo fmt --all -- --check
cargo test --all
./scripta/check-markers
cargo clippy --all-targets --all-features -- -D warnings
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

Run `cargo run -p faber -- init hello` to create a starter package. The embedded
explain entries cover the current package and test contracts:

```bash
cargo run -p faber -- explain manifest
cargo run -p faber -- explain proba
cargo run -p faber -- explain probandum
```

## CLI Roles (v0.35)

**Faber** is the user-facing project and package tool:

- `faber check`, `faber build`, `faber targets`
- `faber explain <term>`, `faber explain --search <query>`, `faber explain --list`, `faber explain --json <term>`
- `faber init`
- `faber run`; `faber test`
- Compatibility aliases: `faber lex`, `faber parse`, `faber hir`, `faber emit`, `faber cli-ir`

**Radix** is the compiler-developer tool:

- `radix lex`, `radix parse`, `radix hir`, `radix mir`, `radix emit`, `radix check`, `radix targets`, `radix cli-ir`
- Package policy lives in Faber; use `faber check --package` and `faber build` for local packages.

The active compiler has a validated MIR inspection branch:

```text
Lex -> Parse -> Collect -> Resolve -> Lower -> Typecheck -> Analysis
  -> typed HIR -> default target codegen
  -> validated MIR -> inspection / internal executable probe
```

`radix mir` prints the validated middle IR for compiler-development inspection. Normal Rust output still uses the stable HIR-to-Rust backend; the MIR Rust probe is an internal boundary test, not the user-facing Rust backend.

v0.33 shipped a single `faber` binary that combined both layers. v0.34 splits the binaries while keeping Faber compatibility aliases for compiler inspection commands.

## Standard Library

Faber stdlib definitions live in [`stdlib/norma`](stdlib/norma):

```fab
@ verte ts "push"
@ verte rs "push"
functio adde(T elem) → vacuum
```

Runtime-backed Rust support lives in [`crates/norma`](crates/norma).

## Commandments

These are the rules that make Faber feel like Faber. Syntax can evolve, but changes should preserve these laws.

1. **Types before names.** Declarations read from shape to binding: `textus nomen`, `numerus aetas`, `functio salve(textus name) → textus`.
2. **Mechanical over magical.** The same construct should mean the same thing everywhere. If a reader needs distant context to know what a symbol does, the syntax is suspect.
3. **Glyphs carry structure.** Use glyphs for operators, control-flow joints, and value-flow edges: `←`, `→`, `⇥`, `∴`, `≡`, `∪`.
4. **Latin carries behavior.** Use Latin words for declarations, statements, lifecycle, and behavioral intent: `functio`, `genus`, `fixum`, `varia`, `redde`, `cape`.
5. **One sign, one job.** A glyph or keyword may have exact aliases, but it should not carry unrelated meanings. Aliases must point back to one canonical concept.
6. **Runtime flow is explicit.** Runtime binding, reassignment, and mutation use `←`; structural definition uses `=`.
7. **Absence is typed.** Nullable value types are written as unions, such as `T ∪ nihil`; optional declaration slots use post-name markers such as `sponte`.
8. **The compiler does not guess to hide missing information.** Missing type information is an analysis problem to fix upstream, not a codegen detail to paper over.

## Language Notes

Faber uses type-first syntax:

```fab
textus nomen
numerus aetas
functio salve(textus name) → textus
```

Use [`EBNF.md`](EBNF.md) as the formal grammar and spec-commentary source. Use `faber explain <term>` or the Markdown files in [`explain`](explain) for user-facing reference text.

## Archive Note

Historical bootstrap compilers, self-hosting sources, old reference code, and old test harnesses are preserved in `../faber-archivum`. They are useful for archaeology, not for current main-repo commands or CI.

Opus perfectum est when `./scripta/ci` passes.
