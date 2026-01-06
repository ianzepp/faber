---
status: completed
updated: 2026-01-06
note: Core ex preposition fully implemented. Collection DSL and return borrow source remain unimplemented.
implemented:
  - Import (ex...importa)
  - Iteration (ex...pro/fit/fiet)
  - Destructuring (ex object fixum fields)
not_implemented:
  - Collection DSL transforms (ex items filtra ubi...)
  - Return borrow source (-> de Type ex param)
---

# ex — Source/Origin

**Latin:** "from, out of"

`ex` introduces a source from which something is drawn. What happens next depends on grammatical position.

---

## Import

Draw bindings **from** a module:

```fab
ex norma importa scribe, lege
ex "hono" importa Hono
```

**Target mappings:**
- TypeScript: `import { scribe, lege } from 'norma'`
- Python: `from norma import scribe, lege`
- Zig: Handled via build system and `@import`

---

## Iteration

Draw each element **from** the collection:

```fab
ex items pro item {
    scribe item
}
```

The binding keyword encodes sync/async:

| Keyword | Meaning             | Compiles to      |
| ------- | ------------------- | ---------------- |
| `pro`   | for (preposition)   | `for...of`       |
| `fit`   | becomes (sync verb) | `for...of`       |
| `fiet`  | will become (async) | `for await...of` |

**Examples:**

```fab
# Sync (equivalent)
ex items pro item { scribe item }
ex items fit item { scribe item }

# Async
ex stream fiet chunk { scribe chunk }
```

---

## Destructuring

Extract fields **from** an object/expression:

```fab
ex response fixum status, data
ex response fixum status ut s, data ut d   # with aliases
ex fetchData() figendum result             # await + destructure
```

Uses brace-less syntax matching imports.

---

## Collection DSL (Not Implemented)

**Status:** Design only. Use method chaining (`.filtrata()`, `.ordinata()`) instead.

**Planned syntax:**

```fab
ex items filtra ubi active pro item {
    scribe item
}

ex items filtra ubi active, ordina per nomen pro item {
    scribe item
}
```

Draw elements **from** the collection, after applying transforms. The `ex ... pro` frame stays constant; the middle is an optional pipeline.

**Collection expressions:**

```fab
fixum active = ex users filtra ubi active
fixum sorted = ex users filtra ubi active, ordina per nomen
fixum total = ex prices summa
```

Same as iteration transforms, but assigned instead of iterated.

---

## Return Borrow Source (Not Implemented)

**Status:** Design only. Systems targets (Zig, Rust) only.

When a function returns a borrowed value (`de` return type), and multiple parameters are borrowed, `ex` specifies which parameter(s) the return borrows from:

```fab
# Single source - return borrows from 'a'
functio first(de textus a, de textus b) -> de textus ex a {
    redde a
}

# Multiple sources - return could borrow from either
functio pick(de textus a, de textus b, bivalens flag) -> de textus ex a, b {
    si flag { redde a } secus { redde b }
}
```

**When `ex` is required:**
- Return type has `de` (borrowed)
- Multiple parameters have `de`

**When `ex` is optional:**
- Only one `de` parameter exists - source is unambiguous

**Target mappings:**

```rust
// ex a, b -> both share lifetime 'a
fn pick<'a>(a: &'a str, b: &'a str, flag: bool) -> &'a str {
    if flag { a } else { b }
}
```

For Zig/C++, `ex` serves as documentation - these languages lack explicit lifetime syntax but the intent is communicated.

---

## Summary

| Pattern                            | Meaning                         | Status   |
| ---------------------------------- | ------------------------------- | -------- |
| `ex module importa ...`            | Import from module              | Done     |
| `ex source pro var { }`            | Iterate from source             | Done     |
| `ex source transforms pro var { }` | Iterate from transformed source | Not done |
| `ex source transforms` (assigned)  | Collection expression           | Not done |
| `ex source fixum name, email`      | Destructure from source         | Done     |
| `ex source fixum name ut alias`    | Destructure with alias          | Done     |
| `-> de Type ex param`              | Return borrows from param       | Not done |
| `-> de Type ex a, b`               | Return borrows from a and/or b  | Not done |
