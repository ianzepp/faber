# Streaming-First Design

**The fundamental design principle**: Async generators are the primitive. Everything else—promises, blocking calls, collected results—derives from streaming.

This inverts the typical mental model where sync is default and async is added.

## Why Streaming First?

1. **I/O is naturally streaming** — Files read in chunks, networks deliver packets, databases return rows
2. **Memory efficiency** — Process items as they arrive, don't buffer everything
3. **Unified model** — Single primitive handles all async patterns
4. **Backpressure built-in** — Consumer controls pace via poll frequency
5. **Buffer management** — Base streaming impl uses fixed internal buffers; derived batch forms add allocation on top

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

Streaming is strictly more general. You can always derive batch from stream (iterate and collect), but you cannot derive stream from batch without reimplementing.

## The Derivation Chain

Every I/O operation follows this pattern:

```
stream()           # primitive: yields items over time
    ↓ collect
future()           # derived: collect all items → single value (Promise)
    ↓ block_on
sync()             # derived: await future → unwrapped value (blocking)
```

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

## Scope: What This Applies To

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

## Memory Efficiency Example

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

## Zig 0.15 Alignment

The async-generator-first approach naturally fits Zig 0.15's buffer-based I/O APIs ("Writergate"):

```zig
// Old (pre-0.15): allocation-based
const line = reader.readUntilDelimiterAlloc(alloc, '\n', 4096);

// New (0.15): buffer-based
var buf: [4096]u8 = undefined;
var r = file.reader(&buf);
const line = r.interface.takeDelimiter('\n');  // Returns slice into buf
```

Base `legens` uses fixed buffers (matches 0.15 API). Derived `leget`/`lege` add allocation on top.

---

## Edge Cases and Concerns

### Streaming is Not Always Natural

While I/O is often streaming, some operations are inherently atomic:
- File existence check (`solum.exstat`) — yes/no, no streaming
- Small config files (< 4KB) — overhead of streaming exceeds benefit
- DNS lookups — single result, not a stream
- Random access (seek to byte N) — streaming model forces reading preceding bytes

Should these operations bypass streaming and return `.ok` directly? Or force them through streaming for API consistency?

### Partial Read Complications

Streaming file reads can stop midway (user breaks loop, error occurs, etc.). This leaves:
- File descriptor unclosed (resource leak)
- File lock held (blocks other processes)
- Partial state in consumer (half-processed data)

Options:
- Require explicit cleanup: `cape` blocks must close handles
- Automatic cleanup via RAII: `cura` blocks track handles and close on scope exit
- Finalizers (TS/Python only): Register cleanup on cursor, run on GC

### Small File Streaming Overhead

Consider a 10-byte config file. Streaming approach:
1. Allocate buffer (4096 bytes)
2. Open file
3. Read chunk (10 bytes)
4. Yield `.item`
5. Read again (EOF, 0 bytes)
6. Yield `.done`
7. Close file

Batch approach:
1. Open file
2. Read all (10 bytes)
3. Close file
4. Return bytes

Streaming has 3-4x more operations. For files < chunk size, batch is strictly better. Should the compiler or stdlib detect this and optimize automatically?

### Streaming Interrupts Compiler Optimizations

When collecting a stream, the compiler can't optimize across suspend points. Example:

```fab
# Streaming
varia sum = 0
ex solum.legens("numbers.txt") pro chunk {
    sum = sum + parse(chunk)  # Suspend between chunks
}
```

The loop body can't be optimized as a tight loop — each iteration may suspend. Batch form:

```fab
fixum content = solum.lege("numbers.txt")
varia sum = 0
ex content.split("\n") pro line {
    sum = sum + parse(line)  # No suspend, optimizer can vectorize
}
```

Batch enables SIMD, loop unrolling, and other optimizations. For CPU-bound workloads, batch > streaming.

### Latin Conjugation Concerns

**Stem irregularities**: Not all Latin verbs conjugate regularly. Example: "to write" has multiple stems:
- `scribere` (infinitive) → `scribens` (present participle) ✓ regular
- `scribere` → `scribet` (future indicative) ✓ regular
- But imperative is `scribe` not `scribere` ✓ different stem

The stdlib uses `inscribe` (compound form) for sync write. This works, but users defining custom verbs must know Latin morphology. Is this sustainable? Should there be a verb conjugation validator in the compiler?

**Morphological ambiguity**: Some Latin verbs have identical forms across tenses. Example:
- `audit` could be "he hears" (present) or "he heard" (perfect)
- `legit` could be "he reads" (present) or "he read" (perfect)

Faber uses verb endings to encode semantics, not tense. But when users read `leget`, do they parse it as:
- Future tense ("will read")? ✓ intended
- Present tense third person ("he reads")? ✗ misleading

**Verb form collisions with keywords**: The design shows:
- `fit` = sync return
- `fiet` = async return

But `fit` also means "becomes" in Latin (third person singular). If user code says `x fit y`, is this assignment or function declaration? Context disambiguates, but it's subtle.

**Non-Latin developer experience**: Developers unfamiliar with Latin must memorize arbitrary-looking endings:
- `-ens` = streaming
- `-et` = async
- `-e` = sync

These mappings are opaque without Latin knowledge. Should the docs include a "cheat sheet" mapping Latin forms to programmer-familiar concepts (e.g., "legens = async iterator")?

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
