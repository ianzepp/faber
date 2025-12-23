# Collections Design

## Overview

Three core collection types wrap native implementations:

| Faber | JavaScript | Zig | Description |
|-------|------------|-----|-------------|
| `lista<T>` | `Array<T>` | `ArrayList(T)` | Ordered, indexed, duplicates allowed |
| `tabula<K,V>` | `Map<K,V>` | `HashMap(K,V)` | Key-value pairs, unique keys |
| `copia<T>` | `Set<T>` | `HashSet(T)` | Unique values, unordered |

All collection type names are feminine (Latin convention for containers).

---

## lista<T>

**Etymology:** "list, border, edge" — a bounded sequence.

### Core Methods

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `adde(T)` | `push` | Add to end |
| `remove() -> T?` | `pop` | Remove from end |
| `primus() -> T?` | `[0]` | First element |
| `ultimus() -> T?` | `at(-1)` | Last element |
| `longitudo() -> numerus` | `length` | Count |
| `vacua() -> bivalens` | `length === 0` | Is empty |
| `purgare()` | `length = 0` | Clear all |

### Search & Test

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `contine(T) -> bivalens` | `includes` | Contains element |
| `indiceDe(T) -> numerus?` | `indexOf` | Find index |
| `inveni((T) -> bivalens) -> T?` | `find` | Find first match |
| `omnes((T) -> bivalens) -> bivalens` | `every` | All match predicate |
| `aliquis((T) -> bivalens) -> bivalens` | `some` | Any matches predicate |

### Transformation (Functional)

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `mappa((T) -> U) -> lista<U>` | `map` | Transform each |
| `filtra((T) -> bivalens) -> lista<T>` | `filter` | Keep matching |
| `reducere(U, (U,T) -> U) -> U` | `reduce` | Fold to value |
| `coniunge(textus) -> textus` | `join` | Join to string |

### Lodash-Inspired Methods

| Faber | lodash | Description |
|-------|--------|-------------|
| `ordina((T) -> U) -> lista<T>` | `sortBy` | Sort by key function |
| `congrega((T) -> K) -> tabula<K, lista<T>>` | `groupBy` | Group by key function |
| `unica() -> lista<T>` | `uniq` | Remove duplicates |
| `plana() -> lista<T>` | `flatten` | Flatten one level |
| `planaOmnia() -> lista<T>` | `flattenDeep` | Flatten all levels |
| `fragmenta(numerus) -> lista<lista<T>>` | `chunk` | Split into chunks of size n |
| `densa() -> lista<T>` | `compact` | Remove falsy values |
| `partire((T) -> bivalens) -> (lista<T>, lista<T>)` | `partition` | Split by predicate |
| `misce() -> lista<T>` | `shuffle` | Randomize order |
| `specimen() -> T?` | `sample` | Random element |
| `specimina(numerus) -> lista<T>` | `sampleSize` | Random n elements |
| `prima(numerus) -> lista<T>` | `take` | First n elements |
| `ultima(numerus) -> lista<T>` | `takeRight` | Last n elements |
| `omitte(numerus) -> lista<T>` | `drop` | Skip first n |
| `inversa() -> lista<T>` | `reverse` | Reverse order |

### Aggregation

| Faber | lodash | Description |
|-------|--------|-------------|
| `summa() -> numerus` | `sum` | Sum of numbers |
| `medium() -> numerus` | `mean` | Average |
| `minimus() -> T?` | `min` | Minimum value |
| `maximus() -> T?` | `max` | Maximum value |
| `minimusPer((T) -> U) -> T?` | `minBy` | Min by key function |
| `maximusPer((T) -> U) -> T?` | `maxBy` | Max by key function |
| `numera((T) -> bivalens) -> numerus` | `countBy` | Count matching |

### Open Questions

1. **Mutability**: Should `adde`/`remove` mutate in place or return new list?
   - JS convention: mutate in place
   - Functional convention: return new
   - Proposal: mutate in place, provide `cum` variants for immutable ops

2. **Negative indices**: Support `lista[-1]` for last element?
   - JS: No (use `at(-1)`)
   - Python: Yes
   - Proposal: Support via subscript operator

3. **Slicing**: Syntax for sublists?
   - Python: `lista[1:3]`
   - Proposal: `lista.sectio(1, 3)` method

---

## tabula<K, V>

**Etymology:** "board, tablet, table" — a writing surface with entries.

### Core Methods

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `pone(K, V)` | `set` | Set key-value |
| `accipe(K) -> V?` | `get` | Get by key |
| `habet(K) -> bivalens` | `has` | Key exists |
| `dele(K) -> bivalens` | `delete` | Remove key |
| `longitudo() -> numerus` | `size` | Count |
| `vacua() -> bivalens` | `size === 0` | Is empty |
| `purgare()` | `clear` | Clear all |

### Iteration

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `claves() -> cursor<K>` | `keys()` | Iterate keys |
| `valores() -> cursor<V>` | `values()` | Iterate values |
| `paria() -> cursor<(K,V)>` | `entries()` | Iterate pairs |

### Lodash-Inspired Methods

| Faber | lodash | Description |
|-------|--------|-------------|
| `accipeAut(K, V) -> V` | `get` with default | Get or return default |
| `selige(...K) -> tabula<K,V>` | `pick` | Keep only specified keys |
| `omitte(...K) -> tabula<K,V>` | `omit` | Remove specified keys |
| `confla(tabula<K,V>) -> tabula<K,V>` | `merge` | Merge maps |
| `inversa() -> tabula<V,K>` | `invert` | Swap keys and values |
| `mappaValores((V) -> U) -> tabula<K,U>` | `mapValues` | Transform values |
| `mappaClaves((K) -> J) -> tabula<J,V>` | `mapKeys` | Transform keys |

### Open Questions

1. **Object conversion**: `exObjecto(obj)` static constructor?
   - Useful for JS interop
   - Less relevant for Zig target

---

## copia<T>

**Etymology:** "abundance, supply" — a collection of resources.

### Core Methods

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `adde(T)` | `add` | Add element |
| `habet(T) -> bivalens` | `has` | Contains |
| `dele(T) -> bivalens` | `delete` | Remove |
| `longitudo() -> numerus` | `size` | Count |
| `vacua() -> bivalens` | `size === 0` | Is empty |
| `purgare()` | `clear` | Clear all |

### Set Operations

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `unio(copia<T>) -> copia<T>` | `union` | A ∪ B |
| `intersectio(copia<T>) -> copia<T>` | `intersection` | A ∩ B |
| `differentia(copia<T>) -> copia<T>` | `difference` | A - B |
| `symmetrica(copia<T>) -> copia<T>` | — | (A - B) ∪ (B - A) |

### Predicates

| Faber | lodash | Description |
|-------|--------|-------------|
| `subcopia(copia<T>) -> bivalens` | `isSubset` | Is subset of |
| `supercopia(copia<T>) -> bivalens` | `isSuperset` | Is superset of |

### Open Questions

1. **Conversion to lista**: `inLista() -> lista<T>`?

---

## Standalone Functions (norma)

Functions that operate on multiple collections or don't belong to a single type.

| Faber | lodash | Description |
|-------|--------|-------------|
| `iunge(lista<T>, lista<U>) -> lista<(T,U)>` | `zip` | Pair up elements |
| `iungeOmnes(...lista) -> lista<(...)>` | `zipAll` | Zip multiple lists |
| `intervallum(n) -> lista<numerus>` | `range` | 0 to n-1 |
| `intervallum(a, b) -> lista<numerus>` | `range` | a to b-1 |
| `repete(T, n) -> lista<T>` | `times` | Repeat value n times |

---

## Function Utilities (norma)

Higher-order function helpers.

| Faber | lodash | Description |
|-------|--------|-------------|
| `memora((A) -> B) -> (A) -> B` | `memoize` | Cache results |
| `semel((A) -> B) -> (A) -> B` | `once` | Call only once |
| `mora(numerus, fn) -> fn` | `debounce` | Delay execution |
| `tempera(numerus, fn) -> fn` | `throttle` | Rate limit |
| `partialis(fn, ...args) -> fn` | `partial` | Partial application |

---

## Common Patterns

### Method Naming Convention

| Pattern | Meaning | Examples |
|---------|---------|----------|
| `-are` verbs | Actions | `adde`, `dele`, `purgare`, `filtra`, `mappa` |
| Adjectives | Properties/predicates | `vacua`, `primus`, `ultimus`, `unica` |
| Nouns | Derived values | `longitudo`, `claves`, `valores`, `summa` |
| `-Per` suffix | "by" variant | `ordinaPer`, `maximusPer` |

### Verb Conjugation for Mutability and Async

Latin verb forms distinguish between mutable/immutable and sync/async operations:

| | Mutates | Returns New |
|---|---------|-------------|
| **Sync** | `adde` (imperative) | `addita` (perfect participle) |
| **Async** | `addet` (future) | `additura` (future participle) |

**Examples:**

```
// Sync operations
lista.adde(x)                  // mutate in place
fixum nova = lista.addita(x)   // new list with x

lista.ordina(cum aetas)        // sort in place
fixum nova = lista.ordinata(cum aetas)  // new sorted list

lista.filtra({ .activus })     // filter in place
fixum nova = lista.filtrata({ .activus })  // new filtered list

// Async operations
cede lista.addet(x)        // mutate eventually
fixum nova = cede lista.additura(x)  // new list eventually
```

**Participle agreement:** Feminine endings (`-a`) agree with `lista`, `tabula`, `copia`.

**Pattern for any verb:**

| Root | Imperative | Perfect Participle | Future | Future Participle |
|------|------------|-------------------|--------|-------------------|
| addere | adde | addita | addet | additura |
| ordinare | ordina | ordinata | ordinat | ordinatura |
| filtrare | filtra | filtrata | filtrat | filtratura |
| removere | remove | remota | removet | remotura |
| purgare | purga | purgata | purgat | purgatura |

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
    .ordina(cum nomen)
    .mappa({ redde .email })
    .prima(10)
```

---

## Closure Syntax

Three levels of expressiveness, from tersest to most powerful:

### Level 1: `cum property` — Property Shorthand

For simple property access, use `cum` (with) followed by property name(s):

```
users.ordina(cum aetas)
users.congrega(cum civitas)
users.ordina(cum aetas et nomen)
```

Reads as Latin: "order with age and name."

**Sort direction** uses Latin adjectives (default is `ascendens`):

```
users.ordina(cum aetas descendens)
users.ordina(cum aetas descendens et nomen ascendens)
users.ordina(cum aetas et nomen)  // both ascending
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

### Level 3: `var => { }` — Explicit Variable

For complex logic requiring named variable:

```
users.filtra(user => {
    si user.aetas < 18 {
        redde falsum
    }
    redde user.activus et user.verificatus
})
```

### Summary

| Form | Use Case | Example |
|------|----------|---------|
| `cum property` | Property access | `ordina(cum aetas)` |
| `cum a et b` | Multi-property | `ordina(cum aetas et nomen)` |
| `cum a descendens` | Sort direction | `ordina(cum aetas descendens)` |
| `{ .property }` | Expressions | `filtra({ .aetas > 18 })` |
| `{ redde ... }` | Multi-statement | `mappa({ redde .x + .y })` |
| `v => { }` | Complex logic | `filtra(u => { ... })` |

---

## Implementation Notes

### TypeScript Target

Direct mapping to native types:
- `lista<T>` → `Array<T>`
- `tabula<K,V>` → `Map<K,V>`
- `copia<T>` → `Set<T>`

Lodash methods compile to equivalent lodash calls or inline implementations.

### Zig Target

Use standard library types:
- `lista<T>` → `std.ArrayList(T)`
- `tabula<K,V>` → `std.HashMap(K,V,...)`
- `copia<T>` → `std.HashSet(T,...)`

Zig requires allocator handling — need design decision on how to expose this.

Many lodash-style methods need custom implementation for Zig.

---

## Collection DSL: Prefix Operations

An alternative syntax for collection operations using Latin prefix verbs with comma chaining. Reads like natural Latin sentences.

### Basic Syntax

```
<verb> <collection> [<preposition> <arg>], <verb> [<preposition> <arg>], ...
```

The comma acts as an implicit pipe — each operation flows into the next.

### Prefix Verbs

| Latin | Meaning | Method Equivalent |
|-------|---------|-------------------|
| `summa` | sum | `.summa()` |
| `maximum` | max | `.maximus()` |
| `minimum` | min | `.minimus()` |
| `medium` | average | `.medium()` |
| `quota` | count | `.longitudo()` |
| `filtra` | filter | `.filtrata()` |
| `ordina` | sort/order | `.ordinata()` |
| `collige` | collect/pluck | `.mappa()` for property extraction |
| `mappa` | map to key-value | creates `tabula` keyed by property |
| `grupa` | group by | `.congrega()` |
| `prima` | first n | `.prima()` |
| `ultima` | last n | `.ultima()` |
| `inversa` | reverse | `.inversa()` |
| `unica` | unique | `.unica()` |

### Prepositions

| Latin | Meaning | Use |
|-------|---------|-----|
| `ubi` | where | filter condition |
| `cum` | with/by | property selector |
| `per` | by | sort/group field |
| `ex` | from | source collection |

### Examples

**Aggregates:**
```
fixum total = summa numeri
fixum highest = maximum pretia
fixum count = quota users
fixum avg = medium scores
```

**Filtering:**
```
fixum active = filtra users ubi activus
fixum expensive = filtra items ubi pretium > 100
fixum adults = filtra users ubi aetas >= 18
```

**Transformation:**
```
fixum names = collige users cum nomen
fixum sorted = ordina items per pretium
fixum byRole = grupa users per role
fixum indexed = mappa users cum id
```

**Chained with comma:**
```
// Filter active users, extract names, sort
fixum result = filtra users ubi activus, collige cum nomen, ordina

// Filter expensive items, sort by price, take top 5
fixum top5 = filtra items ubi pretium > 100, ordina per pretium, prima 5

// Sum prices of active products
fixum total = filtra products ubi activus, collige cum pretium, summa

// Group users by role, then by department
fixum nested = grupa users per role, mappa cum departmentum
```

### Comparison with Method Chaining

Both syntaxes are valid and equivalent:

```
// Method chaining (OOP style)
fixum result = users
    .filtrata({ .activus })
    .ordinata(cum nomen)
    .prima(10)

// Prefix DSL (Latin sentence style)
fixum result = filtra users ubi activus, ordina cum nomen, prima 10
```

The prefix DSL reads more like natural Latin:
- "filtra users ubi activus" = "filter users where active"
- "ordina cum nomen" = "order by name"
- "prima 10" = "first 10"

### Grammar Notes

When using genitive case for collection names, the syntax becomes fully grammatical Latin:

```
fixum summa numerorum        // sum of the numbers
fixum maximum pretiorum      // maximum of the prices
fixum prima 5 ex itemis      // first 5 from the items
```

However, for practical code with non-Latin variable names, the nominative form works:

```
fixum total = summa orderTotals
fixum result = filtra userList ubi active
```

### Target Compilation

**TypeScript:**
```typescript
// filtra users ubi activus, collige cum nomen, ordina
users.filter(u => u.activus).map(u => u.nomen).sort()
```

**Zig:**
```zig
// Generates iterator chain or explicit loops
// with arena allocator for intermediate results
```

**Rust:**
```rust
// filtra users ubi activus, collige cum nomen, ordina
users.iter()
    .filter(|u| u.activus)
    .map(|u| &u.nomen)
    .sorted()
    .collect()
```
