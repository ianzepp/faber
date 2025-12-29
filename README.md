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
| TypeScript |  1608 |  100%  |
| Python     |   524 |  33%   |
| Rust       |   517 |  32%   |
| C++23      |   397 |  25%   |
| Zig        |   350 |  22%   |

> Status % = passing tests / TypeScript baseline. Run `bun test proba/runner.test.ts -t "<target>"` to verify.

Status key: `[x]` implemented, `[~]` partial, `[ ]` not implemented, `[-]` not applicable, `[c]` convention (no compiler support needed)

## Type System

| Feature                   | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `textus` (string)         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `numerus` (integer)       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `fractus` (float)         |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [x]  |
| `decimus` (decimal)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `magnus` (bigint)         |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `bivalens` (boolean)      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `nihil` (null)            |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `vacuum` (void)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `numquam` (never)         |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `octeti` (bytes)          |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `objectum` (object)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `lista<T>` (array)        |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [ ]  |
| `tabula<K,V>` (map)       |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [ ]  |
| `copia<T>` (set)          |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [ ]  |
| `series<T...>` (tuple)    |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `promissum<T>` (promise)  |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `erratum` (error)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `cursor<T>` (iterator)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ignotum` (unknown)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `curator` (allocator)     |    [-]     | [x] |  [-]   | [ ]  | [-]  |  [-]  |
| Nullable types (`T?`)     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| Union types (`unio<A,B>`) |    [x]     | [~] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Generic type params       |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| Type aliases (`typus`)    |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| typeof (`typus` RHS)      |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |

## Variable Declarations

| Feature                      | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `varia` (mutable)            |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `fixum` (immutable)          |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `figendum` (async immutable) |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `variandum` (async mutable)  |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `nexum` (reactive binding)   |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Type annotations             |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Object destructuring         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Array destructuring          |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Rest in destructuring        |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Negative indices `[-1]`      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Slicing `[1..3]`             |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Initializer expressions      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |

## Enum & Tagged Union Declarations

| Feature                    | TypeScript | Zig | Python | WASM | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `ordo` (enum)              |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| Enum variants              |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| Enum with values           |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `discretio` (tagged union) |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Variant fields             |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Generic discretio          |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `discerne` (variant match) |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [~]  |

## Function Declarations

| Feature                            | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ---------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| Basic functions (`functio`)        |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Parameters                         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Parameter type annotations         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Rest parameters (`ceteri`)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Return type annotation (`->`)      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `futura` (async prefix)            |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [ ]  |
| `cursor` (generator prefix)        |    [x]     | [-] |  [x]   | [ ]  | [-]  |  [-]  |
| Async generator                    |    [x]     | [-] |  [x]   | [ ]  | [-]  |  [-]  |
| Arrow functions                    |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [x]  |
| `fit T` (sync return)              |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `fiet T` (async return)            |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `fiunt T` (generator return)       |    [x]     | [ ] |  [x]   | [ ]  | [-]  |  [-]  |
| `fient T` (async generator return) |    [x]     | [ ] |  [x]   | [ ]  | [-]  |  [-]  |
| `prae` (comptime type param)       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |

## Control Flow Statements

| Feature                       | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `si` (if)                     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `aliter` (else)               |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `aliter si` (else if)         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `sin` (else if, poetic)       |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `dum` (while)                 |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `ex...pro` (for-of)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `de...pro` (for-in)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `ex...fit` (for-of verb form) |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `ex...fiet` (async for)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ex...pro (i, n)` (indexed)   |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Range expressions (`0..10`)   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Range with step (`per`)       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `in` (mutation block)         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `elige` (switch)              |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| Switch cases (`si`)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| Switch default (`aliter`)     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `secus` (else/ternary alt)    |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `fac` (do/block)              |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `ergo` (then, one-liner)      |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `rumpe` (break)               |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `perge` (continue)            |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `custodi` (guard)             |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `cura` (resource management)  |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| `praefixum` (comptime block)  |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| Catch on control flow         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Return/Exit Statements

| Feature            | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :--: | :---: |
| `redde` (return)   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `redde` with value |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `redde` void       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |

## Exception Handling

| Feature              | TypeScript | Zig | Python | WASM | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `tempta` (try)       |    [x]     | [-] |  [x]   | [ ]  | [~]  |  [x]  |
| `cape` (catch)       |    [x]     | [~] |  [x]   | [ ]  | [~]  |  [x]  |
| `demum` (finally)    |    [x]     | [-] |  [x]   | [ ]  | [-]  |  [x]  |
| `iace` (throw)       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `adfirma` (assert)   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Assert with message  |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `mori` (panic/fatal) |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |

## Output/Debug/Events

| Feature            | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :--: | :---: |
| `scribe` statement |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `vide` (debug)     |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `mone` (warn)      |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Multiple args      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |

## Expressions

| Feature                             | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| Identifiers                         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `ego` (this/self)                   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Boolean literals (`verum`/`falsum`) |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `nihil` literal                     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| String literals                     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Number literals                     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Hex literals (`0xFF`)               |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| BigInt hex (`0xFFn`)                |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| Template literals                   |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [ ]  |
| Regex literals (`sed`)              |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Array literals                      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Array spread (`sparge`)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Object literals                     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Object spread (`sparge`)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Binary operators                    |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Comparison operators                |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Logical operators                   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Bitwise operators                   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Unary operators                     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `nulla` (is empty)                  |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `nonnulla` (has content)            |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `nihil` (is null)                   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `nonnihil` (is not null)            |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `negativum` (is negative)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `positivum` (is positive)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| Member access (`.`)                 |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Optional chaining (`?.`)            |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Non-null assertion (`!.`)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Computed access (`[]`)              |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Function calls                      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Call spread (`sparge`)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Method calls                        |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Assignment                          |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Conditional (ternary)               |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `sic`/`secus` ternary syntax        |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `cede` (await/yield)                |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [ ]  |
| `novum` (new)                       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `novum...de` (new with props)       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `===` / `est` (strict equality)     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `!==` / `non est` (strict ineq.)    |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `est` (instanceof/typeof)           |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `qua` (type cast)                   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `aut` (logical or)                  |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [x]  |
| `vel` (nullish coalescing)          |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |

## Lambda Syntax

| Feature                         | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `pro x redde expr` (expression) |    [x]     | [~] |  [~]   | [ ]  | [x]  |  [ ]  |
| `pro x: expr` (colon shorthand) |    [x]     | [~] |  [~]   | [ ]  | [x]  |  [ ]  |
| `pro x { body }` (block)        |    [x]     | [~] |  [~]   | [ ]  | [x]  |  [ ]  |
| `pro redde expr` (zero-param)   |    [x]     | [~] |  [~]   | [ ]  | [x]  |  [ ]  |
| `pro x -> T: expr` (ret. type)  |    [x]     | [x] |  [-]   | [ ]  | [x]  |  [x]  |
| `(x) => expr` (JS-style)        |    [x]     | [~] |  [~]   | [ ]  | [x]  |  [x]  |
| `per property` (shorthand)      |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |

## OOP Features (genus/pactum)

| Feature                   | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `genus` declaration       |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Field declarations        |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Field defaults            |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Computed fields (getters) |    [-]     | [-] |  [-]   | [-]  | [-]  |  [-]  |
| Static fields (`generis`) |    [x]     | [ ] |  [~]   | [ ]  | [ ]  |  [ ]  |
| Public/private visibility |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `creo` (constructor hook) |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `deleo` (destructor)      |    [c]     | [c] |  [c]   | [c]  | [c]  |  [c]  |
| `pingo` (render method)   |    [c]     | [c] |  [c]   | [c]  | [c]  |  [c]  |
| Auto-merge constructor    |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Methods                   |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Async methods             |    [x]     | [~] |  [x]   | [ ]  | [x]  |  [ ]  |
| Generator methods         |    [x]     | [-] |  [x]   | [ ]  | [-]  |  [-]  |
| `implet` (implements)     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `aperit` (index sig)      |    [ ]     | [-] |  [-]   | [-]  | [-]  |  [-]  |
| Generic classes           |    [x]     | [ ] |  [x]   | [ ]  | [x]  |  [ ]  |
| `pactum` declaration      |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| Interface methods         |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |

## Import/Export

| Feature                        | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :--: | :---: |
| `ex...importa` (named imports) |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |
| `ex...importa *` (wildcard)    |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [ ]  |

## Testing

| Feature                         | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `proba` (test case)             |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| `probandum` (test suite)        |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| `cura ante` (beforeEach)        |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| `cura post` (afterEach)         |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| `omitte` modifier (skip)        |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| `solum` modifier (only)         |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| `futurum` modifier (todo)       |    [x]     | [ ] |  [ ]   | [ ]  | [x]  |  [ ]  |
| Table-driven tests (`proba ex`) |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |

## Preamble / Prologue

| Feature                 | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| Preamble infrastructure |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [x]  |
| Panic class/import      |    [x]     | [-] |  [ ]   | [ ]  | [-]  |  [-]  |
| Collection imports      |    [-]     | [ ] |  [ ]   | [ ]  | [ ]  |  [-]  |
| Async imports           |    [-]     | [ ] |  [ ]   | [ ]  | [ ]  |  [-]  |
| Arena allocator         |    [-]     | [x] |  [-]   | [ ]  | [ ]  |  [-]  |
| Curator tracking        |    [-]     | [x] |  [-]   | [ ]  | [ ]  |  [-]  |

## Standard Library Intrinsics

| Feature               | TypeScript | Zig | Python | WASM | Rust | C++23 |
| --------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `_scribe` (print)     |    [x]     | [x] |  [x]   | [ ]  | [x]  |  [x]  |
| `_vide` (debug)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `_mone` (warn)        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `_lege` (read input)  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `_fortuitus` (random) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `_pavimentum` (floor) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `_tectum` (ceiling)   |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `_radix` (sqrt)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `_potentia` (pow)     |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Lista (Array) Methods

| Latin                        | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `adde` (push)                |    [x]     | [x] |  [x]   | [ ]  | [~]  |  [x]  |
| `addita` (push copy)         |    [x]     | [-] |  [x]   | [ ]  | [~]  |  [x]  |
| `praepone` (unshift)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `praeposita` (unshift copy)  |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `remove` (pop)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `remota` (pop copy)          |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `decapita` (shift)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `decapitata` (shift copy)    |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `purga` (clear)              |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `primus` (first)             |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `ultimus` (last)             |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `accipe` (at index)          |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `longitudo` (length)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `vacua` (is empty)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `continet` (includes)        |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `indiceDe` (indexOf)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inveni` (find)              |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inveniIndicem` (findIndex)  |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `filtrata` (filter)          |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `mappata` (map)              |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `reducta` (reduce)           |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `explanata` (flatMap)        |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `plana` (flat)               |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inversa` (reverse copy)     |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `ordinata` (sort copy)       |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `sectio` (slice)             |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `prima` (take first n)       |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `ultima` (take last n)       |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `omitte` (skip first n)      |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `omnes` (every)              |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `aliquis` (some)             |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `coniunge` (join)            |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `perambula` (forEach)        |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `filtra` (filter in-place)   |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `ordina` (sort in-place)     |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inverte` (reverse in-place) |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `congrega` (groupBy)         |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `unica` (unique)             |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `planaOmnia` (flattenDeep)   |    [x]     | [-] |  [ ]   | [ ]  | [ ]  |  [~]  |
| `fragmenta` (chunk)          |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `densa` (compact)            |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `partire` (partition)        |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `misce` (shuffle)            |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `specimen` (sample one)      |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `specimina` (sample n)       |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `summa` (sum)                |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `medium` (average)           |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `minimus` (min)              |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `maximus` (max)              |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `minimusPer` (minBy)         |    [x]     | [-] |  [ ]   | [ ]  | [ ]  |  [x]  |
| `maximusPer` (maxBy)         |    [x]     | [-] |  [ ]   | [ ]  | [ ]  |  [x]  |
| `numera` (count)             |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |

## Tabula (Map) Methods

| Latin                      | TypeScript | Zig | Python | WASM | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `pone` (set)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `accipe` (get)             |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `habet` (has)              |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `dele` (delete)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `longitudo` (size)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `vacua` (isEmpty)          |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `purga` (clear)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `claves` (keys)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `valores` (values)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `paria` (entries)          |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `accipeAut` (getOrDefault) |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `selige` (pick)            |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `omitte` (omit)            |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `confla` (merge)           |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inversa` (invert)         |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `mappaValores` (mapValues) |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `mappaClaves` (mapKeys)    |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inLista` (toArray)        |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inObjectum` (toObject)    |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [-]  |

## Copia (Set) Methods

| Latin                         | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `adde` (add)                  |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `habet` (has)                 |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `dele` (delete)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `longitudo` (size)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `vacua` (isEmpty)             |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `purga` (clear)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `unio` (union)                |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `intersectio` (intersection)  |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `differentia` (difference)    |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `symmetrica` (symmetric diff) |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `subcopia` (isSubset)         |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `supercopia` (isSuperset)     |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `inLista` (toArray)           |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `valores` (values)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `perambula` (forEach)         |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |

## Stdlib: Time (tempus)

| Feature                  | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------ | :--------: | :-: | :----: | :--: | :--: | :---: |
| `nunc()` (current epoch) |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `dormi ms` (sleep)       |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Duration constants       |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |

## Stdlib: File I/O, Filesystem, Network, Crypto, Resources

These modules are planned but not yet implemented for any target.

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
