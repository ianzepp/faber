# Faber Romanus Type System

Type-first syntax: `fixum Textus nomen = "Marcus"`

## Primitive Types

### Numerus (Integer)

Signed integers. Size in bits, defaults to 64.

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Numerus` | `number` | `i64` | `i64` |
| `Numerus<8>` | `number` | `i8` | `i8` |
| `Numerus<16>` | `number` | `i16` | `i16` |
| `Numerus<32>` | `number` | `i32` | `i32` |
| `Numerus<64>` | `number` | `i64` | `i64` |

**Unsigned (Naturalis)** - natural numbers, no negatives:

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Numerus<Naturalis>` | `number` | `u64` | `u64` |
| `Numerus<8, Naturalis>` | `number` | `u8` | `u8` |
| `Numerus<16, Naturalis>` | `number` | `u16` | `u16` |
| `Numerus<32, Naturalis>` | `number` | `u32` | `u32` |
| `Numerus<64, Naturalis>` | `number` | `u64` | `u64` |

### Fractus (Floating Point)

Binary floating-point numbers. Size in bits, defaults to 64.

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Fractus` | `number` | `f64` | `f64` |
| `Fractus<32>` | `number` | `f32` | `f32` |
| `Fractus<64>` | `number` | `f64` | `f64` |

### Decimus (Decimal)

Exact decimal arithmetic. For financial calculations and precision-critical code.

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Decimus` | `Decimal`* | † | `Decimal`* |
| `Decimus<32>` | `Decimal`* | † | `Decimal`* |
| `Decimus<64>` | `Decimal`* | † | `Decimal`* |
| `Decimus<128>` | `Decimal`* | † | `Decimal`* |

\* Requires library (decimal.js, rust_decimal)
† Not natively supported in Zig

### Textus (String)

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Textus` | `string` | `[]const u8` | `String` |
| `Textus<Proprius>` | `string` | `[]u8` | `String` |
| `Textus<Alienus>` | `string` | `[]const u8` | `&str` |

- **Proprius** (own) - owned, heap-allocated
- **Alienus** (other's) - borrowed, reference

### Simple Types

| Type | Meaning | TypeScript | Zig | Rust |
|------|---------|------------|-----|------|
| `Bivalens` | boolean | `boolean` | `bool` | `bool` |
| `Nihil` | null | `null` | `null` | `None` |
| `Incertum` | undefined | `undefined` | — | — |
| `Vacuum` | void | `void` | `void` | `()` |
| `Signum` | symbol | `symbol` | — | — |

### Boolean Values

| Latin | JavaScript |
|-------|------------|
| `verum` | `true` |
| `falsum` | `false` |

### Escape Hatches

| Type | Meaning | TypeScript |
|------|---------|------------|
| `Quodlibet` | "whatever pleases" - opt out of type checking | `any` |
| `Ignotum` | "unknown thing" - must narrow before use | `unknown` |

## Collection Types

### Lista (Array/List)

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Lista<T>` | `T[]` | `[]T` | `Vec<T>` |
| `Lista<T, N>` | `T[]` | `[N]T` | `[T; N]` |

Second parameter `N` specifies fixed length (compiles to array, not slice/vec).

### Tabula (Map/Dictionary)

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Tabula<K, V>` | `Map<K, V>` | `std.HashMap(K, V)` | `HashMap<K, V>` |

### Copia (Set)

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Copia<T>` | `Set<T>` | `std.HashSet(T)` | `HashSet<T>` |

### Tuples

```faber
fixum [Textus, Numerus] pair = ["Marcus", 30]
```

## Iteration & Streaming

| Type | Meaning | TypeScript | Rust |
|------|---------|------------|------|
| `Cursor<T>` | iterator (pull-based) | `Iterator<T>` | `Iterator<Item=T>` |
| `Fluxus<T>` | stream (push-based) | `Observable<T>` | `Stream<Item=T>` |
| `FuturaCursor<T>` | async iterator | `AsyncIterator<T>` | `AsyncIterator` |
| `FuturusFluxus<T>` | async stream | `AsyncIterable<T>` | `futures::Stream` |

Semantic distinction:
- `Cursor` — you pull values ("runner" through data)
- `Fluxus` — values push to you ("flow" past you)

## Async Types

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Promissum<T>` | `Promise<T>` | — | `Future<Output=T>` |

## Structural Types

| Type | Meaning | TypeScript |
|------|---------|------------|
| `Res` | generic object | `object` |
| `Functio` | function type | `Function` |
| `Tempus` | date/time | `Date` |
| `Erratum` | error | `Error` |

## User-Defined Types

### genus (Struct/Class)

Latin: "kind" — defines a data structure with fields and methods.

```faber
genus persona {
  textus nomen
  numerus aetas = 18

  crea(init) {
    ego.nomen = init.nomen
    ego.aetas = init.aetas aut 18
  }

  functio saluta() -> textus {
    redde "Salve, " + ego.nomen
  }
}
```

| Target | Compiles To |
|--------|-------------|
| TypeScript | `class` |
| Zig | `struct` with methods |
| Rust | `struct` + `impl` block |

**Fields:**
- Type-first syntax: `textus nomen`
- Default values allowed: `numerus aetas = 18`
- Private by default, use `publicus` for visibility

**Methods:**
- Defined with `functio` inside the genus body
- Use `ego` (Latin "I") for self-reference
- Private by default, use `publicus` for visibility

**Constructor:**
- Optional `crea(init)` block receives initialization object
- If no `crea` defined, direct field assignment

**Visibility:**

```faber
genus persona {
  publicus textus nomen      // accessible from outside
  numerus secretum           // private (default)

  publicus functio saluta()  // public method
  functio helper()           // private method
}
```

**Module export:**

```faber
exporta genus persona { ... }  // available to other files
```

**Instantiation:**

```faber
// With initialization values
fixum marcus = novum persona cum { nomen: "Marcus", aetas: 30 }

// Partial initialization (uses defaults)
fixum julia = novum persona cum { nomen: "Julia" }

// All defaults (calls crea({}) if defined)
fixum hospes = novum persona
```

### pactum (Interface/Trait)

Latin: "agreement" — defines a contract of required methods.

```faber
pactum salutabilis {
  functio saluta() -> textus
}

pactum comparabilis {
  functio compara(alius) -> numerus
}
```

| Target | Compiles To |
|--------|-------------|
| TypeScript | `interface` |
| Zig | (compile-time duck typing) |
| Rust | `trait` |

**Characteristics:**
- Method signatures only, no implementations
- No fields (methods only)
- Structural typing — a genus satisfies a pactum if it has matching methods

**Usage as type:**

```faber
functio greet(s: salutabilis) -> textus {
  redde s.saluta()
}

// Any genus with saluta() method works
fixum marcus = novum persona cum { nomen: "Marcus" }
greet(marcus)
```

### ordo (Enum)

Latin: "order, rank" — defines named constants.

```faber
// Auto-numbered (starts at 0)
ordo direction {
  north
  east
  south
  west
}

// Explicit values
ordo httpStatus {
  ok = 200
  notFound = 404
  serverError = 500
}

// Mixed (continues from last explicit value)
ordo priority {
  low = 1
  medium
  high
  critical = 10
}
```

| Target | Compiles To |
|--------|-------------|
| TypeScript | `enum` |
| Zig | `enum` |
| Rust | `enum` (C-style) |

**Usage:**

```faber
fixum d = direction.north
fixum status = httpStatus.ok

si status == httpStatus.notFound {
  scribe "Not found"
}
```

Note: The language is case-insensitive, so `direction.NORTH` and `direction.north` are equivalent.

### User-Defined Types Summary

| Keyword | Purpose | TypeScript | Zig | Rust |
|---------|---------|------------|-----|------|
| `genus` | struct/class | `class` | `struct` | `struct` + `impl` |
| `pactum` | interface/trait | `interface` | — | `trait` |
| `ordo` | enum | `enum` | `enum` | `enum` |
| `typus` | type alias | `type` | — | `type` |

## Optional & Error Types

### Forsitan (Optional)

Latin: "perhaps" - value may or may not exist.

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Forsitan<T>` | `T \| null` | `?T` | `Option<T>` |

Shorthand: `T?` is sugar for `Forsitan<T>`

```faber
fixum Textus? cognomen = nihil
```

### Fors (Result)

Latin: "chance/fortune" - operation may succeed or fail.

| Type | TypeScript | Zig | Rust |
|------|------------|-----|------|
| `Fors<T>` | `T` (throws) | `!T` | `Result<T, Error>` |
| `Fors<T, E>` | `T` (throws) | `E!T` | `Result<T, E>` |

## Union & Intersection Types

```faber
// Union: one or the other
Textus | Numerus

// Nullable (sugar for T | Nihil)
Textus?

// Intersection: both combined
Serializable & Comparable
```

## Type Aliases

```faber
typus ID = Textus
typus Handler = Functio<Res, Vacuum>
```

## Utility Types

> **Note:** These are TypeScript-oriented. May not compile to all targets.

### Object Utilities

| Type | TypeScript | Meaning | Target Support |
|------|------------|---------|----------------|
| `Pars<T>` | `Partial<T>` | all fields optional | TS only |
| `Totum<T>` | `Required<T>` | all fields required | TS only |
| `Lectum<T>` | `Readonly<T>` | all fields immutable | TS, Zig, Rust |
| `Registrum<K, V>` | `Record<K, V>` | key-value object type | TS only |

### Selection Utilities

| Type | TypeScript | Meaning | Target Support |
|------|------------|---------|----------------|
| `Selectum<T, K>` | `Pick<T, K>` | subset of fields | TS only |
| `Omissum<T, K>` | `Omit<T, K>` | exclude fields | TS only |
| `Extractum<T, U>` | `Extract<T, U>` | matching union members | TS only |
| `Exclusum<T, U>` | `Exclude<T, U>` | non-matching members | TS only |

### Nullability Utilities

| Type | TypeScript | Meaning | Target Support |
|------|------------|---------|----------------|
| `NonNihil<T>` | `NonNullable<T>` | remove null/undefined | TS only |

### Function Introspection

| Type | TypeScript | Meaning | Target Support |
|------|------------|---------|----------------|
| `Reditus<F>` | `ReturnType<F>` | function's return type | TS only |
| `Parametra<F>` | `Parameters<F>` | function's arg types | TS only |

## Pointer & Reference Types

For systems programming targets (Zig, Rust).

| Type | Meaning | Zig | Rust |
|------|---------|-----|------|
| `Indicium<T>` | pointer | `*T` | `*const T` |
| `Indicium<T, Mutabilis>` | mutable pointer | `*T` | `*mut T` |
| `Refera<T>` | reference | — | `&T` |
| `Refera<T, Mutabilis>` | mutable reference | — | `&mut T` |

> **Note:** Pointer types are ignored when compiling to TypeScript.

## Type Modifiers Summary

| Modifier | Meaning | Context |
|----------|---------|---------|
| `Naturalis` | unsigned | `Numerus<32, Naturalis>` |
| `Proprius` | owned | `Textus<Proprius>` |
| `Alienus` | borrowed | `Textus<Alienus>` |
| `Mutabilis` | mutable | `Indicium<T, Mutabilis>` |

---

# Syntax Reference

## Variables

```faber
esto nomen = "Marcus"        // mutable (let) — "let it be"
fixum PI = 3.14159           // immutable (const) — "fixed"

// Boolean literals
fixum veritatis = verum      // true
fixum falsitatis = falsum    // false

// Null literal
fixum nihilum = nihil        // null

// Reassignment works only with esto
esto valor = 42
valor = 100
```

## Functions

```faber
// Function with return type (-> at end)
functio salve(nomen) -> Textus {
  redde "Salve, " + nomen + "!"
}

// Function with multiple parameters
functio adde(a, b) -> Numerus {
  redde a + b
}

// Function without return (implicit Vacuum)
functio salutaOmnes() {
  scribe "Salve, Mundus"
}

// Async function
futura functio fetchData(url) -> Textus {
  fixum data = exspecta fetch(url)
  redde data
}
```

## Output

`scribe` is a statement keyword (not a function call):

```faber
scribe "Hello"
scribe nomen
scribe "Value:", x
```

## Control Flow

### Conditionals

```faber
si x > 5 {
  scribe "x est magnus"
}

// One-liner with ergo (then)
si x > 5 ergo scribe "x magnus est"

// If-else
si x > 20 {
  scribe "x est maximus"
}
aliter {
  scribe "x non est maximus"
}

// One-liner if-else
si x > 20 ergo scribe "maximus" aliter scribe "non maximus"

// If-else chain
si nota >= 90 {
  scribe "A"
}
aliter si nota >= 80 {
  scribe "B"
}
aliter {
  scribe "F"
}
```

### Switch Statement

`elige` uses `si`/`ergo` for cases (not `quando`/`=>`):

```faber
elige status {
  si "pending" ergo scribe "waiting"
  si "active" ergo scribe "running"
  si "done" {
    scribe "finished"
    cleanup()
  }
  aliter scribe "unknown"
}
```

### Loops

```faber
// While loop
dum i < 3 {
  scribe i
  i = i + 1
}

// While one-liner
dum j > 0 ergo j = j - 1

// For-each (ex ... pro) - source first
ex numeri pro n {
  scribe n
}

// For-each one-liner
ex items pro item ergo scribe item

// Range expression
ex 0..10 pro i {
  scribe i
}

// Range with step
ex 0..10 per 2 pro i {
  scribe i
}

// For-in (keys)
in objeto pro key {
  scribe key
}
```

### Loop Control

| Latin | JavaScript | Meaning |
|-------|------------|---------|
| `rumpe` | `break` | break out of loop |
| `perge` | `continue` | continue to next iteration |

### Logical Operators

```faber
si a et b {
  scribe "both true"
}

si a aut b {
  scribe "one true"
}

// Empty/non-empty checks
si nonnulla items {
  scribe "has items"
}

si nulla data {
  scribe "no data"
}
```

## Guard Clauses

```faber
functio validate(x) -> Numerus {
  custodi {
    si x < 0 { redde -1 }
    si x > 100 { redde -1 }
  }
  redde x
}
```

## Assert Statement

```faber
adfirma x > 0, "x must be positive"
adfirma valid
```

## Error Handling

Any block can have a `cape` clause:

```faber
si riskyCall() {
  process()
} cape erratum {
  handleError(erratum)
}

ex items pro item {
  process(item)
} cape erratum {
  logFailure(erratum)
}
```

Explicit try with finally:

```faber
tempta {
  riskyOperation()
} cape erratum {
  recover()
} demum {
  cleanup()
}
```

Throw:

```faber
iace "Something went wrong"
iace novum Error("message")
```

## Objects

```faber
// Object literal
fixum persona = {
  nomen: "Marcus",
  aetas: 30
}

scribe persona.nomen

// Destructuring
fixum { nomen, aetas } = persona

// Destructuring with rename
fixum { nomen: userName } = persona

// With block - set properties in context
esto config = { host: "", port: 0 }
cum config {
  host = "localhost"
  port = 8080
}
```

## Operators

| Latin | JavaScript | Meaning |
|-------|------------|---------|
| `et` | `&&` | and |
| `aut` | `\|\|` | or |
| `non` | `!` | not |
| `nulla` | — | is null/empty |
| `nonnulla` | — | is non-null/non-empty |

## Keywords Reference

| Latin | JavaScript | Category |
|-------|------------|----------|
| `si` | `if` | control |
| `aliter` | `else` | control |
| `ergo` | (then) | control |
| `elige` | `switch` | control |
| `dum` | `while` | control |
| `in` | `in` | preposition |
| `ex` | `of` | preposition |
| `cum` | `with` | preposition |
| `per` | `by/through` | preposition |
| `rumpe` | `break` | control |
| `perge` | `continue` | control |
| `custodi` | (guard) | control |
| `adfirma` | (assert) | control |
| `tempta` | `try` | error |
| `cape` | `catch` | error |
| `demum` | `finally` | error |
| `iace` | `throw` | error |
| `scribe` | `console.log` | I/O |
| `exspecta` | `await` | async |
| `novum` | `new` | expression |
| `ego` | `this` | expression |
| `esto` | `let` | declaration |
| `fixum` | `const` | declaration |
| `functio` | `function` | declaration |
| `genus` | `class`/`struct` | declaration |
| `pactum` | `interface`/`trait` | declaration |
| `ordo` | `enum` | declaration |
| `typus` | `type` | declaration |
| `importa` | `import` | declaration |
| `exporta` | `export` | declaration |
| `futura` | `async` | modifier |
| `publicus` | `public` | modifier |
| `redde` | `return` | control |
| `verum` | `true` | value |
| `falsum` | `false` | value |
| `nihil` | `null` | value |
| `et` | `&&` | operator |
| `aut` | `\|\|` | operator |
| `non` | `!` | operator |
| `nulla` | — | operator |
| `nonnulla` | — | operator |

## Examples

### Hello World

```faber
functio salve(nomen) -> Textus {
  redde "Salve, " + nomen + "!"
}

fixum nomen = "Mundus"
scribe salve(nomen)
```

### Control Flow

```faber
fixum nota = 85

si nota >= 90 {
  scribe "A"
}
aliter si nota >= 80 {
  scribe "B"
}
aliter {
  scribe "F"
}

// Switch
elige status {
  si "pending" ergo scribe "waiting"
  si "active" ergo scribe "running"
  aliter scribe "unknown"
}
```

### Loops and Collections

```faber
fixum numeri = [1, 2, 3, 4, 5]

ex numeri pro n {
  scribe n
}

// Range with step
ex 0..10 per 2 pro i {
  scribe i
}

// While
esto i = 0
dum i < 3 {
  scribe i
  i = i + 1
}
```

### Functions

```faber
functio adde(a, b) -> Numerus {
  redde a + b
}

functio divide(a, b) -> Numerus {
  custodi {
    si b == 0 { iace "Divisio per nihil" }
  }
  redde a / b
}

scribe adde(2, 3)
```

### Objects

```faber
fixum persona = {
  nomen: "Marcus",
  aetas: 30
}

scribe persona.nomen

fixum { nomen, aetas } = persona
scribe nomen
```
