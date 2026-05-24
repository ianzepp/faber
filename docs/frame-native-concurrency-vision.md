# Frame-Native Concurrency Vision

**Status**: High-level architectural vision captured (May 2026)

This document records a possible direction for Faber's future concurrency and
message-passing model. It is not a committed grammar, MIR contract, host ABI, or
implementation plan. The goal is to preserve the strategic idea while the host
kernel work in Epic 4 is still early enough to influence.

The central idea is simple: if Faber is already moving toward a host kernel that
routes capability calls as frame-shaped syscalls, then language-level
asynchrony, actors, message passing, events, and streams should probably share
that frame lifecycle instead of growing a separate runtime model.

---

## Context

The current language has `@ futura` as an async annotation surface, but no
larger language-level model for independent tasks, message passing, actors,
events, cancellation, or streams.

The execution roadmap already points toward a host architecture where
outside-world effects become host-owned capabilities. In that model, calls such
as:

```fab
ad "pg:query" ("select * from users") → lista<tabula<textus, ignotum>> pro rows {
    nota rows
}
```

lower into host syscalls routed through a Faber-owned kernel. The first macOS
host slice is expected to borrow useful Muninn semantics: frames, request
correlation, prefix routing, syscalls, sigcalls, cancellation, backpressure,
structured errors, and unresolved-route handling.

Those same mechanics are also the mechanics needed for serious language-level
concurrency.

---

## Strategic Direction

Faber should treat frames as the shared runtime substrate for effectful
asynchronous work.

That does not mean source programs should manipulate raw frames. Source code
should use Faber-shaped constructs: actor-like definitions, typed message
handlers, explicit sends, awaits, and eventually event or stream forms. The
compiler should lower those constructs into a typed effectful framed-call model,
and the host/runtime should decide when that framed call physically crosses the
host kernel.

The rough layering should be:

```text
Faber source feature:
  agens / nuntius / mitte / exspecta / future event forms

Compiler and MIR feature:
  typed effectful framed call

Host and runtime feature:
  frame kernel routes requests, streams responses, handles cancellation,
  enforces backpressure, and reports structured errors
```

The programmer gets an explicit language feature. The implementation gets one
coherent async substrate.

---

## Actor Shape

The preferred source-level actor keyword is currently `agens`, with `actor` as
the English documentation term.

Example sketch:

```fab
agens Indexator {
    lista<textus> documenta ← vacua

    nuntius adde(textus doc) → vacuum {
        documenta.adde(doc)
    }

    nuntius quaere(textus verbum) → lista<textus> {
        redde documenta
    }
}

fixum Indexator ix ← agens Indexator()

mitte ix.adde("De bello Gallico")
fixum lista<textus> inventa ← exspecta ix.quaere("bello")
```

The intended semantics:

- `agens` defines an isolated state owner.
- `nuntius` defines a typed message handler.
- External code cannot directly mutate actor fields.
- `mitte` sends a message to an actor mailbox.
- `exspecta` waits for a response when a message returns a value.
- Handlers process messages according to a scheduling policy the runtime owns,
  likely serially per actor instance unless the language later exposes a
  stronger concurrency annotation.

The source language should express the architectural boundary. It should not
expose threads as the primary concept. Threads are a host/runtime scheduling
choice; actors and messages are the language contract.

---

## Frame Mapping

An actor message maps naturally to the existing frame lifecycle:

```text
Request -> Item* / Bulk* -> Done | Error | Cancel
```

For a single-response message:

```text
mitte ix.quaere("bello")

Frame {
  status: Request,
  to:     actor instance ix,
  call:   "quaere",
  data:   encoded arguments
}

Frame {
  parent_id: request.id,
  status:    Done | Error | Cancel,
  data:      encoded result or structured error
}
```

For a streaming message, the same request can yield multiple item frames before
completion:

```text
Request -> Item(line)* -> Done
```

That gives actors, capability calls, streams, cancellation, and backpressure the
same runtime story.

---

## Host Kernel Boundary

The frame lifecycle should be universal. The host kernel should be the router
for host-visible effects, not necessarily the physical path for every local
message.

The rule of thumb:

- If a framed call crosses a trust, component, provider, process, OS, or
  deployment boundary, it goes through the host kernel.
- If a framed call is proven local to one compiled component and one private
  runtime, the compiler/runtime may implement it as an in-component queue.
- The optimization must preserve the same observable lifecycle: request
  identity, response correlation, cancellation, terminal status, error shape,
  and stream behavior.

This keeps local actor sends cheap without creating two semantic models.

Examples:

```text
ad "pg:query" ...
  -> host capability frame
  -> host kernel
  -> built-in syscall or registered provider

mitte ix.quaere(...)
  -> actor mailbox frame
  -> local queue if ix is local and private
  -> host kernel if ix is host-owned, external, persistent, inspectable, or
     cross-component

future event/pubsub form
  -> event frame
  -> host kernel when routed across components, providers, or subscribers
```

---

## Frame Shape Pressure

The host syscall frame currently described for Epic 4 is a good starting point:

```text
Frame {
  id,
  parent_id,
  created_ms,
  expires_in,
  from,
  call,
  status,
  trace,
  data,
}
```

For syscalls, `call` can reasonably carry names such as `pg:query` or
`host:echo`. For actor messages, overloading `call` as both destination and verb
would be awkward. Actor instances need a routeable mailbox identity.

Before the host ABI hardens, Faber should consider an internal destination field:

```text
Frame {
  id,
  parent_id,
  created_ms,
  expires_in,
  from,
  to,
  call,
  status,
  trace,
  data,
}
```

In this model:

- `to` identifies an actor instance, host, provider, component, subscription
  group, or local runtime mailbox.
- `call` identifies the operation or message handler.
- `data` carries typed arguments or payload.
- `trace` carries spans, grants, timings, diagnostics, and observability
  metadata.

The exact field names and ABI layout are open. The important design pressure is
to avoid baking actor addressing into stringly `call` names if frames are likely
to become the shared substrate for more than syscalls.

---

## Relationship To Capabilities

`ad` host capabilities and actor message sends should share the same lower-level
concept: a typed effectful framed call.

They differ in route class:

- `ad "pg:query"` is a host capability frame.
- `mitte ix.quaere(...)` is an actor mailbox frame.
- A future `emitte`/listen feature could be an event or pub/sub frame.
- `@ futura` can remain the function-level async annotation and lower to task,
  promise, or framed-call machinery depending on target and escape behavior.

This lets the compiler reason about effectful work without pretending every
effect is an ordinary pure function call.

---

## Why Not Compiler-Only Magic

The compiler should optimize scheduling and routing, but it should not invent
concurrency architecture invisibly.

Message passing changes program structure:

- work may outlive the caller,
- state ownership moves behind a mailbox,
- failures and cancellation need boundaries,
- streams may produce partial results,
- backpressure can affect progress,
- host grants and provider routes may matter.

Those are source-level concepts. They deserve explicit syntax and typechecking.
The compiler can still choose an efficient implementation path after the program
has made the concurrency boundary clear.

---

## Open Questions

- Is `agens` the right keyword, or should the actor concept use a different
  Latin term?
- Should `nuntius`, `mitte`, and `exspecta` be the user-facing spellings?
- Should actor handlers always process serially per actor instance, or should
  the language have an explicit concurrency annotation for reentrant handlers?
- How should actor lifetime be represented: lexical, owned handle, supervised
  task, host-owned service, or persistent provider?
- Should actor messages return futures implicitly, or should the send syntax
  distinguish fire-and-forget from request/reply?
- Should local actors be manifest-visible when the host can inspect, cancel, or
  supervise them?
- Where should cancellation policy live: source syntax, handler signature, frame
  metadata, or host/runtime policy?
- Should the first Wasm boundary expose full frames or a smaller typed call API
  that becomes frames inside the host?
- How much of Muninn's stream controller should be copied before Faber has a
  language-level stream surface?
- Does the frame need a first-class `to` field before the host syscall ABI is
  considered stable?

---

## Related Documents

- `docs/factory/faber-execution-roadmap/goal.md` — Umbrella execution roadmap,
  including Epic 4 host kernel work.
- `hosts/macos-arm64/ARCHITECTURE.md` — macOS host architecture direction.
- `hosts/macos-arm64/SYSCALL_MODEL.md` — Current host syscall/frame model.
- `/Users/ianzepp/work/ianzepp/muninn/protocol/frames-rs` — Reference frame
  protocol material.
- `/Users/ianzepp/work/ianzepp/muninn/runtimes/kernel-rs` — Reference kernel,
  routing, cancellation, backpressure, and sigcall material.

---

*This document captures a strategic direction for future discussion. It should
inform host-kernel and MIR design, but it should not be treated as accepted
language syntax until the grammar, type system, MIR, and host ABI are designed
together.*
