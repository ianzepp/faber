# Rivus

The bootstrap compiler for Faber Romanus, written entirely in Faber itself. Rivus ("stream" in Latin) demonstrates that Faber is self-hosting: the language can compile its own compiler. This serves as both a proof of concept and a comprehensive test of the language's expressiveness.

Unlike the reference compiler, Rivus uses Latin exclusively in its source code, serving as a showcase for writing substantial programs in pure Faber. The bootstrap compiler currently targets TypeScript output, with Zig support in development.

## Usage

```
rivus compile <file.fab>              # Compile to TypeScript
rivus compile <file.fab> -o out.ts    # Specify output file
```

> **Note:** Rivus must be built before use with `bun run build:rivus`. The bootstrap compiler has a narrower feature set than Faber, focusing on the subset needed for self-hosting.

## Implementation Status

| Target     | Tests | Status |
| ---------- | ----: | :----: |
| TypeScript |   741 |  100%  |
| Python     |     0 |   0%   |
| Rust       |     0 |   0%   |
| C++23      |     0 |   0%   |
| Zig        |     0 |   0%   |

> Status % = passing tests / TypeScript baseline (741). Run `bun test proba/runner.test.ts -t "@rivus @<target>"` to verify. 35 tests skipped (intrinsic I/O functions, deferred).

> **Artifex Status:** The artifex compiler (`bun run build:artifex`) is currently blocked by parser and semantic analyzer gaps. See issues #48 (multi-discriminant discerne), #49 (block scoping), #50 (member assignment), #51 (predeclaration types).

Status: ● implemented, ◐ partial, ○ not implemented, — not applicable, ◌ convention

## Type System

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `textus` (string)         | ● | ● | ○ | ○ | ○ |
| `numerus` (integer)       | ● | ● | ○ | ○ | ○ |
| `fractus` (float)         | ● | ● | ○ | ○ | ○ |
| `decimus` (decimal)       | ● | ● | ○ | ○ | ○ |
| `magnus` (bigint)         | ● | ● | ○ | ○ | ○ |
| `bivalens` (boolean)      | ● | ● | ○ | ○ | ○ |
| `nihil` (null)            | ● | ● | ○ | ○ | ○ |
| `vacuum` (void)           | ● | ● | ○ | ○ | ○ |
| `numquam` (never)         | ● | ● | ○ | ○ | ○ |
| `octeti` (bytes)          | ● | ● | ○ | ○ | ○ |
| `objectum` (object)       | ● | ● | ○ | ○ | ○ |
| `lista<T>` (array)        | ● | ● | ○ | ○ | ○ |
| `tabula<K,V>` (map)       | ● | ● | ○ | ○ | ○ |
| `copia<T>` (set)          | ● | ● | ○ | ○ | ○ |
| `series<T...>` (tuple)    | ○ | ○ | ○ | ○ | ○ |
| `promissum<T>` (promise)  | ● | ◐ | ○ | ○ | ○ |
| `erratum` (error)         | ● | ○ | ○ | ○ | ○ |
| `cursor<T>` (iterator)    | ● | ○ | ○ | ○ | ○ |
| `ignotum` (unknown)       | ● | ● | ○ | ○ | ○ |
| `curator` (allocator)     | — | ● | — | — | — |
| Nullable types (`T?`)     | ● | ● | ○ | ○ | ○ |
| Union types (`unio<A,B>`) | ● | ◐ | ○ | ○ | ○ |
| Generic type params       | ● | ● | ○ | ○ | ○ |
| Type aliases (`typus`)    | ● | ○ | ○ | ○ | ○ |
| typeof (`typus` RHS)      | ● | ○ | ○ | ○ | ○ |

## Variable Declarations

| Feature                      | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `varia` (mutable)            | ● | ● | ○ | ○ | ○ |
| `fixum` (immutable)          | ● | ● | ○ | ○ | ○ |
| `figendum` (async immutable) | ● | ○ | ○ | ○ | ○ |
| `variandum` (async mutable)  | ● | ○ | ○ | ○ | ○ |
| `nexum` (reactive field)     | ● | ○ | ○ | ○ | ○ |
| Type annotations             | ● | ● | ○ | ○ | ○ |
| Object destructuring         | ● | ○ | ○ | ○ | ○ |
| Array destructuring          | ● | ○ | ○ | ○ | ○ |
| Rest in destructuring        | ● | ○ | ○ | ○ | ○ |
| Skip pattern (`_`)           | ● | ○ | ○ | ○ | ○ |
| Negative indices `[-1]`      | ● | ○ | ○ | ○ | ○ |
| Slicing `[1..3]`             | ● | ○ | ○ | ○ | ○ |
| Inclusive slicing (`usque`)  | ● | ○ | ○ | ○ | ○ |
| Initializer expressions      | ● | ● | ○ | ○ | ○ |

## Enum & Tagged Union Declarations

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ordo` (enum)              | ● | ● | ○ | ○ | ○ |
| Enum variants              | ● | ● | ○ | ○ | ○ |
| Enum with values           | ● | ○ | ○ | ○ | ○ |
| `discretio` (tagged union) | ● | ○ | ○ | ○ | ○ |
| Variant fields             | ● | ○ | ○ | ○ | ○ |
| Generic discretio          | ● | ○ | ○ | ○ | ○ |
| `discerne` (variant match) | ● | ○ | ○ | ○ | ○ |

## Function Declarations

| Feature                            | TypeScript | Zig | Python | Rust | C++23 |
| ---------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Basic functions (`functio`)        | ● | ● | ○ | ○ | ○ |
| Parameters                         | ● | ● | ○ | ○ | ○ |
| Parameter type annotations         | ● | ● | ○ | ○ | ○ |
| Parameter aliasing (`ut`)          | ● | ○ | ○ | ○ | ○ |
| Parameter defaults (`vel`)         | ● | ○ | ○ | ○ | ○ |
| Parameter prepositions (`de`/`in`) | ● | ○ | ○ | ○ | ○ |
| Rest parameters (`ceteri`)         | ● | ○ | ○ | ○ | ○ |
| Return type annotation (`->`)      | ● | ● | ○ | ○ | ○ |
| `futura` (async prefix)            | ● | ○ | ○ | ○ | ○ |
| `cursor` (generator prefix)        | ● | ○ | ○ | ○ | ○ |
| Async generator                    | ● | ○ | ○ | ○ | ○ |
| Arrow functions                    | ● | ● | ○ | ○ | ○ |
| `fit T` (sync return)              | ● | ○ | ○ | ○ | ○ |
| `fiet T` (async return)            | ● | ○ | ○ | ○ | ○ |
| `fiunt T` (generator return)       | ● | ○ | ○ | ○ | ○ |
| `fient T` (async generator return) | ● | ○ | ○ | ○ | ○ |
| `prae` (comptime type param)       | ● | ○ | ○ | ○ | ○ |
| `@ externa` (external decl)        | ● | ● | ○ | ○ | ○ |

## Control Flow Statements

| Feature                       | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `si` (if)                     | ● | ● | ○ | ○ | ○ |
| `secus` (else)                | ● | ● | ○ | ○ | ○ |
| `sin` (else if)               | ● | ● | ○ | ○ | ○ |
| `dum` (while)                 | ● | ● | ○ | ○ | ○ |
| `ex...pro` (for-of)           | ● | ● | ○ | ○ | ○ |
| `ex...fit` (for-of verb form) | ● | ○ | ○ | ○ | ○ |
| `ex...fiet` (async for)       | ● | ○ | ○ | ○ | ○ |
| `ex...pro (i, n)` (indexed)   | ● | ○ | ○ | ○ | ○ |
| `de...pro` (for-in)           | ● | ○ | ○ | ○ | ○ |
| Range `..` (exclusive)        | ● | ● | ○ | ○ | ○ |
| Range `ante` (exclusive)      | ● | ● | ○ | ○ | ○ |
| Range `usque` (inclusive)     | ● | ● | ○ | ○ | ○ |
| Range with step (`per`)       | ● | ○ | ○ | ○ | ○ |
| `in` (mutation block)         | ● | ○ | ○ | ○ | ○ |
| `elige` (switch)              | ● | ○ | ○ | ○ | ○ |
| Switch cases (`si`)           | ● | ○ | ○ | ○ | ○ |
| Switch default (`secus`)      | ● | ○ | ○ | ○ | ○ |
| `discerne` (pattern match)    | ● | ○ | ○ | ○ | ○ |
| `secus` (else/ternary alt)    | ● | ● | ○ | ○ | ○ |
| `fac` (do/block)              | ● | ○ | ○ | ○ | ○ |
| `ergo` (then, one-liner)      | ● | ○ | ○ | ○ | ○ |
| `reddit` (then return)        | ● | ● | ○ | ○ | ○ |
| `rumpe` (break)               | ● | ● | ○ | ○ | ○ |
| `perge` (continue)            | ● | ● | ○ | ○ | ○ |
| `custodi` (guard)             | ● | ○ | ○ | ○ | ○ |
| `cura` (resource management)  | ● | ● | ○ | ○ | ○ |
| `praefixum` (comptime block)  | ● | ○ | ○ | ○ | ○ |
| Catch on control flow         | ● | ○ | ○ | ○ | ○ |

## Return/Exit Statements

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `redde` (return)   | ● | ● | ○ | ○ | ○ |
| `redde` with value | ● | ● | ○ | ○ | ○ |
| `redde` void       | ● | ● | ○ | ○ | ○ |

## Exception Handling

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `tempta` (try)       | ● | — | ○ | ○ | ○ |
| `cape` (catch)       | ● | ○ | ○ | ○ | ○ |
| `demum` (finally)    | ● | — | ○ | — | ○ |
| `fac...cape` (block) | ● | ○ | ○ | ○ | ○ |
| `iace` (throw)       | ● | ○ | ○ | ○ | ○ |
| `adfirma` (assert)   | ● | ○ | ○ | ○ | ○ |
| Assert with message  | ● | ○ | ○ | ○ | ○ |
| `mori` (panic/fatal) | ● | ○ | ○ | ○ | ○ |

## Output/Debug/Events

| Feature            | TypeScript | Zig | Python | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :---: |
| `scribe` statement | ● | ● | ○ | ○ | ○ |
| `vide` (debug)     | ● | ○ | ○ | ○ | ○ |
| `mone` (warn)      | ● | ○ | ○ | ○ | ○ |
| Multiple args      | ● | ● | ○ | ○ | ○ |

## Expressions

| Feature                             | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Identifiers                         | ● | ● | ○ | ○ | ○ |
| `ego` (this/self)                   | ● | ● | ○ | ○ | ○ |
| Boolean literals (`verum`/`falsum`) | ● | ● | ○ | ○ | ○ |
| `nihil` literal                     | ● | ● | ○ | ○ | ○ |
| String literals                     | ● | ● | ○ | ○ | ○ |
| Number literals                     | ● | ● | ○ | ○ | ○ |
| Hex literals (`0xFF`)               | ● | ● | ○ | ○ | ○ |
| Binary literals (`0b1010`)          | ● | ● | ○ | ○ | ○ |
| Octal literals (`0o755`)            | ● | ● | ○ | ○ | ○ |
| BigInt literals (`123n`)            | ○ | ○ | ○ | ○ | ○ |
| Template literals                   | ● | ○ | ○ | ○ | ○ |
| `scriptum()` format strings         | ● | ● | ○ | ○ | ○ |
| Regex literals (`sed`)              | ● | — | ○ | ○ | ○ |
| Array literals                      | ● | ● | ○ | ○ | ○ |
| Array spread (`sparge`)             | ● | ○ | ○ | ○ | ○ |
| Object literals                     | ● | ● | ○ | ○ | ○ |
| Object spread (`sparge`)            | ● | ○ | ○ | ○ | ○ |
| Binary operators                    | ● | ● | ○ | ○ | ○ |
| Comparison operators                | ● | ● | ○ | ○ | ○ |
| `intra` (range containment)         | ● | ● | ○ | ○ | ○ |
| `inter` (set membership)            | ● | ● | ○ | ○ | ○ |
| Logical operators                   | ● | ● | ○ | ○ | ○ |
| Bitwise operators                   | ● | ○ | ○ | ○ | ○ |
| Unary operators                     | ● | ● | ○ | ○ | ○ |
| `nulla` (is empty)                  | ● | ○ | ○ | ○ | ○ |
| `nonnulla` (has content)            | ● | ○ | ○ | ○ | ○ |
| `nihil x` (is null)                 | ● | ● | ○ | ○ | ○ |
| `nonnihil x` (is not null)          | ● | ● | ○ | ○ | ○ |
| `negativum` (is negative)           | ● | ● | ○ | ○ | ○ |
| `positivum` (is positive)           | ● | ● | ○ | ○ | ○ |
| `verum x` (is true)                 | ● | ● | ○ | ○ | ○ |
| `falsum x` (is false)               | ● | ● | ○ | ○ | ○ |
| Member access (`.`)                 | ● | ● | ○ | ○ | ○ |
| Optional chaining (`?.`)            | ● | ● | ○ | ○ | ○ |
| Non-null assertion (`!.`)           | ● | ○ | ○ | ○ | ○ |
| Computed access (`[]`)              | ● | ● | ○ | ○ | ○ |
| Function calls                      | ● | ● | ○ | ○ | ○ |
| Call spread (`sparge`)              | ● | ○ | ○ | ○ | ○ |
| Method calls                        | ● | ● | ○ | ○ | ○ |
| Assignment                          | ● | ● | ○ | ○ | ○ |
| Compound assignment (`+=`, etc.)    | ● | ○ | ○ | ○ | ○ |
| Conditional (ternary)               | ● | ● | ○ | ○ | ○ |
| `sic`/`secus` ternary syntax        | ● | ● | ○ | ○ | ○ |
| `cede` (await/yield)                | ● | ○ | ○ | ○ | ○ |
| `novum` (new)                       | ● | ○ | ○ | ○ | ○ |
| `novum...de` (new with props)       | ● | ○ | ○ | ○ | ○ |
| `===` / `est` (strict equality)     | ● | ○ | ○ | ○ | ○ |
| `!==` / `non est` (strict ineq.)    | ● | ○ | ○ | ○ | ○ |
| `est` (instanceof/typeof)           | ● | ○ | ○ | ○ | ○ |
| `qua` (type cast)                   | ● | ● | ○ | ○ | ○ |
| `innatum` (native construction)     | ● | ● | ○ | ○ | ○ |
| `numeratum` (to integer)            | ● | ○ | ○ | ○ | ○ |
| `fractatum` (to float)              | ● | ○ | ○ | ○ | ○ |
| `textatum` (to string)              | ● | ○ | ○ | ○ | ○ |
| `bivalentum` (to boolean)           | ● | ○ | ○ | ○ | ○ |
| `aut` (logical or)                  | ● | ● | ○ | ○ | ○ |
| `vel` (nullish coalescing)          | ● | ○ | ○ | ○ | ○ |
| `praefixum` (comptime expr)         | ● | ○ | ○ | ○ | ○ |

## Lambda Syntax

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `pro x: expr` (expression)     | ● | ● | ○ | ○ | ○ |
| `pro x { body }` (block)       | ● | ◐ | ○ | ○ | ○ |
| `pro: expr` (zero-param)       | ● | ● | ○ | ○ | ○ |
| `pro x -> T: expr` (ret. type) | ● | ◐ | ○ | ○ | ○ |
| `fit x: expr` (sync binding)   | ● | ○ | ○ | ○ | ○ |
| `per property` (shorthand)     | ● | ○ | ○ | ○ | ○ |

## OOP Features (genus/pactum)

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `genus` declaration       | ● | ● | ○ | ○ | ○ |
| Field declarations        | ● | ● | ○ | ○ | ○ |
| Field defaults            | ● | ○ | ○ | ○ | ○ |
| `nexum` (reactive field)  | ● | ○ | ○ | ○ | ○ |
| Static fields (`generis`) | ● | ○ | ○ | ○ | ○ |
| `@ privatum` (private)    | ● | ○ | ○ | ○ | ○ |
| `@ protectum` (protected) | ● | ○ | ○ | ○ | ○ |
| `creo` (constructor hook) | ● | ○ | ○ | ○ | ○ |
| `deleo` (destructor)      | ◌ | ◌ | ◌ | ◌ | ◌ |
| `pingo` (render method)   | ◌ | ◌ | ◌ | ◌ | ◌ |
| Auto-merge constructor    | ● | ○ | ○ | ○ | ○ |
| Methods                   | ● | ○ | ○ | ○ | ○ |
| Async methods             | ● | ○ | ○ | ○ | ○ |
| Generator methods         | ● | ○ | ○ | ○ | ○ |
| `sub` (extends)           | ● | — | ○ | ○ | ○ |
| `implet` (implements)     | ● | — | ○ | ○ | ○ |
| Multiple `implet`         | ● | — | ○ | ○ | ○ |
| `@ abstractum` class      | ● | — | ○ | ○ | ○ |
| `@ abstracta` method      | ● | — | ○ | ○ | ○ |
| `aperit` (index sig)      | ● | — | — | — | — |
| Generic classes           | ● | ○ | ○ | ○ | ○ |
| `pactum` declaration      | ● | — | ○ | ○ | ○ |
| Interface methods         | ● | — | ○ | ○ | ○ |

## Import/Export

| Feature                        | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :---: |
| `ex...importa` (named imports) | ● | ○ | ○ | ○ | ○ |
| `ex...importa *` (wildcard)    | ● | ○ | ○ | ○ | ○ |
| `ut` alias (import renaming)   | ● | ○ | ○ | ○ | ○ |

## Testing

| Feature                         | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `proba` (test case)             | ● | ○ | ○ | ○ | ○ |
| `probandum` (test suite)        | ● | ○ | ○ | ○ | ○ |
| `praepara` (beforeEach)         | ● | ○ | ○ | ○ | ○ |
| `postpara` (afterEach)          | ● | ○ | ○ | ○ | ○ |
| `praepara omnia` (beforeAll)    | ● | ○ | ○ | ○ | ○ |
| `postpara omnia` (afterAll)     | ● | ○ | ○ | ○ | ○ |
| `praeparabit` (async beforeEach)| ● | ○ | ○ | ○ | ○ |
| `postparabit` (async afterEach) | ● | ○ | ○ | ○ | ○ |
| `omitte` modifier (skip)        | ● | ○ | ○ | ○ | ○ |
| `solum` modifier (only)         | ○ | ○ | ○ | ○ | ○ |
| `futurum` modifier (todo)       | ● | ○ | ○ | ○ | ○ |
| Table-driven tests (`proba ex`) | ○ | ○ | ○ | ○ | ○ |

## Preamble / Prologue

| Feature                 | TypeScript | Zig | Python | Rust | C++23 |
| ----------------------- | :--------: | :-: | :----: | :--: | :---: |
| Preamble infrastructure | ● | ◐ | ○ | ○ | ○ |
| Panic class/import      | ● | — | ○ | — | — |
| Decimal import          | ● | — | ○ | — | — |
| Enum import             | — | — | ○ | — | — |
| Regex import            | — | — | ○ | ○ | — |
| Collection imports      | — | ◐ | ○ | ○ | — |
| Async imports           | — | ○ | ○ | ○ | — |
| Arena allocator         | — | ● | — | ○ | — |
| Curator tracking        | — | ● | — | ○ | — |
| Flumina/Responsum       | ○ | ○ | ○ | ○ | ○ |

## I/O Intrinsics

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `_scribe` (print)    | ○ | ○ | ○ | ○ | ○ |
| `_vide` (debug)      | ○ | ○ | ○ | ○ | ○ |
| `_mone` (warn)       | ○ | ○ | ○ | ○ | ○ |
| `_lege` (read input) | ○ | ○ | ○ | ○ | ○ |

## Standard Library (norma)

See [`fons/norma/README.md`](../norma/README.md) for stdlib method implementations per target. Rivus auto-inherits translations from `norma-registry.gen.fab`.

## Collection DSL

| Feature                   | TypeScript | Zig | Python | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ex...prima n` (take)     | ○ | ○ | ○ | ○ | ○ |
| `ex...ultima n` (last)    | ○ | ○ | ○ | ○ | ○ |
| `ex...summa` (sum)        | ○ | ○ | ○ | ○ | ○ |
| `ab...ubi` (filter where) | ○ | ○ | ○ | ○ | ○ |
| `ab...pro` (filter iter)  | ○ | ○ | ○ | ○ | ○ |

## External Dispatch (ad)

| Feature              | TypeScript | Zig | Python | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :---: |
| `ad "target" (args)` | ○ | ○ | ○ | ○ | ○ |
| Syscall dispatch     | ○ | ○ | ○ | ○ | ○ |
| URL protocol routing | ○ | ○ | ○ | ○ | ○ |
| Package dispatch     | ○ | ○ | ○ | ○ | ○ |

## Nucleus Runtime

The Nucleus is Faber's micro-kernel runtime providing unified I/O dispatch, message-passing protocol, and async execution across all targets. See `consilia/futura/nucleus.md` for full design.

| Feature                    | TypeScript | Zig | Python | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :---: |
| Responsum protocol         | ○ | ○ | ○ | ○ | ○ |
| Handle abstraction         | ○ | ○ | ○ | ○ | ○ |
| Dispatcher (syscall table) | ○ | ○ | ○ | ○ | ○ |
| Request correlation        | ○ | ○ | ○ | ○ | ○ |
| AsyncContext executor      | ○ | ○ | ○ | ○ | ○ |
| State machine codegen      | — | ○ | — | ○ | ○ |

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

