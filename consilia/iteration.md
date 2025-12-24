# Iteration Design

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `ex...pro/fit/fiet` loops | Done | For-of iteration |
| `in...pro/fit` loops | Done | For-in iteration |
| Range `0..10` | Done | Inclusive endpoints |
| Range step `per` | Done | `0..10 per 2` |
| `dum` while loop | Done | Basic while |
| Generators `fiunt/fient` | Done | Verb conjugation |
| Prefix `cursor/fluxus` | Done | Alternative syntax |
| `cede` yield | Done | Context-aware (yield/await) |
| One-liner `ergo` | Done | `ex items pro n ergo scribe n` |
| Error handling `cape` | Done | `ex items pro n { } cape err { }` |
| `rumpe` (break) | **Not done** | Designed only |
| `perge` (continue) | **Not done** | Designed only |
| Pipeline `per` transforms | **Not done** | Future feature |
| Indexed iteration | **Not done** | Open question |
| `cede ex` delegation | **Not done** | yield* equivalent |

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

| Keyword | Meaning | Compiles to |
|---------|---------|-------------|
| `pro` | for (preposition) | `for...of` |
| `fit` | becomes (sync verb) | `for...of` |
| `fiet` | will become (async) | `for await...of` |

```
// Sync (equivalent)
ex items pro item { scribe item }
ex items fit item { scribe item }

// Async
ex stream fiet chunk { scribe chunk }
```

### in...pro (For-Each Keys)

**Syntax:** `in <object> (pro | fit) <key> { body }`

```
fixum persona = { nomen: "Marcus", aetas: 30 }
in persona pro clavis {
    scribe clavis + ": " + persona[clavis]
}
```

Compiles to `for...in`.

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

**Syntax:** `start..end [per step]`

**End is exclusive** (matches Rust, Python, lodash conventions).

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

**Why exclusive?** Matches Rust (`0..5`), Python (`range(0, 5)`), and lodash (`_.range(0, 5)`). Avoids off-by-one errors when working across targets. To include 5, use `0..6`.

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

| Declaration | Meaning | JS Output |
|-------------|---------|-----------|
| `functio f() fit T` | sync, returns one | `function f(): T` |
| `functio f() fiet T` | async, returns one | `async function f(): Promise<T>` |
| `functio f() fiunt T` | sync, yields many | `function* f(): Generator<T>` |
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

| Faber | TypeScript | Purpose |
|-------|------------|---------|
| `cursor<T>` | `Iterator<T>` | Sync iterator |
| `fluxus<T>` | `AsyncIterator<T>` | Async iterator |

These are recognized as generic types but don't yet have method implementations (`.proximus()`, etc.).

---

## Future: Pipeline Syntax

**Status: Designed, not implemented**

The `per` keyword is envisioned to chain transformations inline:

```
// Future syntax
ex users per filtra({ .activus }) per ordina(cum nomen) pro user {
    scribe user.nomen
}
```

Reading: "from users, through filter, through sort, as user..."

Currently, use method chaining on the iterable instead:
```
ex users.filtrata({ .activus }).ordinata(cum nomen) pro user {
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

## Open Questions

1. **Indexed iteration**: Best syntax for index access?
   - `ex numeri pro (i, n) { }` — tuple destructuring
   - `ex numeri pro n cum indice i { }` — explicit clause
   - `ex numeri.cumIndice() pro (i, n) { }` — method approach

2. **`cede ex` delegation**: Yield from another iterator (like JS `yield*`)
   ```
   cede ex numerare(5)  // yields 0,1,2,3,4
   ```

3. **Early return**: Should `redde` inside loops return from enclosing function? (Yes, like JS)

---

## Target Mappings

### TypeScript

| Faber | TypeScript |
|-------|------------|
| `ex...pro/fit` | `for...of` |
| `ex...fiet` | `for await...of` |
| `in...pro/fit` | `for...in` |
| `dum` | `while` |
| `cede` (generator) | `yield` |
| `cede` (async) | `await` |
| `fiunt` | `function*` |
| `fient` | `async function*` |

### Python

| Faber | Python |
|-------|--------|
| `ex...pro` | `for x in iterable:` |
| `ex 0..10` | `for x in range(0, 11):` (note +1 for inclusive) |
| `dum` | `while` |
| `cede` | `yield` |

### Zig

| Faber | Zig |
|-------|-----|
| `ex...pro` | `for (slice) \|item\|` |
| `ex 0..10` | `while (i <= 10) : (i += 1)` |
| `dum` | `while` |

Zig has no async iteration — requires different approach.
