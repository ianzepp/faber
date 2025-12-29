# Flumina: Streams-First Architecture

**Status:** Exploratory
**Inspiration:** Monk OS syscall architecture

## Vision

Make streams the fundamental execution model in Faber. Every function is a generator; single-value returns are streams that yield once. This unifies:

- Single-value functions (`fit`)
- Multi-value generators (`fiunt`)
- Async variants (`fiet`, `fient`)
- Error handling
- Backpressure

The goal: streams-first should feel *natural*, not like extra work.

## The Response Protocol

Inspired by Monk OS, all yields produce a `Responsum` with an `op` field signaling what kind of response it is and whether the stream continues.

### Response Operations

| Op | Latin | Terminal | Purpose |
|----|-------|----------|---------|
| `ok` | `bene` | yes | Single value, stream ends |
| `error` | `error` | yes | Failure, stream ends |
| `done` | `factum` | yes | Multi-value complete, stream ends |
| `redirect` | `refer` | yes | Go elsewhere, stream ends |
| `item` | `res` | no | One item of many, continue |
| `data` | `bytes` | no | Binary chunk, continue |
| `event` | `eventum` | no | Notification, continue |
| `progress` | `progressus` | no | Status update, continue |

**Terminal ops** (`bene`, `error`, `factum`, `refer`) signal "no more responses."
**Non-terminal ops** (`res`, `bytes`, `eventum`, `progressus`) signal "more may follow."

### Type Definition

```fab
genus Responsum {
    op: textus
    data: quidlibet?
    bytes: Bytes?
}

// Or as a discriminated union:
unio Responsum {
    Bene { data: quidlibet? }
    Error { code: textus, message: textus }
    Factum
    Refer { location: textus }
    Res { data: quidlibet }
    Bytes { bytes: Bytes }
    Eventum { typus: textus, data: quidlibet? }
    Progressus { percent: fractus?, current: numerus?, total: numerus? }
}
```

## Mapping to Faber Syntax

### Current Model

| Verb | Async | Generator | Meaning |
|------|-------|-----------|---------|
| `fit` / `->` | no | no | Returns single value |
| `fiet` | yes | no | Returns promise of single value |
| `fiunt` | no | yes | Yields multiple values |
| `fient` | yes | yes | Async generator |

### Proposed Model

All functions are generators internally. The verb indicates *expected cardinality*:

| Verb | Cardinality | Internal Behavior |
|------|-------------|-------------------|
| `fit` | exactly one | yields `bene(value)` |
| `fiet` | exactly one (async) | awaits, then yields `bene(value)` |
| `fiunt` | zero or more | yields `res(value)` per item, then `factum()` |
| `fient` | zero or more (async) | async yields `res(value)` per item |

### Syntax Mapping

```fab
// Single-value function
functio getId() fit textus {
    redde "abc"
}
// Compiles to: yield bene("abc")

// Multi-value generator
functio getItems() fiunt textus {
    cede "a"
    cede "b"
}
// Compiles to: yield res("a"); yield res("b"); yield factum()

// Error case
functio mightFail() fit textus {
    iace Defectum("oops")
}
// Compiles to: yield error("EFAIL", "oops")
```

### Keywords

| Faber | Purpose | Response Op |
|-------|---------|-------------|
| `redde x` | Return single value | `bene(x)` |
| `cede x` | Yield one of many | `res(x)` |
| `iace err` | Throw/raise error | `error(code, msg)` |
| (implicit) | End of `fiunt` function | `factum()` |

## Consumption Patterns

The consumer iterates until hitting a terminal op:

```fab
// Uniform iteration (works for single or multi-value)
ex getData() pro item {
    // runs once for fit, many times for fiunt
}

// Explicit single-value extraction
fixum x = expecta getData()  // errors if multiple items

// Collect to array
fixum items = collige(getData())  // collects all res items
```

### Consumer Helpers

```fab
// Wait for single bene response
functio expecta<T>(flumen: Flumen<T>) fiet T

// Collect all res responses into array
functio collige<T>(flumen: Flumen<T>) fiet Lista<T>

// Iterate with full Response access
functio inspice<T>(flumen: Flumen<T>) fiunt Responsum
```

## The Dispatcher Pattern

With streams-first, a dispatcher becomes natural:

```fab
functio dispatch(nuntius: Nuntius) fiunt Responsum {
    elige nuntius.op {
        si "file:open" {
            cede* fileOpen(nuntius.args)
        }
        si "file:read" {
            cede* fileRead(nuntius.args)
        }
        si "proc:getpid" {
            cede* procGetpid(nuntius.args)
        }
        aliter {
            iace Defectum("ENOSYS", "Unknown operation")
        }
    }
}
```

The `cede*` operator (like `yield*`) forwards an entire stream transparently.

## Target Compilation

### TypeScript

```typescript
// fit function
async function* getId(): AsyncGenerator<Response> {
    yield respond.ok("abc");
}

// fiunt function
async function* getItems(): AsyncGenerator<Response> {
    yield respond.item("a");
    yield respond.item("b");
    yield respond.done();
}
```

### Python

```python
# fit function
async def getId():
    yield respond_ok("abc")

# fiunt function
async def getItems():
    yield respond_item("a")
    yield respond_item("b")
    yield respond_done()
```

### Zig / Rust / C++

These languages don't have cheap generators. Options:

1. **Callback-based** — Transform to callback style internally
2. **State machine** — Compile generator to explicit state machine
3. **Coroutine library** — Use language-specific coroutine support
4. **Semantic preservation** — `fit` compiles to regular return (optimization)

The `fit` optimization is key: if a function only yields once (`bene`), emit a regular function. Only `fiunt`/`fient` need generator machinery.

## Open Questions

### 1. Type System

How does the type system represent streams?

```fab
// Option A: Verb implies type
functio foo() fit textus      // Flumen<textus> that yields once
functio bar() fiunt textus    // Flumen<textus> that yields many

// Option B: Explicit stream types
functio foo() fit Singulum<textus>
functio bar() fiunt Flumen<textus>
```

### 2. Void Functions

What does a void function yield?

```fab
functio doSomething() fit vacuum {
    // do work
}
// yields bene(nihil)? or factum()?
```

### 3. Backpressure

Should backpressure be built into the language runtime?

- Monk OS: ping/cancel protocol, high/low water marks
- Faber: TBD — may depend on target

### 4. Interop

How do Faber streams interop with target language iterables?

```fab
// Consuming external iterator
ex externIter() pro item { }

// Exposing Faber stream to external code
exporta functio myStream() fiunt textus { }
```

### 5. Error Codes

Should Faber have built-in error codes like POSIX?

```fab
genus Defectum {
    code: textus    // "EINVAL", "ENOENT", etc.
    message: textus
}
```

Or leave error structure to the user?

## Relationship to Other Features

### Async (`fiet`, `fient`)

Async is orthogonal to streaming. An async generator can await between yields:

```fab
functio fetchAll(urls: Lista<textus>) fient textus {
    ex urls pro url {
        fixum response = expecta fetch(url)  // await
        cede response.corpus                  // yield
    }
}
```

### Resource Management (`cura...fit`)

Resource scopes interact with streams — the resource must stay alive while yielding:

```fab
functio readLines(via: textus) fiunt textus {
    cura aperi(via) fit file {
        dum non file.finis {
            cede file.legeLine()
        }
    }
}
// file closed after last yield
```

### Pattern Matching (`discerne`)

Pattern matching on `Responsum` variants:

```fab
ex stream pro resp {
    discerne resp {
        si Bene pro data { redde data }
        si Error pro e { iace e }
        si Res pro item { processa(item) }
        si Factum { exi }
    }
}
```

## Next Steps

1. **Prototype** — Implement `Responsum` type and `responde` helper in norma/
2. **Syntax** — Decide on `cede*` for stream forwarding
3. **Codegen** — TypeScript target first (native generators)
4. **Optimization** — `fit` functions compile to regular returns when possible
5. **Backpressure** — Design runtime protocol (or defer to target)

## References

- Monk OS AGENTS.md — syscall architecture, Response protocol
- Monk OS `src/message.ts` — Response types, `respond` helpers
- Monk OS `src/dispatch/` — dispatcher pattern, backpressure
