# Async-Generator-First Design

For IO-bound stdlib types (solum, caelum, arca, nucleus), the **async generator is the base implementation**. Sync and async-batch variants are derived from it.

## The Inversion

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

## Why This Matters

### 1. Streaming Is the Primitive

You can always derive batch operations from streaming:
- **Batch from stream**: Iterate and collect
- **Stream from batch**: Not possible without reimplementing

Streaming is strictly more general. Making it the base ensures the derived forms are simple wrappers, not separate implementations.

### 2. Memory Efficiency

Streaming operations process data in chunks without loading everything into memory:

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

The batch forms (`lege`, `leget`) internally collect the stream, but users who need memory efficiency can use the streaming form directly.

### 3. Buffer Management

The base streaming implementation manages buffers internally:

```zig
// Base impl: fixed internal buffer, yields slices
pub fn legens(path: []const u8) ChunkIterator {
    return ChunkIterator{
        .file = openFile(path),
        .buffer = undefined,  // Stack-allocated, fixed size
    };
}

// ChunkIterator.next() fills buffer, yields slice into it
// Buffer is reused across iterations - zero allocation in steady state
```

Derived batch forms handle allocation:

```zig
// Derived: allocates to collect stream
pub fn leget(alloc: Allocator, path: []const u8) Future([]u8) {
    // Iterate legens(), append each chunk to ArrayList
    // Return collected result
}
```

### 4. Zig 0.15 Alignment

Zig 0.15's "Writergate" changed I/O to buffer-based APIs:

```zig
// Old (pre-0.15): allocation-based
const line = reader.readUntilDelimiterAlloc(alloc, '\n', 4096);

// New (0.15): buffer-based
var buf: [4096]u8 = undefined;
var r = file.reader(&buf);
const line = r.interface.takeDelimiter('\n');  // Returns slice into buf
```

The async-generator-first approach naturally fits this model:
- Base `legens` uses fixed buffers (matches 0.15 API)
- Derived `leget`/`lege` add allocation on top

---

## Implementation Architecture

### Per-Target Structure

Each target has a subsidia implementation:

```
fons/subsidia/zig/solum.zig     # Zig native impl
fons/subsidia/ts/solum.ts       # TS native impl (optional)
fons/norma/solum.fab            # Annotations mapping to native
```

### Base Implementation (Zig Example)

```zig
// fons/subsidia/zig/solum.zig

const std = @import("std");

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
        return ChunkIterator{ .file = undefined, .done = true, ... };
    };
    return ChunkIterator{ .file = file, .done = false, ... };
}

// Derived: async batch read (leget)
pub fn leget(alloc: std.mem.Allocator, path: []const u8) LegetFuture {
    return LegetFuture.init(alloc, path);
}

// Derived: sync batch read (lege)
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

### Morphology Annotations

```fab
# fons/norma/solum.fab

@ innatum ts "fs", py "os", zig "solum"
genus solum { }

# Base: streaming read
@ radix leg, participium_praesens
@ verte ts (path) -> "createReadStream(§)"
@ verte py (path) -> "open(§, 'rb')"
@ verte zig (path) -> "solum.legens(§)"
@ externa
functio legens()

# Derived: async batch read
@ radix leg, futurum_indicativum
@ verte ts (path) -> "fs.promises.readFile(§)"
@ verte py (path) -> "await aiofiles.open(§).read()"
@ verte zig (path, alloc) -> "solum.leget(§, §)"
@ externa
functio leget()

# Derived: sync batch read
@ radix leg, imperativus
@ verte ts (path) -> "fs.readFileSync(§)"
@ verte py (path) -> "open(§).read()"
@ verte zig (path, alloc) -> "solum.lege(§, §)"
@ externa
functio lege()
```

---

## Derivation Patterns

### Stream → Async Batch

The async batch form collects the stream into a Future:

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
                        return .{ .err = ... };
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
            .failed => return .{ .err = ... },
        }
    }
};
```

### Async Batch → Sync Batch

The sync batch form blocks on the async:

```zig
pub fn lege(alloc: Allocator, path: []const u8) ![]u8 {
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

Or more simply, iterate the stream directly (no Future overhead for sync):

```zig
pub fn lege(alloc: Allocator, path: []const u8) ![]u8 {
    var iter = legens(path);
    defer iter.deinit();

    var result = ArrayList(u8).init(alloc);
    while (iter.next()) |chunk| {
        try result.appendSlice(chunk);
    }
    return result.toOwnedSlice();
}
```

---

## Applies To

This pattern applies to IO-bound stdlib types:

| Type       | Base Form       | Meaning                    |
|------------|-----------------|----------------------------|
| **solum**  | `legens`        | Stream file chunks         |
| **solum**  | `scribens`      | Stream writes              |
| **caelum** | `petens`        | Stream HTTP response       |
| **caelum** | `auscultans`    | Stream WebSocket messages  |
| **arca**   | `quaerens`      | Stream query results       |
| **nucleus**| `accipiens`     | Stream IPC messages        |

Does **not** apply to:
- In-memory collections (lista, tabula, copia) - sync is natural base
- Pure computation (mathesis, tempus) - no I/O involved

---

## Trade-offs

### Advantages

1. **Single implementation path**: Base streaming impl is authoritative
2. **Derived forms are trivial**: Just collect or block
3. **Memory control**: Users choose streaming vs batch based on needs
4. **Zig 0.15 fit**: Matches buffer-based API model naturally

### Disadvantages

1. **Sync has overhead**: Even `lege` goes through iterator machinery
2. **Simple cases verbose**: Reading a small config file requires allocator
3. **Implementation complexity**: Base streaming impl is more complex than naive sync

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

## References

- `morphologia.md` - Verb conjugation as semantic dispatch
- `zig-async.md` - State machine compilation for Zig
- `flumina.md` - Responsum protocol design
