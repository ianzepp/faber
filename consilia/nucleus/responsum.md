# Responsum Protocol

The `Responsum<T>` tagged union is the universal return type for all I/O operations in Nucleus.

## Variants

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

## Protocol Invariants

- **Single-value operation**: `.pending* -> (.ok | .err)`
- **Streaming operation**: `.pending* -> (.item | .pending)* -> (.done | .err)`
- `.item` MUST NOT appear after a terminal variant
- `.done` and `.err` are terminal variants

## Why Protocol Everywhere?

The Responsum protocol is not overhead—it's the unifying abstraction:

1. **Uniform dispatch** — Every syscall produces a stream of `Responsum<T>`
2. **Terminal is explicit** — `.done` vs implicit generator end
3. **Errors are values** — No exception unwinding, streaming errors possible
4. **Cross-target consistency** — Same semantics on all targets
5. **Observability** — Every response is inspectable/loggable

## Detailed Rationale

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

## Known Issues

### Protocol Naming Inconsistency

**Problem**: The existing TypeScript preamble uses `{ op: 'bene' }`, `{ op: 'res' }`, `{ op: 'factum' }` while this design shows `.ok`, `.item`, `.done`.

**Impact**: Silent semantic drift between implementation and design. Cross-target conformance tests will fail.

**Resolution needed**: Decide on canonical names and update either the code or the design.

| Current Code | Design Doc | Latin Meaning |
|--------------|------------|---------------|
| `bene` | `ok` | "well" (success) |
| `res` | `item` | "thing" (item) |
| `factum` | `done` | "done/made" (complete) |

### Missing `.pending` in TypeScript

**Problem**: The TS implementation has no `.pending` variant, but the design requires it for poll-based targets.

**Impact**: If TS code ever needs to interoperate with Zig (via FFI or IPC), the protocol shapes diverge.

**Resolution needed**: Add `.pending` to TypeScript Responsum type, even if native async iteration means it's rarely used.

### No Error Code Taxonomy

**Problem**: Responsum uses `{ code: string, message: string }` for errors, but what codes exist? Different syscalls will produce different error conditions.

**Impact**: Without a defined taxonomy, error handling becomes string-matching guesswork. Zig's approach (typed error sets) and the design's approach (opaque strings) are fundamentally incompatible.

**Resolution needed**: Define error code registry mapping POSIX codes (ENOENT, EACCES, etc.) to Responsum `.err` codes, with target-specific translations.
