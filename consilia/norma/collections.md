---
status: partial
targets: [ts, py, zig, cpp]
note: Core types and methods complete; DSL syntax (see ab.md) and `per property` closures are planned
updated: 2024-12
---

# Collections Design

## Overview

Three core collection types wrap native implementations:

| Faber         | JavaScript | Zig            | Description                          |
| ------------- | ---------- | -------------- | ------------------------------------ |
| `T[]`         | `Array<T>` | `ArrayList(T)` | Ordered, indexed, duplicates allowed |
| `tabula<K,V>` | `Map<K,V>` | `HashMap(K,V)` | Key-value pairs, unique keys         |
| `copia<T>`    | `Set<T>`   | `HashSet(T)`   | Unique values, unordered             |

All collection type names are feminine (Latin convention for containers).

---

## Implementation Status

Method registries implemented in `fons/codegen/*/norma/`:

### TypeScript

| Collection    | Status       | File        | Notes      |
| ------------- | ------------ | ----------- | ---------- |
| `T[]`         | [x] Complete | `lista.ts`  | 46 methods |
| `tabula<K,V>` | [x] Complete | `tabula.ts` | 17 methods |
| `copia<T>`    | [x] Complete | `copia.ts`  | 14 methods |

### Zig

| Collection    | Status      | File        | Notes                                                 |
| ------------- | ----------- | ----------- | ----------------------------------------------------- |
| `T[]`         | [~] Partial | `lista.ts`  | 12 core methods; functional methods use @compileError |
| `tabula<K,V>` | [~] Partial | `tabula.ts` | 11 core methods                                       |
| `copia<T>`    | [~] Partial | `copia.ts`  | 7 core methods; set operations use @compileError      |

Zig implementation uses arena allocation. Programs using collections automatically emit:

```zig
var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
defer arena.deinit();
const alloc = arena.allocator();
```

Functional methods (`filtrata`, `mappata`, `reducta`, etc.) emit `@compileError` — use `ex...pro` loops instead.

### Python

| Collection    | Status       | File        | Notes      |
| ------------- | ------------ | ----------- | ---------- |
| `T[]`         | [x] Complete | `lista.ts`  | 46 methods |
| `tabula<K,V>` | [x] Complete | `tabula.ts` | 17 methods |
| `copia<T>`    | [x] Complete | `copia.ts`  | 14 methods |

**Not yet implemented (any target):**

- [-] Collection DSL (`ex`/`ab` filtering) — see `ab.md`
- [-] Closure syntax (`per property`, `{ .property }`) — requires parser work
- [-] Tabula/copia literals — no syntax for `tabula { k: v }` or `copia { a, b }`

Method dispatch uses `resolvedType` from semantic analysis to correctly route overlapping method names (e.g., `accipe` on lista vs tabula) to their respective implementations.

---

## T[]

**Etymology:** "list, border, edge" — a bounded sequence.

### Core Methods

| Faber                    | JS Equivalent  | Description     |
| ------------------------ | -------------- | --------------- |
| `adde(T)`                | `push`         | Add to end      |
| `remove() -> T?`         | `pop`          | Remove from end |
| `primus() -> T?`         | `[0]`          | First element   |
| `ultimus() -> T?`        | `at(-1)`       | Last element    |
| `longitudo() -> numerus` | `length`       | Count           |
| `vacua() -> bivalens`    | `length === 0` | Is empty        |
| `purgare()`              | `length = 0`   | Clear all       |

### Search & Test

| Faber                                  | JS Equivalent | Description           |
| -------------------------------------- | ------------- | --------------------- |
| `contine(T) -> bivalens`               | `includes`    | Contains element      |
| `indiceDe(T) -> numerus?`              | `indexOf`     | Find index            |
| `inveni((T) -> bivalens) -> T?`        | `find`        | Find first match      |
| `omnes((T) -> bivalens) -> bivalens`   | `every`       | All match predicate   |
| `aliquis((T) -> bivalens) -> bivalens` | `some`        | Any matches predicate |

### Transformation (Functional)

| Faber                            | JS Equivalent | Description    |
| -------------------------------- | ------------- | -------------- |
| `mappa((T) -> U) -> U[]`         | `map`         | Transform each |
| `filtra((T) -> bivalens) -> T[]` | `filter`      | Keep matching  |
| `reducere(U, (U,T) -> U) -> U`   | `reduce`      | Fold to value  |
| `coniunge(textus) -> textus`     | `join`        | Join to string |

### Lodash-Inspired Methods

| Faber                                    | lodash        | Description                 |
| ---------------------------------------- | ------------- | --------------------------- |
| `ordina((T) -> U) -> T[]`                | `sortBy`      | Sort by key function        |
| `congrega((T) -> K) -> tabula<K, T[]>`   | `groupBy`     | Group by key function       |
| `unica() -> T[]`                         | `uniq`        | Remove duplicates           |
| `plana() -> T[]`                         | `flatten`     | Flatten one level           |
| `planaOmnia() -> T[]`                    | `flattenDeep` | Flatten all levels          |
| `fragmenta(numerus) -> T[][]`            | `chunk`       | Split into chunks of size n |
| `densa() -> T[]`                         | `compact`     | Remove falsy values         |
| `partire((T) -> bivalens) -> (T[], T[])` | `partition`   | Split by predicate          |
| `misce() -> T[]`                         | `shuffle`     | Randomize order             |
| `specimen() -> T?`                       | `sample`      | Random element              |
| `specimina(numerus) -> T[]`              | `sampleSize`  | Random n elements           |
| `prima(numerus) -> T[]`                  | `take`        | First n elements            |
| `ultima(numerus) -> T[]`                 | `takeRight`   | Last n elements             |
| `omitte(numerus) -> T[]`                 | `drop`        | Skip first n                |
| `inversa() -> T[]`                       | `reverse`     | Reverse order               |

### Aggregation

| Faber                                | lodash    | Description         |
| ------------------------------------ | --------- | ------------------- |
| `summa() -> numerus`                 | `sum`     | Sum of numbers      |
| `medium() -> numerus`                | `mean`    | Average             |
| `minimus() -> T?`                    | `min`     | Minimum value       |
| `maximus() -> T?`                    | `max`     | Maximum value       |
| `minimusPer((T) -> U) -> T?`         | `minBy`   | Min by key function |
| `maximusPer((T) -> U) -> T?`         | `maxBy`   | Max by key function |
| `numera((T) -> bivalens) -> numerus` | `countBy` | Count matching      |

### Slicing and Negative Indices

**Status:** Implemented for all targets (TS, Py, Zig, C++).

**Negative indices** access elements from the end:

```fab
fixum last = nums[-1]      # last element
fixum secondLast = nums[-2] # second to last
```

Target mappings:

- TypeScript: `nums.at(-1)` (native `.at()` method)
- Python: `nums[-1]` (native)
- Zig: `nums[nums.len - 1]` (computed)
- C++: `nums[nums.size() - 1]` (computed)

**Slicing** uses range syntax inside brackets:

```fab
fixum slice = nums[1..3]       # elements 1, 2 (exclusive end)
fixum slice = nums[1 usque 3]  # elements 1, 2, 3 (inclusive end)
fixum tail = nums[-3..-1]      # last 3 except final
fixum tail = nums[-3 usque -1] # last 3 elements (to end)
```

Target mappings:

- TypeScript: `nums.slice(1, 3)`
- Python: `nums[1:3]` (native)
- Zig: `nums[1..3]` (native)
- C++: `std::vector<T>(nums.begin() + 1, nums.begin() + 3)`

### Open Questions

1. **Mutability**: Should `adde`/`remove` mutate in place or return new list?
    - JS convention: mutate in place
    - Functional convention: return new
    - Proposal: mutate in place, provide variants for immutable ops

---

## tabula<K, V>

**Etymology:** "board, tablet, table" — a writing surface with entries.

### Core Methods

| Faber                    | JS Equivalent | Description   |
| ------------------------ | ------------- | ------------- |
| `pone(K, V)`             | `set`         | Set key-value |
| `accipe(K) -> V?`        | `get`         | Get by key    |
| `habet(K) -> bivalens`   | `has`         | Key exists    |
| `dele(K) -> bivalens`    | `delete`      | Delete key    |
| `longitudo() -> numerus` | `size`        | Count         |
| `vacua() -> bivalens`    | `size === 0`  | Is empty      |
| `purgare()`              | `clear`       | Clear all     |

### Iteration

| Faber                      | JS Equivalent | Description    |
| -------------------------- | ------------- | -------------- |
| `claves() -> cursor<K>`    | `keys()`      | Iterate keys   |
| `valores() -> cursor<V>`   | `values()`    | Iterate values |
| `paria() -> cursor<(K,V)>` | `entries()`   | Iterate pairs  |

### Lodash-Inspired Methods

| Faber                                   | lodash             | Description              |
| --------------------------------------- | ------------------ | ------------------------ |
| `accipeAut(K, V) -> V`                  | `get` with default | Get or return default    |
| `selige(...K) -> tabula<K,V>`           | `pick`             | Keep only specified keys |
| `omitte(...K) -> tabula<K,V>`           | `omit`             | Remove specified keys    |
| `confla(tabula<K,V>) -> tabula<K,V>`    | `merge`            | Merge maps               |
| `inversa() -> tabula<V,K>`              | `invert`           | Swap keys and values     |
| `mappaValores((V) -> U) -> tabula<K,U>` | `mapValues`        | Transform values         |
| `mappaClaves((K) -> J) -> tabula<J,V>`  | `mapKeys`          | Transform keys           |

### Open Questions

1. **Object conversion**: `exObjecto(obj)` static constructor?
    - Useful for JS interop
    - Less relevant for Zig target

---

## copia<T>

**Etymology:** "abundance, supply" — a collection of resources.

### Core Methods

| Faber                    | JS Equivalent | Description |
| ------------------------ | ------------- | ----------- |
| `adde(T)`                | `add`         | Add element |
| `habet(T) -> bivalens`   | `has`         | Contains    |
| `dele(T) -> bivalens`    | `delete`      | Delete      |
| `longitudo() -> numerus` | `size`        | Count       |
| `vacua() -> bivalens`    | `size === 0`  | Is empty    |
| `purgare()`              | `clear`       | Clear all   |

### Set Operations

| Faber                               | JS Equivalent  | Description       |
| ----------------------------------- | -------------- | ----------------- |
| `unio(copia<T>) -> copia<T>`        | `union`        | A ∪ B             |
| `intersectio(copia<T>) -> copia<T>` | `intersection` | A ∩ B             |
| `differentia(copia<T>) -> copia<T>` | `difference`   | A - B             |
| `symmetrica(copia<T>) -> copia<T>`  | —              | (A - B) ∪ (B - A) |

### Predicates

| Faber                              | lodash       | Description    |
| ---------------------------------- | ------------ | -------------- |
| `subcopia(copia<T>) -> bivalens`   | `isSubset`   | Is subset of   |
| `supercopia(copia<T>) -> bivalens` | `isSuperset` | Is superset of |

### Open Questions

1. **Conversion to lista**: `inLista() -> T[]`?

---

## Array Destructuring

**Status:** Implemented for all targets (TS, Py, Zig, C++).

Positional extraction from arrays, using the same `ex` syntax as object destructuring.

### Why Array Destructuring?

Originally excluded because without rest syntax, it's limited to fixed-length patterns which are fragile. With `ceteri` (rest), array destructuring enables:

- Head/tail splitting for recursive algorithms
- Extracting first N elements while collecting remainder
- Tuple-style returns from functions

### Basic Syntax

```fab
fixum coords = [10, 20, 30]

# Extract by position
ex coords fixum [x, y, z]
scribe x  # 10
scribe y  # 20
scribe z  # 30
```

Parallels object destructuring: `ex obj fixum { a, b }` vs `ex arr fixum [a, b]`

### Rest Pattern with ceteri

The primary use case—split head from tail:

```fab
fixum items = [1, 2, 3, 4, 5]

# First element + rest
ex items fixum [first, ceteri rest]
scribe first  # 1
scribe rest   # [2, 3, 4, 5]

# First two + rest
ex items fixum [a, b, ceteri tail]
scribe a     # 1
scribe b     # 2
scribe tail  # [3, 4, 5]
```

### Practical Examples

**Recursive list processing:**

```fab
functio sum(numerus[] nums) -> numerus {
    si nums.vacua() { redde 0 }
    ex nums fixum [head, ceteri tail]
    redde head + sum(tail)
}

scribe sum([1, 2, 3, 4, 5])  # 15
```

**Command argument parsing:**

```fab
functio parseArgs(textus[] args) {
    ex args fixum [command, ceteri flags]

    elige {
        command est "build" => handleBuild(flags)
        command est "test" => handleTest(flags)
        secus => scribe "Unknown command:", command
    }
}
```

**Tuple-style returns:**

```fab
functio divide(numerus a, numerus b) -> numerus[] {
    redde [a / b, a % b]
}

ex divide(17, 5) fixum [quotient, remainder]
scribe quotient   # 3
scribe remainder  # 2
```

**Swapping values:**

```fab
varia a = 1
varia b = 2
[a, b] = [b, a]
scribe a  # 2
scribe b  # 1
```

### Skipping Elements

Use `_` (underscore) to skip positions:

```fab
fixum data = [1, 2, 3, 4, 5]

ex data fixum [_, second, _]     # skip first and third
ex data fixum [_, _, third]      # skip first two
ex data fixum [first, _, _, fourth]  # skip middle
```

### Mutable Bindings

Use `varia` for mutable destructured values:

```fab
ex coords varia [x, y, z]
x = x + 10
scribe x  # 20
```

### Comparison with Object Destructuring

| Aspect           | Object                  | Array                 |
| ---------------- | ----------------------- | --------------------- |
| Syntax           | `ex obj fixum { a, b }` | `ex arr fixum [a, b]` |
| Extraction       | By name                 | By position           |
| Rest             | `{ a, ceteri rest }`    | `[a, ceteri rest]`    |
| Self-documenting | Yes (names)             | No (positions)        |
| Fragile          | No                      | Yes (order matters)   |

**Recommendation:** Prefer object destructuring when possible. Use array destructuring for:

- Tuple returns where position has clear meaning
- Head/tail recursion patterns
- Coordinate/vector operations

### Target Mappings

| Target     | Syntax                                              |
| ---------- | --------------------------------------------------- |
| TypeScript | `const [a, b, ...rest] = arr`                       |
| Python     | `a, b, *rest = arr`                                 |
| Zig        | Expanded to indexed access: `_tmp[0]`, `_tmp[1..]`  |
| C++        | Expanded to indexed access with iterators           |
| Rust       | `let [a, b, rest @ ..] = arr` (not yet implemented) |

### Design Decisions

1. **Nested destructuring**: Not supported. Keep patterns flat.
2. **Default values**: Not supported. No `ex arr fixum [a, b = 0]` syntax.
3. **Length mismatch**: Runtime behavior depends on target (undefined/nil for missing positions).

---

## Standalone Functions (norma)

Functions that operate on multiple collections or don't belong to a single type.

| Faber                             | lodash   | Description          |
| --------------------------------- | -------- | -------------------- |
| `iunge(T[], U[]) -> (T,U)[]`      | `zip`    | Pair up elements     |
| `iungeOmnes(...lista) -> (...)[]` | `zipAll` | Zip multiple lists   |
| `series(n) -> numerus[]`          | `range`  | 0 to n-1             |
| `series(a, b) -> numerus[]`       | `range`  | a to b-1             |
| `repete(T, n) -> T[]`             | `times`  | Repeat value n times |

---

## Function Utilities (norma)

Higher-order function helpers.

| Faber                          | lodash     | Description         |
| ------------------------------ | ---------- | ------------------- |
| `memora((A) -> B) -> (A) -> B` | `memoize`  | Cache results       |
| `semel((A) -> B) -> (A) -> B`  | `once`     | Call only once      |
| `mora(numerus, fn) -> fn`      | `debounce` | Delay execution     |
| `tempera(numerus, fn) -> fn`   | `throttle` | Rate limit          |
| `partialis(fn, ...args) -> fn` | `partial`  | Partial application |

---

## Common Patterns

### Method Naming Convention

| Pattern       | Meaning               | Examples                                     |
| ------------- | --------------------- | -------------------------------------------- |
| `-are` verbs  | Actions               | `adde`, `dele`, `purgare`, `filtra`, `mappa` |
| Adjectives    | Properties/predicates | `vacua`, `primus`, `ultimus`, `unica`        |
| Nouns         | Derived values        | `longitudo`, `claves`, `valores`, `summa`    |
| `-Per` suffix | "by" variant          | `ordinaPer`, `maximusPer`                    |

### Verb Conjugation for Mutability and Async

Latin verb forms distinguish between mutable/immutable and sync/async operations:

|           | Mutates             | Returns New                    |
| --------- | ------------------- | ------------------------------ |
| **Sync**  | `adde` (imperative) | `addita` (perfect participle)  |
| **Async** | `addet` (future)    | `additura` (future participle) |

**Examples:**

```
// Sync operations
lista.adde(x)                  // mutate in place
fixum nova = lista.addita(x)   // new list with x

lista.ordina(per aetas)        // sort in place
fixum nova = lista.ordinata(per aetas)  // new sorted list

lista.filtra({ .activus })     // filter in place
fixum nova = lista.filtrata({ .activus })  // new filtered list

// Async operations
cede lista.addet(x)        // mutate eventually
fixum nova = cede lista.additura(x)  // new list eventually
```

**Participle agreement:** Feminine endings (`-a`) agree with `lista`, `tabula`, `copia`.

**Pattern for any verb:**

| Root     | Imperative | Perfect Participle | Future  | Future Participle |
| -------- | ---------- | ------------------ | ------- | ----------------- |
| addere   | adde       | addita             | addet   | additura          |
| ordinare | ordina     | ordinata           | ordinat | ordinatura        |
| filtrare | filtra     | filtrata           | filtrat | filtratura        |
| removere | remove     | remota             | removet | remotura          |
| purgare  | purga      | purgata            | purgat  | purgatura         |

The grammar carries semantic weight that other languages encode with symbols (`!`, `async`, `Immutable.`).

### Iteration Integration

All collections work with `ex...pro`:

```
ex lista pro elementum { ... }
ex tabula.claves() pro clavis { ... }
ex copia pro valor { ... }
```

### Chaining

Methods that return collections can be chained:

```
fixum result = users
    .filtra({ redde .activus })
    .ordina(per nomen)
    .mappa({ redde .email })
    .prima(10)
```

---

## Closure Syntax

Three levels of expressiveness, from tersest to most powerful:

### Level 1: `per property` — Property Shorthand

For simple property access, use `per` (by) followed by property name(s):

```
users.ordina(per aetas)
users.congrega(per civitas)
users.ordina(per aetas et nomen)
```

Reads as Latin: "order by age and name."

**Sort direction** uses Latin adjectives (default is `ascendens`):

```
users.ordina(per aetas descendens)
users.ordina(per aetas descendens et nomen ascendens)
users.ordina(per aetas et nomen)  // both ascending
```

### Level 2: `{ redde .property }` — Implicit Subject Block

For expressions involving the current item, use `.` as implicit subject:

```
users.filtra({ redde .aetas > 18 })
users.mappa({ redde .nomen + " " + .cognomen })
users.ordina({ redde .aetas * -1 })  // manual descending
```

The `.property` means "property of the current item."

For single expressions, `redde` may be implicit:

```
users.filtra({ .aetas > 18 })
users.mappa({ .nomen + " " + .cognomen })
```

### Level 3: Explicit Variable

For complex logic requiring named variable, two syntaxes are supported:

**`pro var { }` block form (preferred — aligns with iteration syntax):**

```
users.filtra(pro user {
    si user.aetas < 18 {
        redde falsum
    }
    redde user.activus et user.verificatus
})
```

**`pro var redde expr` expression form:**

```
users.filtra(pro user redde user.activus et user.verificatus)
lista.mappa(pro item redde item.nomen)
```

**`var => { }` (supported — familiar to JS/TS developers):**

```
users.filtra(user => {
    si user.aetas < 18 {
        redde falsum
    }
    redde user.activus et user.verificatus
})
```

The `pro` keyword aligns with iteration: `ex items pro x { }` iterates, `pro x { }` is a closure.

### Summary

| Form               | Use Case                  | Example                        |
| ------------------ | ------------------------- | ------------------------------ |
| `per property`     | Property access           | `ordina(per aetas)`            |
| `per a et b`       | Multi-property            | `ordina(per aetas et nomen)`   |
| `per a descendens` | Sort direction            | `ordina(per aetas descendens)` |
| `{ .property }`    | Expressions               | `filtra({ .aetas > 18 })`      |
| `{ redde ... }`    | Multi-statement           | `mappa({ redde .x + .y })`     |
| `pro v redde expr` | Expression closure        | `mappa(pro x redde x * 2)`     |
| `pro v { }`        | Block closure (preferred) | `filtra(pro u { ... })`        |
| `v => { }`         | Block closure (JS-style)  | `filtra(u => { ... })`         |

---

## Implementation Notes

### TypeScript Target

Direct mapping to native types:

- `T[]` → `Array<T>`
- `tabula<K,V>` → `Map<K,V>`
- `copia<T>` → `Set<T>`

Lodash methods compile to equivalent lodash calls or inline implementations.

### Zig Target

Use standard library types:

- `T[]` → `std.ArrayList(T)`
- `tabula<K,V>` → `std.HashMap(K,V,...)`
- `copia<T>` → `std.HashSet(T,...)`

Zig requires allocator handling — need design decision on how to expose this.

Many lodash-style methods need custom implementation for Zig.

---

## Collection DSL

See `ab.md` for the collection filtering DSL using `ex`/`ab` prepositions.

Until DSL is implemented, use method chaining:

```fab
fixum result = users.filtrata(pro u: u.activus).ordinata(pro u: u.nomen).prima(10)
```
