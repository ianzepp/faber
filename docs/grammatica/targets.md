# Target Compatibility

This page describes the current target contract of the active `radix-rs` compiler.

Treat `cargo run --manifest-path compilers/radix-rs/Cargo.toml -- targets` as the live source of truth for the target list and high-level capability notes. Older bootstrap compilers and planned future targets are real repository surfaces, but they are not the authoritative contract of the primary compiler.

## Current `radix-rs` Target Surface

As of the current `radix-rs` CLI, the supported targets are:

| Target | `check` | `build` | `package` | Notes |
| ------ | ------- | ------- | --------- | ----- |
| `rust` | yes | yes | yes | Primary backend; package compilation supported |
| `go` | yes | yes | no | File emission supported; package compilation not yet supported |
| `ts` | yes | yes | no | File emission supported; package compilation not yet supported |
| `faber` | yes | yes | no | Canonical pretty-print target; package compilation not yet supported |

The current `targets` command prints these same notes directly:

```text
rust check=yes build=yes run=no package=yes note=primary backend; package compilation supported
go check=yes build=yes run=no package=no note=file emission supported; package compilation not yet supported
ts check=yes build=yes run=no package=no note=file emission supported; package compilation not yet supported
faber check=yes build=yes run=no package=no note=canonical pretty-print target; package compilation not yet supported
```

## Primary Surface Versus Secondary Surfaces

[`compilers/radix-rs`](../../compilers/radix-rs) is the active delivery compiler.

- Rust is the strongest target today and the primary package-compilation path.
- Go and TypeScript are real `radix-rs` file-emission targets, but package support is still intentionally absent.
- Faber output is useful as a canonical pretty-printer and inspection target.
- Python remains a maintenance/bootstrap surface elsewhere in the repository, not part of the current `radix-rs targets` contract.

If you need current repo status at a glance, see [`project.yaml`](../../project.yaml) and the root [`README.md`](../../README.md).

## Current Syntax Reminder

Some older docs used verb-form return syntax such as `fit`, `fiet`, `fiunt`, and `fient`. The current grammar contract lives in [`EBNF.md`](../../EBNF.md) and uses arrow returns plus annotations:

```fab
functio parse() -> numerus

@ futura
functio fetch(textus url) -> textus

@ cursor
functio count(numerus n) -> numerus

@ futura
@ cursor
functio stream() -> textus
```

When reading target examples, prefer this syntax over older prose that still reflects pre-annotation documentation.

## Package Compilation

Package compilation is a target capability, not a generic promise.

- `rust`: package compilation is supported today.
- `go`, `ts`, `faber`: file emission is supported, but package compilation is not yet supported.

This matters for command selection:

```bash
cargo run --manifest-path compilers/radix-rs/Cargo.toml -- build examples/exempla/salve-munde.fab
cargo run --manifest-path compilers/radix-rs/Cargo.toml -- build --package examples/exempla/cli/main.fab
cargo run --manifest-path compilers/radix-rs/Cargo.toml -- emit -t go examples/exempla/salve-munde.fab
```

Use `build --package` only when the target actually supports package compilation.

## Portability Guidance For The Current Compiler

If you want the least surprising cross-target results on the active `radix-rs` surface:

1. Prefer single-file examples unless you specifically need Rust package output.
2. Treat Rust as the strongest correctness gate.
3. Use `emit` for stdout/debug workflows and `build` for writing outputs to disk.
4. Re-check [`README.md`](../../README.md) or `radix targets` before assuming a target or package mode exists.

## What This Page Does Not Promise

This page does not claim a current `radix-rs` support matrix for Python, Zig, or C++. Those surfaces either live in bootstrap/older compiler paths or remain future/planned work. If broader target support becomes part of the primary compiler contract later, this page should be expanded from live compiler evidence rather than from historical notes.
