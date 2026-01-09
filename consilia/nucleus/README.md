# Nucleus: Faber's Micro-Kernel Runtime

A minimal kernel layer providing unified I/O dispatch, message-passing protocol, and async execution across all targets.

## Documents

| Document | Purpose |
|----------|---------|
| [responsum.md](responsum.md) | Protocol specification — the `Responsum<T>` tagged union |
| [streaming.md](streaming.md) | Async-generator-first design philosophy |
| [executor.md](executor.md) | Runtime architecture — OpStream, Executor, Handle |
| [dispatch.md](dispatch.md) | Syscall dispatch and `ad` integration |
| [targets.md](targets.md) | Per-target implementations (TS, Python, Zig, Rust, C++) |
| [implementation.md](implementation.md) | Implementation plan and phases |

## Status

| Feature                 | Status      | Notes                              |
| ----------------------- | ----------- | ---------------------------------- |
| Responsum protocol      | Partial     | TS has it; Zig/Rust/C++/Py need it |
| Syscall dispatch (`ad`) | Design only | See `ad.md`                        |
| Request correlation     | Not started | IDs for concurrent ops             |
| Handle abstraction      | Not started | Unified I/O interface              |
| AsyncContext            | Not started | Executor for state machines        |
| Target runtimes         | TS only     | Zig is next priority               |

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

## Core Insight

**Async generators are the primitive.** Everything else—promises, blocking calls, collected results—derives from streaming.

```
Async Generator (primitive)
       │
       ├── fient/fiunt → raw stream, yields .item repeatedly, ends with .done
       │
       ├── fiet        → collect stream into single value, return .ok
       │
       └── fit         → block until .ok, unwrap and return raw value
```

See [streaming.md](streaming.md) for the full design philosophy.

## Design Review (Opus Analysis)

*Reviewed: 2026-01-09*

### Key Findings

1. **Protocol discriminant naming inconsistency** — Existing TS implementation uses `bene`/`res`/`factum` but design shows `ok`/`item`/`done`. This will cause silent semantic drift.

2. **Missing `.pending` in TS implementation** — The design requires it for poll-based targets. If TS ever needs to interoperate with Zig (via FFI or IPC), the protocol shapes diverge.

3. **Circular dependency between Nucleus and `ad`** — Document says "Nucleus runtime is the execution layer beneath `ad`" but also shows syscall handlers being dispatched through Nucleus. Unclear who owns registration.

4. **Allocator threading declared but not designed** — Decision says "Per-request allocator, inherited from parent" but no mechanism shown.

5. **User-defined `fiet` functions don't fit derivation chain** — Only stdlib operations shown going through syscall table.

6. **stdlib `@ verte` annotations bypass derivation model** — Shows direct function calls, not `block_on(collect(...))`.

7. **No error code taxonomy** — Zig typed errors vs opaque strings are fundamentally incompatible.

8. **Blocking I/O in Phase 3 can't validate async design** — Bugs won't surface until Phase 4.

### Recommendations

1. **Establish protocol conformance tests immediately** — Before more codegen work, validate Responsum semantics are identical across targets.

2. **Sequence semantic analysis work explicitly** — Create dependency graph: (liveness analysis) → (state machine shape) → (Zig codegen). Don't start Phase 3 without semantic prerequisites.

3. **Resolve stdlib annotation vs derivation chain conflict** — Either update annotations to emit derivation calls, or document why hand-written implementations are acceptable.

4. **Define error code registry** — Document stdlib error codes and how they map to each target's native error system.

## References

- `ad.md` — Dispatch syntax design
- `zig-async.md` — Zig-specific state machine details
- `flumina.md` — Original Responsum protocol (TypeScript)
- `two-pass.md` — Semantic analysis for liveness
- `fons/norma/solum.fab` — File I/O stdlib definitions
- `fons/norma/caelum.fab` — Network I/O stdlib definitions
- `fons/norma/arca.fab` — Database stdlib definitions
