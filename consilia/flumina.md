# Flumina: Streams-First Architecture

**Status:** Phases 1-3 Complete (TypeScript target)
**Phase 4:** Deferred (Zig/Rust/C++ targets)

## Overview

Flumina makes streams the fundamental execution model. All `fit`/`fiunt`/`fiet`/`fient` functions use the `Responsum` protocol internally, while callers see raw values.

| Verb    | Sync/Async | Cardinality | Boundary    | Return Type         |
| ------- | ---------- | ----------- | ----------- | ------------------- |
| `fit`   | sync       | single      | `utFit()`   | `T`                 |
| `fiunt` | sync       | multi       | `utFiunt()` | `Generator<T>`      |
| `fiet`  | async      | single      | `utFiet()`  | `Promise<T>`        |
| `fient` | async      | multi       | `utFient()` | `AsyncGenerator<T>` |

The `->` arrow syntax bypasses the protocol entirely for zero-overhead direct returns.

### Two Syntax Patterns

**Verb syntax** (stream protocol, Responsum wrapper):

```fab
functio getId() fit textus { redde "abc" }     // sync, single (utFit)
functio items() fiunt numerus { cede 1; cede 2 }  // sync, multi (utFiunt)
functio fetch() fiet textus { redde data }     // async, single (utFiet)
functio stream() fient textus { cede data }    // async, multi (utFient)
```

**Arrow syntax** (direct returns, traditional semantics):

```fab
functio getId() -> textus { redde "abc" }            // sync, direct return
futura functio fetch() -> textus { redde data }      // async, direct return
cursor functio items() -> numerus { cede 1; cede 2 } // sync, traditional yield
futura cursor functio stream() -> textus { cede data } // async, traditional async yield
```

**Important**: `futura`/`cursor` prefixes cannot be combined with verbs:

- ✗ `futura functio f() fit T` (ERROR: prefixes only with arrow)
- ✗ `cursor functio f() fiunt T` (ERROR: prefixes only with arrow)
- ✓ `functio f() fiet T` (verb carries semantic info)
- ✓ `futura functio f() -> T` (arrow uses prefix semantics)

## The Responsum Protocol

All yields produce a `Responsum` with an `op` field:

| Op      | Latin    | Terminal | Purpose                   |
| ------- | -------- | -------- | ------------------------- |
| `ok`    | `bene`   | yes      | Single value, stream ends |
| `error` | `error`  | yes      | Failure, stream ends      |
| `done`  | `factum` | yes      | Multi-value complete      |
| `item`  | `res`    | no       | One item of many          |

```typescript
type Responsum<T> = { op: 'bene'; data: T } | { op: 'error'; code: string; message: string } | { op: 'factum' } | { op: 'res'; data: T };
```

## Keyword Mapping

| Faber      | Purpose             | Emits                      |
| ---------- | ------------------- | -------------------------- |
| `redde x`  | Return single value | `yield respond.ok(x)`      |
| `cede x`   | Yield one of many   | `yield respond.item(x)`    |
| `iace err` | Recoverable error   | `yield respond.error(...)` |
| `mori err` | Fatal panic         | `throw new Panic(...)`     |

## Examples

### Single Value (`fit`)

```fab
functio getId() fit textus {
    redde "abc"
}
```

Compiles to:

```typescript
function getId(): string {
    return utFit(function* () {
        yield respond.ok('abc');
    });
}
```

### Multi-Value (`fiunt`)

```fab
functio items() fiunt numerus {
    cede 1
    cede 2
}
```

Compiles to:

```typescript
function* items(): Generator<number> {
    yield* utFiunt(
        (function* () {
            yield respond.item(1);
            yield respond.item(2);
            yield respond.done();
        })(),
    );
}
```

### Async Single (`fiet`)

```fab
functio fetchData() fiet textus {
    redde cede getData()
}
```

Compiles to:

```typescript
async function fetchData(): Promise<string> {
    return await utFiet(async function* () {
        yield respond.ok(await getData());
    });
}
```

### Async Multi (`fient`)

```fab
functio fetchAll(lista<textus> urls) fient textus {
    ex urls pro url {
        cede fetch(url)
    }
}
```

Compiles to:

```typescript
async function* fetchAll(urls: Array<string>): AsyncGenerator<string> {
    yield* utFient(
        (async function* () {
            for (const url of urls) {
                yield respond.item(await fetch(url));
            }
            yield respond.done();
        })(),
    );
}
```

## Boundary Helpers

The `ut*` helpers convert internal protocol to external values:

```typescript
// utFit: sync single-value
function utFit<T>(gen: () => Generator<Responsum<T>>): T {
    for (const resp of gen()) {
        if (resp.op === 'bene') return resp.data;
        if (resp.op === 'error') throw new Error(`${resp.code}: ${resp.message}`);
    }
    throw new Error('EPROTO: No terminal response');
}

// utFiunt: sync multi-value
function* utFiunt<T>(gen: Generator<Responsum<T>>): Generator<T> {
    for (const resp of gen) {
        if (resp.op === 'res') yield resp.data;
        else if (resp.op === 'error') throw new Error(`${resp.code}: ${resp.message}`);
        else if (resp.op === 'factum') return;
        else if (resp.op === 'bene') { yield resp.data; return; }
    }
}

// utFiet: async single-value
async function utFiet<T>(gen: () => AsyncGenerator<Responsum<T>>): Promise<T> { ... }

// utFient: async multi-value
async function* utFient<T>(gen: AsyncGenerator<Responsum<T>>): AsyncGenerator<T> { ... }
```

## Error Handling

- `iace` (recoverable): Emits `respond.error()`, converted to exception at boundary
- `mori` (fatal): Always throws `Panic`, bypasses protocol

```fab
functio validate(numerus x) fit numerus {
    si x < 0 {
        iace "negative not allowed"  // -> yield respond.error(...)
    }
    si x > 1000 {
        mori "value too large"       // -> throw new Panic(...)
    }
    redde x
}
```

## Implementation

### Files Modified

| File                                    | Purpose                                                    |
| --------------------------------------- | ---------------------------------------------------------- |
| `fons/codegen/ts/index.ts`              | Preamble with `Responsum`, `respond`, `ut*` helpers        |
| `fons/codegen/ts/generator.ts`          | Context flags: `inFlumina`, `inFiunt`, `inFiet`, `inFient` |
| `fons/codegen/ts/statements/functio.ts` | Verb detection, wrapper generation                         |
| `fons/codegen/ts/statements/redde.ts`   | `yield respond.ok()` in flumina context                    |
| `fons/codegen/ts/statements/iace.ts`    | `yield respond.error()` in flumina context                 |
| `fons/parser/ast.ts`                    | `ReturnVerb` type, `returnVerb` field                      |

### Tests

See `proba/codegen/statements/flumina.yaml` for comprehensive tests.

## Phase 4: Other Targets (Deferred)

Zig/Rust/C++ lack native generators. Options:

1. **Callback-based**: Transform to push-style callbacks
2. **State machine**: Compile generator to explicit state machine
3. **Target idioms**: Use `impl Iterator` (Rust), etc.

Example Zig output (conceptual):

```zig
fn numeri(emit: *const fn(Responsum(i64)) void) void {
    emit(respond.item(1));
    emit(respond.item(2));
    emit(respond.done());
}
```

## Design Decisions

1. **Protocol is internal**: Callers see raw values, not `Responsum`
2. **`->` bypasses protocol**: Zero overhead for simple functions
3. **`mori` bypasses protocol**: Panics are fatal, not recoverable
4. **Helpers named `ut*`**: Latin consistency (`ut` = "as")
5. **Backpressure deferred**: Pull-based generators have implicit backpressure

## References

- Monk OS syscall architecture (inspiration)
- `proba/codegen/statements/flumina.yaml` (tests)
