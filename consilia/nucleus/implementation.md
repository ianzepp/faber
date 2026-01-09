# Implementation Plan

Phased approach to implementing Nucleus across all targets.

## Phase 1: Protocol Foundation

1. Define `Responsum<T>` for all targets in preamble
2. Define `OpStream<T>` interface per target
3. Implement basic `Executor` with `run()` and `collect()`

**Concerns:**
- **Preamble explosion**: Each target needs its own preamble. Maintaining 5 copies of Responsum definition risks divergence. Consider codegen from a single source of truth.
- **Testing across targets**: Protocol semantics must be identical. How do we validate this? Cross-target integration tests? Formal specification?
- **Responsum size**: For Zig, a `union(enum)` with `.err` containing strings may be large. Stack vs heap allocation trade-off affects every operation.

---

## Phase 2: TypeScript Runtime

1. Use native `AsyncIterable<Responsum<T>>` for OpStream
2. Implement `run()`, `collect()` as async functions
3. Wire solum/caelum handlers using Node.js APIs

**Concerns:**
- **Node.js vs Bun APIs**: The doc says "using Node.js APIs" but the project uses Bun. Are Node APIs sufficient, or do Bun-specific APIs offer performance wins (e.g., `Bun.file()` vs `fs.readFile`)?
- **Error translation**: Node.js errors (e.g., `ENOENT`) must be translated to Responsum `.err` format. What's the canonical mapping? Some errors have structured data (errno codes, syscall names). Do we preserve this?
- **Stream abort handling**: If user breaks from `for await` loop, the stream should clean up. Does `AsyncIterable` finalization handle this, or do we need explicit `.return()` calls?

---

## Phase 3: Zig Runtime

1. Implement `Responsum(T)` union
2. Implement `OpStream` vtable pattern
3. Implement `ExecutorContext` with blocking I/O (v1)
4. State machine codegen for `fiet`/`fient` functions
5. Implement solum handlers using `std.fs`
6. Implement caelum handlers using `std.http`

**Concerns:**
- **Zig stdlib volatility**: Zig 0.15 just overhauled I/O APIs ("Writergate"). If Phase 3 starts on Zig 0.15 but Zig 0.16 changes APIs again, handlers need rewriting. Mitigation: Wrap Zig APIs in a stable internal interface.
- **Allocator propagation**: Every handler needs an allocator. Tracking allocators through deeply nested calls is error-prone. Should `ExecutorContext` embed a standard allocator (e.g., arena per request)?
- **Blocking I/O performance**: Phase 3 intentionally uses blocking I/O. This means `wait_for_io()` is a no-op, and `poll()` never returns `.pending`. This defeats the async model. Is this acceptable for initial implementation? Or should Phase 3 and 4 be merged?

**Critical dependency**: State machine generation requires semantic analysis for liveness tracking. The current compiler lacks this. Must complete semantic infrastructure before Phase 3 can proceed.

---

## Phase 4: Async I/O (Zig)

1. Integrate io_uring or epoll for non-blocking I/O
2. Implement proper `.pending` handling with I/O multiplexing
3. Add request correlation for concurrent operations

**Concerns:**
- **Library choice**: `io_uring` (Linux-only, newest, fastest) vs `epoll` (Linux-only, older, compatible) vs `libxev` (cross-platform, external dep) vs `std.io.poll` (cross-platform, limited). Each has trade-offs. Which aligns with Faber's goals (portability vs performance)?
- **Completion queue management**: io_uring uses submission/completion queue rings. Mapping completions back to futures requires request IDs. This is where "Request Correlation" comes in, but the design is vague. Need concrete data structures.
- **Error handling in event loop**: If `io_uring_wait_cqe()` returns an error, what happens to in-flight futures? Do they all return `.err`? Or do we panic? Or silently retry?

---

## Phase 5: Other Targets

1. Python — Similar to TypeScript (native async)
2. Rust — State machines like Zig, or native async
3. C++ — Coroutines (C++20) or callback-based

**Concerns:**
- **Python GIL**: Native async in Python still contends with GIL for CPU work. Nucleus can't fix this. Should Faber expose multi-process parallelism for Python (via `multiprocessing`), or is that out of scope?
- **Rust borrow checker vs state machines**: Generated Rust state machines must satisfy borrow checker. Captured variables may need explicit lifetimes. Rust's async ecosystem (Tokio, async-std) has mature state machine generation. Can we leverage `async-trait` or similar, or do we fully DIY?
- **C++20 coroutine support**: Not all compilers support C++20 coroutines (older GCC/Clang, MSVC). Do we require C++20, or provide fallback (callback-based, worse UX)?

---

## Dependency Graph

```
Phase 1: Protocol Foundation
    │
    ├── Phase 2: TypeScript Runtime
    │       (can start immediately after Phase 1)
    │
    └── Semantic Analysis: Liveness Tracking
            │
            └── Phase 3: Zig Runtime
                    │
                    └── Phase 4: Async I/O (Zig)
                            │
                            └── Phase 5: Other Targets
```

**Critical path**: Semantic analysis for liveness tracking must complete before Zig state machine codegen can begin.

---

## Lessons from Monk OS

Analysis of Monk OS source code revealed patterns worth adopting and avoiding.

### What Monk Solved

**Worker Boundary Isolation**: Monk uses Bun Workers for process isolation, requiring message passing across thread boundaries.

**Faber's advantage**: Compiles to native code in single address space. Direct function calls, no serialization overhead.

**Streaming Backpressure**: Monk implements ping/ack protocol with per-stream timers. Side effect: 1000 concurrent syscalls = 1000 active timers. GC pressure.

**Faber's advantage**: Use language-native generator pause/resume. No timers needed for native targets.

**Request Correlation**: Monk uses UUID v4 for request IDs (128-bit, cryptographically unique). Trade-off: 16 bytes per ID vs 8 bytes. String allocation + hashing overhead.

**Faber's decision**: Use 64-bit counter for Zig/Rust/C++ (simpler, faster). UUID acceptable for TS/Python.

**Handle Abstraction**: Monk's unified interface is clean and worth adopting. Nucleus mirrors this pattern with vtable for Zig.

### Concerns Discovered

**Partial Results on Midstream Error**: Consumer has already processed partial results before error arrives.

**Nucleus decision**: Document that consumers must handle partial results. Consider adding `.partial_error` variant that includes count of successful items.

**Nested Cancellation Leaks**: Cleanup only happens for outermost stream.

**Nucleus decision**: For native targets, use RAII/defer patterns. Each stream cleans itself on scope exit.

**Stall Detection False Positives**: Monk aborts streams after 5s without ping. Slow networks can trigger false timeouts.

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

## Open Questions

1. **Error context enrichment** — Should `.err` include stack trace? Performance vs debuggability.
   - For Zig targets without native error unwinding, stack traces would require storing frame pointers or using Debug builds.
   - TypeScript/Python have native stack traces. How do we unify error reporting across targets?

2. **Streaming timeout** — Per-operation or global? Syntax for specifying?
   - Should timeout be a verb modifier? `functio leget(...) fiet tempore 5000 textus`?
   - How does timeout interact with backpressure watermarks?

3. **Future size threshold** — 256 bytes is arbitrary. Profile real-world futures.
   - Nested futures multiply state size. A 200-byte outer + 200-byte inner = 400 bytes, exceeding threshold.

4. **Cross-target ID consistency** — TS uses UUID, Zig uses counter. Acceptable divergence?
   - For distributed tracing, request IDs must be compatible. UUID is standard; counter is local-only.

5. **Partial error semantics** — Consumer sees 100 items then `.err`. How to handle?
   - Should `.err` include metadata about how many items succeeded before failure?

6. **Convenience wrappers** — Is `legeBrevis` worth the API surface, or just accept allocator params?
   - Temporary allocator wrapper makes allocation invisible, violating Zig's explicit allocation principle.

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

**Recommendation**: Add optional `max_bytes` for syscalls that stream large chunks.

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
