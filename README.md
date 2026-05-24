# Faber Romanus

**The Roman Craftsman** is a Latin programming language and compiler centered on
the root Rust workspace in this repository.

Faber's active surface is intentionally narrow: the Radix compiler library, the
Faber project/package tool, Rust runtime support, Faber standard library
definitions, examples, and docs. Historical bootstrap compilers, self-hosting
sources, old reference code, and old test harnesses live in the sibling archive
repository; they are useful for archaeology, not for current commands or CI.

## Contents

- [Codebase Index](#codebase-index)
- [Quick Start](#quick-start)
- [Package Manifest](#package-manifest)
- [CLI Roles](#cli-roles)
- [Compilation Pipeline](#compilation-pipeline)
- [Standard Library](#standard-library)
- [Language Snapshot](#language-snapshot)
- [Commandments](#commandments)
- [Language Orientation](#language-orientation)
- [Common LLM Failure Modes](#common-llm-failure-modes)
- [Website / Documentation Site](#website--documentation-site)
- [Archive Note](#archive-note)

## Codebase Index

| Area | Path | Use When |
| ---- | ---- | -------- |
| Workspace manifest | [`Cargo.toml`](Cargo.toml) | You need the active Cargo workspace members. |
| User tool | [`crates/faber`](crates/faber) | You are changing package/project behavior, `faber build`, `faber run`, `faber test`, `faber explain`, manifests, or generated Cargo layouts. |
| Compiler | [`crates/radix`](crates/radix) | You are changing lexing, parsing, semantic analysis, HIR/MIR, diagnostics, target codegen, or the `radix` developer CLI. |
| Rust runtime | [`crates/norma`](crates/norma) | You are changing runtime support used by generated Rust and stdlib-backed features. |
| Stdlib source | [`stdlib/norma`](stdlib/norma) | You are changing Faber stdlib declarations, HAL contracts, or `@ verte` target translation metadata. |
| Examples | [`examples/exempla`](examples/exempla) | You need small Faber programs for syntax, behavior, and backend coverage. Treat older migration examples with care. |
| Grammar | [`EBNF.md`](EBNF.md) | You need the canonical grammar and spec commentary. |
| Explain corpus | [`explain`](explain) | You need user-facing keyword/glyph docs embedded by `faber explain`. |
| Docs | [`docs`](docs) | You need delivery plans, release notes, and design history. |
| Website / Docs Site | [`website`](website) | You are updating the public static documentation site (templates, styles, curated content sources). The site is part of the monorepo so docs stay in sync with the compiler. |
| Scripts | [`scripta`](scripta) | You need repo-local CI, test, lint, or helper wrappers. |
| macOS host placeholder | [`hosts/macos-arm64`](hosts/macos-arm64) | You are looking at future host runtime work for Faber-produced Wasm components on macOS arm64. |

## Quick Start

Install the current released CLI with Homebrew:

```bash
brew install ianzepp/tap/faber
faber --version
```

The `faber` crate name on crates.io is not this project. Use the Homebrew tap or
build from this repository.

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

## CLI Roles

**Faber** is the user-facing project and package tool:

- `faber check`, `faber build`, `faber targets`
- `faber explain <term>`, `faber explain --search <query>`, `faber explain --list`, `faber explain --json <term>`
- `faber init`
- `faber run`; `faber test`
- Compatibility aliases: `faber lex`, `faber parse`, `faber hir`, `faber emit`, `faber cli-ir`

**Radix** is the compiler-developer tool:

- `radix lex`, `radix parse`, `radix hir`, `radix mir`, `radix emit`, `radix check`, `radix targets`, `radix cli-ir`
- Package policy lives in Faber; use `faber check --package` and `faber build` for local packages.

v0.33 shipped a single `faber` binary that combined both layers. v0.34 split the
binaries while keeping Faber compatibility aliases for compiler inspection
commands.

## Compilation Pipeline

The active compiler authority is Radix. The main HIR-backed pipeline is:

```text
Source (.fab)
  -> Lex
  -> Parse
  -> Collect + Resolve + Lower
  -> Typecheck + Analysis
  -> Codegen (Rust, Faber, TypeScript, or Go)
```

The active compiler also has a validated MIR inspection branch:

```text
Lex -> Parse -> Collect -> Resolve -> Lower -> Typecheck -> Analysis
  -> typed HIR -> default target codegen
  -> validated MIR -> inspection / internal executable probe
```

`radix mir` prints the validated middle IR for compiler-development inspection.
Normal Rust output still uses the stable HIR-to-Rust backend; the MIR Rust probe
is an internal boundary test, not the user-facing Rust backend.

Current target shape:

| Target | Status |
| ------ | ------ |
| Rust | Primary executable backend; full package build, run, and test through `faber`. |
| Faber | Canonical pretty-printer and round-trip target. |
| TypeScript | File emission supported; package compilation is not yet supported. |
| Go | File emission supported; package compilation is not yet supported. |

## Standard Library

Faber stdlib definitions live in [`stdlib/norma`](stdlib/norma):

```fab
@ verte ts "push"
@ verte rs "push"
functio appende(T elem) → vacuum
```

Runtime-backed Rust support lives in [`crates/norma`](crates/norma). The stdlib
source is Faber declaration metadata first: source code speaks in Faber's
semantic vocabulary, while backend-specific names remain `@ verte` translation
metadata.

## Language Snapshot

Faber is type-first and glyph-forward:

```fab
functio divide(numerus a, numerus b) → numerus ∪ nihil {
    si b ≡ 0 ∴ redde nihil
    redde a / b
}

genus Point {
    numerus x
    numerus y
}

incipit {
    fixum _ p ← Point {
        x = 10,
        y = 20
    }

    nota "Salve, §"(p.x)
}
```

The fastest way to recognize Faber:

- Types come before names: `textus nomen`, not `nomen: textus`.
- Runtime binding and assignment use `←`.
- Function returns use `→`.
- Compact branch bodies use `∴` or accepted alias `ergo`.
- Nullable values use `T ∪ nihil`.
- Latin words carry declarations, statements, lifecycle, and behavior.
- Glyphs carry value flow, type flow, and structural joints.

Use [`EBNF.md`](EBNF.md) as the formal grammar and spec-commentary source. Use
`faber explain <term>` or the Markdown files in [`explain`](explain) for
user-facing reference text.

## Commandments

These are the rules that make Faber feel like Faber. Syntax can evolve, but
changes should preserve these laws.

1. **Types before names.** Declarations read from shape to binding: `textus nomen`, `numerus aetas`, `functio salve(textus name) → textus`.
2. **Mechanical over magical.** The same construct should mean the same thing everywhere. If a reader needs distant context to know what a symbol does, the syntax is suspect.
3. **Glyphs carry structure.** Use glyphs for operators, control-flow joints, and value-flow edges: `←`, `→`, `⇥`, `∴`, `≡`, `∪`.
4. **Latin carries behavior.** Use Latin words for declarations, statements, lifecycle, and behavioral intent: `functio`, `genus`, `fixum`, `varia`, `redde`, `cape`.
5. **Conjugation carries time and flow.** When the same root logic can run synchronously, asynchronously, or as a generator, the conjugated form should carry that execution mode.
6. **One sign, one job.** A glyph or keyword may have exact aliases, but it should not carry unrelated meanings. Aliases must point back to one canonical concept.
7. **Runtime flow is explicit.** Runtime binding, reassignment, and mutation use `←`; structural definition uses `=`.
8. **Absence is typed.** Nullable value types are written as unions, such as `T ∪ nihil`; optional declaration slots use post-name markers such as `sponte`.
9. **The compiler does not guess to hide missing information.** Missing type information is an analysis problem to fix upstream, not a codegen detail to paper over.

## Language Orientation

This section is written for readers and LLMs that already know languages like
Python, Rust, TypeScript, and Go. It is an orientation guide, not the grammar
reference.

The short version:

- Python: say the thing directly.
- Rust: prove the thing mechanically.
- TypeScript: describe the thing structurally and let the ecosystem carry the runtime.
- Go: keep the thing ordinary and explicit.
- Faber: state the thing in a regular, reviewable form before it lowers to a target language.

Faber's center is not "Latin Python" or "Rust with different keywords." It is a
small source language that uses type-first declarations, Latin behavioral words,
and structural glyphs to make program intent easy to scan and hard to confuse.

### How Faber Feels

Faber feels declarative, narrated, and review-oriented.

This target-language shape is not Faber:

```text
functio divide(numerus a, numerus b) -> numerus
```

Faber writes the type first and uses the canonical return glyph:

```fab
functio divide(numerus a, numerus b) → numerus
```

Local binding is not `let name: Type = value`; it is a declaration role, a type,
a name, and runtime binding:

```fab
fixum numerus count ← 0
varia textus label ← "ready"
```

If the initializer determines the type, use `_`:

```fab
fixum _ names ← ["Marcus", "Julia"]
```

Declarations read from semantic shape toward binding name. Runtime value flow is
marked with `←`; compile-time definition uses `=`.

### Compared To Python

Python optimizes for immediate expression:

```python
def greet(name):
    return f"Salve, {name}"
```

The closest Faber shape is more explicit:

```fab
functio greet(textus name) → textus {
    redde "Salve, §"(name)
}
```

Python often relies on surrounding context and library convention. Faber puts
more of the shape in the source line itself: `textus name`, `→ textus`, `redde`,
and string-template application rather than target-specific interpolation.

For quick scripts, Python is more fluid. For source that should be inspected by
another tool or another person before lowering to Rust, TypeScript, or Go, Faber
prefers steadier structure.

### Compared To Rust

Rust makes ownership, mutability, lifetimes, traits, and failure extremely
visible. Faber borrows some of that seriousness but not Rust's full semantic
pressure.

Rust:

```rust
fn divide(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        return None;
    }
    Some(a / b)
}
```

Faber:

```fab
functio divide(numerus a, numerus b) → numerus ∪ nihil {
    si b ≡ 0 ∴ redde nihil
    redde a / b
}
```

The nullable type is explicit, but it is written as a value-domain union:

```fab
numerus ∪ nihil
```

Do not invent `numerus?`, `Option<numerus>`, or `nihil numerus`.

Faber has ownership-flavored markers such as `de`, `in`, and `ex`, and Radix
enables Rust-specific borrow analysis for the Rust target. Still, Faber is not
Rust. Its source is more about preserving high-level intent and giving the
compiler enough structure to lower honestly. When Rust-specific representation
or borrow facts matter, check Radix and the Rust backend.

### Compared To TypeScript

TypeScript and Faber both have structural instincts, but they put structure in
different places.

TypeScript:

```ts
type UserId = number;

interface Drawable {
  draw(): void;
}
```

Faber:

```fab
typus UserId = numerus

pactum Drawable {
    functio draw() → vacuum
}
```

Faber declarations are more uniform and less punctuation-heavy. `genus` covers
class or struct-shaped declarations; `pactum` covers interface-shaped contracts;
`discretio` covers tagged unions.

Object construction is also declarative:

```fab
genus Point {
    numerus x
    numerus y
}

fixum _ p ← Point {
    x = 10,
    y = 20
}
```

`=` defines fields inside a structural literal or declaration metadata. `←`
binds the runtime value to `p`.

### Compared To Go

Go and Faber both value plain control flow, but Faber is more grammatical.

Go:

```go
for _, item := range items {
    if item == target {
        return item
    }
}
```

Faber:

```fab
itera ex items fixum item {
    si item ≡ target ∴ redde item
}
```

Iteration says whether it is walking values, keys, or a range:

```fab
itera ex items fixum item { ... }  # values
itera de table fixum key { ... }   # keys
itera pro 0‥10 fixum i { ... }     # range
```

Branch bodies use a block or a one-statement consequent introduced by `∴` or its
accepted Latin alias `ergo`:

```fab
si score ≥ 90 ∴ redde "A"
sin score ≥ 80 ∴ redde "B"
secus ∴ redde "C"
```

Prefer `∴` in canonical examples. `ergo` is accepted, but glyph-first source is
the current style for structural joints.

### Canonical Surface

When generating or reviewing Faber, default to these forms:

```fab
fixum textus name ← "Marcus"
varia numerus count ← 0
functio greet(textus name) → textus { ... }
si ready ∴ redde value
fixum _ n ← raw ⇒ numerus vel 0
fixum _ x ← value ∷ textus
functio maybe() → textus ∪ nihil { ... }
```

Avoid these stale or non-canonical forms:

```fab
name: textus
functio greet(name: textus): textus
textus?
Option<textus>
tempta { ... } cape err { ... }
demum { ... }
value qua textus
value innatum textus
novum Point { x = 1 }
```

Current error handling uses `fac` with `cape` for a local recoverable boundary:

```fab
fac {
    iace "bad input"
} cape err {
    nota err
}
```

`tempta` is legacy and Radix rejects it with a migration diagnostic. `demum`
cleanup semantics are deferred.

### Glyphs And Words

Faber uses glyphs where the symbol is structural:

- `←` for runtime binding and assignment.
- `→` for function return type.
- `⇥` for alternate recoverable exit type.
- `∴` for therefore or compact body.
- `≡` and `≠` for equality.
- `∪` for inline union types.
- `∷` for static type ascription.
- `⇒` for runtime conversion.

Faber uses Latin words where the construct has behavioral or grammatical shape:

- `functio`, `genus`, `pactum`, `typus`, `ordo`, `discretio`.
- `fixum`, `varia`, `redde`, `iace`, `mori`, `tacet`.
- `si`, `sin`, `secus`, `dum`, `itera`, `elige`, `discerne`.
- `fac`, `cape`, `cura`.

A good Faber source file should look like a stable grammar, not like a target
language wearing Latin labels.

### Nullability And Optionality

Faber distinguishes absence in a value from optional provision at a declaration
site.

Use `T ∪ nihil` when the value can be absent:

```fab
functio find(textus key) → numerus ∪ nihil
```

Use `sponte` after the name when a parameter or field may be omitted by the
caller or constructor:

```fab
functio connect(textus host, numerus port sponte) → vacuum

genus User {
    textus email sponte
}
```

Use `fixus` after the name for late-initialized slots that become fixed after
assignment. The canonical combined order is `sponte fixus`.

Do not use `ignotum` as a nullability escape. `ignotum` is the top-level unknown
type for escape hatches and incomplete knowledge, not "maybe null."

### Conversion And Construction

There are two important arrows:

```fab
fixum _ parsed ← "42" ⇒ numerus
fixum _ safe ← input ⇒ numerus vel 0
fixum _ text ← value ∷ textus
```

Use `⇒` for runtime conversion, especially parsing or coercion that may need a
fallback.

Use `∷` for explicit static type ascription. It is postfix and target-type
driven. Prefer ordinary typed construction and `vacua` where they say the same
thing more directly:

```fab
fixum lista<numerus> xs ← vacua
fixum _ p ← Point { x = 10, y = 20 }
```

### Best Mental Model

Faber is a readable semantic staging language.

It is more formal than Python, less mechanically forceful than Rust, more
grammar-shaped than TypeScript, and less deliberately plain than Go. Its value
is strongest when the source needs to be reviewed, transformed, and lowered
without losing the reader's sense of intent.

The guiding question for Faber source is:

> Can a reader see the semantic role of every construct before knowing the
> target backend?

If yes, the code probably feels like Faber. Opus bene formatum est.

## Common LLM Failure Modes

These are the mistakes to avoid during warm-up or code generation:

- Writing `name: Type` instead of `Type name`.
- Using ASCII arrows when glyph forms are canonical.
- Treating every file under `examples/exempla` as canonical current style. Some example names and files preserve older surfaces for migration or coverage.
- Reintroducing old aliases such as `qua`, `innatum`, or `novum` as conversion syntax.
- Using `tempta` or `demum` for current error handling.
- Guessing missing types in codegen-shaped explanations. Missing type information is an upstream analysis issue.
- Assuming package builds work for every file-emission target. Package builds are Rust-backed today.
- Treating `faber` and `radix` as competing compilers. `faber` is the user tool; Radix is the compiler implementation and developer inspection tool.

## Website / Documentation Site

The public Faber documentation site lives in [`website/`](website/) as part of this monorepo.

- Presentation layer: `templates/layout.html` + `styles/main.css`
- Curated content sources: `content/`
- Legacy pre-migration content (for reference during refresh): `content/legacy-from-faber-www/`
- Migration plan and history: `docs/faber-website-refresh-plan.md`

The site is intentionally lightweight and static. A repo-local generator (Rust/xtask preferred) will be added to produce `website/dist/`, `llms.txt`, and `faber-complete.md` from the content while pulling live grammar and examples from the repository root. See the plan doc for phases, acceptance criteria, and open questions.

Until the generator lands, the assets here are the source of truth for any manual or external publishing process.

## Archive Note

Historical bootstrap compilers, self-hosting sources, old reference code, and
old test harnesses are preserved in `../faber-archivum`. They are not current
main-repo commands or CI surfaces.

Opus perfectum est when `./scripta/ci` passes.
