# Iteration Design

## Overview

Faber provides several iteration constructs:

| Construct | Purpose | Example |
|-----------|---------|---------|
| `ex...pro` | Iterate values (sync) | `ex items pro item { }` |
| `ex...fit` | Iterate values (sync, verb) | `ex items fit item { }` |
| `ex...fiet` | Iterate values (async) | `ex stream fiet chunk { }` |
| `in...pro` | Iterate keys | `in object pro key { }` |
| `dum` | While loop | `dum x > 0 { }` |
| Range | Numeric sequence | `ex 0..10 fit n { }` |

Plus iterator types:
- `cursor<T>` — synchronous iterator
- `fluxus<T>` — async iterator/stream

---

## Verb Conjugation in Loops

The binding keyword can use verb conjugation to encode sync/async semantics, consistent with function return types:

| Keyword | Meaning | Compiles to |
|---------|---------|-------------|
| `pro` | for (preposition) | `for...of` / `for...in` |
| `fit` | becomes (sync verb) | `for...of` / `for...in` |
| `fiet` | will become (async verb) | `for await...of` / `for await...in` |

Examples:
```
// Sync iteration (equivalent)
ex items pro item { scribe item }
ex items fit item { scribe item }

// Async iteration
ex stream fiet chunk { scribe chunk }
```

The verb forms (`fit`/`fiet`) parallel function return types:
- Functions: `functio f() fit T` (sync) vs `functio f() fiet T` (async)
- Loops: `ex items fit item` (sync) vs `ex items fiet item` (async)

---

## ex...pro (For-Each Values)

**Syntax:** `ex <source> [per <transform>]... (pro | fit | fiet) <variable> { body }`

Iterates over values, optionally through a transformation pipeline.

```
fixum numeri = [1, 2, 3]
ex numeri pro n {
    scribe n
}
```

### Pipeline Syntax with `per`

The `per` keyword chains transformations inline:

```
// From users, through filter, as user
ex users per filtra({ .activus }) pro user {
    scribe user.nomen
}

// Multiple transforms
ex users per filtra({ .activus }) per ordina(cum nomen) pro user {
    scribe user.nomen
}

// Numeric pipeline
ex 0..100 per filtra({ . % 2 == 0 }) per mappa({ . * 2 }) pro n {
    scribe n
}
```

**Reading:** "from users, through filter, through sort, as user..."

| Keyword | Meaning | Role |
|---------|---------|------|
| `ex` | from | source |
| `per` | through | transformation |
| `pro` | as/for | variable binding |

Compiles to:
```typescript
for (const user of users.filter(u => u.activus).sort((a,b) => a.nomen.localeCompare(b.nomen))) {
    console.log(user.nomen);
}
```

### Custom Functions in Pipeline

Any function that transforms an iterable works in `per`:

```
// Custom filter with complex logic
functio soloActivi(lista<user> users) -> lista<user> {
    redde users.filtrata({
        .activus et .verificatus et .aetas >= 18
    })
}

ex users per soloActivi pro user {
    scribe user.nomen
}
```

Generators work too — useful for stateful transformations:

```
cursor functio dedupe(lista<T> items) -> T {
    fixum seen = copia.nova()
    ex items pro item {
        si non seen.habet(item) {
            seen.adde(item)
            cede item
        }
    }
}

ex users per dedupe pro user {
    scribe user.nomen
}
```

The `per` clause accepts:
- Built-in methods (`filtra`, `ordina`, `mappa`)
- Custom functions returning iterables
- Generators (`cursor functio`, `fluxus functio`)

### One-liner Form

```
ex numeri pro n ergo scribe n
ex users per filtra({ .activus }) pro u ergo scribe u.nomen
```

### With Index

Open question: How to access index during iteration?

Options:
1. Tuple destructuring: `ex numeri pro (i, n) { }`
2. Separate construct: `ex numeri pro n cum indice i { }`
3. Method: `ex numeri.cumIndice() pro (i, n) { }`

---

## in...pro (For-Each Keys)

**Syntax:** `in <object> pro <key> { body }`

Iterates over keys/properties of an object.

```
fixum persona = { nomen: "Marcus", aetas: 30 }
in persona pro clavis {
    scribe clavis + ": " + persona[clavis]
}
```

Compiles to:
```typescript
const persona = { nomen: "Marcus", aetas: 30 };
for (const clavis in persona) {
    console.log(clavis + ": " + persona[clavis]);
}
```

### Key-Value Iteration

For maps, use `ex` with `.paria()`:

```
ex tabula.paria() pro (clavis, valor) {
    scribe clavis + " => " + valor
}
```

---

## Range Expressions

**Syntax:** `start..end [per step]`

Creates a numeric sequence. **Both endpoints are inclusive.**

```
ex 0..5 pro n {
    scribe n  // 0, 1, 2, 3, 4, 5
}

ex 0..10 per 2 pro n {
    scribe n  // 0, 2, 4, 6, 8, 10
}

ex 10..1 per -1 pro n {
    scribe n  // 10, 9, 8, ... 1
}
```

### Why Inclusive?

"From 0 to 10" in natural language includes both endpoints. The exclusive convention in other languages exists because of `i < 10` in C-style loops — but Faber doesn't expose `<` or `++` in range syntax, so exclusion would be arbitrary and unintuitive.

```
ex 0..9 pro n { ... }   // ten digits: 0 through 9
ex 1..10 pro n { ... }  // ten numbers: 1 through 10
```

No mental gymnastics. Say what you mean.

---

## dum (While)

**Syntax:** `dum <condition> { body }`

```
varia i = 0
dum i < 10 {
    scribe i
    i = i + 1
}
```

### One-liner Form

```
dum i > 0 ergo i = i - 1
```

### Infinite Loop

```
dum verum {
    // forever
    si done ergo rumpe
}
```

---

## Control Flow

### rumpe (break)

Exit loop immediately.

```
ex items pro item {
    si item == target {
        rumpe
    }
}
```

### perge (continue)

Skip to next iteration.

```
ex items pro item {
    si item < 0 {
        perge
    }
    scribe item
}
```

---

## cursor<T> — Synchronous Iterator

**Etymology:** "runner" — one who runs through a sequence.

### Protocol

```
structura cursor<T> {
    functio proximus() -> { valor: T?, factum: bivalens }
}
```

Maps to JavaScript Iterator protocol:
```typescript
interface Iterator<T> {
    next(): { value: T | undefined, done: boolean }
}
```

### Creating Iterators

From collections:
```
fixum iter = lista.valores()
```

Custom iterator (requires generator syntax — see Open Questions):
```
// Proposed generator syntax
functio* numerare(numerus n) -> cursor<numerus> {
    varia i = 0
    dum i < n {
        cede i    // yield
        i = i + 1
    }
}
```

---

## fluxus<T> — Async Iterator

**Etymology:** "flow, stream" — continuous flow of values.

For async iteration over streams, events, etc.

### Async For-Each

Use `fiet` (will become) for async iteration:

```
ex stream fiet chunk {
    scribe chunk
}
```

Compiles to:
```typescript
for await (const chunk of stream) {
    console.log(chunk);
}
```

The `fiet` verb form replaces the older `futura ex...pro` syntax, providing consistency with function return type conjugation.

### Protocol

```
structura fluxus<T> {
    futura functio proximus() -> { valor: T?, factum: bivalens }
}
```

---

## Generators

### Declaration Syntax

Generators use verb conjugation to encode their semantics:

| Declaration | Produces | JS Equivalent |
|-------------|----------|---------------|
| `functio f() fiunt T` | `Generator<T>` | `function*` |
| `functio f() fient T` | `AsyncGenerator<T>` | `async function*` |
| `cursor functio f() -> T` | `Generator<T>` | `function*` (prefix style) |
| `fluxus functio f() -> T` | `AsyncGenerator<T>` | `async function*` (prefix style) |

The verb forms (`fiunt`/`fient`) are preferred — they encode semantics grammatically:
- `fiunt` = "they become" (present plural) → yields many values synchronously
- `fient` = "they will become" (future plural) → yields many values asynchronously

**Sync generator:**

```
functio numerare(numerus n) fiunt numerus {
    varia i = 0
    dum i < n {
        cede i
        i = i + 1
    }
}

// Usage
ex numerare(10) fit n {
    scribe n
}
```

**Async generator:**

```
functio lege(numerus fd) fient bytes {
    ex syscall('file:read', fd) fiet r {
        si r.op == 'data' {
            cede r.bytes
        }
        aliter si r.op == 'done' {
            redde
        }
    }
}

// Usage
ex lege(fd) fiet chunk {
    process(chunk)
}
```

### Yield Keyword: `cede`

**Etymology:** "yield, give way, withdraw" — the root of "cede" and "concede."

- `cede value` — yield a single value
- `cede ex iterator` — delegate to another iterator (yield*)

```
cursor functio omnia() -> numerus {
    cede ex numerare(5)    // yields 0,1,2,3,4
    cede ex numerare(3)    // yields 0,1,2
}
```

The `cede ex` pattern mirrors `ex...pro` — "yield from this source."

### Return Type

The return type annotation is the *element* type, not the full iterator type:

```
cursor functio foo() -> numerus { ... }   // produces cursor<numerus>
fluxus functio bar() -> textus { ... }    // produces fluxus<textus>
```

The `cursor` or `fluxus` keyword already indicates the wrapper type.

---

## Open Questions

1. **Indexed iteration**: Best syntax for index access?
   - Proposal: `ex lista.cumIndice() pro (i, val) { }`

2. **Early return from iteration**: Allow `redde` inside `ex...pro`?
   - JS: Yes, returns from enclosing function
   - Should work the same in Faber

3. **Labeled loops**: For breaking outer loops?
   ```
   // Proposed
   @exterior: ex items pro item {
       ex subitems pro sub {
           si found ergo rumpe @exterior
       }
   }
   ```

---

## Implementation Notes

### TypeScript Target

| Faber | TypeScript |
|-------|------------|
| `ex...pro` / `ex...fit` | `for...of` |
| `ex...fiet` | `for await...of` |
| `in...pro` / `in...fit` | `for...in` |
| `in...fiet` | `for await...in` |
| `dum` | `while` |
| `rumpe` | `break` |
| `perge` | `continue` |
| `cursor<T>` | `Iterator<T>` |
| `fluxus<T>` | `AsyncIterator<T>` |

### Zig Target

| Faber | Zig |
|-------|-----|
| `ex...pro` / `ex...fit` | `for` loop over slice/iterator |
| `dum` | `while` |
| `rumpe` | `break` |
| `perge` | `continue` |

Zig iteration is more explicit — may need adapter patterns. Async iteration (`fiet`) will require special handling for Zig's different concurrency model.
