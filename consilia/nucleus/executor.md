# Executor and Runtime Architecture

The Nucleus runtime consists of several core components that work together to execute async operations.

## Core Components

### 1. OpStream (Async Generator Interface)

The primitive type representing an async generator:

**Conceptual interface:**

```
OpStream<T>
├── poll(ctx) → Responsum<T>    # Get next item or status
└── cancel(ctx) → void          # Request cancellation (optional)
```

**Native targets (TS/Python):**

```typescript
type OpStream<T> = AsyncIterable<Responsum<T>>;
```

**Poll-based targets (Zig/Rust/C++):**

```zig
const OpStream = struct {
    ptr: *anyopaque,
    vtable: *const OpStreamVTable,

    pub fn poll(self: *OpStream, ctx: *ExecutorContext) Responsum(T) {
        return self.vtable.poll(self.ptr, ctx);
    }

    pub fn cancel(self: *OpStream, ctx: *ExecutorContext) void {
        self.vtable.cancel(self.ptr, ctx);
    }
};
```

### 2. Executor

Drives state machines and manages I/O:

**Core operations:**

```
Executor
├── run(stream) → T              # Poll until .ok, return value
├── collect(stream) → []T        # Poll until .done, collect items
├── block_on(future) → T         # Sync wrapper around run()
└── for_each(stream, fn) → void  # Poll and call fn on each .item
```

**Zig implementation:**

```zig
pub fn run(comptime T: type, stream: *OpStream(T), ctx: *ExecutorContext) !T {
    while (true) {
        switch (stream.poll(ctx)) {
            .pending => ctx.wait_for_io(),
            .ok => |value| return value,
            .err => |e| return error.ResponsumError,
            .item, .done => unreachable, // single-value context
        }
    }
}

pub fn collect(comptime T: type, stream: *OpStream(T), ctx: *ExecutorContext, allocator: Allocator) ![]T {
    var list = std.ArrayList(T).init(allocator);
    while (true) {
        switch (stream.poll(ctx)) {
            .pending => ctx.wait_for_io(),
            .item => |v| try list.append(v),
            .done => return list.toOwnedSlice(),
            .err => |e| return error.ResponsumError,
            .ok => unreachable, // streaming context
        }
    }
}
```

### 3. Handle Abstraction

Unified interface for all I/O sources:

```
Handle
├── id: HandleId
├── type: HandleType (file, socket, channel, timer, ...)
├── exec(msg: Message) → OpStream<T>
└── close() → void
```

**Zig vtable pattern:**

```zig
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

### 4. Request Correlation

For concurrent operations, every request gets an ID:

```
Request
├── id: u64                              # Unique within executor
├── namespace: "caelum"
├── op: "pete"
└── args: ("https://api.example.com",)

Response
├── id: u64                              # Matches request
└── result: Responsum<T>
```

**ID generation:**
- Native targets (TS/Python): UUID acceptable
- Poll-based targets (Zig/Rust/C++): 64-bit monotonic counter

---

## Backpressure

For streaming operations, the executor implements flow control:

- **Native async targets**: Backpressure is naturally enforced by consumer-driven iteration
- **Poll-based targets**: Explicit high/low watermarks

| Constant             | Value  | Purpose                 |
| -------------------- | ------ | ----------------------- |
| `AQUA_ALTA`          | 1000   | Pause producing         |
| `AQUA_BASSA`         | 100    | Resume producing        |
| `PULSUS_INTERVALLUM` | 100ms  | Consumer liveness check |
| `MORA_MAXIMUM`       | 5000ms | Abort if consumer dead  |

(Latin: aqua alta = high water, aqua bassa = low water)

---

## Executor Context

```zig
const ExecutorContext = struct {
    allocator: Allocator,
    cancelled: bool = false,
    // For async I/O (Phase 4):
    // io_context: *IoContext,
    // pending_requests: std.AutoHashMap(u64, *anyopaque),

    pub fn wait_for_io(self: *ExecutorContext) void {
        // Phase 1: blocking (no-op, I/O is sync)
        // Phase 4: epoll_wait / io_uring_wait
    }
};
```

---

## Concerns and Open Questions

### Unbounded Collection

**Problem**: The `collect()` function allocates unbounded memory. If a stream produces millions of items, `collect()` will allocate a massive slice. This can OOM.

**Considerations:**
- Should `collect()` accept a max item count? `collect(stream, ctx, allocator, max_items: 10000)`
- Should there be a separate `collectLimited()` variant?
- Should the user be warned (via documentation) that `collect()` is unsafe for unbounded streams?

### Error Detail Loss

**Problem**: `collect()` returns generic `error.ResponsumError`, discarding the actual error code and message. Caller can't distinguish "file not found" from "permission denied".

**Options:**
- Return `ResponsumError` struct with full context?
- Thread Zig error unions through (`!T` where T can be another error union)?
- Keep it simple and require caller to catch errors before collection?

### Cancellation During Wait

**Problem**: If `ctx.wait_for_io()` blocks indefinitely and user wants to cancel, how? The function signature doesn't support cancellation tokens.

**Impact:**
- Long-running streams can't be interrupted gracefully
- Ctrl+C during `wait_for_io()` may leak resources
- Need to clarify: Is cancellation in-scope for v1 Nucleus, or deferred to later?

### Concurrent Executor Instances

**Problem**: If multiple threads each have their own `ExecutorContext`, can they safely coexist? The design doc doesn't specify thread-safety guarantees.

**Questions:**
- Is `ExecutorContext` thread-local only?
- Can futures be transferred between executors?
- What if syscall handlers use shared global state (e.g., file descriptor table)?

### Allocator Threading (Resolved)

**Decision**: Use existing `cura`/`curator` mechanism. No new Nucleus-specific design needed.

**Rationale**: Faber already solves allocator threading at the language level:
- `cura` blocks push allocators onto a stack for implicit injection
- `curator` function suffix binds an allocator to a function without polluting its signature

Nucleus doesn't need its own mechanism—it inherits whatever allocator is in scope via the existing stack injection model.

**Codegen per target:**
- **Zig/Rust/C++**: Syscall implementations access the current allocator from the `cura` stack (thread-local or context pointer)
- **TS/Python**: Ignored entirely; GC handles it

The "per-request allocator, inherited from parent" statement describes the *semantic guarantee*, not a new mechanism. The mechanism is `cura`.

### Backpressure Concerns

**Watermark granularity**: The constants above are item-count based. For streams where items vary widely in size (e.g., file chunks: 4KB vs 4MB), item count is a poor proxy for memory pressure.

**Consumer liveness detection failure modes**: The 100ms ping interval and 5s timeout assume low-latency networks. In high-latency or lossy networks:
- Ping may arrive late, falsely triggering timeout
- Consumer may be alive but slow-processing (e.g., writing to slow disk)
- Spurious timeouts abort streams prematurely

**Cross-executor backpressure**: If producer runs in executor A and consumer runs in executor B (multi-threaded), how is backpressure signaled? The design assumes single-executor model.

**Native async backpressure assumptions**: The doc states "Backpressure is naturally enforced by consumer-driven iteration" for TS/Python. But what if the consumer dispatches to a queue?

```typescript
for await (const item of stream) {
    queue.add(() => slowOperation(item));  // Non-blocking
}
```

The queue fills unbounded. "Natural backpressure" doesn't apply.

**Backpressure bypass**: What if user code manually calls `poll()` in a tight loop, ignoring `.pending`? The watermark system is advisory, not enforced. Malicious or buggy code can OOM the system.

---

## Design Decisions

### 1. Cancellation

**Decision:** Out-of-band signal, not Responsum variant.

**Rationale:** Adding `.cancelled` complicates every switch. Separate flag is cleaner.

### 2. Nested Composition

**Decision:** Inline for small futures (< 256 bytes), pointer for large.

**Rationale:** Inline avoids allocation for common case; pointer prevents bloat.

### 3. Allocator Threading (Zig)

**Decision:** Per-request allocator, inherited from parent.

**Rationale:** Avoids global lock contention, matches per-stream isolation.

### 4. User-Defined Syscalls

**Decision:** Not in v1. Compile-time syscall table only.

**Rationale:** Runtime registration requires type erasure and has security concerns.
