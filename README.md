# Faber Romanus

**The Roman Craftsman** — A Latin programming language that compiles to TypeScript, Python, or Zig.

## Why Latin?

Latin makes implicit programming concepts explicit. By mapping Latin's morphological system to code semantics, Faber Romanus makes visible what other languages hide: the roles that values play, the flow of data, the relationship between caller and callee.

Case systems aren't a gimmick—they scaffold understanding of semantic roles. English hides this; only word order distinguishes subject from object. Latin makes roles explicit in the words themselves: *canis* (subject) vs *canem* (object). Programming has the same structures, just less visible. A function's parameters are objects (accusative). Its return value is a subject (nominative).

This is not a novelty language. It is an experiment in whether linguistic structure can scaffold understanding of computation.

## Principles

**Compiler as Tutor.** Error messages teach Latin grammar in context. The compiler never crashes on malformed input—it collects errors and continues, showing multiple issues at once. Each error is an opportunity to teach.

**Accessibility Over Purity.** Pragmatism beats philological correctness. You don't need to know Latin case declensions to write code. When Latin conventions conflict with programming conventions, we ask: which choice helps more people understand?

**Single-Target Pragmatism.** Faber projects compile to one target language. Foreign imports work natively—`ex hono importa Hono` becomes `import { Hono } from 'hono'`. You use your target's libraries directly.

## Quick Start

```bash
bun install
bun run fons/cli.ts compile exempla/salve.fab        # TypeScript (default)
bun run fons/cli.ts compile exempla/salve.fab -t zig # Zig
bun run fons/cli.ts run exempla/salve.fab            # Compile and run
bun test                                              # Run tests
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

Status key: `[x]` implemented, `[~]` partial, `[ ]` not implemented, `[-]` not applicable, `[c]` convention (no compiler support needed)

## Type System

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `textus` (string) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `numerus` (integer) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `fractus` (float) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `decimus` (decimal) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `bivalens` (boolean) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nihil` (null) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `vacuum` (void) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `octeti` (bytes) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `objectum` (object) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `lista<T>` (array) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `tabula<K,V>` (map) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `copia<T>` (set) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `promissum<T>` (promise) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `erratum` (error) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `cursor<T>` (iterator) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Nullable types (`T?`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Union types (`T | U`) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Generic type params | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Type aliases (`typus`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Variable Declarations

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `varia` (mutable) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `fixum` (immutable) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `figendum` (async immutable) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `variandum` (async mutable) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `nexum` (reactive binding) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Type annotations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Object destructuring | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Initializer expressions | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Enum Declarations

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `ordo` (enum) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Enum variants | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Enum with values | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Function Declarations

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| Basic functions (`functio`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Parameters | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Parameter type annotations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Return type annotation (`->`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `futura` (async prefix) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `cursor` (generator prefix) | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| Async generator | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| Arrow functions | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `fit T` (sync return) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `fiet T` (async return) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `fiunt T` (generator return) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `fient T` (async generator return) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Control Flow Statements

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `si` (if) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `aliter` (else) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `aliter si` (else if) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `sin` (else if, poetic) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `dum` (while) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...pro` (for-of) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `in...pro` (for-in) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...fit` (for-of verb form) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...fiet` (async for) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Range expressions (`0..10`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Range with step (`per`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `cum` (with/context) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `elige` (switch) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Switch cases (`si`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Switch default (`aliter`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `secus` (else/ternary alt) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `fac` (do/block) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `ergo` (then, one-liner) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `rumpe` (break) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `perge` (continue) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `custodi` (guard) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Catch on control flow | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## Return/Exit Statements

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `redde` (return) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `redde` with value | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `redde` void | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Exception Handling

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `tempta` (try) | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| `cape` (catch) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `demum` (finally) | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| `iace` (throw) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `adfirma` (assert) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Assert with message | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `mori` (panic/fatal) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Output/Debug/Events

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `scribe` statement | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `vide` (debug) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `mone` (warn) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `emitte` (emit event) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `ausculta` (event stream) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Multiple args | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Expressions

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| Identifiers | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ego` (this/self) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Boolean literals (`verum`/`falsum`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nihil` literal | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| String literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Number literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Template literals | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| Array literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Object literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Binary operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Comparison operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Logical operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Unary operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nulla` (is empty) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nonnulla` (has content) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `negativum` (is negative) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `positivum` (is positive) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Member access (`.`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Computed access (`[]`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Function calls | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Method calls | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Assignment | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Conditional (ternary) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `sic`/`secus` ternary syntax | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `cede` (await/yield) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `novum` (new) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `novum...cum` (new with props) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `est` (strict equality) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `aut` (logical or) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Lambda Syntax

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `pro x redde expr` (expression) | [x] | [~] | [~] | [ ] | [ ] | [ ] |
| `pro x { body }` (block) | [x] | [~] | [~] | [ ] | [ ] | [ ] |
| `pro redde expr` (zero-param) | [x] | [~] | [~] | [ ] | [ ] | [ ] |
| `(x) => expr` (JS-style) | [x] | [~] | [~] | [ ] | [ ] | [ ] |

## OOP Features (genus/pactum)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `genus` declaration | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Field declarations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Field defaults | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Computed fields (getters) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Static fields (`generis`) | [x] | [ ] | [~] | [ ] | [ ] | [ ] |
| Public/private visibility | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `creo` (constructor hook) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `deleo` (destructor) | [c] | [c] | [c] | [c] | [c] | [c] |
| `pingo` (render method) | [c] | [c] | [c] | [c] | [c] | [c] |
| Auto-merge constructor | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Methods | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Async methods | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| Generator methods | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| `implet` (implements) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Generic classes | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `pactum` declaration | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Interface methods | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Import/Export

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `ex...importa` (named imports) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...importa *` (wildcard) | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Preamble / Prologue

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| Preamble infrastructure | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Panic class/import | [x] | [-] | [ ] | [ ] | [-] | [ ] |
| Collection imports | [-] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Async imports | [-] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Arena allocator | [-] | [ ] | [-] | [ ] | [ ] | [ ] |

## Standard Library Intrinsics

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `_scribe` (print) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `_vide` (debug) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_mone` (warn) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_lege` (read input) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_fortuitus` (random) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_pavimentum` (floor) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_tectum` (ceiling) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_radix` (sqrt) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_potentia` (pow) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## Lista (Array) Methods

| Latin | TypeScript | Zig | Python | WASM | Rust | C++23 |
|-------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `adde` (push) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `addita` (push copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `praepone` (unshift) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `praeposita` (unshift copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `remove` (pop) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `remota` (pop copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `decapita` (shift) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `decapitata` (shift copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `purga` (clear) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `primus` (first) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `ultimus` (last) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `accipe` (at index) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `longitudo` (length) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `vacua` (is empty) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `continet` (includes) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `indiceDe` (indexOf) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `inveni` (find) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `inveniIndicem` (findIndex) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `filtrata` (filter) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `mappata` (map) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `reducta` (reduce) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `explanata` (flatMap) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `plana` (flat) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `inversa` (reverse copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `ordinata` (sort copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `sectio` (slice) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `prima` (take first n) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `ultima` (take last n) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `omitte` (skip first n) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `omnes` (every) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `aliquis` (some) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `coniunge` (join) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `perambula` (forEach) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `filtra` (filter in-place) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `ordina` (sort in-place) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `inverte` (reverse in-place) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `congrega` (groupBy) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `unica` (unique) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `planaOmnia` (flattenDeep) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `fragmenta` (chunk) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `densa` (compact) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `partire` (partition) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `misce` (shuffle) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `specimen` (sample one) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `specimina` (sample n) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `summa` (sum) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `medium` (average) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `minimus` (min) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `maximus` (max) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `minimusPer` (minBy) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `maximusPer` (maxBy) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `numera` (count) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Tabula (Map) Methods

| Latin | TypeScript | Zig | Python | WASM | Rust | C++23 |
|-------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `pone` (set) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `accipe` (get) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `habet` (has) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `dele` (delete) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `longitudo` (size) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `vacua` (isEmpty) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `purga` (clear) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `claves` (keys) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `valores` (values) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `paria` (entries) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `accipeAut` (getOrDefault) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `selige` (pick) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `omitte` (omit) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `confla` (merge) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `inversa` (invert) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `mappaValores` (mapValues) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `mappaClaves` (mapKeys) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `inLista` (toArray) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `inObjectum` (toObject) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Copia (Set) Methods

| Latin | TypeScript | Zig | Python | WASM | Rust | C++23 |
|-------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `adde` (add) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `habet` (has) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `dele` (delete) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `longitudo` (size) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `vacua` (isEmpty) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `purga` (clear) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `unio` (union) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `intersectio` (intersection) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `differentia` (difference) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `symmetrica` (symmetric diff) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `subcopia` (isSubset) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `supercopia` (isSuperset) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `inLista` (toArray) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `valores` (values) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `perambula` (forEach) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Stdlib: Time (tempus)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `nunc()` (current epoch) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `dormi ms` (sleep) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Duration constants | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

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

## License

MIT
