# Nucleus: Faber's Micro-Kernel Runtime

A minimal kernel layer providing unified I/O dispatch, message-passing protocol, and async execution across all targets.

## Status

| Feature | Status | Notes |
|---------|--------|-------|
| Responsum protocol | Partial | TS has it; Zig/Rust/C++/Py need it |
| Syscall dispatch (`ad`) | Design only | See `ad.md` |
| Request correlation | Not started | IDs for concurrent ops |
| Handle abstraction | Not started | Unified I/O interface |
| AsyncContext | Not started | Executor for state machines |
| Target runtimes | TS only | Zig is next priority |

## Motivation

Faber targets multiple backends (TS, Python, Zig, Rust, C++). Each has different async primitives:

| Target | Native Async | Native Generators |
|--------|--------------|-------------------|
| TypeScript | `async`/`await` | `function*` |
| Python | `async`/`await` | `yield` |
| Zig | Removed in 0.11 | None |
| Rust | `async`/`.await` | Unstable |
| C++ | Coroutines (C++20) | None |

Rather than emit target-native async everywhere (impossible for Zig), Faber provides a **unified runtime protocol** that compiles to:
- Native async where available (TS, Python)
- State machines where not (Zig, Rust, C++)

The `ad` dispatch system routes all I/O through this protocol.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Faber Source Code                                          │
│  ad "fasciculus:lege" ("file.txt") fiet textus pro content  │
├─────────────────────────────────────────────────────────────┤
│  Compiler (semantic analysis, codegen)                      │
├─────────────────────────────────────────────────────────────┤
│  Nucleus Runtime (per-target implementation)                │
│  ├── Dispatcher (syscall routing)                           │
│  ├── Responsum (message protocol)                           │
│  ├── Handle (unified I/O interface)                         │
│  └── Executor (async scheduling)                            │
├─────────────────────────────────────────────────────────────┤
│  Target Runtime (Bun, Python, Zig std, etc.)                │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Responsum Protocol

All syscalls return `Responsum<T>` — a tagged union describing the result:

```
┌─────────────┬──────────────────────────────────────────┐
│ Variant     │ Meaning                                  │
├─────────────┼──────────────────────────────────────────┤
│ .pending    │ Operation in progress, poll again       │
│ .ok(T)      │ Single value, terminal                  │
│ .item(T)    │ One of many values, non-terminal        │
│ .done       │ Stream complete, terminal               │
│ .err(E)     │ Error, terminal                         │
└─────────────┴──────────────────────────────────────────┘
```

**TypeScript:**
```typescript
type Responsum<T> =
  | { op: 'pending' }
  | { op: 'ok'; data: T }
  | { op: 'item'; data: T }
  | { op: 'done' }
  | { op: 'err'; code: string; message: string };
```

**Zig:**
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

### 2. Handle Abstraction

All I/O sources implement a common interface:

```
Handle
├── id: RequestId
├── type: HandleType (file, socket, channel, timer, ...)
├── exec(msg: Message) → Stream<Responsum<T>>
└── close() → void
```

**Message:**
```
Message
├── op: string        // "read", "write", "get", "query", ...
├── data: any         // Operation-specific payload
└── id: RequestId     // For correlating responses
```

This mirrors Monk's `Handle.exec(msg): AsyncIterable<Response>` pattern.

**Zig:**
```zig
const Handle = struct {
    id: u64,
    handle_type: HandleType,
    vtable: *const HandleVTable,

    pub fn exec(self: *Handle, msg: Message) Responsum(ResultType) {
        return self.vtable.exec(self, msg);
    }

    pub fn close(self: *Handle) void {
        self.vtable.close(self);
    }
};

const HandleVTable = struct {
    exec: *const fn (*Handle, Message) Responsum(ResultType),
    close: *const fn (*Handle) void,
};
```

### 3. Dispatcher (Syscall Table)

Routes `ad` calls to handlers based on target pattern:

```
┌──────────────────────┬─────────────────────┬─────────────────┐
│ Pattern              │ Handler             │ Returns         │
├──────────────────────┼─────────────────────┼─────────────────┤
│ fasciculus:lege      │ FileReadHandler     │ Responsum<text> │
│ fasciculus:scribe    │ FileWriteHandler    │ Responsum<void> │
│ caelum:request       │ HttpHandler         │ Responsum<Resp> │
│ caelum:websocket     │ WebSocketHandler    │ Stream<Message> │
│ tempus:dormi         │ SleepHandler        │ Responsum<void> │
│ https://*            │ → caelum:request    │ (rewrite)       │
│ file://*             │ → fasciculus:lege   │ (rewrite)       │
└──────────────────────┴─────────────────────┴─────────────────┘
```

**Compile-time dispatch (Zig):**
```zig
pub fn dispatch(comptime target: []const u8, args: anytype) Responsum(ReturnType(target)) {
    if (comptime std.mem.startsWith(u8, target, "fasciculus:")) {
        return fasciculus.dispatch(target, args);
    } else if (comptime std.mem.startsWith(u8, target, "caelum:")) {
        return caelum.dispatch(target, args);
    } else {
        @compileError("Unknown syscall: " ++ target);
    }
}
```

**Runtime dispatch (TypeScript):**
```typescript
const dispatcher: Record<string, Handler> = {
  'fasciculus:lege': fileReadHandler,
  'fasciculus:scribe': fileWriteHandler,
  'caelum:request': httpHandler,
  // ...
};

async function* dispatch(target: string, ...args: unknown[]): AsyncIterable<Responsum<unknown>> {
  const handler = dispatcher[target] ?? resolvePattern(target);
  yield* handler(...args);
}
```

### 4. Request Correlation

For concurrent operations, every request gets an ID:

```
┌────────────────────────────────────────────────────────────┐
│ Request                                                    │
│ ├── id: u64                                                │
│ ├── target: "caelum:request"                               │
│ └── args: ("https://api.example.com", "GET")               │
├────────────────────────────────────────────────────────────┤
│ Response                                                   │
│ ├── id: u64  (same as request)                             │
│ └── result: Responsum<T>                                   │
└────────────────────────────────────────────────────────────┘
```

The executor tracks pending requests by ID. When I/O completes, the result is routed to the correct waiting state machine.

### 5. Executor

Drives state machines and manages I/O:

**Single-threaded poll loop (Zig):**
```zig
pub fn run(comptime T: type, future: *Future(T)) !T {
    var ctx = ExecutorContext.init();
    defer ctx.deinit();

    while (true) {
        switch (future.poll(&ctx)) {
            .pending => {
                // Wait for any I/O to complete
                const completed = ctx.wait_for_io();
                // Route result to waiting future
                ctx.deliver_result(completed.id, completed.result);
            },
            .ok => |value| return value,
            .err => |e| return error.FaberError,
            .item, .done => unreachable, // single-value future
        }
    }
}
```

**Async wrapper (TypeScript):**
```typescript
async function run<T>(gen: AsyncIterable<Responsum<T>>): Promise<T> {
    for await (const resp of gen) {
        switch (resp.op) {
            case 'ok': return resp.data;
            case 'err': throw new FaberError(resp.code, resp.message);
            case 'pending': continue; // implicit in async iteration
            case 'item': continue;    // collect or ignore
            case 'done': throw new Error('Unexpected done in single-value context');
        }
    }
    throw new Error('Generator ended without result');
}
```

## Compilation Strategy

### TypeScript / Python

Use native async. The nucleus is a thin wrapper:

```fab
ad "fasciculus:lege" ("file.txt") fiet textus pro content { ... }
```

Becomes:
```typescript
const content = await run(dispatch('fasciculus:lege', 'file.txt'));
```

Or with direct codegen (preferred):
```typescript
const content = await fs.readFile('file.txt', 'utf-8');
```

The dispatcher exists for uniformity but can be bypassed for performance.

### Zig / Rust / C++

Compile to state machines. Each `fiet`/`fient` function becomes:

1. **State union** — captures locals at each suspend point
2. **Poll function** — advances state machine, returns `Responsum<T>`
3. **Wrapper function** — returns the future

```fab
functio fetch(textus url) fiet Response {
    ad "caelum:request" (url, "GET") fiet Response pro resp
    redde resp
}
```

Becomes (Zig):
```zig
const FetchState = union(enum) {
    start: struct { url: []const u8 },
    awaiting_request: struct { request_id: u64 },
};

const FetchFuture = struct {
    state: FetchState,

    pub fn poll(self: *FetchFuture, ctx: *ExecutorContext) Responsum(Response) {
        switch (self.state) {
            .start => |s| {
                const id = ctx.submit("caelum:request", .{ s.url, "GET" });
                self.state = .{ .awaiting_request = .{ .request_id = id } };
                return .pending;
            },
            .awaiting_request => |s| {
                if (ctx.get_result(s.request_id, Response)) |resp| {
                    return .{ .ok = resp };
                }
                return .pending;
            },
        }
    }
};

pub fn fetch(url: []const u8) FetchFuture {
    return FetchFuture{ .state = .{ .start = .{ .url = url } } };
}
```

## Syscall Namespaces

Latin names for stdlib modules (matching `ad.md`):

| Namespace | Domain | Example Syscalls |
|-----------|--------|------------------|
| `fasciculus` | Files | `lege`, `scribe`, `tolle`, `movere` |
| `caelum` | Network | `request`, `websocket`, `listen` |
| `tempus` | Time | `dormi`, `nunc`, `intervallum` |
| `memoria` | Memory | `alloca`, `libera` (Zig/Rust/C++) |
| `processus` | Process | `genera`, `occide`, `expecta` |
| `canalis` | Channels | `crea`, `mitte`, `recipe` |
| `aleator` | Random | `numerus`, `bytes`, `uuid` |
| `crypto` | Crypto | `hash`, `signa`, `verifica` |

## Backpressure

For streaming syscalls (`fiunt`/`fient`), the executor implements flow control:

| Constant | Value | Purpose |
|----------|-------|---------|
| `AQUA_ALTA` | 1000 | Pause producing |
| `AQUA_BASSA` | 100 | Resume producing |
| `PULSUS_INTERVALLUM` | 100ms | Consumer liveness check |
| `MORA_MAXIMUM` | 5000ms | Abort if consumer dead |

(Latin: aqua alta = high water, aqua bassa = low water, pulsus = pulse/ping, mora = delay)

## Error Handling

Errors flow through Responsum:

```fab
ad "fasciculus:lege" ("missing.txt") fiet textus pro content {
    scribe content
} cape err {
    scribe "Error: " + err.message
}
```

The `cape` clause catches `.err` responses. Without it, errors propagate to caller.

**Zig codegen:**
```zig
switch (future.poll(&ctx)) {
    .ok => |content| { ... },
    .err => |e| {
        // cape block
        std.debug.print("Error: {s}\n", .{e.message});
    },
    // ...
}
```

## Relationship to Existing Designs

| Document | Relationship |
|----------|--------------|
| `ad.md` | Nucleus provides the runtime that `ad` dispatches to |
| `zig-async.md` | State machine compilation is one strategy within Nucleus |
| `flumina.md` | Responsum protocol originated here; Nucleus generalizes it |
| `two-pass.md` | Liveness analysis feeds into state machine generation |

## Implementation Plan

### Phase 1: Protocol Unification
1. Define `Responsum<T>` for all targets in preamble
2. Standardize syscall table schema
3. Add request ID generation

### Phase 2: TypeScript Runtime
1. Implement dispatcher with pattern matching
2. Wire `ad` codegen to dispatcher
3. Test with file/http syscalls

### Phase 3: Zig Runtime
1. Implement `Handle` vtable pattern
2. Implement `ExecutorContext` with I/O polling
3. State machine codegen for `fiet`/`fient`
4. Syscall handlers for `fasciculus`, `caelum`, `tempus`

### Phase 4: Other Targets
1. Python — similar to TypeScript
2. Rust — state machines like Zig, or native async
3. C++ — coroutines or callback-based

---

## Lessons from Monk OS

Analysis of Monk OS source code revealed patterns, concerns, and opportunities relevant to Nucleus.

### What Monk Solved

#### 1. Worker Boundary Isolation

Monk uses Bun Workers for process isolation, requiring message passing across thread boundaries:

```typescript
// Monk: Every syscall crosses Worker boundary
worker.postMessage({
    type: 'syscall:request',
    id: requestId,
    name: 'file:read',
    args: [fd, 4096],
});
```

**Faber's advantage**: Compiles to native code in single address space. Direct function calls, no serialization overhead.

#### 2. Streaming Backpressure

Monk implements ping/ack protocol with per-stream timers:

```typescript
// Monk: setInterval per active syscall stream
stream.pingTimer = setInterval(() => {
    self.postMessage({
        type: 'syscall:ping',
        id,
        processed: stream.processed,
    });
}, 100);
```

**Side effect**: 1000 concurrent syscalls = 1000 active timers. GC pressure.

**Faber's advantage**: Use language-native generator pause/resume. No timers needed for native targets.

#### 3. Request Correlation

Monk uses UUID v4 for request IDs (128-bit, cryptographically unique):

```typescript
const id = crypto.randomUUID();
pending.set(id, { stream });
```

**Trade-off**: 16 bytes per ID vs 8 bytes. String allocation + hashing overhead.

**Faber's decision**: Use 64-bit counter for Zig/Rust/C++ (simpler, faster). UUID acceptable for TS/Python.

#### 4. Handle Abstraction

Monk's unified interface is clean and worth adopting:

```typescript
interface Handle {
    readonly id: string;
    readonly type: HandleType;
    exec(msg: Message): AsyncIterable<Response>;
    close(): Promise<void>;
}
```

**Adopted**: Nucleus mirrors this pattern with vtable for Zig.

### Concerns Discovered

#### 1. Partial Results on Midstream Error

```typescript
// Handler yields 100 items, then crashes
yield respond.item(item1);   // Consumer sees this
yield respond.item(item2);   // Consumer sees this
// ... 98 more items ...
// CRASH
yield respond.error('...'); // Consumer sees this AFTER 100 items
```

**Problem**: Consumer has already processed partial results before error arrives.

**Nucleus decision**: Document that consumers must handle partial results. Consider adding `.partial_error` variant that includes count of successful items.

#### 2. Nested Cancellation Leaks

```typescript
// Outer handler uses inner syscall
async *handler() {
    for await (const item of syscall('inner')) {
        yield respond.item(item);
        // If cancelled here, inner stream.pingTimer not cleared!
    }
}
```

**Problem**: Cleanup only happens for outermost stream.

**Nucleus decision**: For native targets, use RAII/defer patterns. Each stream cleans itself on scope exit.

#### 3. Stall Detection False Positives

Monk aborts streams after 5s without ping. Slow networks can trigger false timeouts.

**Nucleus decision**: Make stall timeout configurable per-syscall. Network ops get longer timeout than file ops.

### Opportunities for Simplification

| Monk Complexity | Faber Simplification |
|-----------------|---------------------|
| Message serialization across Workers | Direct function calls |
| Per-stream ping timers | Language-native yield/suspend |
| Virtual process validation | Trust OS process isolation |
| UUID request IDs | 64-bit counter (native targets) |
| Runtime syscall dispatch | Compile-time dispatch (Zig comptime) |
| Per-syscall auth checking | Entry-point or compile-time auth |

### HAL Device Model

Monk's 17-device HAL is worth adopting selectively:

| Device | Faber Equivalent | Priority |
|--------|------------------|----------|
| `block` | `fasciculus` (files) | High |
| `network` | `caelum` (network) | High |
| `timer` | `tempus` | High |
| `entropy` | `aleator` | Medium |
| `crypto` | `crypto` | Medium |
| `storage` | EMS-style (future) | Low |
| `console` | `scriba` | High |

---

## Design Decisions

Based on Monk analysis, the following open questions are resolved:

### 1. Allocator Threading (Zig)

**Decision**: Per-request allocator, inherited from parent.

```zig
pub fn poll(self: *FetchFuture, ctx: *ExecutorContext) Responsum(Response) {
    // ctx.allocator inherited from parent future or executor
    const result = try ctx.allocator.alloc(u8, size);
    // ...
}
```

**Rationale**: Matches Monk's per-Worker heap isolation. Avoids global lock contention.

### 2. Cancellation

**Decision**: Out-of-band signal (like Monk), not Responsum variant.

```zig
// Executor signals cancellation via context flag
if (ctx.cancelled) {
    // Cleanup and exit
    return .done;
}
```

**Rationale**: Adding `.cancelled` to Responsum complicates every switch statement. Separate signal is cleaner.

For streaming consumers, provide explicit cancel:

```fab
ad "fasciculus:lege" ("large.bin") fiunt bytes pro chunk {
    si chunk.size > MAX {
        exi  // Break from stream (triggers cleanup)
    }
}
```

### 3. Nested Composition

**Decision**: Inline for small futures (< 256 bytes), pointer for large.

```zig
// Small future: inline
const OuterFuture = struct {
    inner: InnerFuture,  // Inline, no allocation
    // ...
};

// Large future: pointer
const OuterFuture = struct {
    inner_ptr: *LargeFuture,  // Heap-allocated
    // ...
};
```

**Rationale**: Inline avoids allocation overhead for common case. Pointer prevents bloat for complex compositions.

**Implementation**: Compiler tracks future sizes during codegen. Threshold configurable.

### 4. User-Defined Syscalls

**Decision**: Not in v1. Compile-time syscall table only.

**Rationale**: Runtime registration requires:
- Type erasure (fighting Zig/Rust)
- Dynamic dispatch overhead
- Security concerns (arbitrary code execution)

Future consideration: Allow `importa` to bring in typed syscall handlers at compile time.

### 5. Direct Codegen Escape Hatch

**Decision**: Opt-in via `ad!` syntax.

```fab
// Standard: via dispatcher (observable, testable)
ad "fasciculus:lege" ("file.txt") fiet textus pro content

// Direct: bypass dispatcher (fast, less observable)
ad! "fasciculus:lege" ("file.txt") fiet textus pro content
```

**Rationale**: Default should be observable/testable. Performance-critical paths can opt out explicitly.

**Trade-offs**:
- `ad!` bypasses logging, metrics, mocking
- Some targets may not support bypass (falls back to dispatcher)

---

## Additional Concerns

### Byte-Based Backpressure

Current design uses item count (gap = sent - acked). For large-object streams, consider memory-aware backpressure:

```zig
const BackpressureConfig = struct {
    max_items: u32 = 1000,        // Item count limit
    max_bytes: usize = 100_MB,    // Memory limit
};
```

**Recommendation**: Add optional `max_bytes` for syscalls that stream large chunks (file reads, network buffers).

### Process Isolation

Monk uses Workers for isolation. Faber compiles to native code without Worker boundaries.

**Options for isolation**:
1. **None** — Single process, shared memory (fastest, least safe)
2. **OS processes** — Use `processus:genera` to spawn isolated processes
3. **WASM sandboxing** — Future consideration for untrusted code

**v1 decision**: No built-in isolation. Users who need it use OS processes.

### Auth Consolidation

Monk checks auth on every syscall (lazy expiry). This is scattered and inefficient.

**Faber approach**: Check auth at entry points, not per-syscall.

```fab
// Auth middleware wraps entire handler
@requiresAuth
functio protectedEndpoint() fiet Response {
    // Auth already validated
}
```

Compiler injects auth check at function entry, not inside dispatcher.

---

## Open Questions (Remaining)

1. **Error context enrichment** — Should `.err` include stack trace or causality chain? Performance vs debuggability trade-off.

2. **Streaming timeout configuration** — Per-syscall or global? How to specify in Faber syntax?

3. **Future size threshold** — 256 bytes for inline composition is arbitrary. Profile real-world futures to tune.

4. **Cross-target consistency** — How much divergence is acceptable between targets? (e.g., TS uses UUID, Zig uses counter)

## References

- `ad.md` — Dispatch syntax design
- `zig-async.md` — Zig-specific state machine details
- `flumina.md` — Original Responsum protocol (TypeScript)
- `two-pass.md` — Semantic analysis for liveness
- Monk OS `AGENTS.md` — Inspiration for Handle/Message/Response patterns
