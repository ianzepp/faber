# Faber Romanus

**The Roman Craftsman** is a Latin programming language compiler centered on the Rust `radix` workspace.

The current repository is intentionally narrow: the active compiler, Rust runtime crate, Faber standard library definitions, examples, and docs all live around `radix/`. Older bootstrap and self-hosting implementations were moved to the sibling archive repository during the pruning pass and are no longer live main-repo surfaces.

## Current Shape

| Path | Purpose |
| ---- | ------- |
| [`radix/Cargo.toml`](radix/Cargo.toml) | Canonical Cargo workspace manifest |
| [`radix/crates/radix`](radix/crates/radix) | Active compiler crate that builds the `faber` CLI binary |
| [`radix/crates/norma`](radix/crates/norma) | Rust runtime support crate for standard library and HAL features |
| [`radix/stdlib/norma`](radix/stdlib/norma) | Faber standard library definitions with `@ verte` translation metadata |
| [`examples/exempla`](examples/exempla) | Example Faber programs |
| [`docs/grammatica`](docs/grammatica) | Language documentation |
| [`scripta`](scripta) | Current helper scripts only |

## Quick Start

```bash
bun install

bun run check:radix
bun run test:radix
bun run ci

bun run build:radix
cargo run --manifest-path radix/Cargo.toml -p radix -- targets
cargo run --manifest-path radix/Cargo.toml -p radix -- check examples/exempla/salve-munde.fab
cargo run --manifest-path radix/Cargo.toml -p radix -- emit -t rust examples/exempla/salve-munde.fab
cargo run --manifest-path radix/Cargo.toml -p radix -- emit -t ts examples/exempla/salve-munde.fab
```

Root CI is scoped to the `radix` workspace:

```bash
cargo fmt --manifest-path radix/Cargo.toml --all -- --check
cargo test --manifest-path radix/Cargo.toml
```

## Compiler Surface

The active CLI is the `faber` binary from `radix/crates/radix`.

Product-facing commands:

- `targets` shows supported targets and capability notes.
- `check` runs semantic analysis.
- `build` writes generated output to disk.
- `emit` writes generated output to stdout.

Inspection commands:

- `lex` emits lexer output.
- `parse` emits parsed syntax.
- `hir` emits lowered HIR.

Current targets are documented in [`docs/grammatica/targets.md`](docs/grammatica/targets.md). Rust is the strongest backend and the package-compilation path. Go, TypeScript, and canonical Faber output are file-emission surfaces.

## Standard Library

The Faber standard library definitions live in [`radix/stdlib/norma`](radix/stdlib/norma). These files describe collection methods, HAL contracts, and target translations:

```fab
@ verte ts "push"
@ verte rs "push"
functio adde(T elem) -> vacuum
```

The Rust runtime crate that supports runtime-backed stdlib and HAL behavior lives in [`radix/crates/norma`](radix/crates/norma).

## Language Notes

Faber uses type-first syntax:

```fab
textus nomen
numerus aetas
functio salve(textus name) -> textus
```

Use [`EBNF.md`](EBNF.md) as the formal grammar source, and [`docs/grammatica`](docs/grammatica) for prose explanations.

Block syntax follows regular keyword patterns:

```fab
itera ex items fixum item {
    si item.pretium > 100 {
        scribe item.nomen
    }
}
```

## Archive Note

Historical bootstrap compilers, self-hosting sources, old reference code, and old test harnesses are preserved in `../faber-archivum`. They are useful for archaeology, not for current main-repo commands or CI.

Opus perfectum est only when `bun run ci` passes against `radix/Cargo.toml`.
