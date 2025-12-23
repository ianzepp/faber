# Async Design

## TODO: Verb Conjugation for Async/Iterator Semantics

Consider replacing `futura`/`cursor` prefixes with Latin verb conjugation of *fieri* (to become):

| | Sync | Async |
|---|---|---|
| **Single return** | `fit textus` | `fiet textus` |
| **Iterator/yield** | `fiunt textus` | `fient textus` |

Examples:
```
functio greet(name) fit textus { ... }       // sync, returns string
functio fetch(url) fiet textus { ... }       // async, returns Promise<string>
functio range(n) fiunt numerus { ... }       // sync generator, yields numbers
functio stream(url) fient textus { ... }     // async generator, yields strings
```

Grammar encodes both dimensions:
- **Tense**: present (sync) vs future (async)
- **Number**: singular (return once) vs plural (yield many)

This eliminates `futura`/`cursor` prefixes — the return type verb carries all semantics.

---

## Overview

Faber's async model maps to native async/await patterns:

| Faber | JavaScript | Zig |
|-------|------------|-----|
| `futura functio` | `async function` | `fn` returning frame |
| `cede` | `await` | `await` / suspend |
| `promissum<T>` | `Promise<T>` | Future-like pattern |

**Unified suspension model:** `cede` is shared with generators (`cursor functio`). The function modifier determines semantics:
- In `futura functio`: `cede expr` awaits a promise
- In `cursor functio`: `cede expr` yields a value

---

## Keywords

### futura

Marks a function as asynchronous. Returns `promissum<T>` implicitly.

```
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

### cede (in async context)

Awaits a promise. Only valid inside `futura functio`.

```
fixum data = cede fetchData("https://api.example.com")
```

**Etymology:** "yield, give way, surrender" — ceding control to the event loop until the promise resolves.

**See also:** In `cursor functio`, `cede` yields values to the caller (see iteration.md).

---

## promissum<T>

**Etymology:** "promise, guarantee" — something pledged for the future.

### Instance Methods

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `tunc((T) -> U) -> promissum<U>` | `then` | Chain success |
| `cape((erratum) -> U) -> promissum<U>` | `catch` | Handle error |
| `demum(() -> vacuum) -> promissum<T>` | `finally` | Always run |

**Naming rationale:**
- `tunc` = "then" (Latin temporal adverb)
- `cape` = "catch, seize" (imperative of capere)
- `demum` = "finally, at last" (Latin adverb)

### Static Constructors

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `promissum.da(T)` | `Promise.resolve` | Immediate success |
| `promissum.nega(erratum)` | `Promise.reject` | Immediate failure |

**Naming rationale:**
- `da` = "give" (imperative of dare) — here's your value
- `nega` = "deny" (imperative of negare) — refused with error

### Static Combinators

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `promissum.omnes(lista<promissum<T>>)` | `Promise.all` | All must succeed |
| `promissum.primus(lista<promissum<T>>)` | `Promise.race` | First to settle |
| `promissum.quilibet(lista<promissum<T>>)` | `Promise.any` | First to succeed |
| `promissum.finita(...)` | `Promise.allSettled` | All settled |

**Naming rationale:**
- `omnes` = "all" (must succeed)
- `primus` = "first" (race — first to settle wins)
- `quilibet` = "any, whichever" (first to succeed)
- `finita` = "finished" (all done, regardless of success/failure)

---

## Error Handling

### With tempta/cape

```
tempta {
    fixum data = cede fetchData(url)
    scribe data
}
cape err {
    scribe "Error: " + err.message
}
```

### With .cape() method

```
fetchData(url)
    .tunc((data) => scribe data)
    .cape((err) => scribe "Error: " + err.message)
```

### Unhandled Rejections

Open question: How to handle unhandled promise rejections?
- Compiler warning for promises not awaited or `.cape()`'d?
- Runtime hook like Node's `unhandledRejection`?

---

## Patterns

### Sequential Async

```
futura functio processAll(lista<textus> urls) -> lista<textus> {
    varia results = []
    ex urls pro url {
        fixum data = cede fetchData(url)
        results.adde(data)
    }
    redde results
}
```

### Parallel Async

```
futura functio fetchAll(lista<textus> urls) -> lista<textus> {
    fixum promissa = urls.mappa((url) => fetchData(url))
    redde cede promissum.omnes(promissa)
}
```

### Timeout Pattern

```
futura functio cumTimeout(promissum<T> p, numerus ms) -> T {
    redde cede promissum.primus([
        p,
        delay(ms).tunc(() => iace "Timeout")
    ])
}
```

---

## Open Questions

1. **Top-level await**: Allow `cede` at module level?
   - JS: Yes (ES modules)
   - Proposal: Yes, for scripts

2. **Async iterators**: Support `futura ex...pro`?
   ```
   futura ex stream pro chunk {
       // process each chunk as it arrives
   }
   ```

3. **Cancellation**: Add cancellation token pattern?
   - Not native to JS Promises
   - Could wrap with AbortController

4. **Zig target**: How to map to Zig's async model?
   - Zig uses suspend/resume with frames
   - May need different abstraction

---

## Implementation Notes

### TypeScript Target

Direct mapping:
- `futura functio` → `async function`
- `cede` (in async) → `await`
- `promissum<T>` → `Promise<T>`

All instance methods map directly to Promise prototype.

### Zig Target

More complex — Zig's async is different:
- Functions return frames
- Explicit suspend points
- No built-in Promise type

Options:
1. Generate Zig async frames directly
2. Provide runtime promise library
3. Limit async to TypeScript target initially
