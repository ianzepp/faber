# Faber Romanus

**The Roman Craftsman** — A Latin programming language that compiles to TypeScript, Python, Zig, or C++.

## Why Latin?

Latin makes implicit programming concepts explicit. By mapping Latin's morphological system to code semantics, Faber Romanus makes visible what other languages hide: the roles that values play, the flow of data, the relationship between caller and callee.

Case systems aren't a gimmick—they scaffold understanding of semantic roles. English hides this; only word order distinguishes subject from object. Latin makes roles explicit in the words themselves: _canis_ (subject) vs _canem_ (object). Programming has the same structures, just less visible. A function's parameters are objects (accusative). Its return value is a subject (nominative).

This is not a novelty language. It is an experiment in whether linguistic structure can scaffold understanding of computation.

## Principles

**Compiler as Tutor.** Error messages teach Latin grammar in context. The compiler never crashes on malformed input—it collects errors and continues, showing multiple issues at once. Each error is an opportunity to teach.

**Accessibility Over Purity.** Pragmatism beats philological correctness. You don't need to know Latin case declensions to write code. When Latin conventions conflict with programming conventions, we ask: which choice helps more people understand?

**Single-Target Pragmatism.** Faber projects compile to one target language. Foreign imports work natively—`ex hono importa Hono` becomes `import { Hono } from 'hono'`. You use your target's libraries directly.

## Quick Start

```bash
bun install
bun run fons/cli.ts compile exempla/fundamenta/salve.fab        # TypeScript (default)
bun run fons/cli.ts compile exempla/fundamenta/salve.fab -t zig # Zig
bun run fons/cli.ts run exempla/fundamenta/salve.fab            # Compile and run
bun test                                                         # Run tests
```

## Example

```
functio salve(nomen) -> textus {
  redde "Salve, " + nomen + "!"
}

fixum nomen = "Mundus"
scribe salve(nomen)
```

---

# Implementation Status

| Target     | Features |
| ---------- | :------: |
| TypeScript |   96%    |
| Python     |   87%    |
| Zig        |   24%    |
| C++23      |   17%    |
| Rust       |    0%    |

> Features % = count of `[x]` entries in the tables below divided by total feature rows (~200).

Status key: `[x]` implemented, `[~]` partial, `[ ]` not implemented, `[-]` not applicable, `[c]` convention (no compiler support needed)

## Type System

| Feature                   | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `textus` (string)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `numerus` (integer)       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `fractus` (float)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [x]  |
| `decimus` (decimal)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `magnus` (bigint)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `bivalens` (boolean)      |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `nihil` (null)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `vacuum` (void)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `numquam` (never)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `octeti` (bytes)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `objectum` (object)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `lista<T>` (array)        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `tabula<K,V>` (map)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `copia<T>` (set)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `series<T...>` (tuple)    |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `promissum<T>` (promise)  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `erratum` (error)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `cursor<T>` (iterator)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ignotum` (unknown)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Nullable types (`T?`)     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Union types (`unio<A,B>`) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Generic type params       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Type aliases (`typus`)    |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| typeof (`typus` RHS)      |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |

## Variable Declarations

| Feature                      | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ---------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `varia` (mutable)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `fixum` (immutable)          |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `figendum` (async immutable) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `variandum` (async mutable)  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `nexum` (reactive binding)   |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Type annotations             |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Object destructuring         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Array destructuring          |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Rest in destructuring        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Initializer expressions      |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |

## Enum Declarations

| Feature          | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ---------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `ordo` (enum)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Enum variants    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Enum with values |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Function Declarations

| Feature                            | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ---------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| Basic functions (`functio`)        |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Parameters                         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Parameter type annotations         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Rest parameters (`ceteri`)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Return type annotation (`->`)      |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `futura` (async prefix)            |    [x]     | [~] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `cursor` (generator prefix)        |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [-]  |
| Async generator                    |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [-]  |
| Arrow functions                    |    [x]     | [~] |  [x]   | [ ]  | [ ]  |  [x]  |
| `fit T` (sync return)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `fiet T` (async return)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `fiunt T` (generator return)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [-]  |
| `fient T` (async generator return) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [-]  |
| `prae` (comptime type param)       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Control Flow Statements

| Feature                       | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `si` (if)                     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `aliter` (else)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `aliter si` (else if)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `sin` (else if, poetic)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `dum` (while)                 |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `ex...pro` (for-of)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `de...pro` (for-in)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ex...fit` (for-of verb form) |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ex...fiet` (async for)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ex...pro (i, n)` (indexed)   |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Range expressions (`0..10`)   |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Range with step (`per`)       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `in` (mutation block)         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `elige` (switch)              |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Switch cases (`si`)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Switch default (`aliter`)     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `secus` (else/ternary alt)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `fac` (do/block)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ergo` (then, one-liner)      |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `rumpe` (break)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `perge` (continue)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `custodi` (guard)             |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `cura` (resource management)  |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `praefixum` (comptime block)  |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Catch on control flow         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Return/Exit Statements

| Feature            | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :--: | :---: |
| `redde` (return)   |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `redde` with value |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `redde` void       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |

## Exception Handling

| Feature              | TypeScript | Zig | Python | WASM | Rust | C++23 |
| -------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `tempta` (try)       |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [x]  |
| `cape` (catch)       |    [x]     | [~] |  [x]   | [ ]  | [ ]  |  [x]  |
| `demum` (finally)    |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `iace` (throw)       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `adfirma` (assert)   |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Assert with message  |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `mori` (panic/fatal) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Output/Debug/Events

| Feature            | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------ | :--------: | :-: | :----: | :--: | :--: | :---: |
| `scribe` statement |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `vide` (debug)     |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `mone` (warn)      |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Multiple args      |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |

## Expressions

| Feature                             | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| Identifiers                         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `ego` (this/self)                   |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Boolean literals (`verum`/`falsum`) |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `nihil` literal                     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| String literals                     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Number literals                     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Hex literals (`0xFF`)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| BigInt hex (`0xFFn`)                |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Template literals                   |    [x]     | [~] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Regex literals (`sed`)              |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Array literals                      |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Array spread (`sparge`)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Object literals                     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Object spread (`sparge`)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Binary operators                    |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Comparison operators                |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Logical operators                   |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Bitwise operators (`&\|^~<<>>`)     |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Unary operators                     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `nulla` (is empty)                  |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `nonnulla` (has content)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `negativum` (is negative)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `positivum` (is positive)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Member access (`.`)                 |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Optional chaining (`?.`)            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Non-null assertion (`!.`)           |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Computed access (`[]`)              |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Function calls                      |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Call spread (`sparge`)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Method calls                        |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Assignment                          |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Conditional (ternary)               |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `sic`/`secus` ternary syntax        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `cede` (await/yield)                |    [x]     | [~] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `novum` (new)                       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `novum...de` (new with props)       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `===` / `est` (strict equality)     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `!==` / `non est` (strict ineq.)    |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| `est` (instanceof/typeof)           |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `ut` (type cast)                    |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `aut` (logical or)                  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [x]  |
| `vel` (nullish coalescing)          |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |

## Lambda Syntax

| Feature                         | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `pro x redde expr` (expression) |    [x]     | [~] |  [~]   | [ ]  | [ ]  |  [ ]  |
| `pro x: expr` (colon shorthand) |    [x]     | [~] |  [~]   | [ ]  | [ ]  |  [ ]  |
| `pro x { body }` (block)        |    [x]     | [~] |  [~]   | [ ]  | [ ]  |  [ ]  |
| `pro redde expr` (zero-param)   |    [x]     | [~] |  [~]   | [ ]  | [ ]  |  [ ]  |
| `pro x -> T: expr` (ret. type)  |    [x]     | [x] |  [-]   | [ ]  | [ ]  |  [x]  |
| `(x) => expr` (JS-style)        |    [x]     | [~] |  [~]   | [ ]  | [ ]  |  [x]  |
| `per property` (shorthand)      |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |

## OOP Features (genus/pactum)

| Feature                   | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `genus` declaration       |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Field declarations        |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Field defaults            |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Computed fields (getters) |    [-]     | [-] |  [-]   | [-]  | [-]  |  [-]  |
| Static fields (`generis`) |    [x]     | [ ] |  [~]   | [ ]  | [ ]  |  [ ]  |
| Public/private visibility |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `creo` (constructor hook) |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `deleo` (destructor)      |    [c]     | [c] |  [c]   | [c]  | [c]  |  [c]  |
| `pingo` (render method)   |    [c]     | [c] |  [c]   | [c]  | [c]  |  [c]  |
| Auto-merge constructor    |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Methods                   |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Async methods             |    [x]     | [~] |  [x]   | [ ]  | [ ]  |  [ ]  |
| Generator methods         |    [x]     | [-] |  [x]   | [ ]  | [ ]  |  [-]  |
| `implet` (implements)     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `aperit` (index sig)      |    [ ]     | [-] |  [-]   | [-]  | [-]  |  [-]  |
| Generic classes           |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `pactum` declaration      |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
| Interface methods         |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |

## Import/Export

| Feature                        | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------------ | :--------: | :-: | :----: | :--: | :--: | :---: |
| `ex...importa` (named imports) |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ex...importa *` (wildcard)    |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Testing

| Feature                         | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ------------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `proba` (test case)             |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `probandum` (test suite)        |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `cura ante` (beforeEach)        |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `cura post` (afterEach)         |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `omitte` modifier (skip)        |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `solum` modifier (only)         |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `futurum` modifier (todo)       |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| Table-driven tests (`proba ex`) |    [ ]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |

## Preamble / Prologue

| Feature                 | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| Preamble infrastructure |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [x]  |
| Panic class/import      |    [x]     | [-] |  [ ]   | [ ]  | [-]  |  [-]  |
| Collection imports      |    [-]     | [ ] |  [ ]   | [ ]  | [ ]  |  [-]  |
| Async imports           |    [-]     | [ ] |  [ ]   | [ ]  | [ ]  |  [-]  |
| Arena allocator         |    [-]     | [ ] |  [-]   | [ ]  | [ ]  |  [-]  |

## Standard Library Intrinsics

| Feature               | TypeScript | Zig | Python | WASM | Rust | C++23 |
| --------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `_scribe` (print)     |    [x]     | [x] |  [x]   | [ ]  | [ ]  |  [x]  |
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
| `adde` (push)                |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `addita` (push copy)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `praepone` (unshift)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `praeposita` (unshift copy)  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `remove` (pop)               |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `remota` (pop copy)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `decapita` (shift)           |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `decapitata` (shift copy)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `purga` (clear)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `primus` (first)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ultimus` (last)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `accipe` (at index)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `longitudo` (length)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `vacua` (is empty)           |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `continet` (includes)        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `indiceDe` (indexOf)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inveni` (find)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inveniIndicem` (findIndex)  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `filtrata` (filter)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `mappata` (map)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `reducta` (reduce)           |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `explanata` (flatMap)        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `plana` (flat)               |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inversa` (reverse copy)     |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ordinata` (sort copy)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `sectio` (slice)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `prima` (take first n)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ultima` (take last n)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `omitte` (skip first n)      |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `omnes` (every)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `aliquis` (some)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `coniunge` (join)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `perambula` (forEach)        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `filtra` (filter in-place)   |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `ordina` (sort in-place)     |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inverte` (reverse in-place) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `congrega` (groupBy)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `unica` (unique)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `planaOmnia` (flattenDeep)   |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `fragmenta` (chunk)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `densa` (compact)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `partire` (partition)        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `misce` (shuffle)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `specimen` (sample one)      |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `specimina` (sample n)       |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `summa` (sum)                |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `medium` (average)           |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `minimus` (min)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `maximus` (max)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `minimusPer` (minBy)         |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `maximusPer` (maxBy)         |    [x]     | [ ] |  [ ]   | [ ]  | [ ]  |  [ ]  |
| `numera` (count)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Tabula (Map) Methods

| Latin                      | TypeScript | Zig | Python | WASM | Rust | C++23 |
| -------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `pone` (set)               |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `accipe` (get)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `habet` (has)              |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `dele` (delete)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `longitudo` (size)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `vacua` (isEmpty)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `purga` (clear)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `claves` (keys)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `valores` (values)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `paria` (entries)          |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `accipeAut` (getOrDefault) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `selige` (pick)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `omitte` (omit)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `confla` (merge)           |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inversa` (invert)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `mappaValores` (mapValues) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `mappaClaves` (mapKeys)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inLista` (toArray)        |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inObjectum` (toObject)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

## Copia (Set) Methods

| Latin                         | TypeScript | Zig | Python | WASM | Rust | C++23 |
| ----------------------------- | :--------: | :-: | :----: | :--: | :--: | :---: |
| `adde` (add)                  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `habet` (has)                 |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `dele` (delete)               |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `longitudo` (size)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `vacua` (isEmpty)             |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `purga` (clear)               |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `unio` (union)                |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `intersectio` (intersection)  |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `differentia` (difference)    |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `symmetrica` (symmetric diff) |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `subcopia` (isSubset)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `supercopia` (isSuperset)     |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `inLista` (toArray)           |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `valores` (values)            |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |
| `perambula` (forEach)         |    [x]     | [ ] |  [x]   | [ ]  | [ ]  |  [ ]  |

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

No classes (structs with methods), no interfaces (duck typing), no exceptions (error unions), no generators, comptime generics. `genus` becomes `const Name = struct { ... };`.

### Rust (2021 edition)

Ownership system, borrowing (`&`/`&mut`), `Option<T>`/`Result<T,E>` instead of null/exceptions, traits instead of interfaces, exhaustive pattern matching.

### C++23

`std::expected<T,E>` for errors, `std::print` for output, concepts for interfaces, coroutines for async, RAII for cleanup.

---

## Critical Note: Method Handler Architecture

The codegen method handlers (for `lista`, `tabula`, `copia`) receive arguments as a pre-joined string rather than a structured array. This creates a parsing ambiguity when arguments contain commas—such as multi-parameter lambdas like `lambda acc, n: (acc + n)`.

**Current flow (problematic):**

1. `genCallExpression` generates each argument as a string
2. Joins them with `", "` into a single string
3. Passes that string to the method handler
4. Method handler attempts to parse the string back apart

This is a lossy transformation. Once `["0", "lambda acc, n: (acc + n)"]` becomes `"0, lambda acc, n: (acc + n)"`, the boundary between arguments is lost.

**Required fix:** Pass structured data to method handlers:

```typescript
type PyGenerator = (obj: string, args: string[]) => string;
```

This affects all target languages (TypeScript, Python, Zig) since any language with multi-parameter functions/lambdas will have commas in argument strings that can't be reliably distinguished from argument separators.

---

## License

MIT
