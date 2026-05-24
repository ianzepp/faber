---
title: Faber (Reference)
description: Reference compiler implementation status
section: compilers
order: 0
---

# Faber (Reference)

The reference compiler for the Faber Romanus programming language, implemented in TypeScript. Faber ("craftsman" in Latin) is the primary compiler used for all daily development, supporting multi-target code generation to TypeScript, Python, Rust, Zig, and C++.

This compiler uses mixed Latin/English identifiers in its implementation, prioritizing clarity for contributors familiar with TypeScript conventions while maintaining the project's Latin aesthetic in user-facing APIs.

## Usage

```
faber compile <file.fab>              # Compile to TypeScript (default)
faber compile <file.fab> -t py        # Compile to Python
faber compile <file.fab> -t zig       # Compile to Zig
faber compile <file.fab> -t rs        # Compile to Rust
faber compile <file.fab> -t cpp       # Compile to C++23
faber compile <file.fab> -t fab       # Format/normalize Faber source
faber compile <file.fab> -o out.ts    # Specify output file
faber run <file.fab>                  # Compile and execute (TypeScript only)
faber check <file.fab>                # Validate syntax without output
faber format <file.fab>               # Format source in place
```

## Implementation Status

| Target     | Tests | Status |
| ---------- | ----: | :----: |
| TypeScript |   760 |  100%  |
| Python     |   613 |  81%   |
| Rust       |   580 |  76%   |
| C++23      |   475 |  63%   |
| Zig        |   470 |  62%   |

> Status % = passing tests / TypeScript baseline. Run `bun test proba/runner.test.ts -t "@<target>"` to verify (e.g., `-t "@zig"`).

Status: ● implemented, ◐ partial, ○ not implemented, — not applicable, ◌ convention

## Type System

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `textus` (string)         |    ●     | ● |  ●   | ●  |  ●  |
| `numerus` (integer)       |    ●     | ● |  ●   | ●  |  ●  |
| `fractus` (float)         |    ●     | ● |  ●   | ●  |  ●  |
| `decimus` (decimal)       |    ●     | ◐ |  ●   | ○  |  ◐  |
| `magnus` (bigint)         |    ●     | ◐ |  ●   | ●  |  ○  |
| `bivalens` (boolean)      |    ●     | ● |  ●   | ●  |  ●  |
| `nihil` (null)            |    ●     | ● |  ●   | ●  |  ●  |
| `vacuum` (void)           |    ●     | ● |  ●   | ●  |  ●  |
| `numquam` (never)         |    ●     | ● |  ●   | ●  |  ○  |
| `octeti` (bytes)          |    ●     | ● |  ●   | ●  |  ●  |
| `objectum` (object)       |    ●     | ◐ |  ●   | ○  |  ◐  |
| `lista<T>` (array)        |    ●     | ◐ |  ●   | ●  |  ●  |
| `tabula<K,V>` (map)       |    ●     | ◐ |  ●   | ●  |  ●  |
| `copia<T>` (set)          |    ●     | ◐ |  ●   | ●  |  ●  |
| `series<T...>` (tuple)    |    ○     | ○ |  ○   | ○  |  ○  |
| `promissum<T>` (promise)  |    ●     | ○ |  ●   | ●  |  ○  |
| `erratum` (error)         |    ●     | ○ |  ●   | ○  |  ○  |
| `cursor<T>` (iterator)    |    ●     | ○ |  ●   | ○  |  ○  |
| `ignotum` (unknown)       |    ●     | ● |  ●   | ○  |  ○  |
| `curator` (allocator)     |    —     | ● |  —   | —  |  —  |
| Nullable types (`T?`)     |    ●     | ● |  ●   | ●  |  ○  |
| Union types (`unio<A,B>`) |    ●     | ◐ |  ●   | ○  |  ○  |
| Generic type params       |    ●     | ○ |  ●   | ●  |  ○  |
| Type aliases (`typus`)    |    ●     | ● |  ●   | ●  |  ○  |
| typeof (`typus` RHS)      |    ●     | ◐ |  ○   | ○  |  ○  |

## Variable Declarations

| Feature                      | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `varia` (mutable)            |    ●     | ● |  ●   | ●  |  ●  |
| `fixum` (immutable)          |    ●     | ● |  ●   | ●  |  ●  |
| `figendum` (async immutable) |    ●     | ○ |  ●   | ●  |  ○  |
| `variandum` (async mutable)  |    ●     | ○ |  ●   | ●  |  ○  |
| `nexum` (reactive field)     |    ●     | ○ |  ●   | ○  |  ○  |
| Type annotations             |    ●     | ● |  ●   | ●  |  ●  |
| Object destructuring         |    ●     | ● |  ●   | ●  |  ●  |
| Array destructuring          |    ●     | ● |  ●   | ●  |  ●  |
| Rest in destructuring        |    ●     | ● |  ●   | ●  |  ●  |
| Skip pattern (`_`)           |    ●     | ● |  ●   | ●  |  ●  |
| Negative indices `[-1]`      |    ●     | ● |  ●   | ●  |  ●  |
| Slicing `[1..3]`             |    ●     | ● |  ●   | ●  |  ●  |
| Inclusive slicing (`usque`)  |    ●     | ● |  ●   | ●  |  ●  |
| Initializer expressions      |    ●     | ● |  ●   | ●  |  ●  |

## Enum & Tagged Union Declarations

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ordo` (enum)              |    ●     | ● |  ●   | ●  |  ●  |
| Enum variants              |    ●     | ● |  ●   | ●  |  ●  |
| Enum with values           |    ●     | ● |  ●   | ●  |  ◐  |
| `discretio` (tagged union) |    ●     | ● |  ●   | ●  |  ●  |
| Variant fields             |    ●     | ● |  ●   | ●  |  ●  |
| Generic discretio          |    ●     | ● |  ●   | ●  |  ●  |
| `discerne` (variant match) |    ●     | ● |  ●   | ●  |  ◐  |

## Function Declarations

| Feature                            | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Basic functions (`functio`)        |    ●     | ● |  ●   | ●  |  ●  |
| Parameters                         |    ●     | ● |  ●   | ●  |  ●  |
| Parameter type annotations         |    ●     | ● |  ●   | ●  |  ●  |
| Parameter aliasing (`ut`)          |    ●     | ● |  ●   | ●  |  ●  |
| Parameter defaults (`vel`)         |    ●     | ○ |  ●   | ○  |  ●  |
| Parameter prepositions (`de`/`in`) |    ●     | ● |  ●   | ●  |  ●  |
| Rest parameters (`ceteri`)         |    ●     | ○ |  ●   | ○  |  ○  |
| Return type annotation (`->`)      |    ●     | ● |  ●   | ●  |  ●  |
| `futura` (async prefix)            |    ●     | ◐ |  ●   | ●  |  ○  |
| `cursor` (generator prefix)        |    ●     | — |  ●   | —  |  —  |
| Async generator                    |    ●     | — |  ●   | —  |  —  |
| Arrow functions                    |    ●     | ◐ |  ●   | ●  |  ●  |
| `fit T` (sync return)              |    ●     | ● |  ●   | ●  |  ○  |
| `fiet T` (async return)            |    ●     | ○ |  ●   | ●  |  ○  |
| `fiunt T` (generator return)       |    ●     | ○ |  ●   | —  |  —  |
| `fient T` (async generator return) |    ●     | ○ |  ●   | —  |  —  |
| `prae` (comptime type param)       |    ●     | ● |  ●   | ●  |  ○  |
| `@ externa` (external decl)        |    ●     | ○ |  ○   | ○  |  ○  |

## Control Flow Statements

| Feature                       | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `si` (if)                     |    ●     | ● |  ●   | ●  |  ●  |
| `secus` (else)                |    ●     | ● |  ●   | ●  |  ●  |
| `sin` (else if)               |    ●     | ● |  ●   | ●  |  ●  |
| `dum` (while)                 |    ●     | ● |  ●   | ●  |  ●  |
| `ex...pro` (for-of)           |    ●     | ● |  ●   | ●  |  ●  |
| `ex...fit` (for-of verb form) |    ●     | ● |  ●   | ●  |  ○  |
| `ex...fiet` (async for)       |    ●     | ○ |  ●   | ○  |  ○  |
| `ex...pro (i, n)` (indexed)   |    ○     | ○ |  ○   | ○  |  ○  |
| `de...pro` (for-in)           |    ●     | ● |  ●   | ●  |  ○  |
| Range `..` (exclusive)        |    ●     | ● |  ●   | ●  |  ●  |
| Range `ante` (exclusive)      |    ●     | ● |  ●   | ●  |  ●  |
| Range `usque` (inclusive)     |    ●     | ● |  ●   | ●  |  ●  |
| Range with step (`per`)       |    ●     | ● |  ●   | ●  |  ○  |
| `in` (mutation block)         |    ●     | ● |  ●   | ●  |  ○  |
| `elige` (switch)              |    ●     | ● |  ●   | ●  |  ●  |
| Switch cases (`si`)           |    ●     | ● |  ●   | ●  |  ●  |
| Switch default (`secus`)      |    ●     | ● |  ●   | ●  |  ●  |
| `discerne` (pattern match)    |    ●     | ● |  ●   | ●  |  ◐  |
| `discerne` multi-discriminant |    ●     | ○ |  ○   | ○  |  ○  |
| `secus` (else/ternary alt)    |    ●     | ○ |  ●   | ●  |  ○  |
| `fac` (do/block)              |    ●     | ● |  ●   | ●  |  ●  |
| `ergo` (then, one-liner)      |    ●     | ○ |  ●   | ○  |  ○  |
| `rumpe` (break)               |    ●     | ● |  ●   | ●  |  ●  |
| `perge` (continue)            |    ●     | ● |  ●   | ●  |  ●  |
| `custodi` (guard)             |    ●     | ● |  ●   | ●  |  ●  |
| `cura` (resource management)  |    ●     | ● |  ○   | ●  |  ○  |
| `praefixum` (comptime block)  |    ●     | ● |  ●   | ●  |  ○  |
| Catch on control flow         |    ●     | ○ |  ●   | ○  |  ○  |

## Return/Exit Statements

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `redde` (return)   |    ●     | ● |  ●   | ●  |  ●  |
| `redde` with value |    ●     | ● |  ●   | ●  |  ●  |
| `redde` void       |    ●     | ● |  ●   | ●  |  ●  |

## Exception Handling

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `tempta` (try)       |    ●     | — |  ●   | ◐  |  ●  |
| `cape` (catch)       |    ●     | ◐ |  ●   | ◐  |  ●  |
| `demum` (finally)    |    ●     | — |  ●   | —  |  ●  |
| `fac...cape` (block) |    ●     | ● |  ●   | ●  |  ●  |
| `iace` (throw)       |    ●     | ● |  ●   | ●  |  ●  |
| `adfirma` (assert)   |    ●     | ● |  ●   | ●  |  ●  |
| Assert with message  |    ●     | ● |  ●   | ●  |  ●  |
| `mori` (panic/fatal) |    ●     | ● |  ●   | ●  |  ●  |

## Output/Debug/Events

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `scribe` statement |    ●     | ● |  ●   | ●  |  ●  |
| `vide` (debug)     |    ●     | ● |  ●   | ○  |  ●  |
| `mone` (warn)      |    ●     | ● |  ●   | ○  |  ●  |
| Multiple args      |    ●     | ● |  ●   | ●  |  ●  |

## Expressions

| Feature                             | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Identifiers                         |    ●     | ● |  ●   | ●  |  ●  |
| `ego` (this/self)                   |    ●     | ● |  ●   | ●  |  ●  |
| Boolean literals (`verum`/`falsum`) |    ●     | ● |  ●   | ●  |  ●  |
| `nihil` literal                     |    ●     | ● |  ●   | ●  |  ●  |
| String literals                     |    ●     | ● |  ●   | ●  |  ●  |
| Number literals                     |    ●     | ● |  ●   | ●  |  ●  |
| Hex literals (`0xFF`)               |    ●     | ● |  ●   | ●  |  ○  |
| Binary literals (`0b1010`)          |    ●     | ● |  ●   | ●  |  ○  |
| Octal literals (`0o755`)            |    ●     | ● |  ●   | ●  |  ○  |
| BigInt literals (`123n`)            |    ●     | ● |  ●   | ●  |  ○  |
| Template literals                   |    ●     | ◐ |  ●   | ●  |  ○  |
| `scriptum()` format strings         |    ●     | ● |  ●   | ●  |  ●  |
| Regex literals (`sed`)              |    ●     | ○ |  ●   | ●  |  ○  |
| Array literals                      |    ●     | ● |  ●   | ●  |  ●  |
| Array spread (`sparge`)             |    ●     | ○ |  ●   | ○  |  ○  |
| Object literals                     |    ●     | ● |  ●   | ●  |  ●  |
| Object spread (`sparge`)            |    ●     | ○ |  ●   | ○  |  ○  |
| Binary operators                    |    ●     | ● |  ●   | ●  |  ●  |
| Comparison operators                |    ●     | ● |  ●   | ●  |  ●  |
| Logical operators                   |    ●     | ● |  ●   | ●  |  ●  |
| Bitwise operators                   |    ●     | ● |  ●   | ●  |  ●  |
| Unary operators                     |    ●     | ● |  ●   | ●  |  ●  |
| `nulla` (is empty)                  |    ●     | ● |  ●   | ●  |  ○  |
| `nonnulla` (has content)            |    ●     | ● |  ●   | ●  |  ○  |
| `nihil x` (is null)                 |    ●     | ● |  ●   | ●  |  ○  |
| `nonnihil x` (is not null)          |    ●     | ● |  ●   | ●  |  ○  |
| `negativum` (is negative)           |    ●     | ● |  ●   | ●  |  ○  |
| `positivum` (is positive)           |    ●     | ● |  ●   | ●  |  ○  |
| `verum x` (is true)                 |    ○     | ● |  ●   | ●  |  ○  |
| `falsum x` (is false)               |    ○     | ● |  ●   | ●  |  ○  |
| Member access (`.`)                 |    ●     | ● |  ●   | ●  |  ●  |
| Optional chaining (`?.`)            |    ●     | ● |  ●   | ●  |  ●  |
| Non-null assertion (`!.`)           |    ●     | ● |  ●   | ●  |  ●  |
| Computed access (`[]`)              |    ●     | ● |  ●   | ●  |  ●  |
| Function calls                      |    ●     | ● |  ●   | ●  |  ●  |
| Call spread (`sparge`)              |    ●     | ○ |  ●   | ○  |  ○  |
| Method calls                        |    ●     | ● |  ●   | ●  |  ●  |
| Assignment                          |    ●     | ● |  ●   | ●  |  ●  |
| Compound assignment (`+=`, etc.)    |    ●     | ● |  ●   | ●  |  ●  |
| Conditional (ternary)               |    ●     | ● |  ●   | ●  |  ●  |
| `sic`/`secus` ternary syntax        |    ●     | ○ |  ●   | ●  |  ○  |
| `cede` (await/yield)                |    ●     | ◐ |  ●   | ●  |  ○  |
| `novum` (new)                       |    ●     | ● |  ●   | ●  |  ●  |
| `novum...de` (new with props)       |    ●     | ● |  ●   | ●  |  ●  |
| `===` / `est` (strict equality)     |    ●     | ● |  ●   | ●  |  ●  |
| `!==` / `non est` (strict ineq.)    |    ●     | ● |  ●   | ●  |  ●  |
| `est` (instanceof/typeof)           |    ●     | ● |  ●   | ●  |  ○  |
| `qua` (type cast)                   |    ●     | ● |  ●   | ●  |  ○  |
| `innatum` (native construction)     |    ●     | ● |  ●   | ●  |  ●  |
| `aut` (logical or)                  |    ●     | ● |  ●   | ●  |  ●  |
| `vel` (nullish coalescing)          |    ●     | ● |  ●   | ●  |  ●  |
| `praefixum` (comptime expr)         |    ●     | ● |  ●   | ●  |  ○  |

## Lambda Syntax

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `pro x: expr` (expression)     |    ●     | ◐ |  ◐   | ●  |  ○  |
| `pro x { body }` (block)       |    ●     | ◐ |  ◐   | ●  |  ○  |
| `pro: expr` (zero-param)       |    ●     | ◐ |  ◐   | ●  |  ○  |
| `pro x -> T: expr` (ret. type) |    ●     | ● |  —   | ●  |  ●  |
| `fit x: expr` (sync binding)   |    ●     | ◐ |  ◐   | ●  |  ○  |
| `per property` (shorthand)     |    ○     | ○ |  ○   | ○  |  ○  |

## OOP Features (genus/pactum)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `genus` declaration       |    ●     | ● |  ●   | ●  |  ●  |
| Field declarations        |    ●     | ● |  ●   | ●  |  ●  |
| Field defaults            |    ●     | ● |  ●   | ●  |  ●  |
| `nexum` (reactive field)  |    ●     | ○ |  ●   | ○  |  ○  |
| Static fields (`generis`) |    ●     | ○ |  ◐   | ○  |  ○  |
| `@ privatum` (private)    |    ●     | ○ |  ●   | ●  |  ○  |
| `@ protectum` (protected) |    ●     | ○ |  ●   | ○  |  ○  |
| `creo` (constructor hook) |    ●     | ● |  ●   | ●  |  ●  |
| `deleo` (destructor)      |    ◌     | ◌ |  ◌   | ◌  |  ◌  |
| `pingo` (render method)   |    ◌     | ◌ |  ◌   | ◌  |  ◌  |
| Auto-merge constructor    |    ●     | ● |  ●   | ●  |  ●  |
| Methods                   |    ●     | ● |  ●   | ●  |  ●  |
| Async methods             |    ●     | ◐ |  ●   | ●  |  ○  |
| Generator methods         |    ●     | — |  ●   | —  |  —  |
| `sub` (extends)           |    ●     | ○ |  ●   | ○  |  ○  |
| `implet` (implements)     |    ●     | ● |  ●   | ●  |  ○  |
| Multiple `implet`         |    ●     | ○ |  ●   | ●  |  ○  |
| `@ abstractum` class      |    ●     | ○ |  ●   | ○  |  ○  |
| `@ abstracta` method      |    ●     | ○ |  ●   | ○  |  ○  |
| `aperit` (index sig)      |    ○     | — |  —   | —  |  —  |
| Generic classes           |    ●     | ○ |  ●   | ●  |  ○  |
| `pactum` declaration      |    ●     | ● |  ●   | ●  |  ●  |
| Interface methods         |    ●     | ● |  ●   | ●  |  ●  |

## Import/Export

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `ex...importa` (named imports) |    ●     | ● |  ●   | ●  |  ○  |
| `ex...importa *` (wildcard)    |    ●     | ● |  ●   | ●  |  ○  |
| `ut` alias (import renaming)   |    ●     | ● |  ●   | ●  |  ○  |

## Testing

| Feature                         | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `proba` (test case)             |    ●     | ○ |  ○   | ●  |  ○  |
| `probandum` (test suite)        |    ●     | ○ |  ○   | ●  |  ○  |
| `cura ante` (beforeEach)        |    ●     | ○ |  ○   | ●  |  ○  |
| `cura post` (afterEach)         |    ●     | ○ |  ○   | ●  |  ○  |
| `cura ante omnia` (beforeAll)   |    ●     | ○ |  ○   | ○  |  ○  |
| `cura post omnia` (afterAll)    |    ●     | ○ |  ○   | ○  |  ○  |
| `omitte` modifier (skip)        |    ●     | ○ |  ○   | ●  |  ○  |
| `solum` modifier (only)         |    ○     | ○ |  ○   | ○  |  ○  |
| `futurum` modifier (todo)       |    ●     | ○ |  ○   | ●  |  ○  |
| Table-driven tests (`proba ex`) |    ○     | ○ |  ○   | ○  |  ○  |

## Preamble / Prologue

| Feature                 | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------- | :--------: | :-: | :----: | :--: | :---: |
| Preamble infrastructure |    ●     | ◐ |  ●   | ◐  |  ●  |
| Panic class/import      |    ●     | — |  ○   | —  |  —  |
| Decimal import          |    ●     | — |  ●   | —  |  —  |
| Enum import             |    —     | — |  ●   | —  |  —  |
| Regex import            |    —     | — |  ●   | ●  |  —  |
| Collection imports      |    —     | ○ |  ○   | ○  |  —  |
| Async imports           |    —     | ○ |  ○   | ○  |  —  |
| Arena allocator         |    —     | ● |  —   | ○  |  —  |
| Curator tracking        |    —     | ● |  —   | ○  |  —  |
| Flumina/Responsum       |    ●     | ○ |  ○   | ○  |  ○  |

## I/O Intrinsics

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `_scribe` (print)    |    ●     | ● |  ●   | ●  |  ●  |
| `_vide` (debug)      |    ●     | ● |  ●   | ●  |  ●  |
| `_mone` (warn)       |    ●     | ● |  ●   | ●  |  ●  |
| `_lege` (read input) |    ●     | ● |  ●   | ●  |  ●  |

## Stdlib: Math (mathesis)

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pavimentum(x)` (floor)    |    ●     | ● |  ●   | ●  |  ●  |
| `tectum(x)` (ceiling)      |    ●     | ● |  ●   | ●  |  ●  |
| `radix(x)` (sqrt)          |    ●     | ● |  ●   | ●  |  ●  |
| `potentia(x, n)` (pow)     |    ●     | ● |  ●   | ●  |  ●  |
| `absolutum(x)` (abs)       |    ●     | ● |  ●   | ●  |  ●  |
| `signum(x)` (sign)         |    ●     | ● |  ●   | ●  |  ●  |
| `rotundum(x)` (round)      |    ●     | ● |  ●   | ●  |  ●  |
| `truncatum(x)` (trunc)     |    ●     | ● |  ●   | ●  |  ●  |
| `logarithmus(x)` (log)     |    ●     | ● |  ●   | ●  |  ●  |
| `logarithmus10(x)` (log10) |    ●     | ● |  ●   | ●  |  ●  |
| `exponens(x)` (exp)        |    ●     | ● |  ●   | ●  |  ●  |
| `sinus(x)` (sin)           |    ●     | ● |  ●   | ●  |  ●  |
| `cosinus(x)` (cos)         |    ●     | ● |  ●   | ●  |  ●  |
| `tangens(x)` (tan)         |    ●     | ● |  ●   | ●  |  ●  |
| `minimus(a, b)` (min)      |    ●     | ● |  ●   | ●  |  ●  |
| `maximus(a, b)` (max)      |    ●     | ● |  ●   | ●  |  ●  |
| `constringens(x, lo, hi)`  |    ●     | ● |  ●   | ●  |  ●  |
| `PI` (constant)            |    ●     | ● |  ●   | ●  |  ●  |
| `E` (constant)             |    ●     | ● |  ●   | ●  |  ●  |
| `TAU` (constant)           |    ●     | ● |  ●   | ●  |  ●  |

## Stdlib: Random (aleator)

| Feature                       | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `fractus()` (random 0-1)      |    ●     | ● |  ●   | ●  |  ●  |
| `inter(min, max)` (int)       |    ●     | ● |  ●   | ●  |  ●  |
| `octeti(n)` (random bytes)    |    ●     | ● |  ●   | ●  |  ●  |
| `uuid()` (UUID v4)            |    ●     | ● |  ●   | ●  |  ●  |
| `selige(lista)` (random pick) |    ●     | ● |  ●   | ●  |  ●  |
| `misce(lista)` (shuffle copy) |    ●     | ● |  ●   | ●  |  ●  |
| `semen(n)` (seed)             |    ●     | ● |  ●   | ●  |  ●  |

## Lista (Array) Methods

| Latin                        | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `adde` (push)                |    ●     | ● |  ●   | ●  |  ●  |
| `addita` (push copy)         |    ●     | — |  ●   | ●  |  ●  |
| `praepone` (unshift)         |    ●     | ● |  ●   | ●  |  ●  |
| `praeposita` (unshift copy)  |    ●     | — |  ●   | ●  |  ●  |
| `remove` (pop)               |    ●     | ● |  ●   | ●  |  ●  |
| `remota` (pop copy)          |    ●     | — |  ●   | ●  |  ●  |
| `decapita` (shift)           |    ●     | ● |  ●   | ●  |  ●  |
| `decapitata` (shift copy)    |    ●     | — |  ●   | ●  |  ●  |
| `purga` (clear)              |    ●     | ● |  ●   | ●  |  ●  |
| `primus` (first)             |    ●     | ● |  ●   | ●  |  ●  |
| `ultimus` (last)             |    ●     | ● |  ●   | ●  |  ●  |
| `accipe` (at index)          |    ●     | ● |  ●   | ●  |  ●  |
| `longitudo` (length)         |    ●     | ● |  ●   | ●  |  ●  |
| `vacua` (is empty)           |    ●     | ● |  ●   | ●  |  ●  |
| `continet` (includes)        |    ●     | ● |  ●   | ●  |  ●  |
| `indiceDe` (indexOf)         |    ●     | ● |  ●   | ●  |  ●  |
| `inveni` (find)              |    ●     | ● |  ●   | ●  |  ●  |
| `inveniIndicem` (findIndex)  |    ●     | ● |  ●   | ●  |  ●  |
| `filtrata` (filter)          |    ●     | ● |  ●   | ●  |  ●  |
| `mappata` (map)              |    ●     | ● |  ●   | ●  |  ●  |
| `reducta` (reduce)           |    ●     | ● |  ●   | ●  |  ●  |
| `explanata` (flatMap)        |    ●     | — |  ●   | ●  |  ●  |
| `plana` (flat)               |    ●     | — |  ●   | ●  |  ●  |
| `inversa` (reverse copy)     |    ●     | ● |  ●   | ●  |  ●  |
| `ordinata` (sort copy)       |    ●     | ● |  ●   | ●  |  ●  |
| `sectio` (slice)             |    ●     | ● |  ●   | ●  |  ●  |
| `prima` (take first n)       |    ●     | ● |  ●   | ●  |  ●  |
| `ultima` (take last n)       |    ●     | ● |  ●   | ●  |  ●  |
| `omitte` (skip first n)      |    ●     | ● |  ●   | ●  |  ●  |
| `omnes` (every)              |    ●     | ● |  ●   | ●  |  ●  |
| `aliquis` (some)             |    ●     | ● |  ●   | ●  |  ●  |
| `coniunge` (join)            |    ●     | — |  ●   | ●  |  ●  |
| `perambula` (forEach)        |    ●     | ● |  ●   | ●  |  ●  |
| `filtra` (filter in-place)   |    ●     | — |  ●   | ○  |  ●  |
| `ordina` (sort in-place)     |    ●     | ● |  ●   | ●  |  ●  |
| `inverte` (reverse in-place) |    ●     | ● |  ●   | ●  |  ●  |
| `congrega` (groupBy)         |    ●     | — |  ●   | ○  |  ●  |
| `unica` (unique)             |    ●     | — |  ●   | ●  |  ●  |
| `planaOmnia` (flattenDeep)   |    ●     | — |  ○   | ○  |  ◐  |
| `fragmenta` (chunk)          |    ●     | — |  ●   | ●  |  ●  |
| `densa` (compact)            |    ●     | — |  ●   | ○  |  ●  |
| `partire` (partition)        |    ●     | — |  ●   | ●  |  ●  |
| `misce` (shuffle)            |    ●     | — |  ●   | ○  |  ●  |
| `specimen` (sample one)      |    ●     | — |  ●   | ○  |  ●  |
| `specimina` (sample n)       |    ●     | — |  ●   | ○  |  ●  |
| `summa` (sum)                |    ●     | ● |  ●   | ●  |  ●  |
| `medium` (average)           |    ●     | ● |  ●   | ●  |  ●  |
| `minimus` (min)              |    ●     | ● |  ●   | ●  |  ●  |
| `maximus` (max)              |    ●     | ● |  ●   | ●  |  ●  |
| `minimusPer` (minBy)         |    ●     | — |  ○   | ●  |  ●  |
| `maximusPer` (maxBy)         |    ●     | — |  ○   | ●  |  ●  |
| `numera` (count)             |    ●     | ● |  ●   | ●  |  ●  |

## Tabula (Map) Methods

| Latin                      | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pone` (set)               |    ●     | ● |  ●   | ●  |  ●  |
| `accipe` (get)             |    ●     | ● |  ●   | ●  |  ●  |
| `habet` (has)              |    ●     | ● |  ●   | ●  |  ●  |
| `dele` (delete)            |    ●     | ● |  ●   | ●  |  ●  |
| `longitudo` (size)         |    ●     | ● |  ●   | ●  |  ●  |
| `vacua` (isEmpty)          |    ●     | ● |  ●   | ●  |  ●  |
| `purga` (clear)            |    ●     | ● |  ●   | ●  |  ●  |
| `claves` (keys)            |    ●     | ● |  ●   | ●  |  ●  |
| `valores` (values)         |    ●     | ● |  ●   | ●  |  ●  |
| `paria` (entries)          |    ●     | ● |  ●   | ●  |  ●  |
| `accipeAut` (getOrDefault) |    ●     | ● |  ●   | ●  |  ●  |
| `selige` (pick)            |    ●     | — |  ●   | ○  |  ●  |
| `omitte` (omit)            |    ●     | — |  ●   | ○  |  ●  |
| `confla` (merge)           |    ●     | — |  ●   | ●  |  ●  |
| `inversa` (invert)         |    ●     | — |  ●   | ○  |  ●  |
| `mappaValores` (mapValues) |    ●     | — |  ●   | ○  |  ●  |
| `mappaClaves` (mapKeys)    |    ●     | — |  ●   | ○  |  ●  |
| `inLista` (toArray)        |    ●     | — |  ●   | ●  |  ●  |
| `inObjectum` (toObject)    |    ●     | — |  ●   | —  |  —  |

## Copia (Set) Methods

| Latin                         | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `adde` (add)                  |    ●     | ● |  ●   | ●  |  ●  |
| `habet` (has)                 |    ●     | ● |  ●   | ●  |  ●  |
| `dele` (delete)               |    ●     | ● |  ●   | ●  |  ●  |
| `longitudo` (size)            |    ●     | ● |  ●   | ●  |  ●  |
| `vacua` (isEmpty)             |    ●     | ● |  ●   | ●  |  ●  |
| `purga` (clear)               |    ●     | ● |  ●   | ●  |  ●  |
| `unio` (union)                |    ●     | — |  ●   | ●  |  ●  |
| `intersectio` (intersection)  |    ●     | — |  ●   | ●  |  ●  |
| `differentia` (difference)    |    ●     | — |  ●   | ●  |  ●  |
| `symmetrica` (symmetric diff) |    ●     | — |  ●   | ●  |  ●  |
| `subcopia` (isSubset)         |    ●     | — |  ●   | ●  |  ●  |
| `supercopia` (isSuperset)     |    ●     | — |  ●   | ●  |  ●  |
| `inLista` (toArray)           |    ●     | — |  ●   | ●  |  ●  |
| `valores` (values)            |    ●     | ● |  ●   | ○  |  ●  |
| `perambula` (forEach)         |    ●     | — |  ●   | ●  |  ●  |

## Stdlib: Time (tempus)

| Feature                  | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `nunc()` (current epoch) |    ●     | ● |  ●   | ●  |  ●  |
| `nunc_nano()` (nanos)    |    ●     | ● |  ●   | ●  |  ●  |
| `nunc_secunda()` (secs)  |    ●     | ● |  ●   | ●  |  ●  |
| `dormi ms` (sleep)       |    ●     | ● |  ●   | ●  |  ●  |
| Duration constants       |    ●     | ● |  ●   | ●  |  ●  |

## Stdlib: File I/O (solum)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `lege` (read file)        |    ○     | ○ |  ○   | ○  |  ○  |
| `inscribe` (write file)   |    ○     | ○ |  ○   | ○  |  ○  |
| `appone` (append)         |    ○     | ○ |  ○   | ○  |  ○  |
| `aperi` / `claude` (open) |    ○     | ○ |  ○   | ○  |  ○  |
| `exstat` (exists)         |    ○     | ○ |  ○   | ○  |  ○  |
| `dele` (delete)           |    ○     | ○ |  ○   | ○  |  ○  |
| `duplica` (copy)          |    ○     | ○ |  ○   | ○  |  ○  |
| `move` (rename)           |    ○     | ○ |  ○   | ○  |  ○  |
| `crea` (mkdir)            |    ○     | ○ |  ○   | ○  |  ○  |
| `elenca` (readdir)        |    ○     | ○ |  ○   | ○  |  ○  |
| `via.*` (path utils)      |    ○     | ○ |  ○   | ○  |  ○  |

## Stdlib: Network (caelum)

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pete` (HTTP GET)    |    ○     | ○ |  ○   | ○  |  ○  |
| `mitte` (HTTP POST)  |    ○     | ○ |  ○   | ○  |  ○  |
| `pone` (HTTP PUT)    |    ○     | ○ |  ○   | ○  |  ○  |
| `dele` (HTTP DELETE) |    ○     | ○ |  ○   | ○  |  ○  |
| WebSocket client     |    ○     | ○ |  ○   | ○  |  ○  |
| TCP/UDP sockets      |    ○     | ○ |  ○   | ○  |  ○  |

## Stdlib: Crypto

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `digere` (hash)           |    ○     | ○ |  ○   | ○  |  ○  |
| `hmac` (HMAC)             |    ○     | ○ |  ○   | ○  |  ○  |
| `cifra` (encrypt)         |    ○     | ○ |  ○   | ○  |  ○  |
| `decifra` (decrypt)       |    ○     | ○ |  ○   | ○  |  ○  |
| `fortuita` (random bytes) |    ○     | ○ |  ○   | ○  |  ○  |
| `deriva` (key derivation) |    ○     | ○ |  ○   | ○  |  ○  |

## Stdlib: Encoding (codex)

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `coda` (encode)      |    ○     | ○ |  ○   | ○  |  ○  |
| `decoda` (decode)    |    ○     | ○ |  ○   | ○  |  ○  |
| Base64/Base64URL     |    ○     | ○ |  ○   | ○  |  ○  |
| Hex encoding         |    ○     | ○ |  ○   | ○  |  ○  |
| URL percent-encoding |    ○     | ○ |  ○   | ○  |  ○  |

## Stdlib: Compression (comprimo)

| Feature               | TypeScript | Zig | Python | Rust | C++23 |
| --------------------- | :--------: | :-: | :----: | :--: | :---: |
| `comprimo` (compress) |    ○     | ○ |  ○   | ○  |  ○  |
| `laxo` (decompress)   |    ○     | ○ |  ○   | ○  |  ○  |
| gzip/zstd/brotli      |    ○     | ○ |  ○   | ○  |  ○  |

## Stdlib: Database (arca)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Query DSL (`de...quaere`) |    ○     | ○ |  ○   | ○  |  ○  |
| Mutations (`in...muta`)   |    ○     | ○ |  ○   | ○  |  ○  |
| Transactions              |    ○     | ○ |  ○   | ○  |  ○  |
| SQLite embedded           |    —     | ○ |  —   | —  |  —  |

## Collection DSL

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ex...prima n` (take)     |    ●     | ○ |  ●   | ○  |  ○  |
| `ex...ultima n` (last)    |    ●     | ○ |  ●   | ○  |  ○  |
| `ex...summa` (sum)        |    ●     | ○ |  ●   | ○  |  ○  |
| `ab...ubi` (filter where) |    ◐     | ○ |  ◐   | ○  |  ○  |
| `ab...pro` (filter iter)  |    ○     | ○ |  ○   | ○  |  ○  |

## External Dispatch (ad)

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ad "target" (args)` |    ○     | ○ |  ○   | ○  |  ○  |
| Syscall dispatch     |    ○     | ○ |  ○   | ○  |  ○  |
| URL protocol routing |    ○     | ○ |  ○   | ○  |  ○  |
| Package dispatch     |    ○     | ○ |  ○   | ○  |  ○  |

## Nucleus Runtime

The Nucleus is Faber's micro-kernel runtime providing unified I/O dispatch, message-passing protocol, and async execution across all targets. See `consilia/futura/nucleus.md` for full design.

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Responsum protocol         |    ●     | ○ |  ○   | ○  |  ○  |
| Handle abstraction         |    ○     | ○ |  ○   | ○  |  ○  |
| Dispatcher (syscall table) |    ○     | ○ |  ○   | ○  |  ○  |
| Request correlation        |    ○     | ○ |  ○   | ○  |  ○  |
| AsyncContext executor      |    ○     | ○ |  ○   | ○  |  ○  |
| State machine codegen      |    —     | ○ |  —   | ○  |  ○  |

The Responsum protocol defines a tagged union for all syscall results.

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
