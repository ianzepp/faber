# Zig Async via Responsum State Machines

Zig removed native async/await in version 0.11. This document describes Faber's approach to async and generators for Zig targets using the Responsum protocol and compiled state machines.

## Syntax Split

Faber has two syntax systems for async/generators. Only verb syntax is supported for Zig:

| Syntax | Zig Support | Notes |
|--------|-------------|-------|
| `futura`/`cursor` + `->` | **Error** | TS/Python only |
| `fit`/`fiet`/`fiunt`/`fient` | **Supported** | Compiles to state machine |

```fab
# Error on Zig target:
futura functio fetch() -> textus { ... }
cursor functio items() -> numerus { ... }

# Supported on all targets including Zig:
functio fetch() fiet textus { ... }
functio items() fiunt numerus { ... }
```

## Responsum Protocol

All verb-syntax functions use the Responsum protocol. Emitted in Zig preamble:

```zig
fn Responsum(comptime T: type) type {
    return union(enum) {
        pending,
        ok: T,
        item: T,
        done,
        err: struct { code: []const u8, message: []const u8 },
    };
}
```

| Faber | Responsum Variant | Terminal |
|-------|-------------------|----------|
| `redde x` (in `fit`/`fiet`) | `.ok` | yes |
| `cede x` (in `fiunt`/`fient`) | `.item` | no |
| end of `fiunt`/`fient` | `.done` | yes |
| `iace err` | `.err` | yes |
| suspend point | `.pending` | no |

## State Machine Compilation

Each `fiet`/`fient` function compiles to:
1. A state `union(enum)` capturing locals across suspend points
2. A `poll` function that advances the state machine

### Example

```fab
functio fetchUser(textus url) fiet User {
    fixum resp = cede http.get(url)
    fixum json = cede resp.json()
    redde User.parse(json)
}
```

Compiles to:

```zig
const FetchUserState = union(enum) {
    start: struct { url: []const u8 },
    awaiting_get: struct { url: []const u8 },
    awaiting_json: struct { resp: Response },
};

const FetchUserFuture = struct {
    state: FetchUserState,

    pub fn init(url: []const u8) FetchUserFuture {
        return .{ .state = .{ .start = .{ .url = url } } };
    }

    pub fn poll(self: *FetchUserFuture, ctx: *AsyncContext) Responsum(User) {
        switch (self.state) {
            .start => |s| {
                ctx.submit_io(http.get(s.url));
                self.state = .{ .awaiting_get = .{ .url = s.url } };
                return .pending;
            },
            .awaiting_get => |s| {
                const resp = ctx.get_result(Response);
                ctx.submit_io(resp.json());
                self.state = .{ .awaiting_json = .{ .resp = resp } };
                return .pending;
            },
            .awaiting_json => |s| {
                const json = ctx.get_result(Json);
                return .{ .ok = User.parse(json) };
            },
        }
    }
};

pub fn fetchUser(url: []const u8) FetchUserFuture {
    return FetchUserFuture.init(url);
}
```

## State Variants

Each state variant captures the variables live at that suspend point:

| Suspend Point | Captured Variables |
|---------------|-------------------|
| `start` | function parameters |
| after `cede` N | params + locals assigned before suspend N |

The compiler performs liveness analysis to minimize captured state.

## Generators (`fiunt`/`fient`)

Generators follow the same pattern but yield `.item` multiple times:

```fab
functio range(numerus n) fiunt numerus {
    varia i = 0
    dum i < n {
        cede i
        i = i + 1
    }
}
```

```zig
const RangeState = union(enum) {
    init: struct { n: i64 },
    yielding: struct { n: i64, i: i64 },
    done,
};

const RangeIterator = struct {
    state: RangeState,

    pub fn next(self: *RangeIterator) Responsum(i64) {
        switch (self.state) {
            .init => |s| {
                self.state = .{ .yielding = .{ .n = s.n, .i = 0 } };
                return self.next();
            },
            .yielding => |s| {
                if (s.i < s.n) {
                    const value = s.i;
                    self.state = .{ .yielding = .{ .n = s.n, .i = s.i + 1 } };
                    return .{ .item = value };
                }
                self.state = .done;
                return .done;
            },
            .done => return .done,
        }
    }
};
```

## Async Context

The `AsyncContext` provides I/O scheduling. Options for implementation:

| Approach | Library | Notes |
|----------|---------|-------|
| `std.io.poll` | stdlib | Basic, limited |
| `libxev` | external | Production-quality event loop |
| `zig-aio` | external | io_uring focused |
| Custom | — | Faber-specific minimal executor |

Minimal interface:

```zig
const AsyncContext = struct {
    pub fn submit_io(self: *AsyncContext, op: IoOp) void { ... }
    pub fn get_result(self: *AsyncContext, comptime T: type) T { ... }
    pub fn wait_for_io(self: *AsyncContext) void { ... }
};
```

## Executor

Simple executor for running futures:

```zig
pub fn block_on(comptime T: type, future: anytype) !T {
    var ctx = AsyncContext.init();
    defer ctx.deinit();

    var f = future;
    while (true) {
        switch (f.poll(&ctx)) {
            .pending => ctx.wait_for_io(),
            .ok => |v| return v,
            .err => |e| return error.FaberError,
            .item => unreachable, // single-value future
            .done => unreachable,
        }
    }
}
```

## Relationship to discretio

The state machine `union(enum)` is structurally identical to `discretio`:

```fab
# User-defined discretio
discretio Event {
    Click { numerus x, numerus y }
    Quit
}

# Compiler-generated for async
discretio FetchState {
    Start { textus url }
    AwaitingGet { textus url }
    Done { User result }
}
```

Both compile to Zig `union(enum)`. The async codegen can reuse `discretio` infrastructure.

## Error Handling

| Faber | State Machine Behavior |
|-------|----------------------|
| `iace err` | Return `.{ .err = .{ .code = ..., .message = ... } }` |
| `mori err` | `@panic(...)` (immediate, no state transition) |
| Called function errors | Propagate via Responsum |

## Implementation Plan

1. **Preamble**: Add `Responsum(T)` and `AsyncContext` to Zig preamble
2. **AST**: Identify `fiet`/`fient` functions for state machine transform
3. **Analysis**: Compute suspend points and liveness for each
4. **Codegen**: Emit state union + poll function per async function
5. **Executor**: Provide minimal `block_on` in stdlib or preamble
6. **I/O**: Integrate with chosen event loop library

## Target Validation

At semantic analysis, reject arrow-syntax async for Zig:

```
Error P192: `futura` with `->` not supported for Zig target. Use `fiet` instead.
Error P193: `cursor` with `->` not supported for Zig target. Use `fiunt` instead.
```

---

## Current Implementation Status

### What Exists

**Parser** (`fons/parser/`):
- ✓ `ReturnVerb` type: `'arrow' | 'fit' | 'fiet' | 'fiunt' | 'fient'`
- ✓ `FunctioDeclaration.returnVerb` field
- ✓ Parsing of all verb forms
- ✓ Validation: `futura`/`cursor` not allowed with verbs (P191)
- ✓ `cede` statement parsing with context validation

**Semantic Analyzer** (`fons/semantic/`):
- ✓ Basic syntax validation
- ✗ Two-pass analysis for `canThrow` propagation
- ✗ Liveness analysis for state machine
- ✗ Zig-specific validation (reject arrow syntax on Zig)

**TypeScript Codegen** (`fons/codegen/ts/`):
- ✓ `asFit`, `asFiunt`, `asFiet`, `asFient` boundary helpers
- ✓ Responsum protocol type definition
- ✓ Context tracking (`inFlumina`, `inFiunt`, `inFiet`, `inFient`)
- ✓ `redde` → `yield respond.ok()`
- ✓ `cede` → `yield respond.item()` (with `respond.done()` at end)
- ✓ `iace` → `yield respond.error()` + early return
- ✓ Preamble generation with feature detection

**Zig Codegen** (`fons/codegen/zig/`):
- ✓ Basic Zig generation (types, functions, control flow)
- ✓ Error union support (`!T` for functions containing `iace`)
- ✓ `discretio` → `union(enum)` compilation
- ✓ Module-level vs runtime code separation
- ✓ Allocator tracking (curator stack)
- ✗ Any async/generator handling
- ✗ Responsum protocol types
- ✗ State machine generation
- ✗ `fit`/`fiet`/`fiunt`/`fient` codegen

### What Needs to Be Built

**1. Semantic Analysis** (blocker):
- Two-pass analysis: collect declarations, then analyze bodies
- Throwability propagation: mark `canThrow` on functions
- Target-specific validation: reject arrow syntax for Zig (P192/P193)

**2. Liveness Analysis** (performance optimization):
- Identify suspend points (each `cede`)
- Determine live variables at each point
- Minimize state struct payloads

**3. Zig Codegen**:
- Preamble: `Responsum(T)`, `AsyncContext`, `block_on`
- State machine generation: union + poll function per async function
- Statement handling: `cede` → context submit, `redde` → return `.ok`, `iace` → return `.err`

**4. Testing**:
- Add `proba/codegen/statements/flumina.yaml` tests for Zig target
- Validate state machine structure
- Test suspend/resume cycles
- Test error propagation via `.err` variant

---

## Open Questions

### AsyncContext Interface

The `AsyncContext` interface is underspecified. Key questions:

1. **What is `IoOp`?** The document shows `ctx.submit_io(http.get(s.url))` but `IoOp` isn't defined. How do http.get, resp.json(), file operations become `IoOp` instances?

2. **Result correlation**: How does `ctx.get_result()` know which operation completed when multiple are in flight?

3. **Threading model**: Is this single-threaded polling or multi-threaded dispatch?

4. **Extensibility**: Does Faber emit a fixed set of I/O operations, or is this user-extensible?

Proposed `IoOp` definition (minimum viable):

```zig
const IoOp = union(enum) {
    http_get: struct { url: []const u8 },
    http_post: struct { url: []const u8, body: []const u8 },
    file_read: struct { path: []const u8 },
    file_write: struct { path: []const u8, data: []const u8 },
    sleep: struct { ms: u64 },
    // User-defined operations would require a different approach
};
```

### Cede Semantics

The document uses `cede` for two distinct operations:

```fab
# In fiet: await-like (waiting for a result)
fixum resp = cede http.get(url)

# In fiunt: yield-like (producing a value)
cede i
```

The Zig codegen shows `ctx.submit_io()` + `ctx.get_result()` as separate operations. The control flow needs clarity:
- How does the caller resume the future after I/O completes?
- How is the result passed back into the state machine?

### Mutable Variables Across Suspend Points

Consider:

```fab
functio example() fiet textus {
    varia x = "hello"
    fixum resp = cede fetch()
    x = "goodbye"  # mutation after suspend
    redde x
}
```

If `x` is captured in the `awaiting_fetch` state, mutations after resume must be preserved. But the state struct is value-typed—each `poll()` call works with a copy unless we're careful about pointer semantics.

Options:
1. Capture mutable locals by pointer (requires allocator)
2. Re-assign updated values back to state struct before returning `.pending`
3. Disallow mutable locals across suspend points (restrictive)

### Nested Async Composition

How does a `fiet` function call another `fiet` function?

```fab
functio outer() fiet User {
    fixum profile = cede fetchProfile()
    fixum settings = cede fetchSettings(profile.id)
    redde User.create(profile, settings)
}
```

The outer state machine must store the inner future in its state:

```zig
const OuterState = union(enum) {
    start,
    awaiting_profile: struct { inner: FetchProfileFuture },
    awaiting_settings: struct { profile: Profile, inner: FetchSettingsFuture },
};
```

This compositional pattern isn't shown in the current examples.

### Cancellation

What happens when a future is dropped mid-execution? The poll-based design implies you can just stop calling `poll()`. But:
- What about cleanup of resources acquired before a suspend point?
- Should there be a `cancel()` method on futures?
- How does this interact with `cura` (resource management)?

### Error Union in Responsum

For functions that can throw (`iace`), should the return type be:
- `Responsum(T)` with `.err` variant (current design)
- `Responsum(!T)` with Zig error union inside `.ok`
- Both (`.err` for Faber errors, `!T` for Zig errors)

The `canThrow` propagation from two-pass analysis affects this decision.

---

## TypeScript Responsum Protocol (Reference)

The TS implementation uses these operation tags:

```typescript
type Responsum<T = unknown> =
  | { op: 'bene'; data: T }       // single value, stream ends
  | { op: 'error'; code: string; message: string }  // failure, stream ends
  | { op: 'factum' }              // multi-value complete, stream ends
  | { op: 'res'; data: T };       // one item of many, continues
```

Boundary helpers (`asFit`, `asFiunt`, `asFiet`, `asFient`) convert internal protocol to external values—callers never see the protocol, only raw values.

The Zig design adds `.pending` for suspend points, which TS doesn't need (native async handles suspension).

---

## Discretio Infrastructure (Reference)

The existing `discretio` → `union(enum)` compilation (`fons/codegen/zig/statements/discretio.ts`) can be reused:

```fab
discretio Event {
    Click { numerus x, numerus y }
    Keypress { textus key }
    Quit
}
```

Compiles to:

```zig
const Event = union(enum) {
    click: struct { x: i64, y: i64 },
    keypress: struct { key: []const u8 },
    quit,
};
```

The async state union follows the same structure—generated state unions are structurally identical to user-defined `discretio`.

---

## References

- `flumina.md` — Responsum protocol design (TS implementation)
- `unio.md` — `discretio` to `union(enum)` compilation
- `two-pass.md` — Liveness analysis for captured variables
