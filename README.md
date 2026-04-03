# Faber Romanus

**The Roman Craftsman** — An LLM-oriented intermediate representation that compiles to Zig, Rust, C++, TypeScript, or Python.

**[faberlang.dev](https://faberlang.dev)** — Official project website

## The Problem

LLMs write code. Humans review it. But systems languages — Zig, Rust, C++ — are hard for both:

- **LLMs struggle with symbol-dense syntax.** Lifetimes, borrow checkers, template metaprogramming, `&&` vs `&`, `->` vs `.` — these create semantic chaos that increases error rates.
- **Humans can't skim generated code.** Reviewing 50 lines of Rust requires understanding Rust. You can't verify "yes, that logic looks right" without parsing the syntax mentally.

You don't need an IR to generate TypeScript. You need one to generate Rust without lifetime annotation chaos.

## The Solution

Faber Romanus is an **intermediate representation** optimized for LLM generation and human review.

```
itera ex items fixum item {
    si item.price > 100 {
        scribe item.name
    }
}
```

- **LLMs write Faber.** Word-based, regular syntax. No lifetime annotations, no pointer semantics, no template noise. One language regardless of compile target.
- **Humans skim Faber.** You see `si` (if), `itera` (iterate), `scribe` (print). You don't need to know Zig to verify the loop logic is correct.
- **Compiler emits target code.** Zig, Rust, C++, TypeScript, or Python. The generated code is what actually runs — you never need to read it.

The workflow: LLM drafts Faber → Human approves → Compiler emits production code.

## Why It Works

**No ecosystem problem.** Faber compiles to the target language, so you use its libraries directly. `importa ex "hono" privata Hono` becomes `import { Hono } from 'hono'`. No need to rewrite npm/PyPI/crates.io in a new language.

**Grammar designed for LLMs.** The [EBNF.md](EBNF.md) specification is built for LLM consumption: formal grammar, type tables, keyword mappings. Trials show models achieve 96-98% accuracy after reading the grammar specification alone — no prose documentation required.

**Regular structure.** Type-first declarations, consistent block patterns, no operator overloading. The regularity may matter more than the vocabulary — a hypothesis we're testing with ablations (same grammar, English keywords).

**Semantic vocabulary.** Latin keywords encode intent: `fixum` (fixed/immutable) vs `varia` (variable/mutable), `cede` (yield/await), `redde` (give back/return). Whether this helps beyond structure is an open question, but it makes code skimmable for humans unfamiliar with the target language.

## Research & Evidence

The [faber-trials](https://github.com/ianzepp/faber-trials) research harness tests Faber's learnability with reproducible evaluation: falsifiable claims, controlled comparisons, automated grading.

### Trial Results (Framework 1.1)

~13,000 trials across 17 models, testing Faber↔TypeScript translation tasks:

| Finding                              | Result                                                                  |
| ------------------------------------ | ----------------------------------------------------------------------- |
| **Faber is learnable**               | 11 of 12 models achieve 86%+ accuracy with grammar-only context         |
| **Grammar beats prose**              | Formal EBNF (87%) matches verbose docs (87%), outperforms minimal (83%) |
| **Reading > Writing**                | Models score 90-95% on Faber→TS but 54-65% on TS→Faber                  |
| **Coding models are cost-effective** | qwen3-coder (96%) matches gpt-4o (98%) at 1/10th cost                   |

Top performers with grammar-only context:

| Model             | Accuracy |
| ----------------- | -------- |
| gpt-4o            | 98%      |
| claude-3.5-sonnet | 98%      |
| qwen3-coder       | 96%      |
| llama-3.1-70b     | 95%      |
| deepseek-v3.1     | 95%      |

**What this validates**: LLMs can learn Faber syntax from a formal grammar specification. TypeScript was used for grading (mature tooling), but the value proposition is strongest for systems languages where direct generation is error-prone.

**Open questions**: Do Latin keywords help, or is it just the regular structure? (Faber-English ablation planned.) How do error rates compare for Faber→Zig vs direct Zig generation?

See [faber-trials/docs/framework-1.1-results.md](https://github.com/ianzepp/faber-trials/blob/main/docs/framework-1.1-results.md) for methodology, or [faber-trials/thesis.md](https://github.com/ianzepp/faber-trials/blob/main/thesis.md) for the research strategy.

## The § Symbol

Faber uses the `§` symbol as a distinctive marker for declarative constructs:

- **String formatting**: `scriptum("Hello, §!", name)` → `"Hello, World!"`
- **Imports**: `importa ex "hono" privata Hono` → `import { Hono } from 'hono'`
- **File-level directives**: `§ dependentia "hono" github "honojs/hono#main" via "."`

The `§` symbol (section marker) distinguishes file-level configuration and imports from executable code. Use `@` for code annotations like `@ futura`, `@ radix`, `@ verte`.

## Principles

**LLM-First, Human-Readable.** The language is optimized for LLMs to write and humans to review. Not the other way around. Humans don't type Faber; they approve it.

**Compiler as Safety Net.** The compiler never crashes on malformed input — it collects errors and continues. When an LLM generates broken code, you see all the issues at once, not one at a time.

**Target Transparency.** You pick a compile target (TypeScript, Zig, etc.) and use that ecosystem directly. Faber is a skin over the target, not a replacement for it.

**Target Compatibility.** Not all targets support all features — the compiler validates your code against target capabilities and reports clear errors. See [docs/grammatica/targets.md](docs/grammatica/targets.md) for the compatibility matrix.

## Repository Contract

Faber is a multi-project family repository. Components live in one tree, but they are not treated as one blocking build surface.

- **Primary delivery target:** [`compilers/radix-rs`](compilers/radix-rs) is the active compiler and the main quality gate.
- **Older project trees are isolated:** bootstrap compilers, rivus, and legacy TypeScript tooling may lag behind without blocking `radix-rs`.
- **Root CI is scoped:** pull requests are gated on `radix-rs` checks, not a repo-wide lint/typecheck sweep.
- **Root inventory lives in [`project.yaml`](project.yaml):** use it to understand project status and ownership at a glance.

Current status model:

- `active` — expected to support ongoing development and blocking checks
- `maintenance` — useful, but not allowed to block unrelated work
- `experimental` — informative or exploratory surfaces that may be broken

## Quick Start

```bash
# radix-rs (primary compiler — Rust target)
cd compilers/radix-rs && cargo build --release
cargo run -- emit examples/exempla/salve-munde.fab        # Emit Rust

# Root-scoped primary checks
bun run check:radix-rs
bun run test:radix-rs
bun run ci

# Bootstrap compilers (TS/Go/Python targets)
bun install
bun run build                                             # Build nanus-* compilers
./opus/bin/nanus-go compile examples/exempla/salve-munde.fab -t ts  # TypeScript
./opus/bin/nanus-go compile examples/exempla/salve-munde.fab -t go  # Go

# Legacy / secondary checks
bun run test:nanus-ts
bun run typecheck:legacy
bun run lint:nanus-ts
```

## Project Structure

| Component               | Description                                           |
| ----------------------- | ----------------------------------------------------- |
| **radix-rs**            | Primary compiler in Rust (~17k LOC, 39 tests)         |
| **rivus**               | Self-hosting compiler in Faber (deprioritized)         |
| **nanus-{ts,go,py,rs}** | Bootstrap compilers (minimal, stable)                  |
| **norma/**              | Stdlib definitions with `@ verte` annotations          |
| **norma-{ts,go,py,rs}** | Target-specific stdlib runtimes                        |
| **proba/**              | YAML test specs (~75 files)                            |
| **exempla/**            | Example .fab programs (~90 files)                      |
| **golden/**             | Golden test outputs                                    |
| **grammatica/**         | Language documentation (prose tutorials)               |

**Primary Compiler:** radix-rs (Rust, emits Rust and Faber targets)
**Bootstrap Compilers:** nanus-go, nanus-ts, nanus-py, nanus-rs (emit TS, Go, Python, Rust)

## Block Syntax Patterns

Faber uses a consistent `keyword expr VERB name { body }` pattern for scoped constructs:

| Construct       | Syntax                                      | Binding | Purpose        |
| --------------- | ------------------------------------------- | ------- | -------------- |
| `itera ex`      | `itera ex expr fixum name { }`              | `name`  | iterate values |
| `itera de`      | `itera de expr fixum key { }`               | `name`  | iterate keys   |
| `cura`          | `cura expr fixum name { }`                  | `name`  | resource scope |
| `tempta...cape` | `tempta { } cape err { }`                   | `err`   | error handling |
| `dum`           | `dum expr { }`                              | —       | while loop     |
| `si`            | `si expr { } secus { }`                     | —       | conditional    |
| `custodi`       | `custodi { si expr { } }`                   | —       | guard clauses  |
| `elige`         | `elige expr { casu val { } }`               | —       | switch         |
| `discerne`      | `discerne expr { casu Variant ut x { } }`   | `x`     | pattern match  |
| `probandum`     | `probandum "label" { }`                     | —       | test suite     |
| `proba`         | `proba "label" { }`                         | —       | test case      |

Bindings use `fixum` (immutable) or `varia` (mutable).

## Primitive Types

| Faber      | TypeScript   | Python     | Zig          | C++                    | Rust      |
| ---------- | ------------ | ---------- | ------------ | ---------------------- | --------- |
| `textus`   | `string`     | `str`      | `[]const u8` | `std::string`          | `String`  |
| `numerus`  | `number`     | `int`      | `i64`        | `int64_t`              | `i64`     |
| `fractus`  | `number`     | `float`    | `f64`        | `double`               | `f64`     |
| `decimus`  | `number`     | `Decimal`  | —            | —                      | —         |
| `magnus`   | `bigint`     | `int`      | `i128`       | —                      | `i128`    |
| `bivalens` | `boolean`    | `bool`     | `bool`       | `bool`                 | `bool`    |
| `nihil`    | `null`       | `None`     | `null`       | `std::nullopt`         | `None`    |
| `vacuum`   | `void`       | `None`     | `void`       | `void`                 | `()`      |
| `numquam`  | `never`      | `NoReturn` | `noreturn`   | `[[noreturn]]`         | `!`       |
| `octeti`   | `Uint8Array` | `bytes`    | `[]u8`       | `std::vector<uint8_t>` | `Vec<u8>` |
| `ignotum`  | `unknown`    | `Any`      | —            | —                      | —         |

## Function Annotations

Function return types use arrow syntax (`->`) with optional annotations for async and generator semantics:

```fab
functio parse() -> numerus                        # sync function
@ futura
functio fetch() -> textus                         # async function
@ cursor
functio items() -> numerus                        # generator function
@ futura
@ cursor
functio stream() -> datum                         # async generator
```

The `@ futura` annotation marks async functions, `@ cursor` marks generators. Arrow syntax (`->`) is the standard return type syntax.

## Method Morphology

Standard library methods use Latin verb conjugations to encode behavior. Instead of separate names like `filter`/`filterAsync`/`filterNew`, Faber uses different endings on the same stem:

| Form                   | Ending        | Behavior                 | Example                         |
| ---------------------- | ------------- | ------------------------ | ------------------------------- |
| **Imperative**         | -a/-e/-i      | Mutates in place, sync   | `adde`, `filtra`                |
| **Perfect**            | -ata/-ita/-sa | Returns new value, sync  | `addita`, `filtrata`, `inversa` |
| **Future Indicative**  | -abit/-ebit   | Mutates in place, async  | `filtrabit`, `scribet`          |
| **Future Active**      | -atura/-itura | Returns new value, async | `filtratura`                    |
| **Present Participle** | -ans/-ens     | Streaming/generator      | `filtrans`, `legens`            |

**Collections** use morphology for mutation vs allocation:

```fab
lista.adde(x)       # mutates list, adds x
lista.addita(x)     # returns NEW list with x added
```

**I/O operations** use morphology for sync vs async vs streaming:

```fab
solum.lege(path)    # sync read (imperative)
solum.leget(path)   # async read (future)
solum.legens(path)  # streaming read (participle)
```

The morphology system is implemented via `@ radix` and `@ verte` annotations in the standard library definitions. `@ radix` declares the verb stem and valid morphological forms, while `@ verte` provides target-specific code generation for each form. The compiler validates morphology usage against declared forms. See [docs/grammatica/morphologia.md](docs/grammatica/morphologia.md) for the complete specification.

## Example

```fab
functio salve(nomen) -> textus {
    redde scriptum("Salve, §!", nomen)
}

fixum nomen = "Mundus"
scribe salve(nomen)
```

Compiles to TypeScript:

```typescript
function salve(nomen): string {
    return 'Salve, ' + nomen + '!';
}

const nomen = 'Mundus';
console.log(salve(nomen));
```
