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
- [Codegen Targets and HIR/MIR Split](#codegen-targets-and-hir-mir-split)
- [Exempla End-to-End Harnesses](#exempla-end-to-end-harnesses)
- [Standard Library](#standard-library)
- [Language Snapshot](#language-snapshot)
- [Commandments](#commandments)
- [Language Orientation](#language-orientation)
- [Runtime binding vs structural definition](#runtime-binding--vs-structural-definition)
- [String-template application](#string-template-application-textus)
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
| Boundary fixtures | [`examples/fixtures/exempla-boundary`](examples/fixtures/exempla-boundary) | You need proba package fixtures, harness edge cases, or negative selection examples. |
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
cargo run -p faber -- test examples/fixtures/exempla-boundary/proba/packages/passing
cargo run -p faber -- emit -t rust examples/exempla/salve-munde.fab
cargo run -p faber -- emit -t ts examples/exempla/salve-munde.fab
cargo run -p faber -- emit -t go examples/exempla/salve-munde.fab
cargo run -p faber -- emit -t wasm-text examples/exempla/salve-munde.fab

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

The active compiler authority is Radix. Every target shares the same frontend:

```text
Source (.fab)
  -> Lex
  -> Parse
  -> Collect + Resolve + Lower
  -> Typecheck + Analysis
```

After analysis, Radix chooses one of two emission paths:

```text
HIR backends (typed HirProgram + TypeTable)
  -> Rust | Faber | TypeScript | Go

MIR backends (validated MirProgram)
  -> wasm-text (.wat probe) | llvm-text (LLVM IR text probe)
```

`radix mir` prints the validated middle IR for compiler-development inspection.
User-facing Rust still comes from the stable HIR-to-Rust backend in
`crates/radix/src/codegen/rust/`. The MIR path is where Wasm and LLVM text probes
grow before any binary Component Model or native codegen work lands.

## Codegen Targets and HIR/MIR Split

See `faber targets` or `radix targets` for live capability flags. Package builds
from `faber.toml` remain **Rust-only** today; other targets support single-file
`check`, `emit`, and `build -t <target>`.

| Target | CLI | Backend path | Package build | Notes |
| ------ | --- | ------------ | ------------- | ----- |
| `rust` | `-t rust` | HIR → `codegen/rust` | Yes | Primary executable backend; `rustc` + Cargo via `faber run` / `faber test`. |
| `faber` | `-t faber` | HIR → `codegen/faber` | No | Canonical pretty-printer and round-trip surface. |
| `ts` | `-t ts` | HIR → `codegen/ts` | No | TypeScript file emission; optional `tsc` / Node in the exempla harness. |
| `go` | `-t go` | HIR → `codegen/go` | No | Go file emission; `gofmt`, `go vet`, and `go run` in the exempla harness. |
| `wasm-text` | `-t wasm-text` (alias `wasm`) | MIR → `mir/wasm_text` | No | Experimental WAT probe with `faber_diag` imports; not a `.wasm` binary backend. |
| `llvm-text` | `-t llvm-text` | MIR → `mir/llvm_text` | No | Experimental LLVM IR text probe; not native codegen. |

**HIR backends** map the typed high-level IR directly to target source. They are
the right place for language-shaped features: generics, classes, failable flow,
and stdlib `@ verte` translation metadata.

**MIR backends** lower the analyzed unit to validated MIR first, then emit a
small probe artifact. This keeps Wasm and LLVM experiments behind one semantic
lowering layer instead of duplicating control-flow and runtime policy in every
HIR backend.

## Exempla End-to-End Harnesses

The compiler ships slow, explicit exempla harnesses under
`crates/radix/src/exempla_e2e/`. They are `#[ignore]` in normal `cargo test` runs;
invoke them when validating a backend lane:

```bash
cargo test -p radix --lib exempla_rust_e2e -- --ignored --nocapture
cargo test -p radix --lib exempla_ts_e2e -- --ignored --nocapture
cargo test -p radix --lib exempla_go_e2e -- --ignored --nocapture
cargo test -p radix --lib exempla_wasm_e2e -- --ignored --nocapture
cargo test -p radix --lib exempla_faber_roundtrip_e2e -- --ignored --nocapture
```

Latest run on this repository (101 files under `examples/exempla/`, Jun 2026):

| Harness | Result | What it exercises |
| ------- | ------ | ----------------- |
| `exempla_rust_e2e` | **101/101** compile + `rustc` + run | HIR → Rust, format/lint hooks, stdout `.expected` checks |
| `exempla_go_e2e` | **94/101** `go run` (7 expected failures) | HIR → Go, `gofmt`, best-effort `go vet`, stdout checks |
| `exempla_ts_e2e` | **100/101** emit; **76/101** `tsc`; **75/101** runnable | HIR → TS; needs `tsc` + `node` for typecheck/run tiers |
| `exempla_wasm_e2e` | **71/101** `wasm-tools validate` | MIR → `.wat`; tier floors in `wasm_expectations.rs` |
| `exempla_faber_roundtrip_e2e` | **91/101** stabilize after one Faber emit | HIR → Faber → re-parse; asserts `salve-munde.fab` |

Toolchain notes from that run:

- **Go** requires `go` on `PATH`.
- **TypeScript** benefits from `tsc` and `node` (formatter/linter tiers skip without
  `prettier`/`deno` or `biome`/`eslint`).
- **Wasm** used `wasm-tools validate` for compile-valid tiers; `wasmtime` was not
  installed, so instantiate/run tiers stayed at 0/101.

The TS harness is informational (tier report, no hard assert). Rust, Go, and Wasm
harnesses enforce expected-failure metadata and tier floors respectively.

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
- Runtime binding and reassignment use `←`; `=` is for structural field shape only.
- Function returns use `→`.
- Compact branch bodies use `∴` or accepted alias `ergo`.
- Nullable values use `T ∪ nihil`.
- Latin words carry declarations, statements, lifecycle, and behavior.
- Glyphs carry value flow, type flow, and structural joints.
- Formatted `textus` uses **string-template application**: a double-quoted template
  with `§` holes, then `(args…)` to fill them — not target `f"…"` / `format!` in source.

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

The lines below can look identical in some fonts because `->` (two ASCII
characters) and `→` (one return glyph) are easy to confuse. Faber is not “the
same function with a prettier arrow.” The whole surface is ordered and marked
differently.

This target-shaped sketch is **not** Faber — it mixes familiar backend habits
(name-after-colon types, ASCII `->`, `?`, `if`/`return`, `==`):

```text
functio divide(a: numerus, b: numerus) -> numerus? {
    if (b == 0) return null
    return a / b
}
```

The Faber shape for the same idea keeps types before names, Latin control words,
and structural glyphs:

```fab
functio divide(numerus a, numerus b) → numerus ∪ nihil {
    si b ≡ 0 ∴ redde nihil
    redde a / b
}
```

| Habit readers often carry over | Not Faber | Faber |
| ------------------------------ | --------- | ----- |
| Parameter and return types | `a: numerus`, `-> numerus?` | `numerus a`, `→ numerus ∪ nihil` |
| Absence / nullability | `?`, `Option<…>`, `null` | `T ∪ nihil`, `nihil` |
| Branch + return | `if (…) return …` | `si … ∴ redde …` |
| Equality | `==` | `≡` |
| Return keyword | `return` | `redde` |

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

Declarations read from semantic shape toward binding name. For the full `←` /
`=` split (Faber does **not** use `=` for runtime assignment), see
[Runtime binding vs structural definition](#runtime-binding--vs-structural-definition).

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

#### Borrowing and mutability (`de`, `in`, `ex`)

Faber marks **how a value is passed** with short prepositions on parameters (and
sometimes on types). They are meant to be easy to scan in review-oriented source;
when the target is Rust, Radix lowers them to references and moves and runs a
borrow check (`crates/radix/src/semantic/passes/borrow.rs`) before codegen.

| Faber (parameter) | Intent | Typical Rust lowering |
| ----------------- | ------ | --------------------- |
| *(none)* | Owned value at the call boundary | `T` / `String` / `Vec<…>` by value |
| `de` | Shared borrow — read, do not move or assign through | `&T` |
| `in` | Mutable borrow — caller's value may change | `&mut T` |
| `ex` | Consume — value is moved into the callee | `T` (by move) |

**Read-only borrow (`de`)** — inspect without taking ownership:

```fab
functio imprime(de textus label) → vacuum {
    nota label
}
```

```rust
fn imprime(label: &str) {
    println!("{}", label);
}
```

**Mutable borrow (`in`)** — callee may update the caller's binding:

```fab
functio duplica(in numerus value) → vacuum {
    value ← value * 2
}
```

```rust
fn duplica(value: &mut i64) {
    *value *= 2;
}
```

**Consume (`ex`)** — callee takes ownership (similar to passing a `String` by value):

```fab
functio consume(ex textus buffer) → textus {
    redde buffer
}
```

```rust
fn consume(buffer: String) -> String {
    buffer
}
```

**Owned (no preposition)** — ordinary by-value parameter; Faber still uses type-first order:

```fab
functio salve(textus nomen) → textus {
    redde "Salve, §!"(nomen)
}
```

```rust
fn salve(nomen: String) -> String {
    format!("Salve, {}!", nomen)
}
```

Borrow markers can also appear on **types** (`de textus`, `in numerus`) and combine
with optional parameters (`de numerus depth sponte` — borrowed slot that may be
absent). See [`examples/exempla/functio/optionalis.fab`](examples/exempla/functio/optionalis.fab).

**Same words, different jobs.** Faber reuses `de` and `ex` in other constructs;
do not read every `ex` as “consume”:

| Surface | Role |
| ------- | ---- |
| `de textus name` on a parameter | Shared borrow |
| `in numerus count` on a parameter | Mutable borrow |
| `ex textus buffer` on a parameter | Move / consume into callee |
| `itera ex items fixum item` | Iterate **values** (for-of shape) |
| `itera de tabula fixum key` | Iterate **keys** (for-in shape) |
| `ex source fixum x, ceteri rest` | Destructure fields into locals |
| `importa ex "path"` | Import from a module path |

Go and TypeScript backends accept the same Faber syntax today, but **borrow
analysis and `&T` / `&mut T` lowering are Rust-target concerns**. Faber is not
Rust: it keeps intent explicit in source without requiring lifetime annotations in
the Faber file. Precise lifetime proof remains the generated Rust compiler's job.

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

### Runtime binding (`←`) vs structural definition (`=`)

Most languages overload `=` for both “define this field in a type/literal” and
“put a runtime value in this variable.” Faber splits those jobs:

| Glyph | Role | Use for |
| ----- | ---- | ------- |
| `←` | **Runtime flow** | Initial binding, reassignment, and mutation at execution time |
| `=` | **Structural shape** | Field names inside literals and declaration metadata — not runtime stores |

**Runtime binding** attaches a value to a name when the program runs:

```fab
fixum numerus count ← 0
varia textus label ← "ready"
count ← count + 1
```

In Rust terms, think `let` / reassignment / `*x = …` on a binding — but in Faber
the glyph is always `←`, not `=`.

**Structural definition** lists members inside a composite value or declaration:

```fab
genus Point {
    numerus x
    numerus y
}

fixum _ p ← Point {
    x = 10,
    y = 20,
}
```

Here `x = 10` and `y = 20` spell the **shape** of the `Point` value. The runtime
step is the outer `fixum _ p ← Point { … }`: create the value, then bind `p`.

**What not to write.** These are foreign habits, not Faber:

```text
varia numerus count = 0          # = is not runtime assignment
fixum _ p = Point { x = 1 }     # use ← to bind the whole value
```

Faber keeps `=` from meaning “store into this variable at runtime,” which makes
review easier: every `←` is live data flow; every `=` inside `{ … }` is field
layout. Compare `∷` (static ascription) and `⇒` (runtime conversion) in
[Conversion And Construction](#conversion-and-construction) — three different
arrows, three different jobs.

### String-template application (`textus`)

Faber formats text with **string-template application**: a string literal in
**double quotes**, optional `§` placeholders inside it, then a parenthesized
argument list that supplies values in order. This is the canonical source form;
the compiler lowers it to an internal `scriptum("…", …)` call.

```fab
functio greet(textus nomen) → textus {
    redde "Salve, §!"(nomen)
}

fixum _ msg ← "Page § of §"(pagina, totum)
nota "Salve, §"(p.x)
```

Three details matter because the shape is unusual compared to Rust, Python, or Go:

1. **Double-quoted template.** Ordinary `textus` literals and templates use `"…"`.
   Block text may use the `❝ … ❞` delimiters (see [`EBNF.md`](EBNF.md)). Faber is
   **Unicode-first**: source is normalized (NFC) and string indexing is
   Unicode-scalar based, so templates naturally include Latin, glyphs, and `§`.

2. **`§` is the format hole, not a runtime value.** Inside the quotes, `§` marks
   where the next argument is injected. Positional holes are written `§0`, `§1`,
   … when order must be explicit (`"status: §1 (§0)"("ok", sample_status())`).
   A trailing `!` on the hole (as in `"Salve, §!"`) selects display formatting on
   backends that distinguish it; otherwise the compiler picks a sensible default.

3. **Parentheses apply the template.** The call-like suffix `(nomen)` is not a
   function named `"Salve…"` — it is **template application**: build a `textus`
   value by substituting the arguments into the holes. Prefer this over writing
   `format!(…)` / `f"…"` / `fmt.Sprintf(…)` directly in Faber source.

Do not confuse `§` inside a string with **`§` at the start of a line**, which
introduces a file-scope directive (`§ opus`, `§ dependentia`, …). Same glyph,
different grammar role.

For output statements, `nota`, `vide`, and `mone` accept the same template form.
Use `faber explain scriptum` or `faber explain string` for the embedded reference.

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
- `§` inside `"…"` for string-template holes (and at line start for directives).

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
- Using target interpolation (Rust f-strings, JavaScript template literals, `format!`,
  ad-hoc concatenation) instead of Faber string-template application (`"Salve, §!"(name)`).
- Using `=` for runtime assignment or rebinding; Faber uses `←` for live flow and reserves
  `=` for structural fields inside literals and declarations.
- Using `tempta` or `demum` for current error handling.
- Guessing missing types in codegen-shaped explanations. Missing type information is an upstream analysis issue.
- Assuming package builds work for every file-emission target. Package builds are Rust-backed today.
- Treating `faber` and `radix` as competing compilers. `faber` is the user tool; Radix is the compiler implementation and developer inspection tool.

## Website / Documentation Site

The public Faber documentation site lives in [`website/`](website/) as part of this monorepo.

- Presentation layer: `templates/layout.html` + `styles/main.css`
- Curated content sources: `content/`
- Legacy pre-migration content (for reference during refresh): `content/legacy-from-faber-www/`
- Migration plan and history: [`website/docs/faber-website-refresh-plan.md`](website/docs/faber-website-refresh-plan.md)

The site is intentionally lightweight and static. A repo-local generator (Rust/xtask preferred) will be added to produce `website/dist/`, `llms.txt`, and `faber-complete.md` from the content while pulling live grammar and examples from the repository root. See the plan doc for phases, acceptance criteria, and open questions.

Until the generator lands, the assets here are the source of truth for any manual or external publishing process.

## Archive Note

Historical bootstrap compilers, self-hosting sources, old reference code, and
old test harnesses are preserved in `../faber-archivum`. They are not current
main-repo commands or CI surfaces.

Opus perfectum est when `./scripta/ci` passes.
