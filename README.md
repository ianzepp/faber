# Faber Romanus

**The Roman Craftsman** — A Latin programming language that compiles to TypeScript, Python, Zig, C++, or Rust.

## The Problem

LLMs write code. Humans review it. But the target languages — especially systems languages like Zig, C++, and Rust — are hard for both:

- **LLMs struggle with symbol-dense syntax.** Lifetimes, borrow checkers, template metaprogramming, `&&` vs `&`, `->` vs `.` — these create semantic chaos that increases error rates.
- **Humans can't skim generated code.** Reviewing 50 lines of Rust requires understanding Rust. You can't quickly verify "yes, that logic looks right" without parsing the syntax mentally.

## The Solution

Faber Romanus is a **human-readable intermediate language** for LLM-generated code.

```
ex items pro item {
    si item.price > 100 {
        scribe item.name
    }
}
```

- **LLMs write Latin.** Word-based, consistent syntax. No lifetime annotations, no pointer semantics, no template noise. One language regardless of compile target.
- **Humans skim Latin.** You see `si` (if), `pro` (for), `scribe` (print). You don't need to know Zig to verify the loop logic is correct.
- **Compiler emits target code.** TypeScript, Python, Zig, C++, or Rust. The generated code is what actually runs — you never read it.

The workflow: LLM writes Faber → Human approves → Compiler emits production code.

## Why It Works

**No ecosystem problem.** Faber compiles to the target language, so you use its libraries directly. `ex hono importa Hono` becomes `import { Hono } from 'hono'`. No need to rewrite npm/PyPI/crates.io packages in Latin.

**Grammar designed for LLMs.** The [GRAMMAR.md](GRAMMAR.md) file is built for LLM consumption: type tables, keyword references, style guides, and complete examples. An LLM can read it once and generate valid Faber immediately.

**Semantics, not syntax.** Latin keywords encode meaning: `fixum` (fixed/immutable) vs `varia` (variable/mutable), `cede` (yield/await), `redde` (give back/return). The code reads like intent, not implementation.

**Cross-model validation.** We asked GPT, Gemini, and Claude to independently review Faber source code and describe how it "feels" to read. All three converged on the same observations: "low-entropy", "predictable", "industrial", "Roman". The verb conjugation system (`fit`/`fiet`/`fiunt`/`fient`) encoding async/generator semantics was called "fascinating" — it eliminates modifier stacking while preserving clarity. When three competing model families agree that code is "unbreakable", that's signal.

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

---

# Implementation Status

| Target     | Tests | Status |
| ---------- | ----: | :----: |
| TypeScript |   760 |  100%  |
| Python     |   613 |  81%   |
| Rust       |   580 |  76%   |
| C++23      |   475 |  63%   |
| Zig        |   470 |  62%   |

> Status % = passing tests / TypeScript baseline. Run `bun test proba/runner.test.ts -t "@<target>"` to verify (e.g., `-t "@zig"`).

Status key: `[x]` implemented, `[~]` partial, `[ ]` not implemented, `[-]` not applicable, `[c]` convention (no compiler support needed)

## Type System

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `textus` (string)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `numerus` (integer)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `fractus` (float)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `decimus` (decimal)       |    [x]     | [~] |  [x]   | [ ]  |  [~]  |
| `magnus` (bigint)         |    [x]     | [~] |  [x]   | [x]  |  [ ]  |
| `bivalens` (boolean)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `nihil` (null)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `vacuum` (void)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `numquam` (never)         |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `octeti` (bytes)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `objectum` (object)       |    [x]     | [~] |  [x]   | [ ]  |  [~]  |
| `lista<T>` (array)        |    [x]     | [~] |  [x]   | [x]  |  [x]  |
| `tabula<K,V>` (map)       |    [x]     | [~] |  [x]   | [x]  |  [x]  |
| `copia<T>` (set)          |    [x]     | [~] |  [x]   | [x]  |  [x]  |
| `series<T...>` (tuple)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `promissum<T>` (promise)  |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `erratum` (error)         |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `cursor<T>` (iterator)    |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `ignotum` (unknown)       |    [x]     | [x] |  [x]   | [ ]  |  [ ]  |
| `curator` (allocator)     |    [-]     | [x] |  [-]   | [-]  |  [-]  |
| Nullable types (`T?`)     |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Union types (`unio<A,B>`) |    [x]     | [~] |  [x]   | [ ]  |  [ ]  |
| Generic type params       |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| Type aliases (`typus`)    |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| typeof (`typus` RHS)      |    [x]     | [~] |  [ ]   | [ ]  |  [ ]  |

## Variable Declarations

| Feature                      | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `varia` (mutable)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `fixum` (immutable)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `figendum` (async immutable) |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `variandum` (async mutable)  |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `nexum` (reactive field)     |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| Type annotations             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Object destructuring         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Array destructuring          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Rest in destructuring        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Skip pattern (`_`)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Negative indices `[-1]`      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Slicing `[1..3]`             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Inclusive slicing (`usque`)  |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Initializer expressions      |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Enum & Tagged Union Declarations

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ordo` (enum)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Enum variants              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Enum with values           |    [x]     | [x] |  [x]   | [x]  |  [~]  |
| `discretio` (tagged union) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Variant fields             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Generic discretio          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `discerne` (variant match) |    [x]     | [x] |  [x]   | [x]  |  [~]  |

## Function Declarations

| Feature                            | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Basic functions (`functio`)        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Parameters                         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Parameter type annotations         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Parameter aliasing (`ut`)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Parameter defaults (`vel`)         |    [x]     | [ ] |  [x]   | [ ]  |  [x]  |
| Parameter prepositions (`de`/`in`) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Rest parameters (`ceteri`)         |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| Return type annotation (`->`)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `futura` (async prefix)            |    [x]     | [~] |  [x]   | [x]  |  [ ]  |
| `cursor` (generator prefix)        |    [x]     | [-] |  [x]   | [-]  |  [-]  |
| Async generator                    |    [x]     | [-] |  [x]   | [-]  |  [-]  |
| Arrow functions                    |    [x]     | [~] |  [x]   | [x]  |  [x]  |
| `fit T` (sync return)              |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `fiet T` (async return)            |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `fiunt T` (generator return)       |    [x]     | [ ] |  [x]   | [-]  |  [-]  |
| `fient T` (async generator return) |    [x]     | [ ] |  [x]   | [-]  |  [-]  |
| `prae` (comptime type param)       |    [x]     | [x] |  [x]   | [x]  |  [ ]  |

## Control Flow Statements

| Feature                       | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `si` (if)                     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `secus` (else)                |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `sin` (else if)               |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `dum` (while)                 |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `ex...pro` (for-of)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `ex...fit` (for-of verb form) |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `ex...fiet` (async for)       |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `ex...pro (i, n)` (indexed)   |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `de...pro` (for-in)           |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Range `..` (exclusive)        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Range `ante` (exclusive)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Range `usque` (inclusive)     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Range with step (`per`)       |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `in` (mutation block)         |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `elige` (switch)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Switch cases (`si`)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Switch default (`secus`)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `discerne` (pattern match)    |    [x]     | [x] |  [x]   | [x]  |  [~]  |
| `secus` (else/ternary alt)    |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `fac` (do/block)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `ergo` (then, one-liner)      |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `rumpe` (break)               |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `perge` (continue)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `custodi` (guard)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `cura` (resource management)  |    [x]     | [x] |  [ ]   | [x]  |  [ ]  |
| `praefixum` (comptime block)  |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Catch on control flow         |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |

## Return/Exit Statements

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `redde` (return)   |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `redde` with value |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `redde` void       |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Exception Handling

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `tempta` (try)       |    [x]     | [-] |  [x]   | [~]  |  [x]  |
| `cape` (catch)       |    [x]     | [~] |  [x]   | [~]  |  [x]  |
| `demum` (finally)    |    [x]     | [-] |  [x]   | [-]  |  [x]  |
| `fac...cape` (block) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `iace` (throw)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `adfirma` (assert)   |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Assert with message  |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `mori` (panic/fatal) |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Output/Debug/Events

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `scribe` statement |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `vide` (debug)     |    [x]     | [x] |  [x]   | [ ]  |  [x]  |
| `mone` (warn)      |    [x]     | [x] |  [x]   | [ ]  |  [x]  |
| Multiple args      |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Expressions

| Feature                             | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Identifiers                         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `ego` (this/self)                   |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Boolean literals (`verum`/`falsum`) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `nihil` literal                     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| String literals                     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Number literals                     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Hex literals (`0xFF`)               |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Binary literals (`0b1010`)          |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Octal literals (`0o755`)            |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| BigInt literals (`123n`)            |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Template literals                   |    [x]     | [~] |  [x]   | [x]  |  [ ]  |
| `scriptum()` format strings         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Regex literals (`sed`)              |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| Array literals                      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Array spread (`sparge`)             |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| Object literals                     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Object spread (`sparge`)            |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| Binary operators                    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Comparison operators                |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Logical operators                   |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Bitwise operators                   |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Unary operators                     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `nulla` (is empty)                  |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `nonnulla` (has content)            |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `nihil x` (is null)                 |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `nonnihil x` (is not null)          |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `negativum` (is negative)           |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `positivum` (is positive)           |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Member access (`.`)                 |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Optional chaining (`?.`)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Non-null assertion (`!.`)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Computed access (`[]`)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Function calls                      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Call spread (`sparge`)              |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| Method calls                        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Assignment                          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Compound assignment (`+=`, etc.)    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Conditional (ternary)               |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `sic`/`secus` ternary syntax        |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `cede` (await/yield)                |    [x]     | [~] |  [x]   | [x]  |  [ ]  |
| `novum` (new)                       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `novum...de` (new with props)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `===` / `est` (strict equality)     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `!==` / `non est` (strict ineq.)    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `est` (instanceof/typeof)           |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `qua` (type cast)                   |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `aut` (logical or)                  |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `vel` (nullish coalescing)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `praefixum` (comptime expr)         |    [x]     | [x] |  [x]   | [x]  |  [ ]  |

## Lambda Syntax

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `pro x: expr` (expression)     |    [x]     | [~] |  [~]   | [x]  |  [ ]  |
| `pro x { body }` (block)       |    [x]     | [~] |  [~]   | [x]  |  [ ]  |
| `pro: expr` (zero-param)       |    [x]     | [~] |  [~]   | [x]  |  [ ]  |
| `pro x -> T: expr` (ret. type) |    [x]     | [x] |  [-]   | [x]  |  [x]  |
| `fit x: expr` (sync binding)   |    [x]     | [~] |  [~]   | [x]  |  [ ]  |
| `per property` (shorthand)     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## OOP Features (genus/pactum)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `genus` declaration       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Field declarations        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Field defaults            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `nexum` (reactive field)  |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| Static fields (`generis`) |    [x]     | [ ] |  [~]   | [ ]  |  [ ]  |
| `@ privatum` (private)    |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `@ protectum` (protected) |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `creo` (constructor hook) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `deleo` (destructor)      |    [c]     | [c] |  [c]   | [c]  |  [c]  |
| `pingo` (render method)   |    [c]     | [c] |  [c]   | [c]  |  [c]  |
| Auto-merge constructor    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Methods                   |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Async methods             |    [x]     | [~] |  [x]   | [x]  |  [ ]  |
| Generator methods         |    [x]     | [-] |  [x]   | [-]  |  [-]  |
| `sub` (extends)           |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `implet` (implements)     |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| Multiple `implet`         |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `@ abstractum` class      |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `@ abstracta` method      |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `aperit` (index sig)      |    [ ]     | [-] |  [-]   | [-]  |  [-]  |
| Generic classes           |    [x]     | [ ] |  [x]   | [x]  |  [ ]  |
| `pactum` declaration      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Interface methods         |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Import/Export

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `ex...importa` (named imports) |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `ex...importa *` (wildcard)    |    [x]     | [x] |  [x]   | [x]  |  [ ]  |
| `ut` alias (import renaming)   |    [x]     | [x] |  [x]   | [x]  |  [ ]  |

## Testing

| Feature                         | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `proba` (test case)             |    [x]     | [ ] |  [ ]   | [x]  |  [ ]  |
| `probandum` (test suite)        |    [x]     | [ ] |  [ ]   | [x]  |  [ ]  |
| `cura ante` (beforeEach)        |    [x]     | [ ] |  [ ]   | [x]  |  [ ]  |
| `cura post` (afterEach)         |    [x]     | [ ] |  [ ]   | [x]  |  [ ]  |
| `cura ante omnia` (beforeAll)   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cura post omnia` (afterAll)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `omitte` modifier (skip)        |    [x]     | [ ] |  [ ]   | [x]  |  [ ]  |
| `solum` modifier (only)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `futurum` modifier (todo)       |    [x]     | [ ] |  [ ]   | [x]  |  [ ]  |
| Table-driven tests (`proba ex`) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Preamble / Prologue

| Feature                 | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------- | :--------: | :-: | :----: | :--: | :---: |
| Preamble infrastructure |    [x]     | [~] |  [x]   | [~]  |  [x]  |
| Panic class/import      |    [x]     | [-] |  [ ]   | [-]  |  [-]  |
| Decimal import          |    [x]     | [-] |  [x]   | [-]  |  [-]  |
| Enum import             |    [-]     | [-] |  [x]   | [-]  |  [-]  |
| Regex import            |    [-]     | [-] |  [x]   | [x]  |  [-]  |
| Collection imports      |    [-]     | [ ] |  [ ]   | [ ]  |  [-]  |
| Async imports           |    [-]     | [ ] |  [ ]   | [ ]  |  [-]  |
| Arena allocator         |    [-]     | [x] |  [-]   | [ ]  |  [-]  |
| Curator tracking        |    [-]     | [x] |  [-]   | [ ]  |  [-]  |
| Flumina/Responsum       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## I/O Intrinsics

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `_scribe` (print)    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `_vide` (debug)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `_mone` (warn)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `_lege` (read input) |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Stdlib: Math (mathesis)

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pavimentum(x)` (floor)    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `tectum(x)` (ceiling)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `radix(x)` (sqrt)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `potentia(x, n)` (pow)     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `absolutum(x)` (abs)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `signum(x)` (sign)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `rotundum(x)` (round)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `truncatum(x)` (trunc)     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `logarithmus(x)` (log)     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `logarithmus10(x)` (log10) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `exponens(x)` (exp)        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `sinus(x)` (sin)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `cosinus(x)` (cos)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `tangens(x)` (tan)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `minimus(a, b)` (min)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `maximus(a, b)` (max)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `constringens(x, lo, hi)`  |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `PI` (constant)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `E` (constant)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `TAU` (constant)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Stdlib: Random (aleator)

| Feature                       | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `fractus()` (random 0-1)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `inter(min, max)` (int)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `octeti(n)` (random bytes)    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `uuid()` (UUID v4)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `selige(lista)` (random pick) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `misce(lista)` (shuffle copy) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `semen(n)` (seed)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Lista (Array) Methods

| Latin                        | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `adde` (push)                |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `addita` (push copy)         |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `praepone` (unshift)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `praeposita` (unshift copy)  |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `remove` (pop)               |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `remota` (pop copy)          |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `decapita` (shift)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `decapitata` (shift copy)    |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `purga` (clear)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `primus` (first)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `ultimus` (last)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `accipe` (at index)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `longitudo` (length)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `vacua` (is empty)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `continet` (includes)        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `indiceDe` (indexOf)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `inveni` (find)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `inveniIndicem` (findIndex)  |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `filtrata` (filter)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `mappata` (map)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `reducta` (reduce)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `explanata` (flatMap)        |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `plana` (flat)               |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `inversa` (reverse copy)     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `ordinata` (sort copy)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `sectio` (slice)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `prima` (take first n)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `ultima` (take last n)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `omitte` (skip first n)      |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `omnes` (every)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `aliquis` (some)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `coniunge` (join)            |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `perambula` (forEach)        |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `filtra` (filter in-place)   |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `ordina` (sort in-place)     |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `inverte` (reverse in-place) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `congrega` (groupBy)         |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `unica` (unique)             |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `planaOmnia` (flattenDeep)   |    [x]     | [-] |  [ ]   | [ ]  |  [~]  |
| `fragmenta` (chunk)          |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `densa` (compact)            |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `partire` (partition)        |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `misce` (shuffle)            |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `specimen` (sample one)      |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `specimina` (sample n)       |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `summa` (sum)                |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `medium` (average)           |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `minimus` (min)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `maximus` (max)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `minimusPer` (minBy)         |    [x]     | [-] |  [ ]   | [x]  |  [x]  |
| `maximusPer` (maxBy)         |    [x]     | [-] |  [ ]   | [x]  |  [x]  |
| `numera` (count)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Tabula (Map) Methods

| Latin                      | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pone` (set)               |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `accipe` (get)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `habet` (has)              |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `dele` (delete)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `longitudo` (size)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `vacua` (isEmpty)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `purga` (clear)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `claves` (keys)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `valores` (values)         |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `paria` (entries)          |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `accipeAut` (getOrDefault) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `selige` (pick)            |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `omitte` (omit)            |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `confla` (merge)           |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `inversa` (invert)         |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `mappaValores` (mapValues) |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `mappaClaves` (mapKeys)    |    [x]     | [-] |  [x]   | [ ]  |  [x]  |
| `inLista` (toArray)        |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `inObjectum` (toObject)    |    [x]     | [-] |  [x]   | [-]  |  [-]  |

## Copia (Set) Methods

| Latin                         | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `adde` (add)                  |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `habet` (has)                 |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `dele` (delete)               |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `longitudo` (size)            |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `vacua` (isEmpty)             |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `purga` (clear)               |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `unio` (union)                |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `intersectio` (intersection)  |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `differentia` (difference)    |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `symmetrica` (symmetric diff) |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `subcopia` (isSubset)         |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `supercopia` (isSuperset)     |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `inLista` (toArray)           |    [x]     | [-] |  [x]   | [x]  |  [x]  |
| `valores` (values)            |    [x]     | [x] |  [x]   | [ ]  |  [x]  |
| `perambula` (forEach)         |    [x]     | [-] |  [x]   | [x]  |  [x]  |

## Stdlib: Time (tempus)

| Feature                  | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `nunc()` (current epoch) |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `nunc_nano()` (nanos)    |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `nunc_secunda()` (secs)  |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| `dormi ms` (sleep)       |    [x]     | [x] |  [x]   | [x]  |  [x]  |
| Duration constants       |    [x]     | [x] |  [x]   | [x]  |  [x]  |

## Stdlib: File I/O (solum)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `lege` (read file)        |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `inscribe` (write file)   |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `appone` (append)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `aperi` / `claude` (open) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `exstat` (exists)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `dele` (delete)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `duplica` (copy)          |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `move` (rename)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `crea` (mkdir)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `elenca` (readdir)        |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `via.*` (path utils)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Network (caelum)

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pete` (HTTP GET)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `mitte` (HTTP POST)  |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `pone` (HTTP PUT)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `dele` (HTTP DELETE) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| WebSocket client     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| TCP/UDP sockets      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Crypto

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `digere` (hash)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `hmac` (HMAC)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cifra` (encrypt)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `decifra` (decrypt)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `fortuita` (random bytes) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `deriva` (key derivation) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Encoding (codex)

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `coda` (encode)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `decoda` (decode)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Base64/Base64URL     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Hex encoding         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| URL percent-encoding |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Compression (comprimo)

| Feature               | TypeScript | Zig | Python | Rust | C++23 |
| --------------------- | :--------: | :-: | :----: | :--: | :---: |
| `comprimo` (compress) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `laxo` (decompress)   |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| gzip/zstd/brotli      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Database (arca)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Query DSL (`de...quaere`) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Mutations (`in...muta`)   |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Transactions              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| SQLite embedded           |    [-]     | [ ] |  [-]   | [-]  |  [-]  |

## Collection DSL

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ex...prima n` (take)     |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `ex...ultima n` (last)    |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `ex...summa` (sum)        |    [x]     | [ ] |  [x]   | [ ]  |  [ ]  |
| `ab...ubi` (filter where) |    [~]     | [ ] |  [~]   | [ ]  |  [ ]  |
| `ab...pro` (filter iter)  |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## External Dispatch (ad)

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ad "target" (args)` |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Syscall dispatch     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| URL protocol routing |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Package dispatch     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Nucleus Runtime

The Nucleus is Faber's micro-kernel runtime providing unified I/O dispatch, message-passing protocol, and async execution across all targets. See `consilia/futura/nucleus.md` for full design.

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Responsum protocol         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Handle abstraction         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Dispatcher (syscall table) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Request correlation        |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| AsyncContext executor      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| State machine codegen      |    [-]     | [ ] |  [-]   | [ ]  |  [ ]  |

The Responsum protocol defines a tagged union for all syscall results:

| Variant    | Meaning                           |
| ---------- | --------------------------------- |
| `.pending` | Operation in progress, poll again |
| `.ok(T)`   | Single value, terminal            |
| `.item(T)` | One of many values, non-terminal  |
| `.done`    | Stream complete, terminal         |
| `.err(E)`  | Error, terminal                   |

Syscall namespaces (Latin names for stdlib modules):

| Namespace    | Domain  | Example Syscalls                  |
| ------------ | ------- | --------------------------------- |
| `fasciculus` | Files   | `lege`, `scribe`, `tolle`         |
| `caelum`     | Network | `request`, `websocket`, `listen`  |
| `tempus`     | Time    | `dormi`, `nunc`, `intervallum`    |
| `memoria`    | Memory  | `alloca`, `libera` (Zig/Rust/C++) |
| `processus`  | Process | `genera`, `occide`, `expecta`     |
| `canalis`    | Channel | `crea`, `mitte`, `recipe`         |
| `aleator`    | Random  | `numerus`, `bytes`, `uuid`        |
| `crypto`     | Crypto  | `hash`, `signa`, `verifica`       |

---

## Target Notes

### Python (3.10+)

No block braces (indentation-based), no `new` keyword, `asyncio` for async, `typing.Protocol` for interfaces, `match`/`case` for pattern matching.

### Zig (0.11+)

No classes (structs with methods), no interfaces (duck typing), no exceptions (error unions), no generators, comptime generics. `genus` becomes `const Name = struct { ... };`. Memory management via `curator` type which maps to `std.mem.Allocator` — collection methods automatically use the allocator from function parameters or the default arena in `main()`.

### Rust (2021 edition)

Ownership system, borrowing (`&`/`&mut`), `Option<T>`/`Result<T,E>` instead of null/exceptions, traits instead of interfaces, exhaustive pattern matching.

### C++23

`std::expected<T,E>` for errors, `std::print` for output, concepts for interfaces, coroutines for async, RAII for cleanup.

---

## License

MIT
