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
esto Textus nomen = "Marcus"        // mutable (let) — "let it be"
fixum Textus nomen = "Marcus"       // immutable (const) — "fixed"

fixum Numerus PI = 3.14159          // constant with type
esto nomen = "Marcus"               // type inferred from literal
```

## Functions

```faber
functio Textus salve(Textus nomen) {
  redde "Salve, " + nomen
}

// Async function
futura functio Textus fetchData(Textus url) {
  fixum Res response = exspecta fetch(url)
  redde exspecta response.text()
}
```

## Control Flow

### Conditionals

```faber
si conditio {
  // if
}
aliter si alia {
  // else if
}
aliter {
  // else
}
```

### Switch / Pattern Matching

```faber
elige valor {
  quando 1 => scribe("unum")
  quando 2 => scribe("duo")
  aliter => scribe("ignotum")
}
```

### Loops

```faber
dum conditio {
  // while
}

fac {
  // do...while
} dum conditio

in usuarios pro usuario {
  // for...in (keys)
}

ex numeris pro numero {
  // for...of (values)
}

pro (esto i = 0; i < 10; i++) {
  // traditional for (available but discouraged)
}
```

### Loop Control

| Latin | JavaScript | Meaning |
|-------|------------|---------|
| `rumpe` | `break` | break out of loop |
| `perge` | `continue` | continue to next iteration |

## Error Handling

Any block can have a `cape` clause:

```faber
si fetch(url) {
  process()
} cape erratum {
  handleError(erratum)
}

in items pro item {
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
  cleanup()  // finally — always runs
}
```

Throw:

```faber
iace novum Erratum("Something went wrong")
```

## Arrow Functions

```faber
fixum duplex = (x) => x * 2

fixum processare = (x) => {
  fixum Numerus result = x * 2
  redde result
}

// With types
fixum saluta = (Textus nomen) => `Salve, ${nomen}!`

// As callback
usuarios.filter((u) => u.activa)
```

## Operators

| Latin | JavaScript | Meaning |
|-------|------------|---------|
| `et` | `&&` | and |
| `aut` | `\|\|` | or |
| `non` | `!` | not |

Both Latin and symbol operators are valid. Prefer Latin when mixing with symbols for readability.

## Keywords Reference

| Latin | JavaScript | Category |
|-------|------------|----------|
| `si` | `if` | control |
| `aliter` | `else` | control |
| `elige` | `switch` | control |
| `quando` | `case` | control |
| `dum` | `while` | control |
| `fac` | `do` | control |
| `pro` | `for` | control |
| `in` | `in` | preposition |
| `ex` | `of` | preposition |
| `rumpe` | `break` | control |
| `perge` | `continue` | control |
| `tempta` | `try` | error |
| `cape` | `catch` | error |
| `demum` | `finally` | error |
| `iace` | `throw` | error |
| `exspecta` | `await` | async |
| `novum` | `new` | expression |
| `esto` | `let` | declaration |
| `fixum` | `const` | declaration |
| `functio` | `function` | declaration |
| `futura` | `async` | modifier |
| `redde` | `return` | control |
| `verum` | `true` | value |
| `falsum` | `false` | value |
| `nihil` | `null` | value |

## Examples

### Hello World

```faber
functio Textus salve(Textus nomen) {
  si nomen.vacuum {
    redde "Salve, hospes"
  }
  redde "Salve, " + nomen
}

esto Textus nomen = lege("Quid est nomen tuum? ")
scribe(salve(nomen))
```

### With Error Handling

```faber
functio Vacuum mitte(Textus nuntium, Textus recipiens, HttpCliens client) {
  esto Res responsum = client.posta(recipiens, nuntium)
  si responsum.successum {
    scribe("Missum est!")
  }
  aliter {
    scribe("Error: " + responsum.error)
  }
}
```

### Full Program

```faber
// Variables with explicit types
fixum Textus nomen = "Marcus"
esto Numerus<32> aetas = 30
fixum Fractus pretium = 19.99
fixum Decimus<64> saldo = 1000.00

// Collections
fixum Lista<Textus> nomina = ["Marcus", "Julius", "Claudia"]
fixum Tabula<Textus, Numerus> aetates = { "Marcus": 30, "Julius": 45 }

// Optional values
fixum Textus? cognomen = nihil

// Functions
functio Numerus adde(Numerus a, Numerus b) {
  redde a + b
}

functio Fors<Numerus> divide(Numerus a, Numerus b) {
  si b == 0 {
    iace novum Erratum("Divisio per nihil")
  }
  redde a / b
}

// Type inference (type optional when inferable)
fixum salutatio = "Salve"    // inferred Textus
esto summa = 0               // inferred Numerus
```
