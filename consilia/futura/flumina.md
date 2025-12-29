# Flumina: Streams-First Architecture

**Status:** Phase 3 Complete
**Inspiration:** Monk OS syscall architecture

## Vision

Make streams the fundamental execution model in Faber. Every function is a generator; single-value returns are streams that yield once. This unifies:

- Single-value functions (`fit`)
- Multi-value generators (`fiunt`)
- Async variants (`fiet`, `fient`)
- Error handling
- Backpressure

The goal: streams-first should feel _natural_, not like extra work.

## The Response Protocol

Inspired by Monk OS, all yields produce a `Responsum` with an `op` field signaling what kind of response it is and whether the stream continues.

### Response Operations

| Op         | Latin        | Terminal | Purpose                           |
| ---------- | ------------ | -------- | --------------------------------- |
| `ok`       | `bene`       | yes      | Single value, stream ends         |
| `error`    | `error`      | yes      | Failure, stream ends              |
| `done`     | `factum`     | yes      | Multi-value complete, stream ends |
| `redirect` | `refer`      | yes      | Go elsewhere, stream ends         |
| `item`     | `res`        | no       | One item of many, continue        |
| `data`     | `bytes`      | no       | Binary chunk, continue            |
| `event`    | `eventum`    | no       | Notification, continue            |
| `progress` | `progressus` | no       | Status update, continue           |

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

| Verb         | Async | Generator | Meaning                         |
| ------------ | ----- | --------- | ------------------------------- |
| `fit` / `->` | no    | no        | Returns single value            |
| `fiet`       | yes   | no        | Returns promise of single value |
| `fiunt`      | no    | yes       | Yields multiple values          |
| `fient`      | yes   | yes       | Async generator                 |

### Proposed Model

All functions are generators internally. The verb indicates _expected cardinality_:

| Verb    | Cardinality          | Internal Behavior                             |
| ------- | -------------------- | --------------------------------------------- |
| `fit`   | exactly one          | yields `bene(value)`                          |
| `fiet`  | exactly one (async)  | awaits, then yields `bene(value)`             |
| `fiunt` | zero or more         | yields `res(value)` per item, then `factum()` |
| `fient` | zero or more (async) | async yields `res(value)` per item            |

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

| Faber      | Purpose                 | Response Op        |
| ---------- | ----------------------- | ------------------ |
| `redde x`  | Return single value     | `bene(x)`          |
| `cede x`   | Yield one of many       | `res(x)`           |
| `iace err` | Throw/raise error       | `error(code, msg)` |
| (implicit) | End of `fiunt` function | `factum()`         |

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
    yield respond.ok('abc');
}

// fiunt function
async function* getItems(): AsyncGenerator<Response> {
    yield respond.item('a');
    yield respond.item('b');
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

## Design Decisions

### Generator-Everywhere, Not Async-Everywhere

The model is **sync generators** for `fit`/`fiunt`, **async generators** for `fiet`/`fient`. Async is orthogonal to streaming.

| Verb    | Sync/Async | Mechanism                     |
| ------- | ---------- | ----------------------------- |
| `fit`   | sync       | `function*` yields once       |
| `fiet`  | async      | `async function*` yields once |
| `fiunt` | sync       | `function*` yields many       |
| `fient` | async      | `async function*` yields many |

### The File I/O Analogy

The mental model: sync file I/O blocks while the underlying library collects chunks, then returns the whole thing. `fit` does the same — the generator is internal plumbing, not exposed API.

- **Caller sees values, not `Responsum`** — `fixum x = getId()` just works
- **Generator is implementation detail** — The runtime drains internally
- **Protocol is uniform** — Everything speaks `Responsum` under the hood

### Runtime Helper Approach

Protocol logic lives in `norma/flumina.ts`, not inlined in each function. Reasons:

1. **Single source of truth** — Protocol changes update one place
2. **Backpressure ready** — Future enhancements don't require re-codegen
3. **Clean output** — Generated code stays readable
4. **Consistent debugging** — Stack traces point to `drain()`

## Implementation Plan

### Phase 1: `fit` Functions (TS Target Only) — COMPLETE

**Status:** Implemented and tested.

**Scope:** Only `fit` verb triggers flumina; `->` arrow syntax uses direct return.

#### Syntax Distinction

| Syntax | Semantics       | Codegen                                       | Use Case                              |
| ------ | --------------- | --------------------------------------------- | ------------------------------------- |
| `->`   | Direct return   | `return x`                                    | Simple functions, hot paths, interop  |
| `fit`  | Stream protocol | `drain(function* () { yield respond.ok(x) })` | Functions that benefit from streaming |

This allows developers to opt-in to the streaming protocol when needed, while keeping zero overhead for simple functions.

```fab
// Direct return - no protocol overhead
functio fast() -> textus { redde "abc" }

// Stream protocol - backpressure ready
functio streamed() fit textus { redde "abc" }
```

#### AST Changes

Added `returnVerb` field to `FunctioDeclaration`:

```typescript
type ReturnVerb = 'arrow' | 'fit' | 'fiet' | 'fiunt' | 'fient';

interface FunctioDeclaration {
    // ... existing fields ...
    returnVerb?: ReturnVerb;
}
```

#### Preamble (emitted when `features.flumina` is set)

```typescript
type Responsum<T = unknown> = { op: 'bene'; data: T } | { op: 'error'; code: string; message: string } | { op: 'factum' } | { op: 'res'; data: T };

const respond = {
    ok: <T>(data: T): Responsum<T> => ({ op: 'bene', data }),
    error: (code: string, message: string): Responsum<never> => ({ op: 'error', code, message }),
    done: (): Responsum<never> => ({ op: 'factum' }),
    item: <T>(data: T): Responsum<T> => ({ op: 'res', data }),
};

function drain<T>(gen: () => Generator<Responsum<T>>): T {
    for (const resp of gen()) {
        if (resp.op === 'bene') return resp.data;
        if (resp.op === 'error') throw new Error(`${resp.code}: ${resp.message}`);
    }
    throw new Error('EPROTO: No terminal response');
}
```

#### Transformations

| Faber Input                       | Current TS Output                 | Phase 1 TS Output                                                            |
| --------------------------------- | --------------------------------- | ---------------------------------------------------------------------------- |
| `functio foo() fit T { redde x }` | `function foo(): T { return x; }` | `function foo(): T { return drain(function* () { yield respond.ok(x); }); }` |
| `redde x` (in `fit`)              | `return x;`                       | `yield respond.ok(x);`                                                       |
| `redde` (in `fit`, no value)      | `return;`                         | `yield respond.ok(undefined);`                                               |
| `iace "msg"` (in `fit`)           | `throw "msg";`                    | `yield respond.error("EFAIL", "msg"); return;`                               |
| `mori "msg"` (in `fit`)           | `throw new Panic("msg");`         | `throw new Panic("msg");` (unchanged — panics are fatal)                     |

#### Files to Modify

| File                                    | Change                                                                                          |
| --------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `fons/codegen/types.ts`                 | Add `flumina: boolean` to `RequiredFeatures`                                                    |
| `fons/codegen/ts/index.ts`              | Add flumina preamble generation                                                                 |
| `fons/codegen/ts/generator.ts`          | Add `inFlumina: boolean` context flag                                                           |
| `fons/codegen/ts/statements/functio.ts` | Detect `fit` (sync, non-generator), set `inFlumina`, wrap body in `drain(function* () { ... })` |
| `fons/codegen/ts/statements/redde.ts`   | If `inFlumina`: emit `yield respond.ok(...)`                                                    |
| `fons/codegen/ts/statements/iace.ts`    | If `inFlumina` and not `fatal`: emit `yield respond.error(...); return;`                        |
| `fons/parser/ast.ts`                    | Add `ReturnVerb` type and `returnVerb` field to `FunctioDeclaration`                            |
| `fons/parser/index.ts`                  | Track which return syntax was used (`->` vs `fit`/`fiet`/`fiunt`/`fient`)                       |

#### Tests to Add

New file: `proba/codegen/statements/flumina.yaml`

```yaml
- name: fit function with redde
  faber: |
      functio getId() fit textus {
        redde "abc"
      }
  expect:
      ts:
          contains:
              - 'function getId(): string'
              - 'return drain(function* ()'
              - 'yield respond.ok("abc")'

- name: fit function with iace
  faber: |
      functio mightFail() fit textus {
        iace "not found"
      }
  expect:
      ts:
          contains:
              - 'yield respond.error("EFAIL", "not found")'
              - 'return;'

- name: mori unchanged in fit
  faber: |
      functio mustWork() fit textus {
        mori "fatal"
      }
  expect:
      ts:
          contains:
              - 'throw new Panic("fatal")'
          not_contains:
              - 'respond.error'
```

#### Call Sites Unchanged

```faber
fixum x = getId()
```

Compiles to:

```typescript
const x = getId();
```

No `unwrap()`, no ceremony. The `drain()` is inside `getId()`.

#### Implementation Notes

- Only `fit` verb triggers flumina — `->` arrow syntax uses direct return
- Async functions (`fiet`) are NOT yet transformed (Phase 3)
- Generator functions (`fiunt`) are NOT yet transformed (Phase 2)
- Nested `fit` functions inside another `fit` also become flumina-style
- The `inFlumina` context flag tracks whether we're inside a `fit` function body
- The `returnVerb` field in the AST preserves which syntax was used

### Phase 2: `fiunt` Functions (TS Target Only) — COMPLETE

**Decision:** `Responsum` is the universal internal protocol. Caller sees raw values.

#### Design Rationale

The tension: we want `Responsum` everywhere internally (consistency, interceptability, future-proofing), but callers shouldn't deal with protocol wrappers.

**Solution:** Two boundary helpers that convert protocol to target-idiomatic iteration:

| Verb    | Internal                | Boundary  | External    |
| ------- | ----------------------- | --------- | ----------- |
| `fit`   | `yield respond.ok(x)`   | `drain()` | Returns `T` |
| `fiunt` | `yield respond.item(x)` | `flow()`  | Yields `T`  |

This maps naturally to different target error models:

| Target     | Error Model                   | `flow()` Converts To   |
| ---------- | ----------------------------- | ---------------------- |
| TypeScript | Exceptions                    | `throw new Error(...)` |
| Python     | Exceptions                    | `raise Exception(...)` |
| Rust       | `Result<T, E>`                | `Err(e)`               |
| Zig        | Error unions                  | Error return           |
| C++        | Exceptions or `std::expected` | Either model           |

#### Syntax

```fab
functio numeri() fiunt numerus {
    cede 1
    cede 2
    cede 3
}

// Caller iterates raw values
ex numeri() pro n {
    scribe(n)  // n is numerus, not Responsum<numerus>
}
```

#### Internal Protocol

```fab
functio numeri() fiunt numerus {
    cede 1
    iace Error("boom")
    cede 3  // never reached
}
```

Compiles to (internal generator):

```typescript
function* __numeri(): Generator<Responsum<number>> {
    yield respond.item(1);
    yield respond.error('EFAIL', 'boom');
    yield respond.item(3);
}
```

#### The `flow()` Boundary

Converts protocol-aware generator to raw-value generator:

```typescript
function* flow<T>(gen: Generator<Responsum<T>>): Generator<T> {
    for (const resp of gen) {
        if (resp.op === 'res') yield resp.data;
        else if (resp.op === 'error') throw new Error(`${resp.code}: ${resp.message}`);
        else if (resp.op === 'factum') return;
        else if (resp.op === 'bene') {
            yield resp.data;
            return;
        }
    }
}
```

#### Generated Output

```typescript
function* numeri(): Generator<number> {
    yield* flow(
        (function* () {
            yield respond.item(1);
            yield respond.item(2);
            yield respond.done();
        })(),
    );
}
```

#### `cede*` for Delegation

Forwards an entire stream:

```fab
functio omnia() fiunt numerus {
    cede* prima()
    cede* secunda()
}
```

Compiles to:

```typescript
function* omnia(): Generator<number> {
    yield* flow(
        (function* () {
            yield* inspice(prima());
            yield* inspice(secunda());
            yield respond.done();
        })(),
    );
}
```

Where `inspice()` converts a raw-value generator back to protocol (for forwarding).

#### Protocol Access

When you need raw `Responsum` access (middleware, logging, error recovery):

```fab
ex inspice(source) pro resp {
    discerne resp {
        si Res pro x { processa(x) }
        si Error pro e { log(e) }
        si Factum { }
    }
}
```

#### Transformations

| Faber Input       | Internal Protocol          | External Output                   |
| ----------------- | -------------------------- | --------------------------------- |
| `cede x`          | `yield respond.item(x)`    | `yield x` (via `flow()`)          |
| `iace err`        | `yield respond.error(...)` | `throw Error(...)` (via `flow()`) |
| `cede* other()`   | `yield* inspice(other())`  | Forwards all items                |
| (end of function) | `yield respond.done()`     | Generator completes               |

#### Files Modified

| File                                    | Change                                                        |
| --------------------------------------- | ------------------------------------------------------------- |
| `fons/codegen/ts/index.ts`              | Added `flow()` helper to preamble                             |
| `fons/codegen/ts/generator.ts`          | Added `inFiunt: boolean` context flag, updated CedeExpression |
| `fons/codegen/ts/statements/functio.ts` | Detect `fiunt`, set `inFiunt`, wrap in `flow()`, emit done()  |
| `fons/codegen/ts/statements/iace.ts`    | If `inFiunt`: emit `yield respond.error(...)`                 |

Note: `cede` is handled via `CedeExpression` in generator.ts, not a separate statement file.

#### Tests to Add

Extend `proba/codegen/statements/flumina.yaml`:

```yaml
- name: fiunt function with cede
  faber: |
      functio items() fiunt textus {
        cede "a"
        cede "b"
      }
  expect:
      ts:
          contains:
              - 'function* items(): Generator<string>'
              - 'yield* flow('
              - 'yield respond.item("a")'
              - 'yield respond.item("b")'
              - 'yield respond.done()'

- name: fiunt function with iace
  faber: |
      functio items() fiunt textus {
        cede "a"
        iace "error"
      }
  expect:
      ts:
          contains:
              - 'yield respond.item("a")'
              - 'yield respond.error("EFAIL", "error")'
```

### Phase 3: Async Variants (TS Target Only) — COMPLETE

**Status:** Implemented and tested.

`fiet` and `fient` follow the same pattern as `fit` and `fiunt`, but with async generators.

#### Preamble Additions

```typescript
async function drainAsync<T>(gen: () => AsyncGenerator<Responsum<T>>): Promise<T> {
    for await (const resp of gen()) {
        if (resp.op === 'bene') return resp.data;
        if (resp.op === 'error') throw new Error(`${resp.code}: ${resp.message}`);
    }
    throw new Error('EPROTO: No terminal response');
}

async function* flowAsync<T>(gen: AsyncGenerator<Responsum<T>>): AsyncGenerator<T> {
    for await (const resp of gen) {
        if (resp.op === 'res') yield resp.data;
        else if (resp.op === 'error') throw new Error(`${resp.code}: ${resp.message}`);
        else if (resp.op === 'factum') return;
        else if (resp.op === 'bene') {
            yield resp.data;
            return;
        }
    }
}
```

#### Transformations

| Verb    | Internal                      | Boundary       | External Type       |
| ------- | ----------------------------- | -------------- | ------------------- |
| `fit`   | `yield respond.ok(x)`         | `drain()`      | `T`                 |
| `fiunt` | `yield respond.item(x)`       | `flow()`       | `Generator<T>`      |
| `fiet`  | `yield respond.ok(x)`         | `drainAsync()` | `Promise<T>`        |
| `fient` | `yield respond.item(await x)` | `flowAsync()`  | `AsyncGenerator<T>` |

#### Examples

```fab
// fiet: async single-value
functio fetchData() fiet textus {
    redde cede getData()
}
// Compiles to:
// async function fetchData(): Promise<string> {
//   return await drainAsync(async function* () {
//     yield respond.ok(await getData());
//   });
// }

// fient: async multi-value
functio fetchAll(lista<textus> urls) fient textus {
    ex urls pro url {
        cede fetch(url)
    }
}
// Compiles to:
// async function* fetchAll(urls: Array<string>): AsyncGenerator<string> {
//   yield* flowAsync((async function* () {
//     for (const url of urls) {
//       yield respond.item(await fetch(url));
//     }
//     yield respond.done();
//   })());
// }
```

#### Files Modified

| File                                    | Change                                         |
| --------------------------------------- | ---------------------------------------------- |
| `fons/codegen/ts/index.ts`              | Added `drainAsync()` and `flowAsync()` helpers |
| `fons/codegen/ts/generator.ts`          | Added `inFiet` and `inFient` context flags     |
| `fons/codegen/ts/statements/functio.ts` | Handle `fiet` and `fient` verbs                |
| `fons/codegen/ts/statements/iace.ts`    | Extended to handle `fiet`/`fient` contexts     |

### Phase 4: Other Targets (Deferred)

Zig/Rust/C++ use callbacks instead of generators, but emit the same `Responsum` protocol:

```zig
fn numeri(emit: *const fn(Responsum(i64)) void) void {
    emit(respond.item(1));
    emit(respond.item(2));
    emit(respond.done());
}
```

The `flow()` equivalent in these languages converts `respond.error()` to idiomatic error returns.

## References

- Monk OS AGENTS.md — syscall architecture, Response protocol
- Monk OS `src/message.ts` — Response types, `respond` helpers
- Monk OS `src/dispatch/` — dispatcher pattern, backpressure
