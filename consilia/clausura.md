# Clausura (Closures/Lambdas) Design

Latin: *clausura* (from *claudere*, to close) — a closure, enclosure.

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `pro x redde expr` expression | Done | Single-expression lambda |
| `pro x, y redde expr` multi-param | Done | Multiple parameters |
| `pro redde expr` zero-param | Done | No parameters |
| `pro x { body }` block | Done | Multi-statement body |
| `pro { body }` zero-param block | Done | No parameters, block body |
| Nested lambdas | Done | `pro x redde pro y redde x + y` |
| Captures | Done | Implicit capture of outer scope |
| `(x) => expr` JS-style | Done | Alternative syntax |
| Async lambdas | Done | Use block form with `cede` |
| Generator lambdas | **Not done** | Use named functions |
| Typed parameters | **Not done** | Future feature |

---

## Syntax

### Expression Lambda

```
pro <params> redde <expr>
```

Single expression, implicit return. The `redde` keyword makes the return explicit.

```
pro x redde x * 2           // (x) => x * 2
pro a, b redde a + b        // (a, b) => a + b
pro redde 42                // () => 42
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
pro redde 42

// Block
pro {
    scribe "Hello!"
}
```

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
fixum evens = numbers.filtrata(pro n redde n % 2 == 0)

// Map
fixum doubled = numbers.mappata(pro n redde n * 2)

// Reduce
fixum sum = numbers.reducta(pro acc, n redde acc + n, 0)
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
    .then(pro data redde process(data))
    .then(pro result redde display(result))
    .catch(pro error redde logError(error))
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
fixum multiplier = pro factor redde pro x redde x * factor
fixum double = multiplier(2)
fixum triple = multiplier(3)
```

---

## Async Lambdas

For async operations, use block form with `cede` (await):

```
// Async lambda
pro url {
    fixum response = cede fetch(url)
    redde cede response.json()
}
```

The compiler infers async from the presence of `cede` in the body.

---

## Captures

Lambdas implicitly capture variables from enclosing scope:

```
fixum factor = 2
fixum doubled = numbers.mappata(pro x redde x * factor)
// 'factor' is captured from outer scope
```

### Target-Specific Capture Handling

| Target | Capture Strategy |
|--------|-----------------|
| TypeScript | Automatic (JS closures) |
| Python | Automatic (Python closures) |
| Zig | Context struct with captured values |
| Rust | Automatic with ownership inference |
| C++ | Capture clause `[=]` or `[&]` |

---

## Alternative Syntax

JS-style arrow functions are also supported:

```
// These are equivalent:
pro x redde x * 2
(x) => x * 2

// Block form:
pro x { redde x * 2 }
(x) => { redde x * 2 }
```

The `pro...redde` form is preferred for consistency with Latin style.

---

## Codegen Examples

### TypeScript

```
// Faber
pro x redde x * 2

// TypeScript
(x) => x * 2
```

### Python

```
// Faber
pro x redde x * 2

// Python
lambda x: x * 2
```

### Zig

```
// Faber
pro x redde x * 2

// Zig (no closures, uses struct)
struct { fn call(x: i64) i64 { return x * 2; } }.call
```

### Rust

```
// Faber
pro x redde x * 2

// Rust
|x| x * 2
```

### C++

```
// Faber
pro x redde x * 2

// C++
[](auto x) { return x * 2; }
```

---

## Open Questions

1. **Typed parameters** — Should lambdas support type annotations?
   ```
   pro x: numerus redde x * 2
   ```

2. **Generator lambdas** — Worth supporting inline generators?
   ```
   pro x fiunt 1..x   // hypothetical
   ```

3. **Capture annotations** — Explicit control over capture mode?
   ```
   pro [in factor] x redde x * factor   // hypothetical
   ```

---

## See Also

- `exempla/regimen/clausura.fab` — Example code
- `consilia/iteration.md` — Related `pro` keyword usage
- `consilia/collections.md` — Lambda usage with collections
