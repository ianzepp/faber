# Nucleus: Faber's Micro-Kernel Runtime

A minimal kernel layer providing unified I/O dispatch, message-passing protocol, and async execution across all targets.

## Status

| Feature                 | Status      | Notes                              |
| ----------------------- | ----------- | ---------------------------------- |
| Responsum protocol      | Partial     | TS has it; Zig/Rust/C++/Py need it |
| Syscall dispatch (`ad`) | Design only | See `ad.md`                        |
| Request correlation     | Not started | IDs for concurrent ops             |
| Handle abstraction      | Not started | Unified I/O interface              |
| AsyncContext            | Not started | Executor for state machines        |
| Target runtimes         | TS only     | Zig is next priority               |

## Motivation

Faber targets multiple backends (TS, Python, Zig, Rust, C++). Each has different async primitives:

| Target     | Native Async       | Native Generators |
| ---------- | ------------------ | ----------------- |
| TypeScript | `async`/`await`    | `function*`       |
| Python     | `async`/`await`    | `yield`           |
| Zig        | Removed in 0.11    | None              |
| Rust       | `async`/`.await`   | Unstable          |
| C++        | Coroutines (C++20) | None              |

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
type ResponsumError = {
    code: string;
    message: string;
    details?: unknown;
    cause?: ResponsumError;
};

type Responsum<T> = { op: 'pending' } | { op: 'ok'; data: T } | { op: 'item'; data: T } | { op: 'done' } | { op: 'err'; error: ResponsumError };
```

**Zig:**

```zig
const ResponsumError = struct {
    code: []const u8,
    message: []const u8,
    // Optional enrichment (best-effort per target/runtime)
    details: ?[]const u8 = null,
    cause: ?*const ResponsumError = null,
};

fn Responsum(comptime T: type) type {
    return union(enum) {
        pending,
        ok: T,
        item: T,
        done,
        err: ResponsumError,
    };
}
```

**Protocol contract (required invariants):**

- Single-value operation: `.pending* -> (.ok | .err)`
- Streaming operation: `.pending* -> (.item | .pending)* -> (.done | .err)`
- `.item` MUST NOT appear after a terminal variant.
- `.done` and `.err` are terminal variants.

### 2. Handle Abstraction

All I/O sources implement a common interface:

```
Handle
├── id: HandleId
├── type: HandleType (file, socket, channel, timer, ...)
├── exec(msg: Message) → OpStream<T>
└── close() → void

OpStream<T>
├── id: RequestId
├── poll(ctx) → Responsum<T>
└── cancel(ctx) → void (optional)
```

**Message:**

```
Message
├── op: string        // "read", "write", "get", "query", ...
├── data: any         // Operation-specific payload
└── id: RequestId     // For correlating responses
```

**Rationale:** Native targets (TypeScript/Python) can expose `OpStream<T>` as an `AsyncIterable<Responsum<T>>`. Poll-based targets (Zig/C++ and potentially Rust) expose an explicit stream object that the executor drives via `poll()`.

This keeps the _conceptual_ model identical across targets ("a stream of `Responsum`"), while allowing different execution mechanisms.

**Zig:**

```zig
const OpStream = struct {
    id: u64,
    vtable: *const OpStreamVTable,

    pub fn poll(self: *OpStream, ctx: *ExecutorContext) Responsum(ResultType) {
        return self.vtable.poll(self, ctx);
    }

    pub fn cancel(self: *OpStream, ctx: *ExecutorContext) void {
        return self.vtable.cancel(self, ctx);
    }
};

const OpStreamVTable = struct {
    poll: *const fn (*OpStream, *ExecutorContext) Responsum(ResultType),
    cancel: *const fn (*OpStream, *ExecutorContext) void,
};

const Handle = struct {
    id: u64,
    handle_type: HandleType,
    vtable: *const HandleVTable,

    pub fn exec(self: *Handle, msg: Message) OpStream {
        return self.vtable.exec(self, msg);
    }

    pub fn close(self: *Handle) void {
        self.vtable.close(self);
    }
};

const HandleVTable = struct {
    exec: *const fn (*Handle, Message) OpStream,
    close: *const fn (*Handle) void,
};
```

### 3. Dispatcher (Syscall Table)

Routes `ad` calls to handlers based on target pattern.

**Streaming syscalls:** For streaming targets (e.g. `caelum:websocket`), the handler yields `.item(T)` values and terminates with `.done` or `.err`.

```
┌──────────────────────┬─────────────────────┬─────────────────┐
│ Pattern              │ Handler             │ Returns         │
├──────────────────────┼─────────────────────┼─────────────────┤
│ fasciculus:lege      │ FileReadHandler     │ Responsum<text> │
│ fasciculus:scribe    │ FileWriteHandler    │ Responsum<void> │
│ caelum:request       │ HttpHandler         │ Responsum<Resp> │
│ caelum:websocket     │ WebSocketHandler    │ Responsum<Message> │
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

**Dynamic targets:** When the syscall target is not compile-time known (computed string, user input), Zig cannot use comptime dispatch. In that case, codegen uses a small runtime dispatch table (string comparisons) and yields `.err` for unknown syscalls.

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

**Uniqueness scope:** `RequestId` is required to be unique within a single executor instance. Native targets may use UUIDs; native-code targets may use a monotonic 64-bit counter (optionally prefixed with an executor ID if multiple executors are introduced).

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
            .err => |e| return error.ResponsumError,
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
            case 'ok':
                return resp.data;
            case 'err':
                throw new ResponsumError(resp.error);
            case 'pending':
                continue; // implicit in async iteration
            case 'item':
                continue; // collect or ignore
            case 'done':
                throw new Error('Unexpected done in single-value context');
        }
    }
    throw new Error('Generator ended without result');
}
```

## Why Protocol on All Targets

The Responsum protocol is not overhead—it's the unifying abstraction. Monk OS implements Response protocol in TypeScript for reasons that apply equally to Faber.

### 1. Uniform Dispatch

Every syscall conceptually produces a stream of `Responsum<T>`, regardless of semantics.

- TypeScript/Python expose this as `AsyncIterable<Responsum<T>>`.
- Zig/C++ expose this as `OpStream<T>` driven by `poll()`.

Example (TypeScript syntax):

```typescript
// Single value
async function* fileOpen(...): AsyncIterable<Responsum<Fd>> {
    yield respond.ok({ fd: 3 });
}

// Stream
async function* readDir(...): AsyncIterable<Responsum<Entry>> {
    yield respond.item({ name: 'file1.txt' });
    yield respond.item({ name: 'file2.txt' });
    yield respond.done();
}

// Error
async function* fileStat(...): AsyncIterable<Responsum<Stat>> {
    yield respond.error('ENOENT', 'File not found');
}
```

The `ad` dispatcher doesn't need to know if a syscall is single-value, stream, sync, or async. It just routes and yields. Without protocol, dispatch becomes type chaos.

### 2. Terminal vs Non-Terminal is Explicit

Native generators don't distinguish "here's an item" from "I'm done":

```typescript
// Native - done is implicit (just stops iterating)
async function* items(): AsyncIterable<Item> {
    yield item1;
    yield item2;
}

// Protocol - done is explicit
async function* items(): AsyncIterable<Responsum<Item>> {
    yield respond.item(item1);
    yield respond.item(item2);
    yield respond.done(); // Explicit terminal signal
}
```

This matters for:

- **Backpressure** — Executor knows stream ended vs stalled
- **Cleanup** — Terminal op triggers resource release
- **Partial errors** — `.err` after `.item` is distinguishable from normal end

### 3. Errors Are Values, Not Exceptions

```typescript
// Without protocol - exceptions
async function fileRead(path: string): Promise<string> {
    if (!exists(path)) throw new Error('ENOENT'); // Exception
    return content;
}

// With protocol - errors are values
async function* fileRead(path: string): AsyncIterable<Responsum<string>> {
    if (!exists(path)) {
        yield respond.error('ENOENT', 'File not found'); // Value
        return;
    }
    yield respond.ok(content);
}
```

Why errors-as-values matters:

- **No exception unwinding** — Predictable control flow
- **Streaming errors** — Error after 100 items is just another yield
- **Cross-target consistency** — Same semantics on TS, Zig, Rust
- **Uniform logging** — Every response logged the same way

### 4. Cross-Target Consistency

Without protocol, targets diverge:

```typescript
// TS without protocol: exception
try {
    const content = await fileRead('missing.txt');
} catch (e) {
    /* error handling */
}
```

```zig
// Zig with protocol: Responsum.err
switch (future.poll(&ctx)) {
    .ok => |content| { ... },
    .err => |e| { ... },
}
```

Same Faber source, different runtime semantics. Bugs that only appear on one target.

With protocol everywhere, same semantics:

```typescript
// TS with protocol: matches Zig
for await (const resp of fileRead('missing.txt')) {
    switch (resp.op) {
        case 'ok': /* success - same as Zig .ok */
        case 'err': /* error - same as Zig .err */
    }
}
```

### 5. Observability and Cancellation

Every response is inspectable:

```typescript
for await (const resp of syscall('file:read', path)) {
    logger.log(resp.op, resp); // Uniform logging
    metrics.record(resp.op); // Uniform metrics
}
```

Cancellation is explicit via stream finalization + executor signals:

- Consumer breaks from loop → triggers `iterator.return()` (native targets)
- Executor sets a cancellation flag in context (poll-based targets)
- Streams are responsible for cleanup on scope exit (no `.cancelled` Responsum variant)

---

## Compilation Strategy

### TypeScript / Python

Use native async generators with Responsum protocol:

```fab
ad "fasciculus:lege" ("file.txt") fiet textus pro content { ... }
```

Becomes:

```typescript
const content = await run(dispatch('fasciculus:lege', 'file.txt'));
```

The `run()` helper unwraps Responsum, throwing on `.err`:

```typescript
async function run<T>(gen: AsyncIterable<Responsum<T>>): Promise<T> {
    for await (const resp of gen) {
        switch (resp.op) {
            case 'ok':
                return resp.data;
            case 'err':
                throw new ResponsumError(resp.error);
            case 'item':
                continue;
            case 'done':
                throw new Error('Unexpected done in single-value context');
            case 'pending':
                continue;
        }
    }
    throw new Error('Generator ended without result');
}
```

**Arrow binding (`->`) is the escape hatch, not the default:**

```fab
# Default: protocol (observable, consistent)
ad "fasciculus:lege" ("file.txt") fiet textus pro content

# Escape hatch: direct (fast, loses observability)
ad "fasciculus:lege" ("file.txt") -> textus pro content
```

Arrow binding bypasses protocol for performance-critical hot paths, but only works on TS/Python. Zig/Rust/C++ reject arrow binding with error P192.

Most code should use verb binding (`fiet`/`fiunt`/`fient`) for consistency and debuggability.

### Rust

Use native `async fn` with Responsum mapped to Rust idioms:

| Responsum  | Rust                               |
| ---------- | ---------------------------------- |
| `.ok(T)`   | `Poll::Ready(Ok(T))`               |
| `.err(E)`  | `Poll::Ready(Err(ResponsumError))` |
| `.pending` | `Poll::Pending`                    |
| `.item(T)` | `Some(Ok(T))` via Stream           |
| `.done`    | `None` via Stream                  |

```fab
functio fetch(textus url) fiet Response {
    ad "caelum:request" (url, "GET") fiet Response pro resp
    redde resp
}
```

Becomes:

```rust
pub async fn fetch(url: &str) -> Result<Response, ResponsumError> {
    let resp = caelum::request(url, "GET").await?;
    Ok(resp)
}
```

Rust's compiler generates state machines for `async fn`. No manual state machine needed. The `?` operator maps to Responsum error propagation.

For streaming (`fiunt`/`fient`):

```rust
pub fn items() -> impl Stream<Item = Result<Item, ResponsumError>> {
    async_stream::stream! {
        yield Ok(item1);
        yield Ok(item2);
    }
}
```

### Zig / C++

Compile to explicit state machines. Each `fiet`/`fient` function becomes:

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

| Namespace    | Domain   | Example Syscalls                    |
| ------------ | -------- | ----------------------------------- |
| `fasciculus` | Files    | `lege`, `scribe`, `tolle`, `movere` |
| `caelum`     | Network  | `request`, `websocket`, `listen`    |
| `tempus`     | Time     | `dormi`, `nunc`, `intervallum`      |
| `memoria`    | Memory   | `alloca`, `libera` (Zig/Rust/C++)   |
| `processus`  | Process  | `genera`, `occide`, `expecta`       |
| `canalis`    | Channels | `crea`, `mitte`, `recipe`           |
| `aleator`    | Random   | `numerus`, `bytes`, `uuid`          |
| `crypto`     | Crypto   | `hash`, `signa`, `verifica`         |

## Backpressure

For streaming syscalls (`fiunt`/`fient`), the executor implements flow control.

- Native async targets: backpressure is naturally enforced by consumer-driven iteration (the producer runs when polled).
- Poll-based targets: the executor may buffer items, so explicit high/low watermarks apply.

Default constants:

| Constant             | Value  | Purpose                 |
| -------------------- | ------ | ----------------------- |
| `AQUA_ALTA`          | 1000   | Pause producing         |
| `AQUA_BASSA`         | 100    | Resume producing        |
| `PULSUS_INTERVALLUM` | 100ms  | Consumer liveness check |
| `MORA_MAXIMUM`       | 5000ms | Abort if consumer dead  |

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

| Document       | Relationship                                               |
| -------------- | ---------------------------------------------------------- |
| `ad.md`        | Nucleus provides the runtime that `ad` dispatches to       |
| `zig-async.md` | State machine compilation is one strategy within Nucleus   |
| `flumina.md`   | Responsum protocol originated here; Nucleus generalizes it |
| `two-pass.md`  | Liveness analysis feeds into state machine generation      |

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
yield respond.item(item1); // Consumer sees this
yield respond.item(item2); // Consumer sees this
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

| Monk Complexity                      | Faber Simplification                 |
| ------------------------------------ | ------------------------------------ |
| Message serialization across Workers | Direct function calls                |
| Per-stream ping timers               | Language-native yield/suspend        |
| Virtual process validation           | Trust OS process isolation           |
| UUID request IDs                     | 64-bit counter (native targets)      |
| Runtime syscall dispatch             | Compile-time dispatch (Zig comptime) |
| Per-syscall auth checking            | Entry-point or compile-time auth     |

### HAL Device Model

Monk's 17-device HAL is worth adopting selectively:

| Device    | Faber Equivalent     | Priority |
| --------- | -------------------- | -------- |
| `block`   | `fasciculus` (files) | High     |
| `network` | `caelum` (network)   | High     |
| `timer`   | `tempus`             | High     |
| `entropy` | `aleator`            | Medium   |
| `crypto`  | `crypto`             | Medium   |
| `storage` | EMS-style (future)   | Low      |
| `console` | `scriba`             | High     |

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
        exi  # Break from stream (triggers cleanup)
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

**Decision**: Opt-in via arrow binding (`->`), consistent with `functio` syntax.

```fab
# Standard: via protocol (observable, testable)
ad "fasciculus:lege" ("file.txt") fiet textus pro content

# Direct: bypass protocol (fast, less observable)
ad "fasciculus:lege" ("file.txt") -> textus pro content
```

**Rationale**: Reuses existing arrow vs verb distinction from function declarations. No new syntax needed.

**Trade-offs**:

- Arrow bypasses logging, metrics, mocking
- Arrow only works on TS/Python (compile error on Zig/Rust/C++)
- Verb binding works on all targets

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
# Auth middleware wraps entire handler
@requiresAuth
functio protectedEndpoint() fiet Response {
    # Auth already validated
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
