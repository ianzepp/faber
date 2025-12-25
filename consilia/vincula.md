# Vincula - Variable Bindings

## Overview

Variable declaration and binding keywords for sync and async contexts.

## Etymology

- `vincula` - "bonds, bindings" (plural of vinculum)
- `fixum` - "fixed, set" (past participle of figere) — immutable binding
- `varia` - "varied, changeable" — mutable binding
- `figendum` - "that which will be fixed" (gerundive) — async immutable
- `variandum` - "that which will be varied" (gerundive) — async mutable

---

## Binding Keywords

|           | Sync    | Async       |
| --------- | ------- | ----------- |
| Immutable | `fixum` | `figendum`  |
| Mutable   | `varia` | `variandum` |

### Sync Bindings

```
// Immutable — cannot be reassigned
fixum name = "Marcus"
fixum count = 42
fixum users = lege "users.json" ut json

// Mutable — can be reassigned
varia counter = 0
counter = counter + 1

varia status = "pending"
status = "complete"
```

### Async Bindings

Async bindings await the RHS automatically — no explicit `cede` needed:

```
// Immutable async — await then fix
figendum response = pete("https://api.example.com/users")
figendum data = response.json()
figendum config = lege "config.json" ut json

// Mutable async — await then allow mutation
variandum result = fetchInitial()
result = transform(result)
result = cede fetchMore()  // can still use cede for subsequent awaits
```

The gerundive form ("that which will be...") signals futurity — the value isn't available now but will be fixed/set once the operation completes.

---

## Sync/Async Agnostic

A key property: `figendum` and `variandum` work with both sync and async values:

```
figendum a = syncFunction()     // returns immediately
figendum b = asyncFunction()    // awaits the promise
figendum c = maybeAsync()       // handles either case
```

If the RHS is a promise, it awaits. If it's a plain value, it passes through immediately (like JavaScript's `await` on non-promises).

**Benefits:**

1. **Future-proof** — if a function becomes async later, callers don't need to change
2. **Simpler mental model** — "value will be ready on next line" regardless of sync/async
3. **Unified API** — library authors can change sync→async without breaking callers
4. **No tracking** — don't need to know or care if something is async

This makes `figendum` a safe default for any function call binding:

```
figendum config = loadConfig()    // don't care if sync or async
figendum user = getUser(id)       // don't care if sync or async
figendum data = parse(input)      // don't care if sync or async
```

The compiler handles the difference — sync values pass through, async values are awaited.

---

## Grammatical Parallel

The async keywords mirror the `fit`/`fiet` return pattern:

| Concept   | Present/Past     | Future                       |
| --------- | ---------------- | ---------------------------- |
| Return    | `fit` (becomes)  | `fiet` (will become)         |
| Immutable | `fixum` (fixed)  | `figendum` (will be fixed)   |
| Mutable   | `varia` (varied) | `variandum` (will be varied) |

Both use the Latin future/gerundive to indicate "not yet, but will be."

---

## Type Annotations

Type comes first, then binding keyword, then name:

```
// Sync
fixum textus name = "Marcus"
varia numerus count = 0

// Async
figendum user[] users = fetchUsers()
variandum response data = pete(url)
```

---

## Destructuring

Bindings work with destructuring via `ex`:

```
// Sync destructuring
ex config fixum { host, port }
ex point varia { x, y }

// Async destructuring — await then destructure
ex cede fetchConfig() figendum { host, port }
ex cede response.json() figendum { users, total }
```

Note: Async destructuring still needs `cede` on the source expression, since `figendum` applies to the bindings, not the source.

---

## Shadowing

Inner scopes can shadow outer bindings:

```
fixum x = 1
si verum {
    fixum x = 2      // shadows outer x
    scribe x         // 2
}
scribe x             // 1
```

Shadowing can change mutability:

```
varia x = 1
fixum x = x + 1      // shadow with immutable
// x is now immutable in this scope
```

---

## Target Mappings

### TypeScript

```typescript
// fixum
const name = 'Marcus';

// varia
let counter = 0;

// figendum
const response = await fetch(url);

// variandum
let result = await fetchInitial();
```

### Python

```python
# fixum (Python has no const, but convention)
NAME = "Marcus"  # or just: name = "Marcus"

# varia
counter = 0

# figendum
response = await fetch(url)

# variandum
result = await fetch_initial()
```

### Rust

```rust
// fixum
let name = "Marcus";

// varia
let mut counter = 0;

// figendum
let response = fetch(url).await;

// variandum
let mut result = fetch_initial().await;
```

### Zig

```zig
// fixum
const name = "Marcus";

// varia
var counter: i32 = 0;

// figendum / variandum
// Zig async is frame-based, different model
const response = try await async fetch(url);
```

---

## Design Decisions

### Why gerundives for async?

The gerundive is the Latin future passive participle — "that which must/will be done." It perfectly captures the semantics of an async binding: the value _will be_ fixed once the operation completes.

Alternatives considered:

- `cede fixum` — requires two keywords, less elegant
- `fiet` — already a verb form, grammatically inconsistent with `fixum`/`varia`
- New unrelated keyword — loses the Latin grammatical connection

### Why implicit await?

`figendum` and `variandum` imply await. The futurity is in the keyword itself, so explicit `cede` is redundant:

```
// These are equivalent:
figendum data = asyncValue
fixum data = cede asyncValue
```

The gerundive form makes the async nature explicit in the binding declaration.

### Why allow both forms?

Some developers prefer explicit `cede`:

```
fixum data = cede asyncValue
```

This remains valid. `figendum`/`variandum` are syntactic sugar for common async patterns.

### Why sync/async agnostic?

`figendum` handles both sync and async RHS values — if it's a promise, await; if not, pass through. This means:

- Code is future-proof (sync→async changes don't break callers)
- No need to track which functions are async
- `figendum` becomes a safe default for any function call binding
- Library APIs can evolve without breaking changes

This mirrors JavaScript's `await` behavior on non-promises, but makes the intent explicit in the binding declaration.

---

## Implementation Status

| Feature             | TypeScript |  Python  |     Zig      |     Rust     |    C++23     | Notes                           |
| ------------------- | :--------: | :------: | :----------: | :----------: | :----------: | ------------------------------- |
| `fixum`             |  [x] Done  | [x] Done |   [x] Done   |   [x] Done   |   [x] Done   | Immutable binding               |
| `varia`             |  [x] Done  | [x] Done |   [x] Done   |   [x] Done   |   [x] Done   | Mutable binding                 |
| `figendum`          |  [x] Done  | [x] Done | [ ] Not Done | [ ] Not Done | [ ] Not Done | Async immutable - impl. `await` |
| `variandum`         |  [x] Done  | [x] Done | [ ] Not Done | [ ] Not Done | [ ] Not Done | Async mutable - impl. `await`   |
| Type annotations    |  [x] Done  | [x] Done |   [x] Done   |   [x] Done   |   [x] Done   | On all binding forms            |
| Destructuring       |  [x] Done  | [x] Done |   [x] Done   |   [x] Done   | [ ] Not Done | Via `ex` - object pattern only  |
| Async destructuring |  [x] Done  | [x] Done | [ ] Not Done | [ ] Not Done | [ ] Not Done | `ex cede ... figendum`          |
