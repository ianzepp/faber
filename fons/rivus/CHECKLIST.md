# Rivus

The bootstrap compiler for Faber Romanus, written entirely in Faber itself. Rivus ("stream" in Latin) demonstrates that Faber is self-hosting: the language can compile its own compiler. This serves as both a proof of concept and a comprehensive test of the language's expressiveness.

Unlike the reference compiler, Rivus uses Latin exclusively in its source code, serving as a showcase for writing substantial programs in pure Faber. The bootstrap compiler currently targets TypeScript output, with Zig support in development.

## Implementation Status

| Target     | Tests | Status |
| ---------- | ----: | :----: |
| TypeScript |   741 |  100%  |
| Python     |     0 |   0%   |
| Rust       |     0 |   0%   |
| C++23      |     0 |   0%   |
| Zig        |     0 |   0%   |

> Status % = passing tests / TypeScript baseline (741). Run `bun test proba/runner.test.ts -t "@rivus @<target>"` to verify. 35 tests skipped (intrinsic I/O functions, deferred).

Status key: `[x]` implemented, `[~]` partial, `[ ]` not implemented, `[-]` not applicable, `[c]` convention (no compiler support needed)

## Type System

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `textus` (string)         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `numerus` (integer)       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `fractus` (float)         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `decimus` (decimal)       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `magnus` (bigint)         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `bivalens` (boolean)      |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `nihil` (null)            |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `vacuum` (void)           |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `numquam` (never)         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `octeti` (bytes)          |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `objectum` (object)       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `lista<T>` (array)        |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `tabula<K,V>` (map)       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `copia<T>` (set)          |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `series<T...>` (tuple)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `promissum<T>` (promise)  |    [x]     | [~] |  [ ]   | [ ]  |  [ ]  |
| `erratum` (error)         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cursor<T>` (iterator)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ignotum` (unknown)       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `curator` (allocator)     |    [-]     | [x] |  [-]   | [-]  |  [-]  |
| Nullable types (`T?`)     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Union types (`unio<A,B>`) |    [x]     | [~] |  [ ]   | [ ]  |  [ ]  |
| Generic type params       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Type aliases (`typus`)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| typeof (`typus` RHS)      |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Variable Declarations

| Feature                      | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `varia` (mutable)            |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `fixum` (immutable)          |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `figendum` (async immutable) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `variandum` (async mutable)  |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `nexum` (reactive field)     |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Type annotations             |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Object destructuring         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Array destructuring          |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Rest in destructuring        |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Skip pattern (`_`)           |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Negative indices `[-1]`      |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Slicing `[1..3]`             |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Inclusive slicing (`usque`)  |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Initializer expressions      |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |

## Enum & Tagged Union Declarations

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ordo` (enum)              |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Enum variants              |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Enum with values           |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `discretio` (tagged union) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Variant fields             |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Generic discretio          |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `discerne` (variant match) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Function Declarations

| Feature                            | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Basic functions (`functio`)        |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Parameters                         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Parameter type annotations         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Parameter aliasing (`ut`)          |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Parameter defaults (`vel`)         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Parameter prepositions (`de`/`in`) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Rest parameters (`ceteri`)         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Return type annotation (`->`)      |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `futura` (async prefix)            |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cursor` (generator prefix)        |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Async generator                    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Arrow functions                    |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `fit T` (sync return)              |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `fiet T` (async return)            |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `fiunt T` (generator return)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `fient T` (async generator return) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `prae` (comptime type param)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Control Flow Statements

| Feature                       | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `si` (if)                     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `secus` (else)                |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `sin` (else if)               |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `dum` (while)                 |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `ex...pro` (for-of)           |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `ex...fit` (for-of verb form) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ex...fiet` (async for)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ex...pro (i, n)` (indexed)   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `de...pro` (for-in)           |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Range `..` (exclusive)        |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Range `ante` (exclusive)      |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Range `usque` (inclusive)     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Range with step (`per`)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `in` (mutation block)         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `elige` (switch)              |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Switch cases (`si`)           |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Switch default (`secus`)      |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `discerne` (pattern match)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `secus` (else/ternary alt)    |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `fac` (do/block)              |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ergo` (then, one-liner)      |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `rumpe` (break)               |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `perge` (continue)            |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `custodi` (guard)             |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cura` (resource management)  |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `praefixum` (comptime block)  |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Catch on control flow         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Return/Exit Statements

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `redde` (return)   |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `redde` with value |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `redde` void       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |

## Exception Handling

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `tempta` (try)       |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `cape` (catch)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `demum` (finally)    |    [x]     | [-] |  [ ]   | [-]  |  [ ]  |
| `fac...cape` (block) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `iace` (throw)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `adfirma` (assert)   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Assert with message  |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `mori` (panic/fatal) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Output/Debug/Events

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `scribe` statement |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `vide` (debug)     |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `mone` (warn)      |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Multiple args      |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |

## Expressions

| Feature                             | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Identifiers                         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `ego` (this/self)                   |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Boolean literals (`verum`/`falsum`) |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `nihil` literal                     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| String literals                     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Number literals                     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Hex literals (`0xFF`)               |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Binary literals (`0b1010`)          |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Octal literals (`0o755`)            |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| BigInt literals (`123n`)            |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Template literals                   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `scriptum()` format strings         |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Regex literals (`sed`)              |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| Array literals                      |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Array spread (`sparge`)             |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Object literals                     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Object spread (`sparge`)            |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Binary operators                    |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Comparison operators                |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Logical operators                   |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Bitwise operators                   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Unary operators                     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `nulla` (is empty)                  |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `nonnulla` (has content)            |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `nihil x` (is null)                 |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `nonnihil x` (is not null)          |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `negativum` (is negative)           |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `positivum` (is positive)           |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `verum x` (is true)                 |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `falsum x` (is false)               |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Member access (`.`)                 |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Optional chaining (`?.`)            |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Non-null assertion (`!.`)           |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Computed access (`[]`)              |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Function calls                      |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Call spread (`sparge`)              |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Method calls                        |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Assignment                          |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Compound assignment (`+=`, etc.)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Conditional (ternary)               |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `sic`/`secus` ternary syntax        |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `cede` (await/yield)                |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `novum` (new)                       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `novum...de` (new with props)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `===` / `est` (strict equality)     |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `!==` / `non est` (strict ineq.)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `est` (instanceof/typeof)           |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `qua` (type cast)                   |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `aut` (logical or)                  |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `vel` (nullish coalescing)          |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `praefixum` (comptime expr)         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Lambda Syntax

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `pro x: expr` (expression)     |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `pro x { body }` (block)       |    [x]     | [~] |  [ ]   | [ ]  |  [ ]  |
| `pro: expr` (zero-param)       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| `pro x -> T: expr` (ret. type) |    [x]     | [~] |  [ ]   | [ ]  |  [ ]  |
| `fit x: expr` (sync binding)   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `per property` (shorthand)     |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## OOP Features (genus/pactum)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `genus` declaration       |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Field declarations        |    [x]     | [x] |  [ ]   | [ ]  |  [ ]  |
| Field defaults            |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `nexum` (reactive field)  |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Static fields (`generis`) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `@ privatum` (private)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `@ protectum` (protected) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `creo` (constructor hook) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `deleo` (destructor)      |    [c]     | [c] |  [c]   | [c]  |  [c]  |
| `pingo` (render method)   |    [c]     | [c] |  [c]   | [c]  |  [c]  |
| Auto-merge constructor    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Methods                   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Async methods             |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Generator methods         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `sub` (extends)           |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `implet` (implements)     |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| Multiple `implet`         |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `@ abstractum` class      |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `@ abstracta` method      |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `aperit` (index sig)      |    [x]     | [-] |  [-]   | [-]  |  [-]  |
| Generic classes           |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `pactum` declaration      |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |
| Interface methods         |    [x]     | [-] |  [ ]   | [ ]  |  [ ]  |

## Import/Export

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `ex...importa` (named imports) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ex...importa *` (wildcard)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ut` alias (import renaming)   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Testing

| Feature                         | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `proba` (test case)             |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `probandum` (test suite)        |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cura ante` (beforeEach)        |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cura post` (afterEach)         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cura ante omnia` (beforeAll)   |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cura post omnia` (afterAll)    |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `omitte` modifier (skip)        |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `solum` modifier (only)         |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `futurum` modifier (todo)       |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Table-driven tests (`proba ex`) |    [x]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Preamble / Prologue

| Feature                 | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------- | :--------: | :-: | :----: | :--: | :---: |
| Preamble infrastructure |    [x]     | [~] |  [ ]   | [ ]  |  [ ]  |
| Panic class/import      |    [x]     | [-] |  [ ]   | [-]  |  [-]  |
| Decimal import          |    [x]     | [-] |  [ ]   | [-]  |  [-]  |
| Enum import             |    [-]     | [-] |  [ ]   | [-]  |  [-]  |
| Regex import            |    [-]     | [-] |  [ ]   | [ ]  |  [-]  |
| Collection imports      |    [-]     | [~] |  [ ]   | [ ]  |  [-]  |
| Async imports           |    [-]     | [ ] |  [ ]   | [ ]  |  [-]  |
| Arena allocator         |    [-]     | [x] |  [-]   | [ ]  |  [-]  |
| Curator tracking        |    [-]     | [x] |  [-]   | [ ]  |  [-]  |
| Flumina/Responsum       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## I/O Intrinsics

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `_scribe` (print)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `_vide` (debug)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `_mone` (warn)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `_lege` (read input) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Math (mathesis)

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pavimentum(x)` (floor)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `tectum(x)` (ceiling)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `radix(x)` (sqrt)          |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `potentia(x, n)` (pow)     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `absolutum(x)` (abs)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `signum(x)` (sign)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `rotundum(x)` (round)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `truncatum(x)` (trunc)     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `logarithmus(x)` (log)     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `logarithmus10(x)` (log10) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `exponens(x)` (exp)        |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `sinus(x)` (sin)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `cosinus(x)` (cos)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `tangens(x)` (tan)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `minimus(a, b)` (min)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `maximus(a, b)` (max)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `constringens(x, lo, hi)`  |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `PI` (constant)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `E` (constant)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `TAU` (constant)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Random (aleator)

| Feature                       | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `fractus()` (random 0-1)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `inter(min, max)` (int)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `octeti(n)` (random bytes)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `uuid()` (UUID v4)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `selige(lista)` (random pick) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `misce(lista)` (shuffle copy) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `semen(n)` (seed)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Lista (Array) Methods

| Latin                        | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `adde` (push)                |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `addita` (push copy)         |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `praepone` (unshift)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `praeposita` (unshift copy)  |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `remove` (pop)               |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `remota` (pop copy)          |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `decapita` (shift)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `decapitata` (shift copy)    |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `purga` (clear)              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `primus` (first)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ultimus` (last)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `accipe` (at index)          |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `longitudo` (length)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `vacua` (is empty)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `continet` (includes)        |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `indiceDe` (indexOf)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `inveni` (find)              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `inveniIndicem` (findIndex)  |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `filtrata` (filter)          |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `mappata` (map)              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `reducta` (reduce)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `explanata` (flatMap)        |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `plana` (flat)               |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `inversa` (reverse copy)     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ordinata` (sort copy)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `sectio` (slice)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `prima` (take first n)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ultima` (take last n)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `omitte` (skip first n)      |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `omnes` (every)              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `aliquis` (some)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `coniunge` (join)            |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `perambula` (forEach)        |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `filtra` (filter in-place)   |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `ordina` (sort in-place)     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `inverte` (reverse in-place) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `congrega` (groupBy)         |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `unica` (unique)             |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `planaOmnia` (flattenDeep)   |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `fragmenta` (chunk)          |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `densa` (compact)            |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `partire` (partition)        |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `misce` (shuffle)            |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `specimen` (sample one)      |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `specimina` (sample n)       |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `summa` (sum)                |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `medium` (average)           |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `minimus` (min)              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `maximus` (max)              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `minimusPer` (minBy)         |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `maximusPer` (maxBy)         |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `numera` (count)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

## Tabula (Map) Methods

| Latin                      | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `pone` (set)               |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `accipe` (get)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `habet` (has)              |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `dele` (delete)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `longitudo` (size)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `vacua` (isEmpty)          |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `purga` (clear)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `claves` (keys)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `valores` (values)         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `paria` (entries)          |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `accipeAut` (getOrDefault) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `selige` (pick)            |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `omitte` (omit)            |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `confla` (merge)           |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `inversa` (invert)         |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `mappaValores` (mapValues) |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `mappaClaves` (mapKeys)    |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `inLista` (toArray)        |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `inObjectum` (toObject)    |    [ ]     | [-] |  [ ]   | [-]  |  [-]  |

## Copia (Set) Methods

| Latin                         | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `adde` (add)                  |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `habet` (has)                 |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `dele` (delete)               |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `longitudo` (size)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `vacua` (isEmpty)             |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `purga` (clear)               |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `unio` (union)                |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `intersectio` (intersection)  |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `differentia` (difference)    |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `symmetrica` (symmetric diff) |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `subcopia` (isSubset)         |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `supercopia` (isSuperset)     |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `inLista` (toArray)           |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |
| `valores` (values)            |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `perambula` (forEach)         |    [ ]     | [-] |  [ ]   | [ ]  |  [ ]  |

## Stdlib: Time (tempus)

| Feature                  | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `nunc()` (current epoch) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `nunc_nano()` (nanos)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `nunc_secunda()` (secs)  |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `dormi ms` (sleep)       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| Duration constants       |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |

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
| `ex...prima n` (take)     |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ex...ultima n` (last)    |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ex...summa` (sum)        |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
| `ab...ubi` (filter where) |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
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
| Responsum protocol         |    [ ]     | [ ] |  [ ]   | [ ]  |  [ ]  |
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
