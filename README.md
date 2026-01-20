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
ex items fixum item {
    si item.price > 100 {
        scribe item.name
    }
}
```

- **LLMs write Faber.** Word-based, regular syntax. No lifetime annotations, no pointer semantics, no template noise. One language regardless of compile target.
- **Humans skim Faber.** You see `si` (if), `pro` (for), `scribe` (print). You don't need to know Zig to verify the loop logic is correct.
- **Compiler emits target code.** Zig, Rust, C++, TypeScript, or Python. The generated code is what actually runs — you never need to read it.

The workflow: LLM drafts Faber → Human approves → Compiler emits production code.

## Why It Works

**No ecosystem problem.** Faber compiles to the target language, so you use its libraries directly. `§ ex "hono" importa Hono` becomes `import { Hono } from 'hono'`. No need to rewrite npm/PyPI/crates.io in a new language.

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
- **Imports**: `§ ex "hono" importa Hono` → `import { Hono } from 'hono'`
- **File-level directives**: `§ dependentia "hono" github "honojs/hono#main" via "."`

The `§` symbol (section marker) distinguishes file-level configuration and imports from executable code. Use `@` for code annotations like `@ futura`, `@ radix`, `@ verte`.

## Principles

**LLM-First, Human-Readable.** The language is optimized for LLMs to write and humans to review. Not the other way around. Humans don't type Faber; they approve it.

**Compiler as Safety Net.** The compiler never crashes on malformed input — it collects errors and continues. When an LLM generates broken code, you see all the issues at once, not one at a time.

**Target Transparency.** You pick a compile target (TypeScript, Zig, etc.) and use that ecosystem directly. Faber is a skin over the target, not a replacement for it.

**Target Compatibility.** Not all targets support all features — the compiler validates your code against target capabilities and reports clear errors. See [fons/grammatica/targets.md](fons/grammatica/targets.md) for the compatibility matrix.

## Quick Start

```bash
bun install
bun run build                                             # Build all compilers
./opus/bin/nanus-go compile fons/exempla/salve-munde.fab -t ts  # TypeScript (recommended)
./opus/bin/nanus-go compile fons/exempla/salve-munde.fab -t go  # Go (recommended)
./opus/bin/faber run fons/exempla/salve-munde.fab         # Legacy: compile and run (TS only)
bun test                                                  # Run tests
```

## Project Stats

| Component             |       Lines |   Files | Description                                   |
| --------------------- | ----------: | ------: | --------------------------------------------- |
| **Compiler (faber)**  |      34,729 |     141 | Reference compiler in TypeScript (deprecated) |
| **Bootstrap (rivus)** |      35,751 |     312 | Self-hosting compiler in Faber                |
| **Micro-compilers**   |             |         |                                               |
|   **nanus-ts**        |             |         | TypeScript compiler in TypeScript             |
|   **nanus-go**        |             |         | Go compiler in Go                             |
|   **glyph-go**        |             |         | Fun translator in Go                          |
| **Tests**             |      12,002 |       — | Test infrastructure                           |
| **Test Specs**        |      19,573 |       — | YAML test definitions (0 passing)             |
| **Core Phases**       |       5,897 |       3 | Tokenizer, parser, semantic analyzer          |
| **Codegen**           |       7,957 |     171 | Code generators for 3 targets                 |
| **Documentation**     |       5,345 |       — | Grammar spec + prose tutorials                |
| **Examples**          |           0 |       0 | Sample Faber programs                         |
| **Total**             | **121,254** | **627** | Complete implementation                       |

**Primary Compiler:** nanus-go (Go, supports -t ts and -t go targets)
**Compilation Targets:** TypeScript, Go, Faber (round-trip)

## Block Syntax Patterns

Faber uses a consistent `keyword expr VERB name { body }` pattern for scoped constructs. Note: `pro` bindings have been replaced with explicit `fixum`/`varia` declarations:

| Construct       | Syntax                                    | Binding | Purpose        |
| --------------- | ----------------------------------------- | ------- | -------------- |
| `ex...fixum`    | `ex expr fixum name { }`                  | `name`  | iterate values |
| `ex...varia`    | `ex expr varia name { }`                  | `name`  | iterate values |
| `de...fixum`    | `de expr fixum name { }`                  | `name`  | iterate keys   |
| `de...varia`    | `de expr varia name { }`                  | `name`  | iterate keys   |
| `cura...fixum`  | `cura expr fixum name { }`                | `name`  | resource scope |
| `cura...varia`  | `cura expr varia name { }`                | `name`  | resource scope |
| `probandum`     | `probandum "label" { }`                   | —       | test suite     |
| `proba`         | `proba "label" { }`                       | —       | test case      |
| `praepara`      | `praepara { }`                            | —       | setup          |
| `postpara`      | `postpara { }`                            | —       | teardown       |
| `tempta...cape` | `tempta { } cape err { }`                 | `err`   | error handling |
| `fac...cape`    | `fac { } cape err { }`                    | `err`   | scoped block   |
| `dum`           | `dum expr { }`                            | —       | while loop     |
| `si`            | `si expr { }`                             | —       | conditional    |
| `custodi`       | `custodi { si expr { } }`                 | —       | guard clauses  |
| `elige`         | `elige expr { casu val { } }`             | —       | switch         |
| `discerne`      | `discerne expr { casu Variant ut x { } }` | `x`     | pattern match  |
| `in`            | `in expr { }`                             | —       | mutation scope |

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

## Return Type Verbs

Function return types use arrow syntax (`->`) with optional annotations for async and generator semantics:

```fab
functio parse() -> numerus                        # sync function
@ futura
functio fetch() -> textus                         # async function
@ cursor
functio items() -> numerus                         # generator function
@ futura
@ cursor
functio stream() -> datum                          # async generator
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

The morphology system is implemented via `@ radix` and `@ verte` annotations in the standard library definitions. `@ radix` declares the verb stem and valid morphological forms, while `@ verte` provides target-specific code generation for each form. The compiler validates morphology usage against declared forms. See [fons/grammatica/morphologia.md](fons/grammatica/morphologia.md) for the complete specification.

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
