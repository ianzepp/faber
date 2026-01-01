---
status: implemented
targets: [ts, py, zig, cpp]
updated: 2024-12
---

# Clausura (Closures/Lambdas) Design

Latin: _clausura_ (from _claudere_, to close) — a closure, enclosure.

## Implementation Status

| Feature                           | TypeScript |  Python  |     Zig      |     Rust     |              C++23              | Notes                     |
| --------------------------------- | :--------: | :------: | :----------: | :----------: | :-----------------------------: | ------------------------- |
| `pro x redde expr` expression     |  [x] Done  | [x] Done | [~] Partial  | [ ] Not Done |            [x] Done             | Single-expression lambda  |
| `pro x: expr` shorthand           |  [x] Done  | [x] Done | [~] Partial  | [ ] Not Done |            [x] Done             | `:` as alias for `redde`  |
| `pro x, y redde expr` multi-param |  [x] Done  | [x] Done | [~] Partial  | [ ] Not Done |            [x] Done             | Multiple parameters       |
| `pro redde expr` zero-param       |  [x] Done  | [x] Done | [~] Partial  | [ ] Not Done |            [x] Done             | No parameters             |
| `pro x { body }` block            |  [x] Done  | [x] Done | [~] Partial  | [ ] Not Done |            [x] Done             | Multi-statement body      |
| `pro { body }` zero-param block   |  [x] Done  | [x] Done | [~] Partial  | [ ] Not Done |            [x] Done             | No parameters, block body |
| Nested lambdas                    |  [x] Done  | [x] Done | [ ] Not Done | [ ] Not Done |            [x] Done             | `pro x: pro y: x + y`     |
| Captures                          |  [x] Done  | [x] Done | [ ] Not Done |   [x] Done   | Implicit capture of outer scope |

| `fiet x: expr` async lambda | [x] Done | [x] Done | [ ] Not Done | [ ] Not Done | [x] Done | Async closure with verb form |
| `fiet x { body }` async block | [x] Done | [x] Done | [ ] Not Done | [ ] Not Done | [x] Done | Async block closure |
| Generator lambdas | [-] | [-] | [-] | [-] | [-] | Use named functions instead |
| Typed parameters | [ ] Not Done | [ ] Not Done | [ ] Not Done | [ ] Not Done | [ ] Not Done | Future: `pro numerus x: x * 2` |
| Return type annotation | [x] Done | [-] N/A | [x] Done | [ ] Not Done | [x] Done | `pro x -> numerus: x * 2` |

---

## Syntax

### Expression Lambda

```
pro <params>: <expr>
```

Single expression, implicit return. The colon separates parameters from the body.

```
pro x: x * 2                // (x) => x * 2
pro a, b: a + b             // (a, b) => a + b
pro: 42                     // () => 42
```

### Block Lambda

```
pro <params> { <body> }
```

Multi-statement body with explicit `redde` for returns.

```
pro user {
    si user.aetas < 18 { redde falsum }
    redde user.activus
}
```

### Zero-Parameter Forms

```
// Expression
pro: 42

// Block
pro {
    scribe "Hello!"
}
```

### Return Type Annotation

For targets that require explicit return types (e.g., Zig), use `-> Type` after params:

```
pro <params> -> <type>: <expr>
pro <params> -> <type> { <body> }
```

Examples:

```
pro x -> numerus: x * 2           // expression with return type
pro x, y -> numerus: x + y        // multi-param
pro -> textus: "hello"            // zero-param
pro x -> textus { redde "hello" } // block body
```

Mirrors function syntax: `functio double(x) -> numerus { ... }`

---

## Etymology and Design

### Why `pro...redde`?

**`pro`** (for) — Aligns with iteration syntax. Both use the same keyword:

- `ex items pro x { }` — iteration binding
- `pro x { }` — lambda binding

**`redde`** (return, give back) — Explicit about what the lambda produces:

- Reads naturally: "for x, return x times 2"
- Consistent with function bodies using `redde`
- No new keyword needed

### The `:` Shorthand

The colon acts as a "defined as" operator, consistent with object literal syntax:

| Symbol | Meaning         | Examples                    |
| ------ | --------------- | --------------------------- |
| `:`    | "is defined as" | `{ x: 42 }`, `pro x: x * 2` |
| `=`    | "assign to"     | `sit x = 5`, `x = 10`       |

Both forms are valid; `:` is preferred for concise one-liners, `redde` for emphasis or teaching.

### Previous Syntax (Deprecated)

The language previously used `fit` (becomes) for lambdas:

```
// Old syntax (no longer valid)
pro x fit x * 2
```

This was changed to `redde` for:

1. Consistency with function return syntax
2. Clearer semantics (returning vs. becoming)
3. Simpler mental model

---

## Usage Patterns

### With Collection Methods

Expression lambdas are clean for simple transforms:

```
fixum numbers = [1, 2, 3, 4, 5]

// Filter
fixum evens = numbers.filtrata(pro n: n % 2 == 0)

// Map
fixum doubled = numbers.mappata(pro n: n * 2)

// Reduce
fixum sum = numbers.reducta(pro acc, n: acc + n, 0)
```

Block lambdas for complex logic:

```
fixum processed = numbers.filtrata(pro n {
    si n < 0 { redde falsum }
    si n > 100 { redde falsum }
    redde n % 2 == 0
})
```

### Event Handlers

Zero-param block form is cleanest for callbacks:

```
button.onClick(pro { scribe "clicked" })
conn.onClose(pro { cleanup() })
conn.onError(pro err { logError(err) })
```

### Promise Chains

Expression form for one-liners:

```
fetchData()
    .then(pro data: process(data))
    .then(pro result: display(result))
    .catch(pro error: logError(error))
```

Block form for complex handling:

```
fetchData()
    .then(pro data {
        si data.error {
            iace data.error
        }
        redde process(data)
    })
```

### Higher-Order Functions

Currying with nested lambdas:

```
fixum multiplier = pro factor: pro x: x * factor
fixum double = multiplier(2)
fixum triple = multiplier(3)
```

---

## Async Lambdas

Use the `fiet` verb form for async closures. This mirrors the function declaration pattern where `fiet` indicates "will become" (future/async):

```
fiet x: expr              // async (x) => expr
fiet x { body }           // async (x) => { body }
fiet: expr                // async () => expr
fiet { body }             // async () => { body }
```

### Examples

```fab
# Async HTTP handler
app.post("/users", fiet c {
    fixum body = cede c.req.json()
    redde c.json(body)
})

# Async map operation
fixum results = urls.mappata(fiet url: cede fetch(url))

# Async with multiple awaits
fetchData().then(fiet data {
    fixum processed = cede transform(data)
    redde cede save(processed)
})
```

### Design Rationale

The `fiet` keyword is consistent with the verb conjugation system used throughout Faber:

| Context          | Sync                | Async                |
| ---------------- | ------------------- | -------------------- |
| Function return  | `functio f() fit T` | `functio f() fiet T` |
| For-loop binding | `ex items fit x`    | `ex items fiet x`    |
| Lambda/closure   | `fit x: expr`       | `fiet x: expr`       |

The `pro` keyword remains as the casual/default form for sync lambdas:

- `pro x: expr` ≡ `fit x: expr` (both sync)
- `fiet x: expr` (async, no `pro` equivalent)

### Why No `futura pro`?

The `futura` prefix is reserved for function declarations. Lambdas use verb conjugation instead:

```fab
# Functions use prefix
futura functio fetch() -> textus { ... }

# Lambdas use verb form
fiet url: cede fetch(url)
```

This keeps lambda syntax concise while maintaining semantic clarity.

---

## Captures

Lambdas implicitly capture variables from enclosing scope:

```
fixum factor = 2
fixum doubled = numbers.mappata(pro x: x * factor)
// 'factor' is captured from outer scope
```

### Target-Specific Capture Handling

| Target     | Capture Strategy                    |
| ---------- | ----------------------------------- |
| TypeScript | Automatic (JS closures)             |
| Python     | Automatic (Python closures)         |
| Zig        | Context struct with captured values |
| Rust       | Automatic with ownership inference  |
| C++        | Capture clause `[=]` or `[&]`       |

---

## Alternative Syntax

The `pro...` forms are preferred for consistency with Latin style.

```
// These are all equivalent:
pro x: x * 2        // shorthand (preferred)
pro x redde x * 2   // explicit

// Block form:
pro x { redde x * 2 }
```

---

## Codegen Examples

### TypeScript

```
// Faber
pro x: x * 2

// TypeScript
(x) => x * 2
```

### Python

```
// Faber
pro x: x * 2

// Python
lambda x: x * 2
```

### Zig

Zig requires explicit return types. Without a return type annotation, a compile error is generated:

```
// Faber (without return type - compile error)
pro x: x * 2

// Zig
@compileError("Lambda requires return type annotation for Zig target")
```

With return type annotation:

```
// Faber (with return type)
pro x -> numerus: x * 2

// Zig
struct { fn call(x: anytype) i64 { return x * 2; } }.call
```

### Rust

```
// Faber
pro x: x * 2

// Rust
|x| x * 2
```

### C++

```
// Faber
pro x: x * 2

// C++
[](auto x) { return x * 2; }
```

---

## Open Questions

1. **Typed parameters** — Should lambdas support type annotations?

    ```
    pro numerus x: x * 2    // type precedes identifier
    ```

2. **Capture annotations** — Explicit control over capture mode?
    ```
    pro [in factor] x: x * factor   // hypothetical
    ```

## Closed Decisions

1. **Generator lambdas** — Not supported. Use named `cursor functio` instead. Inline generators are rare and add complexity without significant benefit.

2. **Async lambdas** — Use `fiet` verb form. The `futura pro` syntax was rejected as inconsistent with function prefix usage.

---

## See Also

- `exempla/regimen/clausura.fab` — Example code
- `consilia/iteration.md` — Related `pro` keyword usage
- `consilia/collections.md` — Lambda usage with collections
