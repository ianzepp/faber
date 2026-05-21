# Target Compatibility

This page describes the current target contract of the active `radix-rs` compiler.

Treat `cargo run --manifest-path Cargo.toml -p faber -- targets` as the live source of truth for the target list and high-level capability notes. Archived bootstrap compilers and planned future targets are not part of the active main-repo target contract.

## Current `radix-rs` Target Surface

As of the current `radix-rs` CLI, the supported targets are:

| Target  | `check` | `build` | `package` | Notes                                                                |
| ------- | ------- | ------- | --------- | -------------------------------------------------------------------- |
| `rust`  | yes     | yes     | yes       | Primary backend; full package build + run via `faber`                |
| `go`    | yes     | yes     | no        | File emission supported; package compilation not yet supported       |
| `ts`    | yes     | yes     | no        | File emission supported; package compilation not yet supported       |
| `faber` | yes     | yes     | no        | Canonical pretty-print target; package compilation not yet supported |

The current `targets` command prints these same notes directly:

```text
rust check=yes build=yes run=yes package=yes note=primary backend; full package build + run via `faber`
go check=yes build=yes run=no package=no note=file emission supported; package compilation not yet supported
ts check=yes build=yes run=no package=no note=file emission supported; package compilation not yet supported
faber check=yes build=yes run=no package=no note=canonical pretty-print target; package compilation not yet supported
```

## Primary Surface Versus Secondary Surfaces

[`crates/radix`](../../crates/radix) is the active delivery compiler.

- Rust is the strongest target today and the primary package-compilation path.
- Go and TypeScript are real `radix-rs` file-emission targets, but package support is still intentionally absent.
- Faber output is useful as a canonical pretty-printer and inspection target.
- Python is not part of the current `radix-rs targets` contract.

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
cargo run --manifest-path Cargo.toml -p faber -- build examples/exempla/salve-munde.fab
cargo run --manifest-path Cargo.toml -p faber -- build --package examples/exempla/cli/main.fab
cargo run --manifest-path Cargo.toml -p faber -- build --package path/to/faber.toml
cargo run -p radix --bin radix -- emit -t go examples/exempla/salve-munde.fab
```

Use `build --package` only when the target actually supports package compilation.
When a package directory contains `faber.toml`, Faber uses that manifest to find the source root and entry file.

## CLI Framework Support

Declarative CLI programs are runnable only on the Rust target today.

- `rust`: supports single-command CLIs, subcommands, help/version output, and package-local `@ imperia` module mounts.
- `go`, `ts`, `faber`: CLI syntax can be parsed and checked, but runnable CLI codegen is intentionally gated.

Generated Rust CLI programs currently include their parser and dispatcher inline. There is no `norma::cli` runtime
dependency yet.

## Portability Guidance For The Current Compiler

If you want the least surprising cross-target results on the active `radix-rs` surface:

1. Prefer single-file examples unless you specifically need Rust package output.
2. Treat Rust as the strongest correctness gate.
3. Use `emit` for stdout/debug workflows and `build` for writing outputs to disk.
4. Re-check [`README.md`](../../README.md) or `radix targets` before assuming a target or package mode exists.

## What This Page Does Not Promise

This page does not claim a current `radix-rs` support matrix for Python, Zig, or C++. Those targets are either archive history or future/planned work. If broader target support becomes part of the primary compiler contract later, this page should be expanded from live compiler evidence rather than from historical notes.
