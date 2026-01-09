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

---

## Core Insight: Async Generators as the Primitive

**The fundamental design principle**: Async generators are the primitive. Everything else—promises, blocking calls, collected results—derives from streaming.

This inverts the typical mental model where sync is default and async is added. In Nucleus:

```
Async Generator (primitive)
       │
       ├── fient/fiunt → raw stream, yields .item repeatedly, ends with .done
       │
       ├── fiet        → collect stream into single value, return .ok
       │
       └── fit         → block until .ok, unwrap and return raw value
```

### Why Streaming First?

1. **I/O is naturally streaming** — Files read in chunks, networks deliver packets, databases return rows
2. **Memory efficiency** — Process items as they arrive, don't buffer everything
3. **Unified model** — Single primitive handles all async patterns
4. **Backpressure built-in** — Consumer controls pace via poll frequency
5. **Buffer management** — Base streaming impl uses fixed internal buffers; derived batch forms add allocation on top

**Memory efficiency example:**

```fab
# Streaming: constant memory regardless of file size
ex solum.legens("huge.log") pro chunk {
    si chunk.continet("ERROR") {
        scribe chunk
    }
}

# Batch: loads entire file into memory
fixum content = solum.lege("huge.log")  # OOM on large files
```

**Zig 0.15 alignment:** The async-generator-first approach naturally fits Zig 0.15's buffer-based I/O APIs ("Writergate"):

```zig
// Old (pre-0.15): allocation-based
const line = reader.readUntilDelimiterAlloc(alloc, '\n', 4096);

// New (0.15): buffer-based
var buf: [4096]u8 = undefined;
var r = file.reader(&buf);
const line = r.interface.takeDelimiter('\n');  // Returns slice into buf
```

Base `legens` uses fixed buffers (matches 0.15 API). Derived `leget`/`lege` add allocation on top.

### The Derivation Chain

Every I/O operation follows this pattern:

```
stream()           # primitive: yields items over time
    ↓ collect
future()           # derived: collect all items → single value (Promise)
    ↓ block_on
sync()             # derived: await future → unwrapped value (blocking)
```

### The Inversion

Traditional approach (sync-first):
```
lege (sync)           ← Base implementation
    ↓ wrap in Promise
leget (async)         ← Derived
    ↓ add chunking
legens (streaming)    ← Derived (complex)
```

Async-generator-first approach:
```
legens (async generator)    ← Base implementation
    ↓ collect stream
leget (async batch)         ← Derived (simple)
    ↓ block until complete
lege (sync batch)           ← Derived (simple)
```

Streaming is strictly more general. You can always derive batch from stream (iterate and collect), but you cannot derive stream from batch without reimplementing.

### Scope: What This Applies To

This pattern applies to **I/O-bound** stdlib types:

| Type       | Base Form       | Meaning                    |
|------------|-----------------|----------------------------|
| **solum**  | `legens`        | Stream file chunks         |
| **solum**  | `scribens`      | Stream writes              |
| **caelum** | `petens`        | Stream HTTP response       |
| **caelum** | `auscultans`    | Stream WebSocket messages  |
| **arca**   | `quaerens`      | Stream query results       |
| **nucleus**| `accipiens`     | Stream IPC messages        |

Does **not** apply to:
- In-memory collections (`lista`, `tabula`, `copia`) — sync is natural base
- Pure computation (`mathesis`, `tempus`) — no I/O involved

---

## Latin Verb Conjugation Model

The Latin verb conjugation **encodes async semantics**. This isn't arbitrary naming—it maps grammatical meaning to execution model:

| Conjugation | Latin Form | Meaning | Async Semantics | Responsum Pattern |
|-------------|------------|---------|-----------------|-------------------|
| `legens` | participium praesens | "reading" (ongoing) | streaming generator | `.item*` → `.done` |
| `leget` | futurum indicativum | "will read" (future) | async/promise | `.pending*` → `.ok` |
| `lege` | imperativus | "read!" (command) | sync/blocking | `block_on(leget)` |

### Grammar-to-Execution Mapping

| Verb Form | Grammar | Execution | Return Type |
|-----------|---------|-----------|-------------|
| Present participle (`-ens`, `-ans`) | Ongoing action | Async generator | `cursor<T>` / `OpStream<T>` |
| Future indicative (`-et`, `-it`) | Will complete | Promise/Future | `futura<T>` / `Future<T>` |
| Imperative (`-e`, `-a`) | Do it now | Blocking call | `T` directly |

### Concrete Example: File Reading

From `fons/norma/solum.fab`:

```fab
# Stream file chunks (base implementation - the primitive)
@ radix leg, participium_praesens
functio legens(textus path) -> cursor<octeti>

# Async batch read (derived: collect stream)
@ radix leg, futurum_indicativum
@ futura
functio leget(textus path) -> textus

# Sync batch read (derived: block + collect)
@ radix leg, imperativus
functio lege(textus path) -> textus
```

The derivation is explicit:
- `legens()` → primitive, yields chunks
- `leget()` = collect(`legens()`) → single value, async
- `lege()` = block_on(`leget()`) → single value, sync

---

## Stdlib Examples

### File I/O (`solum.fab`)

```fab
# READING
functio legens(textus path) -> cursor<octeti>      # stream chunks
functio leget(textus path) -> textus               # async, collected
functio lege(textus path) -> textus                # sync, blocking

# WRITING
functio scribens(textus path) -> cursor<octeti>    # stream writes
functio scribet(textus path, textus data) -> vacuum  # async batch
functio inscribe(textus path, textus data) -> vacuum # sync batch

# STDIN
functio ausculta() -> cursor<textus>               # stream stdin lines
functio hauri() -> textus                          # read all stdin

# DIRECTORY
functio ambula(textus path) -> cursor<textus>      # walk tree (always streaming)
functio elenca(textus path) -> lista<textus>       # list directory
```

### Network I/O (`caelum.fab`)

```fab
# HTTP CLIENT
functio pete(textus url) -> Replicatio             # GET (async)
functio mitte(textus url, textus corpus) -> Replicatio  # POST
functio pone(textus url, textus corpus) -> Replicatio   # PUT
functio dele(textus url) -> Replicatio             # DELETE
functio roga(textus modus, textus url, tabula<textus, textus> capita, textus corpus) -> Replicatio

# HTTP SERVER
functio exspecta((Rogatio) -> Replicatio handler, numerus portus) -> Servitor
```

### Database (`arca.fab`)

```fab
pactum Connexio {
    # Read returns cursor - streams rows (the primitive!)
    @ futura
    functio lege(textus sql, lista<quidlibet> params) -> cursor<series>

    # Write returns count (single value)
    @ futura
    functio muta(textus sql, lista<quidlibet> params) -> numerus

    @ futura
    functio incipe() -> Transactio
    functio claude() -> vacuum
}
```

Database queries return `cursor<series>` — a stream of rows. The caller chooses how to consume:

```fab
# Stream rows one at a time (memory efficient for large results)
ex connexio.lege("SELECT * FROM users", []) pro row {
    scribe row.nomen
}

# Collect all rows (convenience for small results)
fixum users = collige(connexio.lege("SELECT * FROM users", []))
```

---

## Responsum Protocol

All I/O operations return `Responsum<T>` — a tagged union describing the result:

```
┌─────────────┬──────────────────────────────────────────┐
│ Variant     │ Meaning                                  │
├─────────────┼──────────────────────────────────────────┤
│ .pending    │ Operation in progress, poll again        │
│ .ok(T)      │ Single value, terminal (futures)         │
│ .item(T)    │ One of many values, non-terminal (streams) │
│ .done       │ Stream complete, terminal                │
│ .err(E)     │ Error, terminal                          │
└─────────────┴──────────────────────────────────────────┘
```

### Protocol Invariants

- **Single-value operation**: `.pending* -> (.ok | .err)`
- **Streaming operation**: `.pending* -> (.item | .pending)* -> (.done | .err)`
- `.item` MUST NOT appear after a terminal variant
- `.done` and `.err` are terminal variants

### Why Protocol Everywhere?

The Responsum protocol is not overhead—it's the unifying abstraction:

1. **Uniform dispatch** — Every syscall produces a stream of `Responsum<T>`
2. **Terminal is explicit** — `.done` vs implicit generator end
3. **Errors are values** — No exception unwinding, streaming errors possible
4. **Cross-target consistency** — Same semantics on all targets
5. **Observability** — Every response is inspectable/loggable

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Faber Source Code                                          │
│  solum.legens("file.txt") fient octeti pro chunk { ... }    │
├─────────────────────────────────────────────────────────────┤
│  Compiler (semantic analysis, codegen)                      │
│  ├── Verb detection (fit/fiet/fiunt/fient)                  │
│  ├── State machine generation (Zig/Rust/C++)                │
│  └── Native async emission (TS/Python)                      │
├─────────────────────────────────────────────────────────────┤
│  Nucleus Runtime (per-target implementation)                │
│  ├── Responsum<T> protocol types                            │
│  ├── OpStream (async generator interface)                   │
│  ├── Executor (drives state machines)                       │
│  ├── Collectors (stream → single value)                     │
│  └── Syscall handlers (solum, caelum, arca, etc.)           │
├─────────────────────────────────────────────────────────────┤
│  Target Runtime (Bun, Python, Zig std, etc.)                │
└─────────────────────────────────────────────────────────────┘
```

---

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

### 4. Dispatcher (Syscall Table)

Routes calls to handlers based on namespace:

| Namespace    | Domain   | Example Operations                  |
| ------------ | -------- | ----------------------------------- |
| `solum`      | Files    | `lege`, `scribe`, `ambula`          |
| `caelum`     | Network  | `pete`, `mitte`, `exspecta`         |
| `arca`       | Database | `lege`, `muta`, `incipe`            |
| `tempus`     | Time     | `dormi`, `nunc`, `intervallum`      |
| `memoria`    | Memory   | `alloca`, `libera` (Zig/Rust/C++)   |
| `processus`  | Process  | `genera`, `occide`, `expecta`       |
| `canalis`    | Channels | `crea`, `mitte`, `recipe`           |
| `aleator`    | Random   | `numerus`, `octeti`, `uuid`         |
| `crypto`     | Crypto   | `hash`, `signa`, `verifica`         |

**Compile-time dispatch (Zig):**

```zig
pub fn dispatch(comptime namespace: []const u8, comptime op: []const u8, args: anytype) OpStream(ReturnType(namespace, op)) {
    if (comptime std.mem.eql(u8, namespace, "solum")) {
        return solum.dispatch(op, args);
    } else if (comptime std.mem.eql(u8, namespace, "caelum")) {
        return caelum.dispatch(op, args);
    } else {
        @compileError("Unknown namespace: " ++ namespace);
    }
}
```

### 5. Request Correlation

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

## Error Handling

Errors flow through Responsum as values:

```fab
ex solum.legens("missing.txt") pro chunk {
    scribe chunk
} cape err {
    scribe "Error: " + err.message
}
```

**Why errors-as-values:**
- No exception unwinding
- Streaming errors after partial success
- Cross-target consistency
- Uniform logging

---

## Implementation Plan

### Phase 1: Protocol Foundation

1. Define `Responsum<T>` for all targets in preamble
2. Define `OpStream<T>` interface per target
3. Implement basic `Executor` with `run()` and `collect()`

### Phase 2: TypeScript Runtime

1. Use native `AsyncIterable<Responsum<T>>` for OpStream
2. Implement `run()`, `collect()` as async functions
3. Wire solum/caelum handlers using Node.js APIs

### Phase 3: Zig Runtime

1. Implement `Responsum(T)` union
2. Implement `OpStream` vtable pattern
3. Implement `ExecutorContext` with blocking I/O (v1)
4. State machine codegen for `fiet`/`fient` functions
5. Implement solum handlers using `std.fs`
6. Implement caelum handlers using `std.http`

### Phase 4: Async I/O (Zig)

1. Integrate io_uring or epoll for non-blocking I/O
2. Implement proper `.pending` handling with I/O multiplexing
3. Add request correlation for concurrent operations

### Phase 5: Other Targets

1. Python — Similar to TypeScript (native async)
2. Rust — State machines like Zig, or native async
3. C++ — Coroutines (C++20) or callback-based

---

## Target-Specific Design

### TypeScript

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

### Python

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

### Zig

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

// Executor
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

pub fn run(comptime T: type, stream: *OpStream(T), ctx: *ExecutorContext) !T {
    while (true) {
        switch (stream.poll(ctx)) {
            .pending => ctx.wait_for_io(),
            .ok => |value| return value,
            .err => |e| return error.ResponsumError,
            .item, .done => unreachable,
        }
    }
}
```

**State machine codegen example:**

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

**Subsidia structure:**

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

**Detailed ChunkIterator (base streaming impl):**

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

**LegetFuture (stream → async batch derivation):**

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

**Sync can also block on Future (alternative to direct iteration):**

```zig
pub fn lege_via_future(alloc: Allocator, path: []const u8) ![]u8 {
    var future = leget(alloc, path);
    return block_on([]u8, &future);
}

fn block_on(comptime T: type, future: anytype) !T {
    while (true) {
        switch (future.poll()) {
            .pending => {}, // busy-wait or yield
            .ok => |v| return v,
            .err => return error.IoError,
            else => unreachable,
        }
    }
}
```

Direct iteration (first approach) avoids Future overhead for sync use cases.

---

### Rust

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

### C++

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

## Design Decisions

### 1. Async Generators as Primitive

**Decision:** All I/O fundamentally returns `OpStream<T>`. Futures and sync calls are derived.

**Rationale:**
- Matches reality (I/O is streaming)
- Memory efficient (no mandatory buffering)
- Unified model (one abstraction for all patterns)

### 2. Latin Verb Conjugation

**Decision:** Use grammatical forms to encode execution semantics.

**Rationale:**
- Self-documenting (grammar implies behavior)
- Consistent across all operations
- Leverages existing Faber Latin design

### 3. Responsum Protocol Everywhere

**Decision:** Use protocol on all targets, not just poll-based.

**Rationale:**
- Cross-target consistency
- Uniform observability
- Errors as values

### 4. Allocator Threading (Zig)

**Decision:** Per-request allocator, inherited from parent.

**Rationale:** Avoids global lock contention, matches per-stream isolation.

### 5. Cancellation

**Decision:** Out-of-band signal, not Responsum variant.

**Rationale:** Adding `.cancelled` complicates every switch. Separate flag is cleaner.

### 6. Nested Composition

**Decision:** Inline for small futures (< 256 bytes), pointer for large.

**Rationale:** Inline avoids allocation for common case; pointer prevents bloat.

### 7. User-Defined Syscalls

**Decision:** Not in v1. Compile-time syscall table only.

**Rationale:** Runtime registration requires type erasure and has security concerns.

---

## Arrow Binding Escape Hatch

For performance-critical paths on native async targets, arrow binding bypasses the Responsum protocol:

```fab
# Default: via protocol (observable, consistent)
solum.leget("file.txt") fiet textus pro content

# Escape hatch: direct (fast, loses observability)
solum.leget("file.txt") -> textus pro content
```

**Trade-offs:**
- Arrow bypasses logging, metrics, mocking
- Arrow only works on TS/Python (compile error P192 on Zig/Rust/C++)
- Verb binding works on all targets

**Rationale:** Reuses existing arrow vs verb distinction from function declarations. No new syntax needed. Most code should use verb binding for consistency and debuggability.

---

## Why Protocol on All Targets (Detailed)

### 1. Uniform Dispatch

Every syscall conceptually produces a stream of `Responsum<T>`, regardless of semantics.

- TypeScript/Python expose this as `AsyncIterable<Responsum<T>>`
- Zig/C++ expose this as `OpStream<T>` driven by `poll()`

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

The dispatcher doesn't need to know if a syscall is single-value, stream, sync, or async. It just routes and yields. Without protocol, dispatch becomes type chaos.

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

---

## HAL Device Model

Monk's 17-device HAL is worth adopting selectively:

| Device    | Faber Equivalent     | Priority |
| --------- | -------------------- | -------- |
| `block`   | `solum` (files)      | High     |
| `network` | `caelum` (network)   | High     |
| `timer`   | `tempus`             | High     |
| `entropy` | `aleator`            | Medium   |
| `crypto`  | `crypto`             | Medium   |
| `storage` | EMS-style (future)   | Low      |
| `console` | `scriba`             | High     |

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

## Trade-offs

### Advantages

1. **Single implementation path** — Base streaming impl is authoritative
2. **Derived forms are trivial** — Just collect or block
3. **Memory control** — Users choose streaming vs batch based on needs
4. **Zig 0.15 fit** — Matches buffer-based API model naturally
5. **Cross-target consistency** — Same semantics everywhere

### Disadvantages

1. **Sync has overhead** — Even `lege` goes through iterator machinery
2. **Simple cases verbose** — Reading a small config file requires allocator
3. **Implementation complexity** — Base streaming impl is more complex than naive sync

### Mitigation

For the "simple case overhead" concern, consider convenience wrappers:

```fab
# In user code, for small files:
fixum config = solum.lege("config.json", alloc)

# Sugar for truly simple cases (uses temp allocator internally):
fixum config = solum.legeBrevis("config.json")  # "read briefly"
```

But this adds API surface. The primary recommendation is: accept the allocator parameter, trust the derived forms are efficient enough.

---

## Open Questions

1. **Error context enrichment** — Should `.err` include stack trace? Performance vs debuggability.

2. **Streaming timeout** — Per-operation or global? Syntax for specifying?

3. **Future size threshold** — 256 bytes is arbitrary. Profile real-world futures.

4. **Cross-target ID consistency** — TS uses UUID, Zig uses counter. Acceptable divergence?

5. **Partial error semantics** — Consumer sees 100 items then `.err`. How to handle?

6. **Convenience wrappers** — Is `legeBrevis` worth the API surface, or just accept allocator params?

---

## References

- `ad.md` — Dispatch syntax design
- `zig-async.md` — Zig-specific state machine details
- `flumina.md` — Original Responsum protocol (TypeScript)
- `two-pass.md` — Semantic analysis for liveness
- `fons/norma/solum.fab` — File I/O stdlib definitions
- `fons/norma/caelum.fab` — Network I/O stdlib definitions
- `fons/norma/arca.fab` — Database stdlib definitions
