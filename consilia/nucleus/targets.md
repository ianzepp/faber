# Target-Specific Implementations

Each compilation target has its own Nucleus implementation, adapted to the target's async model.

## TypeScript

**Approach:** Native async generators with Responsum protocol.

```typescript
type Responsum<T> =
    | { op: 'pending' }
    | { op: 'ok'; data: T }
    | { op: 'item'; data: T }
    | { op: 'done' }
    | { op: 'err'; error: ResponsumError };

type OpStream<T> = AsyncIterable<Responsum<T>>;

// Executor
async function run<T>(stream: OpStream<T>): Promise<T> {
    for await (const resp of stream) {
        switch (resp.op) {
            case 'ok': return resp.data;
            case 'err': throw new ResponsumError(resp.error);
            case 'pending': continue;
            case 'item': continue; // unexpected in single-value context
            case 'done': throw new Error('Unexpected done');
        }
    }
    throw new Error('Stream ended without result');
}

async function collect<T>(stream: OpStream<T>): Promise<T[]> {
    const items: T[] = [];
    for await (const resp of stream) {
        switch (resp.op) {
            case 'item': items.push(resp.data); break;
            case 'done': return items;
            case 'err': throw new ResponsumError(resp.error);
            case 'pending': continue;
            case 'ok': throw new Error('Unexpected ok in stream');
        }
    }
    throw new Error('Stream ended without done');
}
```

**Handler example (solum.legens):**

```typescript
async function* legens(path: string): OpStream<Uint8Array> {
    const stream = fs.createReadStream(path);
    for await (const chunk of stream) {
        yield { op: 'item', data: chunk };
    }
    yield { op: 'done' };
}
```

---

## Python

**Approach:** Native async generators, similar to TypeScript.

```python
from typing import TypeVar, Union, AsyncIterator
from dataclasses import dataclass

T = TypeVar('T')

@dataclass
class Ok:
    data: T

@dataclass
class Item:
    data: T

@dataclass
class Done:
    pass

@dataclass
class Err:
    code: str
    message: str

Responsum = Union[Ok, Item, Done, Err]
OpStream = AsyncIterator[Responsum]

async def run(stream: OpStream[T]) -> T:
    async for resp in stream:
        match resp:
            case Ok(data): return data
            case Err(code, message): raise ResponsumError(code, message)
            case _: continue
    raise RuntimeError("Stream ended without result")
```

---

## Zig

**Approach:** Explicit state machines, poll-based execution.

```zig
const ResponsumError = struct {
    code: []const u8,
    message: []const u8,
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

// OpStream via vtable
const OpStreamVTable = struct {
    poll: *const fn (*anyopaque, *ExecutorContext) Responsum(anytype),
    cancel: *const fn (*anyopaque, *ExecutorContext) void,
};

const OpStream = struct {
    ptr: *anyopaque,
    vtable: *const OpStreamVTable,

    pub fn poll(self: *OpStream, ctx: *ExecutorContext) Responsum(T) {
        return @call(.auto, self.vtable.poll, .{ self.ptr, ctx });
    }
};
```

### State Machine Codegen Example

```fab
functio fetch(textus url) fiet Replicatio {
    fixum resp = caelum.pete(url)
    redde resp
}
```

Compiles to:

```zig
const FetchState = union(enum) {
    start: struct { url: []const u8 },
    awaiting_pete: struct { inner: *caelum.PeteStream },
};

const FetchFuture = struct {
    state: FetchState,

    pub fn poll(self: *FetchFuture, ctx: *ExecutorContext) Responsum(Replicatio) {
        switch (self.state) {
            .start => |s| {
                const inner = caelum.pete(s.url);
                self.state = .{ .awaiting_pete = .{ .inner = inner } };
                return .pending;
            },
            .awaiting_pete => |s| {
                switch (s.inner.poll(ctx)) {
                    .pending => return .pending,
                    .ok => |resp| return .{ .ok = resp },
                    .err => |e| return .{ .err = e },
                    else => unreachable,
                }
            },
        }
    }
};
```

### Subsidia Structure

```
fons/subsidia/zig/
├── responsum.zig      # Responsum(T), ResponsumError
├── stream.zig         # OpStream, OpStreamVTable
├── executor.zig       # ExecutorContext, run(), collect()
├── solum.zig          # File I/O handlers
│   ├── legens()       → OpStream([]u8)
│   ├── leget()        → Future([]u8)
│   └── lege()         → []u8
├── caelum.zig         # HTTP handlers
│   ├── pete()         → Future(Replicatio)
│   └── ...
└── mod.zig            # Re-exports
```

### Detailed ChunkIterator (Base Streaming Impl)

```zig
// fons/subsidia/zig/solum.zig

pub const ChunkIterator = struct {
    file: std.fs.File,
    buffer: [4096]u8,
    bytes_read: usize,
    done: bool,

    pub fn next(self: *ChunkIterator) ?[]const u8 {
        if (self.done) return null;

        self.bytes_read = self.file.read(&self.buffer) catch |err| {
            self.done = true;
            return null;
        };

        if (self.bytes_read == 0) {
            self.done = true;
            return null;
        }

        return self.buffer[0..self.bytes_read];
    }

    pub fn deinit(self: *ChunkIterator) void {
        self.file.close();
    }
};

// Base: streaming read (legens)
pub fn legens(path: []const u8) ChunkIterator {
    const file = std.fs.cwd().openFile(path, .{}) catch |err| {
        return ChunkIterator{ .file = undefined, .done = true, .buffer = undefined, .bytes_read = 0 };
    };
    return ChunkIterator{ .file = file, .done = false, .buffer = undefined, .bytes_read = 0 };
}

// Derived: sync batch read (lege) - iterates stream directly
pub fn lege(alloc: std.mem.Allocator, path: []const u8) ![]u8 {
    var iter = legens(path);
    defer iter.deinit();

    var result = std.ArrayList(u8).init(alloc);
    while (iter.next()) |chunk| {
        try result.appendSlice(chunk);
    }
    return result.toOwnedSlice();
}
```

### LegetFuture (Stream → Async Batch Derivation)

```zig
const LegetFuture = struct {
    state: union(enum) {
        iterating: struct {
            alloc: Allocator,
            iter: ChunkIterator,
            buffer: ArrayList(u8),
        },
        done: []u8,
        failed,
    },

    pub fn poll(self: *LegetFuture) Responsum([]u8) {
        switch (self.state) {
            .iterating => |*s| {
                // Non-blocking: process one chunk per poll
                if (s.iter.next()) |chunk| {
                    s.buffer.appendSlice(chunk) catch {
                        self.state = .failed;
                        return .{ .err = .{ .code = "ALLOC", .message = "allocation failed" } };
                    };
                    return .pending;
                }
                // Stream exhausted
                const result = s.buffer.toOwnedSlice();
                s.iter.deinit();
                self.state = .{ .done = result };
                return .{ .ok = result };
            },
            .done => |data| return .{ .ok = data },
            .failed => return .{ .err = .{ .code = "FAILED", .message = "operation failed" } },
        }
    }
};
```

### State Machine Generation Concerns

**Inner future lifetime**: In the example above, `inner: *caelum.PeteStream` is a pointer. Where is the inner future allocated?
- Stack allocation (but then pointer is invalid after function returns)
- Heap allocation (but who owns it? Need allocator threading)
- Inline the inner future struct (but then state size explodes)

**State transition atomicity**: If `poll()` modifies `self.state` and then calls inner `poll()`, but inner `poll()` panics or returns an error, the state is left in an intermediate position.

**Mutable variables across suspend points**: The example only shows `fixum` (const) variables. What if the code mutates a variable after a suspend?

```fab
functio example() fiet textus {
    varia x = 0
    fixum resp = cede fetch()
    x = x + 1  # mutation after suspend
    redde textatum(x)
}
```

The state struct must capture `x` by value in `awaiting_fetch`. After resume, `x = x + 1` must write back to the struct. Does the compiler:
- Emit explicit write-back code?
- Reject mutable locals across suspends?
- Capture mutable locals by pointer (requires allocator)?

**Multiple suspend points in same scope**: Consider:

```fab
functio multi() fiet textus {
    fixum a = cede fetchA()
    fixum b = cede fetchB()
    redde a + b
}
```

This requires three states: `start`, `awaiting_a`, `awaiting_b`. The `awaiting_b` state must capture both `a` (completed) and the inner future for `fetchB()`. State structs grow with each suspend point.

**Control flow suspend points**: What if `cede` appears inside a loop?

```fab
functio loop_fetch() fiet lista<textus> {
    varia results = []
    ex urls pro url {
        fixum resp = cede fetch(url)  # suspend inside loop
        results.adde(resp)
    }
    redde results
}
```

The state machine must capture loop iteration state (`url` iterator position, `results` accumulator). Significantly more complex than linear suspend points.

---

## Rust

**Approach:** Native async where possible, state machines for generators.

```rust
enum Responsum<T> {
    Pending,
    Ok(T),
    Item(T),
    Done,
    Err(ResponsumError),
}

// Futures: use native async
pub async fn leget(path: &str) -> Result<String, ResponsumError> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(content)
}

// Streams: use async-stream or manual impl
pub fn legens(path: &str) -> impl Stream<Item = Result<Vec<u8>, ResponsumError>> {
    async_stream::stream! {
        let file = tokio::fs::File::open(path).await?;
        let mut reader = tokio::io::BufReader::new(file);
        let mut buf = vec![0u8; 8192];
        loop {
            let n = reader.read(&mut buf).await?;
            if n == 0 { break; }
            yield Ok(buf[..n].to_vec());
        }
    }
}
```

**Responsum mapping to Rust idioms:**

| Responsum  | Rust                               |
| ---------- | ---------------------------------- |
| `.ok(T)`   | `Poll::Ready(Ok(T))`               |
| `.err(E)`  | `Poll::Ready(Err(ResponsumError))` |
| `.pending` | `Poll::Pending`                    |
| `.item(T)` | `Some(Ok(T))` via Stream           |
| `.done`    | `None` via Stream                  |

---

## C++

**Approach:** Coroutines (C++20) or callback-based fallback.

```cpp
// With C++20 coroutines
template<typename T>
struct Responsum {
    enum class Op { Pending, Ok, Item, Done, Err };
    Op op;
    std::optional<T> data;
    std::optional<ResponsumError> error;
};

// Generator using coroutines
template<typename T>
struct OpStream {
    struct promise_type { /* ... */ };

    Responsum<T> poll() {
        if (handle.done()) return {.op = Op::Done};
        handle.resume();
        return handle.promise().current;
    }
};

// Example handler
OpStream<std::vector<uint8_t>> legens(std::string_view path) {
    std::ifstream file(path, std::ios::binary);
    std::vector<uint8_t> buf(8192);
    while (file.read(reinterpret_cast<char*>(buf.data()), buf.size())) {
        co_yield Responsum<std::vector<uint8_t>>{.op = Op::Item, .data = buf};
    }
    co_return;
}
```

---

## Target Comparison

| Feature | TypeScript | Python | Zig | Rust | C++ |
|---------|------------|--------|-----|------|-----|
| Async model | Native generators | Native generators | Poll-based state machines | Native async/streams | Coroutines (C++20) |
| Responsum | JS object | Dataclass | Tagged union | Enum | Struct |
| Executor | `for await` | `async for` | Manual `poll()` loop | Tokio runtime | Manual resume |
| Allocation | GC | GC | Explicit allocator | Ownership | Manual/RAII |
| Complexity | Low | Low | High | Medium | High |
