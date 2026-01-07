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
ex items pro item {
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

**No ecosystem problem.** Faber compiles to the target language, so you use its libraries directly. `ex hono importa Hono` becomes `import { Hono } from 'hono'`. No need to rewrite npm/PyPI/crates.io in a new language.

**Grammar designed for LLMs.** The [EBNF.md](EBNF.md) specification is built for LLM consumption: formal grammar, type tables, keyword mappings. Trials show models achieve 96-98% accuracy after reading the grammar specification alone — no prose documentation required.

**Regular structure.** Type-first declarations, consistent block patterns, no operator overloading. The regularity may matter more than the vocabulary — a hypothesis we're testing with ablations (same grammar, English keywords).

**Semantic vocabulary.** Latin keywords encode intent: `fixum` (fixed/immutable) vs `varia` (variable/mutable), `cede` (yield/await), `redde` (give back/return). Whether this helps beyond structure is an open question, but it makes code skimmable for humans unfamiliar with the target language.

## Research & Evidence

The [probationes](probationes/) research harness tests Faber's learnability with reproducible evaluation: falsifiable claims, controlled comparisons, automated grading.

### Trial Results (Framework 1.1)

~13,000 trials across 17 models, testing Faber↔TypeScript translation tasks:

| Finding | Result |
|---------|--------|
| **Faber is learnable** | 11 of 12 models achieve 86%+ accuracy with grammar-only context |
| **Grammar beats prose** | Formal EBNF (87%) matches verbose docs (87%), outperforms minimal (83%) |
| **Reading > Writing** | Models score 90-95% on Faber→TS but 54-65% on TS→Faber |
| **Coding models are cost-effective** | qwen3-coder (96%) matches gpt-4o (98%) at 1/10th cost |

Top performers with grammar-only context:

| Model | Accuracy |
|-------|----------|
| gpt-4o | 98% |
| claude-3.5-sonnet | 98% |
| qwen3-coder | 96% |
| llama-3.1-70b | 95% |
| deepseek-v3.1 | 95% |

**What this validates**: LLMs can learn Faber syntax from a formal grammar specification. TypeScript was used for grading (mature tooling), but the value proposition is strongest for systems languages where direct generation is error-prone.

**Open questions**: Do Latin keywords help, or is it just the regular structure? (Faber-English ablation planned.) How do error rates compare for Faber→Zig vs direct Zig generation?

See [probationes/docs/framework-1.1-results.md](probationes/docs/framework-1.1-results.md) for methodology, or [probationes/thesis.md](probationes/thesis.md) for the research strategy.

## Principles

**LLM-First, Human-Readable.** The language is optimized for LLMs to write and humans to review. Not the other way around. Humans don't type Faber; they approve it.

**Compiler as Safety Net.** The compiler never crashes on malformed input — it collects errors and continues. When an LLM generates broken code, you see all the issues at once, not one at a time.

**Target Transparency.** You pick a compile target (TypeScript, Zig, etc.) and use that ecosystem directly. Faber is a skin over the target, not a replacement for it.

## Quick Start

```bash
bun install
bun run fons/cli.ts compile exempla/fundamenta/salve.fab        # TypeScript (default)
bun run fons/cli.ts compile exempla/fundamenta/salve.fab -t zig # Zig
bun run fons/cli.ts run exempla/fundamenta/salve.fab            # Compile and run
bun test                                                         # Run tests
```

## Project Stats

| Component | Lines | Files | Description |
|-----------|------:|------:|-------------|
| **Compiler (faber)** | 34,208 | 334 | Reference compiler in TypeScript |
| **Bootstrap (rivus)** | 23,236 | 204 | Self-hosting compiler in Faber |
| **Tests** | 7,128 | — | Test infrastructure |
| **Test Specs** | 18,271 | — | YAML test definitions (3,267 passing) |
| **Core Phases** | 9,392 | 3 | Tokenizer, parser, semantic analyzer |
| **Codegen** | 17,592 | 314 | Code generators for 6 targets |
| **Documentation** | 4,189 | — | Grammar spec + prose tutorials |
| **Examples** | 286 | 4 | Sample Faber programs |
| **Research** | 2,087 | — | LLM learnability harness |
| **Total** | **108,997** | **859** | Complete implementation |

**Compilation Targets:** TypeScript, Zig, Python, Rust, C++, Faber (round-trip)

## Block Syntax Patterns

Faber uses a consistent `keyword expr VERB name { body }` pattern for scoped constructs:

| Construct         | Syntax                                   | Binding | Purpose        |
| ----------------- | ---------------------------------------- | ------- | -------------- |
| `ex...pro`        | `ex expr pro name { }`                   | `name`  | iterate values |
| `ex...fiet`       | `ex expr fiet name { }`                  | `name`  | async iterate  |
| `de...pro`        | `de expr pro name { }`                   | `name`  | iterate keys   |
| `cura...fit`      | `cura expr fit name { }`                 | `name`  | resource scope |
| `cura cede...fit` | `cura cede expr fit name { }`            | `name`  | async acquire  |
| `probandum`       | `probandum "label" { }`                  | —       | test suite     |
| `proba`           | `proba "label" { }`                      | —       | test case      |
| `cura ante`       | `cura ante { }`                          | —       | before each    |
| `cura post`       | `cura post { }`                          | —       | after each     |
| `tempta...cape`   | `tempta { } cape err { }`                | `err`   | error handling |
| `fac...cape`      | `fac { } cape err { }`                   | `err`   | scoped block   |
| `dum`             | `dum expr { }`                           | —       | while loop     |
| `si`              | `si expr { }`                            | —       | conditional    |
| `custodi`         | `custodi { si expr { } }`                | —       | guard clauses  |
| `elige`           | `elige expr { si val { } }`              | —       | switch         |
| `discerne`        | `discerne expr { si Variant pro x { } }` | `x`     | pattern match  |
| `in`              | `in expr { }`                            | —       | mutation scope |

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

Function return types use Latin verb forms of `fio` ("to become") to encode async and generator semantics:

| Verb    | Async | Generator | Meaning            | Example                         |
| ------- | :---: | :-------: | ------------------ | ------------------------------- |
| `fit`   |  no   |    no     | "it becomes"       | `functio parse() fit numerus`   |
| `fiet`  |  yes  |    no     | "it will become"   | `functio fetch() fiet textus`   |
| `fiunt` |  no   |    yes    | "they become"      | `functio items() fiunt numerus` |
| `fient` |  yes  |    yes    | "they will become" | `functio stream() fient datum`  |

The verb alone carries the semantic — no `futura` or `cursor` prefix needed. Arrow syntax (`->`) is also supported but requires explicit prefixes for async/generator behavior.

## Example

```fab
functio salve(nomen) -> textus {
    redde "Salve, " + nomen + "!"
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
