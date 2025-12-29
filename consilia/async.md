---
status: implemented
targets: [ts, py]
updated: 2024-12
---

# Async Design

## Verb Conjugation for Return Types

Faber uses Latin verb conjugation of _fieri_ (to become) to encode async/iterator semantics in the return type:

|                    | Sync           | Async          |
| ------------------ | -------------- | -------------- |
| **Single return**  | `fit textus`   | `fiet textus`  |
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

### Syntax Patterns

Two separate syntax patterns for async functions:

**Arrow syntax** (direct returns, traditional semantics):
| Syntax | Semantics |
| -------------------------------- | ----------------------------------- |
| `functio f() -> T` | sync, single (default) |
| `futura functio f() -> T` | async, single (Promise<T>) |
| `cursor functio f() -> T` | sync, generator (yield) |
| `futura cursor functio f() -> T` | async, generator (AsyncGenerator) |

**Verb syntax** (stream protocol via Responsum):
| Syntax | Semantics |
| -------------------------------- | ----------------------------------- |
| `functio f() fit T` | sync, single (Responsum protocol) |
| `functio f() fiet T` | async, single (Responsum protocol) |
| `functio f() fiunt T` | sync, generator (Responsum protocol) |
| `functio f() fient T` | async, generator (Responsum protocol)|

**Validation**: Prefixes (`futura`/`cursor`) cannot be combined with verbs:

- `futura functio f() fit T` - ERROR: prefixes only work with arrow syntax
- `cursor functio f() fiunt T` - ERROR: prefixes only work with arrow syntax
- `functio f() fiet T` - OK: verb carries semantic information
- `futura functio f() -> T` - OK: arrow uses prefix semantics

---

## Overview

Faber's async model maps to native async/await patterns:

| Faber            | JavaScript       | Zig                  |
| ---------------- | ---------------- | -------------------- |
| `futura functio` | `async function` | `fn` returning frame |
| `cede`           | `await`          | `await` / suspend    |
| `promissum<T>`   | `Promise<T>`     | Future-like pattern  |

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

### figendum / variandum (async bindings)

Alternative syntax using gerundive forms ("that which will be...") with implicit await:

```
figendum data = fetchData("https://api.example.com")  // immutable, implicit await
variandum result = fetchInitial()                      // mutable, implicit await
```

These are equivalent to `fixum x = cede y()` and `varia x = cede y()` respectively, but more natural Latin. The gerundive signals futurity — the value will be fixed/varied once the operation completes.

**Sync/async agnostic:** `figendum` works with both sync and async values. If the RHS is a promise, it awaits; if plain, it passes through. This makes code future-proof when APIs evolve from sync to async.

See `vincula.md` for full details.

---

## promissum<T>

**Etymology:** "promise, guarantee" — something pledged for the future.

### Instance Methods

| Faber                                  | JS Equivalent | Description   |
| -------------------------------------- | ------------- | ------------- |
| `tunc((T) -> U) -> promissum<U>`       | `then`        | Chain success |
| `cape((erratum) -> U) -> promissum<U>` | `catch`       | Handle error  |
| `demum(() -> vacuum) -> promissum<T>`  | `finally`     | Always run    |

**Naming rationale:**

- `tunc` = "then" (Latin temporal adverb)
- `cape` = "catch, seize" (imperative of capere)
- `demum` = "finally, at last" (Latin adverb)

### Static Constructors

| Faber                     | JS Equivalent     | Description       |
| ------------------------- | ----------------- | ----------------- |
| `promissum.da(T)`         | `Promise.resolve` | Immediate success |
| `promissum.nega(erratum)` | `Promise.reject`  | Immediate failure |

**Naming rationale:**

- `da` = "give" (imperative of dare) — here's your value
- `nega` = "deny" (imperative of negare) — refused with error

### Static Combinators

| Faber                                | JS Equivalent        | Description      |
| ------------------------------------ | -------------------- | ---------------- |
| `promissum.omnes(promissum<T>[])`    | `Promise.all`        | All must succeed |
| `promissum.primus(promissum<T>[])`   | `Promise.race`       | First to settle  |
| `promissum.quilibet(promissum<T>[])` | `Promise.any`        | First to succeed |
| `promissum.finita(...)`              | `Promise.allSettled` | All settled      |

**Naming rationale:**

- `omnes` = "all" (must succeed)
- `primus` = "first" (race — first to settle wins)
- `quilibet` = "any, whichever" (first to succeed)
- `finita` = "finished" (all done, regardless of success/failure)

---

## Error Handling

### With tempta/cape (TS/Python only)

> **Note:** `tempta`/`demum` implies try/catch/finally semantics. Since `demum` (finally) has no direct equivalent in Rust/Zig (they use RAII/defer), use `fac`/`cape` for systems targets instead. See `codegen/rust.md` and `codegen/zig.md`.

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

```
futura functio fetchAll(textus[] urls) -> textus[] {
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

---

## Nexum (Reactive Fields)

**Status:** Implemented for TypeScript and Python

**Etymology:** _nexum_ = "bound, tied" (past participle of _necto_, "to bind")

### Reactive Fields in Genus

`nexum` marks a field as reactive — changes trigger invalidation and re-render. Unlike regular fields, `nexum` fields emit getter/setter pairs with an invalidation hook.

### Syntax

```
genus counter {
    nexum numerus count: 0
    textus label: "Counter"
}
```

### Codegen Transformation

**TypeScript:**

```typescript
class counter {
    #count = 0; // Private backing field
    label = 'Counter'; // Regular field

    get count(): number {
        return this.#count;
    }

    set count(v: number) {
        this.#count = v;
        this.__invalidate?.('count'); // Invalidation hook
    }
}
```

**Python:**

```python
class counter:
    def __init__(self, overrides={}):
        self._count = 0
        self.label = "Counter"

    @property
    def count(self):
        return self._count

    @count.setter
    def count(self, value):
        self._count = value
        self.__invalidate__('count')
```

### Implementation Details

| Target     | Status       | Notes                                   |
| ---------- | ------------ | --------------------------------------- |
| TypeScript | [x] Done     | Uses private fields (#) + getter/setter |
| Python     | [x] Done     | Uses @property decorator                |
| Zig        | [ ] Not Done | Reactivity needs runtime decision       |
| Rust       | [ ] Not Done | Reactivity needs runtime decision       |
| C++23      | [ ] Not Done | Reactivity needs runtime decision       |

### Use Cases

Reactive fields are designed for UI frameworks where state changes should trigger re-renders:

```faber
genus TodoItem {
    nexum textus title
    nexum bivalens completed: falsum

    functio toggle() {
        ego.completed = non ego.completed
    }
}
```

### Interaction with `pingo` (render)

When a `genus` has a `pingo()` method and reactive fields, the framework calls `__invalidate(field)` on changes, which may trigger re-render by calling `pingo()`.

**Note:** This is framework-integration specific. The exact invalidation → re-render mechanism depends on the target UI framework (Svelte, Solid, Vue, etc.).

### Differences from `figendum`/`variandum`

| Feature | `figendum`/`variandum` | `nexum` (in genus)        |
| ------- | ---------------------- | ------------------------- |
| Scope   | Top-level bindings     | Class/struct fields       |
| Async   | Blocks until resolves  | Non-blocking reactive     |
| Purpose | Async data loading     | Reactive state management |

`figendum` and `variandum` are for awaiting async results. `nexum` is for reactive state in UI components.

**Important:** Top-level `nexum` bindings (non-blocking async variables) remain conceptual and not implemented. `nexum` is only supported as a field modifier within `genus`.
