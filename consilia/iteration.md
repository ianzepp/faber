# Iteration Design

## Overview

Faber provides several iteration constructs:

| Construct | Purpose | Example |
|-----------|---------|---------|
| `ex...pro` | Iterate values | `ex items pro item { }` |
| `in...pro` | Iterate keys | `in object pro key { }` |
| `dum` | While loop | `dum x > 0 { }` |
| Range | Numeric sequence | `ex 0..10 pro n { }` |

Plus iterator types:
- `cursor<T>` — synchronous iterator
- `fluxus<T>` — async iterator/stream

---

## ex...pro (For-Each Values)

**Syntax:** `ex <iterable> pro <variable> { body }`

Iterates over values of a collection.

```
fixum numeri = [1, 2, 3]
ex numeri pro n {
    scribe n
}
```

Compiles to:
```typescript
const numeri = [1, 2, 3];
for (const n of numeri) {
    console.log(n);
}
```

### One-liner Form

```
ex numeri pro n ergo scribe n
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

Creates a numeric sequence.

```
ex 0..5 pro n {
    scribe n  // 0, 1, 2, 3, 4
}

ex 0..10 per 2 pro n {
    scribe n  // 0, 2, 4, 6, 8
}

ex 10..0 per -1 pro n {
    scribe n  // 10, 9, 8, ... 1
}
```

### Inclusive vs Exclusive

Current: `0..5` is exclusive of end (0, 1, 2, 3, 4)

Open question: Add inclusive syntax?
- `0..=5` for inclusive (Rust-style)
- `0...5` for inclusive (Swift-style)
- Keep exclusive only, use `0..6` for inclusive 0-5

---

## dum (While)

**Syntax:** `dum <condition> { body }`

```
esto i = 0
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
    esto i = 0
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

```
futura ex stream pro chunk {
    scribe chunk
}
```

Compiles to:
```typescript
for await (const chunk of stream) {
    console.log(chunk);
}
```

### Protocol

```
structura fluxus<T> {
    futura functio proximus() -> { valor: T?, factum: bivalens }
}
```

---

## Generators

### Declaration Syntax

The keyword matches the return type — one concept, one keyword:

| Declaration | Produces | JS Equivalent |
|-------------|----------|---------------|
| `cursor functio` | `cursor<T>` | `function*` |
| `fluxus functio` | `fluxus<T>` | `async function*` |

**Sync generator:**

```
cursor functio numerare(numerus n) -> numerus {
    esto i = 0
    dum i < n {
        cede i
        i = i + 1
    }
}

// Usage
ex numerare(10) pro n {
    scribe n
}
```

**Async generator:**

```
fluxus functio lege(numerus fd) -> bytes {
    futura ex syscall('file:read', fd) pro r {
        si r.op == 'data' {
            cede r.bytes
        }
        aliter si r.op == 'done' {
            redde
        }
    }
}

// Usage
futura ex lege(fd) pro chunk {
    process(chunk)
}
```

### Yield Keyword: `cede`

**Etymology:** "yield, give way, withdraw" — the root of "cede" and "concede."

- `cede value` — yield a single value
- `cede* iterator` — delegate to another iterator (yield*)

```
cursor functio omnia() -> numerus {
    cede* numerare(5)    // yields 0,1,2,3,4
    cede* numerare(3)    // yields 0,1,2
}
```

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
| `ex...pro` | `for...of` |
| `in...pro` | `for...in` |
| `dum` | `while` |
| `rumpe` | `break` |
| `perge` | `continue` |
| `cursor<T>` | `Iterator<T>` |
| `fluxus<T>` | `AsyncIterator<T>` |

### Zig Target

| Faber | Zig |
|-------|-----|
| `ex...pro` | `for` loop over slice/iterator |
| `dum` | `while` |
| `rumpe` | `break` |
| `perge` | `continue` |

Zig iteration is more explicit — may need adapter patterns.
