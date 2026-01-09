# Syscall Dispatch

Routes calls to handlers based on namespace.

## Syscall Table

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

## Compile-Time Dispatch (Zig)

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

---

## HAL Device Model

Adapted from Monk OS's 17-device HAL:

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

## Integration with `ad` Dispatch

The `ad.md` design document describes a universal dispatch mechanism for syscalls. Nucleus runtime is the execution layer beneath `ad`.

### Target Resolution

`ad` patterns (e.g., `"solum:lege"`, `"https://..."`) must map to Nucleus handlers. The syscall table is compile-time for native targets (Zig, Rust, C++), runtime for dynamic targets (TS, Python).

**Pattern matching complexity**: If users can register custom patterns (e.g., `"myproto://*"`), who validates them? Compiler or runtime?
- Compile-time: Safer, but limits dynamic loading of handlers
- Runtime: Flexible, but bad patterns discovered at runtime, not compile-time

**Namespace collisions**: What if two packages register handlers for the same pattern?
- Package A: `"postgres://*"` → handler A
- Package B: `"postgres://*"` → handler B

First-registered wins? Explicit priority? Compile error?

### Arrow vs Verb Binding Interaction

The `ad` statement supports both arrow (`->`) and verb (`fiet`) binding. Nucleus only provides protocol-based execution.

```fab
# Arrow binding on Zig target
ad "solum:lege" ("file.txt") -> textus pro content { ... }
```

On Zig, this should be a compile error (P192), but `ad.md` says "Arrow bypasses protocol, native targets compile error". Is this:
- Caught at semantic analysis (before codegen)?
- Caught at codegen (Zig generator throws error)?
- Allowed to pass through and fail at Zig compile step (bad UX)?

**Performance escape hatch on TS/Python**: If arrow binding bypasses Nucleus, does it also bypass:
- Backpressure enforcement?
- Request correlation IDs?
- Observability hooks (logging, tracing)?

This needs explicit documentation. Arrow is "fast but opaque", verb is "observable but overhead".

### Syscall Handler Signatures

`ad` allows arbitrary arguments: `ad "solum:lege" (path, flags, mode)`. Nucleus handlers have fixed signatures: `solum.lege(textus path) -> textus`. Mismatch resolution:

- Does the dispatcher validate argument count/types at compile-time?
- Does `ad` support overloading (same name, different arg shapes)?
- If a handler takes optional args, does `ad` support: `ad "solum:lege" (path)` vs `ad "solum:lege" (path, encoding)`?

**Type inference from syscall table**: The `ad.md` doc says "type annotation is optional if syscall table defines return type". This requires:
- Compile-time access to handler signatures (easy for stdlib, hard for external packages)
- Export of type metadata from compiled libraries (Zig/Rust don't have runtime reflection)
- Fallback to `ignotum` if type is unknowable (loses type safety)

### Error Handling Interaction

Both `ad` and Nucleus use errors-as-values. But `ad` allows `cape` blocks:

```fab
ad "solum:lege" ("missing.txt") fiet textus pro content {
    scribe content
} cape err {
    scribe "Failed: " + err.message
}
```

Does the `cape` block:
- Catch `.err` variants from Responsum (Nucleus-level error)?
- Catch Faber `iace` errors (user-level error)?
- Both?

If both, what's the precedence? If Nucleus returns `.err { code: "ENOENT", ... }` and handler also does `iace CustomError`, which `cape` block catches which error?

### Streaming via `ad`

The `ad.md` doc shows `fient` for streaming:

```fab
ad "wss://stream.example.com/events" () fient Event pro event {
    scribe event.data
}
```

This is `cursor<Event>` underneath. But the syntax suggests `event` is bound once per iteration. Clarifications:

- Is `fient` binding sugar for `ex...pro` loop over cursor?
- Does the block execute once per `.item`, or once total with `event` being a cursor?
- If error occurs midstream (`.err` after 100 `.item`s), does `cape` block have access to partial results?

### Concurrent Operations via `ad`

Nucleus supports request correlation for concurrent ops. Does `ad` expose this?

```fab
# Can I launch two requests concurrently?
ad "caelum:pete" (url1) fiet Response pro r1 { ... }
ad "caelum:pete" (url2) fiet Response pro r2 { ... }
```

If both are in the same function, does Nucleus:
- Execute them sequentially (blocking on first before starting second)?
- Execute them concurrently (submit both, poll both)?

If concurrent, how does the user `cede` both and wait for whichever completes first? Needs explicit syntax or helper.

---

## Design Decisions (Resolved)

### `ad` is Compile-Time Syntax; Nucleus Owns Runtime

**Decision**: `ad` is purely syntactic sugar that the compiler erases. Nucleus owns both dispatch and syscall handlers at runtime.

**Rationale**: The apparent circular dependency dissolves when you recognize the layering:

```
Faber source (ad solum.legens)
       ↓ compiler
Syscall invocation (nucleus.solum.legens)
       ↓ runtime dispatch
Syscall handler (yields Responsum)
       ↓
Target I/O (Bun fs, Zig std, etc.)
```

`ad` doesn't exist at runtime. The compiler transforms `ad "solum:lege"` into direct calls to Nucleus handlers. Nucleus owns the syscall table (compile-time for Zig/Rust/C++, potentially runtime for TS/Python).

### stdlib Annotations: Parked

**Status**: Deferred. The tension between `@ verte` hand-written implementations and the derivation model may resolve naturally as other decisions are made.

**Context**: `solum.fab` annotations assume standalone Zig functions exist. The derivation model says `lege` = `block_on(collect(legens))`. These can coexist:
- Derivation model describes the *semantic* relationship
- `@ verte` annotations provide *implementation* escape hatches for performance-critical paths

Document explicitly if hand-written implementations are acceptable for stdlib (likely yes).
