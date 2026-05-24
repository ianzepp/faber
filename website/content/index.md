+++
title = "Faber Romanus"
description = "A Latin programming language and compiler. User-facing project tool (faber) with Rust primary backend; radix for compiler development and inspection."
order = 0
+++

<div class="hero">

# Faber Romanus

<p class="tagline">The Roman Craftsman — A Latin programming language with a Rust-based compiler toolchain.</p>

</div>

<div class="install-block">

**Install (Homebrew)**

```bash
brew install ianzepp/tap/faber
```

Or build from source:

```bash
cargo install --git https://github.com/ianzepp/faber faber
```

</div>

<div class="links-block">

[GitHub](https://github.com/ianzepp/faber) • [CLI Reference](#cli) • [Language Docs](#language-reference)

</div>

## Current Status

Faber Romanus is under active development (v0.35+). The implementation is a Rust workspace:

- `crates/faber` — user-facing project and package CLI (`faber` binary). Primary interface for `faber init`, `faber build`, `faber run`, `faber test`, `faber check`, `faber emit`.
- `crates/radix` — compiler pipeline library + developer CLI (`radix` binary) for diagnostics, explain, emit, etc.
- `crates/norma` — Rust runtime support for the standard library and HAL contracts.
- `stdlib/norma/` — Faber source for the standard library (collection methods, annotations, target translations via `@ verte`).

**Primary target:** Rust (full check/build/run/package support).

**File-emission targets (check/build/emit supported):** Go, TypeScript, Faber (canonical).

Python, Zig, and C++ are not current active emission targets in the v0.3x line.

## The Language

Faber is a **type-first, Latin-keyword** language designed for clarity, LLM-friendliness, and easy human review of generated code.

Key principles from the [EBNF grammar](EBNF.md) and [explain corpus](explain/):

- Type declarations precede names: `textus nomen` not `nomen: textus`.
- Explicit `T ∪ nihil` for nullable/optional in type position; `sponte` for voluntary parameters/fields.
- Rich block syntax: `si`, `dum`, `itera ex/de`, `elige`/`discerne` (match), `fac ... cape` (try), `cura` (arena/scope).
- Function annotations: `@ futura`, `@ cursor`, etc.
- String templates: `"Hello, §!"(name)`

See the language reference pages for full details.

## Documentation In This Site

This site is generated from (or curated alongside) sources of truth in the repository:

- Grammar: `EBNF.md`
- Detailed reference: `explain/` (used by `faber explain <topic>`)
- Project and release notes: `docs/`
- Examples: `examples/exempla/`
- Standard library: `stdlib/norma/`

The monorepo layout ensures documentation, compiler behavior, and examples evolve together.

---

*This is the refreshed homepage seed for the monorepo website migration. Content will be expanded with imported grammar, CLI help, targets matrix, package docs, and curated examples in subsequent phases.*
