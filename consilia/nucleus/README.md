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

### Resolved Findings

| # | Finding | Resolution |
|---|---------|------------|
| 1 | Protocol discriminant naming inconsistency | **Latin canonical.** TS implementation (`bene`/`res`/`factum`/`pendens`) is correct. Update design docs to match. See [responsum.md](responsum.md). |
| 2 | Missing `.pending` in TS implementation | **Not needed.** Protocol shapes vary by target. TS uses native async; `.pendens` is only for poll-based targets. |
| 3 | Circular dependency between Nucleus and `ad` | **Resolved.** `ad` is compile-time syntax; Nucleus owns runtime dispatch. See [dispatch.md](dispatch.md). |
| 4 | Allocator threading declared but not designed | **Use `cura`/`curator`.** Existing language mechanism handles allocator scoping. See [executor.md](executor.md). |
| 5 | User-defined `fiet` functions don't fit derivation chain | **Verb triggers codegen.** The verb (`fit`/`fiet`/`fient`) signals Responsum codegen, regardless of call target. |
| 6 | stdlib `@ verte` annotations bypass derivation model | **Parked.** May resolve naturally. Hand-written implementations acceptable for stdlib if documented. |
| 7 | No error code taxonomy | **Keep `iace` untyped.** Codegen does best-effort mapping per target. Revisit if problems arise. |
| 8 | Blocking I/O in Phase 3 can't validate async design | **Validate async early.** Build minimal proof-of-concept (one syscall, one target) before full syscall surface. |

### Recommendations (Status)

| # | Recommendation | Status |
|---|----------------|--------|
| 1 | Establish protocol conformance tests | **Future work.** New `fons/probationes/` structure using Faber's `proba` syntax. See [probationes.md](../futura/probationes.md). |
| 2 | Sequence semantic analysis work | **Adopted.** Build order: two-pass → canThrow → liveness → state machine → Zig codegen. |
| 3 | Resolve stdlib annotation conflict | **Parked.** See finding #6. |
| 4 | Define error code registry | **Deferred.** See finding #7. |

## References

- `ad.md` — Dispatch syntax design
- `zig-async.md` — Zig-specific state machine details
- `flumina.md` — Original Responsum protocol (TypeScript)
- `two-pass.md` — Semantic analysis for liveness
- `fons/norma/solum.fab` — File I/O stdlib definitions
- `fons/norma/caelum.fab` — Network I/O stdlib definitions
- `fons/norma/arca.fab` — Database stdlib definitions
