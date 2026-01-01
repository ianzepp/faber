---
status: implemented
targets: [ts, py]
updated: 2024-12
---

# Async Design (Arrow Syntax)

This document covers the **arrow syntax** (`->`) for async/generator functions using `futura`/`cursor` prefixes. For the **verb syntax** (`fit`/`fiet`/`fiunt`/`fient`) with the Responsum stream protocol, see `flumina.md`.

## Two Syntax Systems

Faber has two mutually exclusive syntax patterns for function return types:

| Syntax System | Return Clause                | Prefixes Allowed   | Protocol         |
| ------------- | ---------------------------- | ------------------ | ---------------- |
| Arrow         | `-> T`                       | `futura`, `cursor` | Direct return    |
| Verb          | `fit`/`fiet`/`fiunt`/`fient` | None               | Responsum stream |

**They cannot be mixed.** Using `futura`/`cursor` with verb syntax is a parser error (P191).

### Arrow Syntax (this document)

| Declaration                      | Semantics                            |
| -------------------------------- | ------------------------------------ |
| `functio f() -> T`               | sync, single return                  |
| `futura functio f() -> T`        | async, returns `Promise<T>`          |
| `cursor functio f() -> T`        | sync generator, yields `T`           |
| `futura cursor functio f() -> T` | async generator, yields `Promise<T>` |

### Verb Syntax (see flumina.md)

| Declaration           | Semantics                            |
| --------------------- | ------------------------------------ |
| `functio f() fit T`   | sync, single (Responsum protocol)    |
| `functio f() fiet T`  | async, single (Responsum protocol)   |
| `functio f() fiunt T` | sync generator (Responsum protocol)  |
| `functio f() fient T` | async generator (Responsum protocol) |

---

## Keywords

### futura

Marks a function as asynchronous. Must use `->` arrow syntax.

```fab
futura functio fetchData(textus url) -> textus {
    fixum response = cede fetch(url)
    redde response.text()
}
```

Compiles to:

```typescript
async function fetchData(url: string): Promise<string> {
    const response = await fetch(url);
    return response.text();
}
```

### cursor

Marks a function as a generator. Must use `->` arrow syntax.

```fab
cursor functio range(numerus n) -> numerus {
    ex 0..n pro i {
        cede i
    }
}
```

### cede

In `futura functio`: awaits a promise.
In `cursor functio`: yields a value.

```fab
# Async context - await
futura functio getData() -> textus {
    fixum result = cede fetch(url)
    redde result
}

# Generator context - yield
cursor functio items() -> numerus {
    cede 1
    cede 2
    cede 3
}
```

**Etymology:** "yield, give way, surrender" — ceding control until the operation completes.

### figendum / variandum

Gerundive forms with implicit await:

```fab
figendum data = fetchData(url)   # immutable, implicit await
variandum result = fetchInitial() # mutable, implicit await
```

Equivalent to `fixum x = cede y()` and `varia x = cede y()`. The gerundive signals futurity — the value will be fixed/varied once the operation completes.

**Sync/async agnostic:** If the RHS is a promise, it awaits; if plain, it passes through.

---

## Error Handling

```fab
tempta {
    fixum data = cede fetchData(url)
    scribe data
}
cape err {
    scribe "Error: " + err.message
}
```

---

## Patterns

### Sequential Async

```fab
futura functio processAll(textus[] urls) -> textus[] {
    varia results = []
    ex urls pro url {
        fixum data = cede fetchData(url)
        results.adde(data)
    }
    redde results
}
```

### Parallel Async

```fab
futura functio fetchAll(textus[] urls) -> textus[] {
    fixum promises = urls.mappa((url) => fetchData(url))
    redde cede Promise.all(promises)
}
```

Note: For parallel async, use the target's native Promise API directly. Faber passes through method calls on external types.

---

## Implementation Notes

### TypeScript Target

Direct mapping:

- `futura functio` -> `async function`
- `cede` (in async) -> `await`
- `promissum<T>` -> `Promise<T>`

### Zig Target

Zig's async model differs significantly:

- Functions return frames
- Explicit suspend points
- No built-in Promise type

Currently deferred — async limited to TS/Python targets.
