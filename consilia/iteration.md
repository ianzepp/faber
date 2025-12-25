# Iteration Design

## Implementation Status

| Feature                   | Status       | Notes                             |
| ------------------------- | ------------ | --------------------------------- |
| `ex...pro/fit/fiet` loops | Done         | For-of iteration                  |
| `de...pro/fit/fiet` loops | Done         | For-in iteration                  |
| Range `0..10`             | Done         | Exclusive end (like Python/Rust)  |
| Range step `per`          | Done         | `0..10 per 2`                     |
| `dum` while loop          | Done         | Basic while                       |
| Generators `fiunt/fient`  | Done         | Verb conjugation                  |
| Prefix `cursor/fluxus`    | Done         | Alternative syntax                |
| `cede` yield              | Done         | Context-aware (yield/await)       |
| One-liner `ergo`          | Done         | `ex items pro n ergo scribe n`    |
| Error handling `cape`     | Done         | `ex items pro n { } cape err { }` |
| `rumpe` (break)           | Done         | Exits innermost loop              |
| `perge` (continue)        | Done         | Skips to next iteration           |
| Pipeline `per` transforms | **Not done** | Future feature                    |
| Indexed iteration         | **Not done** | Tuple destructuring `(i, n)`      |
| `cede ex` delegation      | **Not done** | yield\* equivalent                |

---

## Basic Loops

### ex...pro (For-Each Values)

**Syntax:** `ex <iterable> (pro | fit | fiet) <variable> { body }`

```
fixum numeri = [1, 2, 3]
ex numeri pro n {
    scribe n
}
```

The binding keyword encodes sync/async:

| Keyword | Meaning             | Compiles to      |
| ------- | ------------------- | ---------------- |
| `pro`   | for (preposition)   | `for...of`       |
| `fit`   | becomes (sync verb) | `for...of`       |
| `fiet`  | will become (async) | `for await...of` |

```
// Sync (equivalent)
ex items pro item { scribe item }
ex items fit item { scribe item }

// Async
ex stream fiet chunk { scribe chunk }
```

### de...pro (For-Each Keys)

**Syntax:** `de <object> (pro | fit) <key> { body }`

```
fixum persona = { nomen: "Marcus", aetas: 30 }
de persona pro clavis {
    scribe clavis + ": " + persona[clavis]
}
```

Compiles to `for...in`. Uses `de` (from/concerning) because we're extracting keys from the object — a read operation.

### One-liner Form

Use `ergo` instead of block:

```
ex numeri pro n ergo scribe n
```

### Error Handling

Attach `cape` clause for error handling within iteration:

```
ex items pro item {
    riskyOperation(item)
} cape err {
    scribe "Error: " + err
}
```

---

## Range Expressions

**Syntax:** `start (.. | ante | usque) end [per step]`

Three range operators with different end semantics:

| Operator | Latin meaning | End       | Example             |
| -------- | ------------- | --------- | ------------------- |
| `..`     | (shorthand)   | exclusive | `0..10` → 0-9       |
| `ante`   | "before"      | exclusive | `0 ante 10` → 0-9   |
| `usque`  | "up to"       | inclusive | `0 usque 10` → 0-10 |

```
ex 0..5 pro n {
    scribe n  // 0, 1, 2, 3, 4
}

ex 0 usque 5 pro n {
    scribe n  // 0, 1, 2, 3, 4, 5
}

ex 0..10 per 2 pro n {
    scribe n  // 0, 2, 4, 6, 8
}

ex 0 usque 10 per 2 pro n {
    scribe n  // 0, 2, 4, 6, 8, 10
}
```

**Why both?** `..` (exclusive) matches target language semantics for direct codegen. `usque` (inclusive) reads naturally in Latin for mathematical ranges ("zero up to ten").

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

One-liner:

```
dum i > 0 ergo i = i - 1
```

---

## Generators

Generators use verb conjugation to encode semantics:

| Declaration           | Meaning            | JS Output                                |
| --------------------- | ------------------ | ---------------------------------------- |
| `functio f() fit T`   | sync, returns one  | `function f(): T`                        |
| `functio f() fiet T`  | async, returns one | `async function f(): Promise<T>`         |
| `functio f() fiunt T` | sync, yields many  | `function* f(): Generator<T>`            |
| `functio f() fient T` | async, yields many | `async function* f(): AsyncGenerator<T>` |

Alternative prefix syntax:

- `cursor functio f() -> T` — sync generator
- `fluxus functio f() -> T` — async generator

**Example:**

```
functio numerare(numerus n) fiunt numerus {
    varia i = 0
    dum i < n {
        cede i
        i = i + 1
    }
}

ex numerare(10) fit n {
    scribe n
}
```

### cede (yield/await)

The `cede` keyword is context-aware:

- In generator: compiles to `yield`
- In async function: compiles to `await`
- In async generator: compiles to `yield`

---

## Iterator Types

| Faber       | TypeScript         | Purpose        |
| ----------- | ------------------ | -------------- |
| `cursor<T>` | `Iterator<T>`      | Sync iterator  |
| `fluxus<T>` | `AsyncIterator<T>` | Async iterator |

These are recognized as generic types but don't yet have method implementations (`.proximus()`, etc.).

---

## Future: Pipeline Syntax

**Status: Designed, not implemented**

The `per` keyword is envisioned to chain transformations inline:

```
// Future syntax
ex users per filtra({ .activus }) per ordina(per nomen) pro user {
    scribe user.nomen
}
```

Reading: "from users, through filter, through sort, as user..."

Currently, use method chaining on the iterable instead:

```
ex users.filtrata({ .activus }).ordinata(per nomen) pro user {
    scribe user.nomen
}
```

---

## Future: Control Flow

**Status: Designed, not implemented**

### rumpe (break)

```
ex items pro item {
    si item == target {
        rumpe
    }
}
```

### perge (continue)

```
ex items pro item {
    si item < 0 {
        perge
    }
    scribe item
}
```

### Labeled Loops

For breaking outer loops:

```
@exterior: ex items pro item {
    ex subitems pro sub {
        si found ergo rumpe @exterior
    }
}
```

---

## Future: Indexed Iteration

**Status: Designed, not implemented**

Use tuple destructuring to bind both index and value:

```
ex numeri pro (i, n) {
    scribe i + ": " + n
}
```

Index comes first, matching Python's `enumerate()` convention. Compiles to:

**TypeScript:**

```typescript
for (const [i, n] of numeri.entries()) {
    console.log(i + ': ' + n);
}
```

**Python:**

```python
for i, n in enumerate(numeri):
    print(f"{i}: {n}")
```

Works with ranges too:

```
ex 0..10 pro (i, n) {
    // i and n are the same for ranges
}
```

---

## Open Questions

1. **`cede ex` delegation**: Yield from another iterator (like JS `yield*`)

    ```
    cede ex numerare(5)  // yields 0,1,2,3,4
    ```

2. **Early return**: Should `redde` inside loops return from enclosing function? (Yes, like JS)

---

## Target Mappings

### TypeScript

| Faber              | TypeScript        |
| ------------------ | ----------------- |
| `ex...pro/fit`     | `for...of`        |
| `ex...fiet`        | `for await...of`  |
| `de...pro/fit`     | `for...in`        |
| `dum`              | `while`           |
| `cede` (generator) | `yield`           |
| `cede` (async)     | `await`           |
| `fiunt`            | `function*`       |
| `fient`            | `async function*` |

### Python

| Faber      | Python                   |
| ---------- | ------------------------ |
| `ex...pro` | `for x in iterable:`     |
| `ex 0..10` | `for x in range(0, 10):` |
| `dum`      | `while`                  |
| `cede`     | `yield`                  |

### Zig

| Faber      | Zig                         |
| ---------- | --------------------------- |
| `ex...pro` | `for (slice) \|item\|`      |
| `ex 0..10` | `while (i < 10) : (i += 1)` |
| `dum`      | `while`                     |

Zig has no async iteration — requires different approach.
