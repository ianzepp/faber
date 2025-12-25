# Eventus - Event System

## Overview

Faber provides two event primitives:
- `emitte` - Push events (statement)
- `ausculta` - Pull events as async stream (expression)

Together they enable both fire-and-forget and reactive consumption patterns.

## Design Philosophy

| Construct | Type | Purpose | Compiles to (TS) |
|-----------|------|---------|------------------|
| `emitte "name", data` | Statement | Push event | `Eventus.emitte("name", data)` |
| `ausculta "name"` | Expression | Pull stream | `Eventus.ausculta("name")` |

The push/pull duality:
- `emitte` = fire and forget, don't know/care who listens
- `ausculta` = subscribe and iterate, async generator pattern

## Syntax

### emitte (Push)

```
emitte <event-name>
emitte <event-name>, <data>
```

Examples:
```
emitte "userLogin"
emitte "userLogin", { userId: 42 }
```

### ausculta (Pull)

```
ausculta <event-name>
```

Returns `fluxus<T>` (AsyncIterator). Use with `fiet` for async iteration:

```
// Using fixum
fixum stream = ausculta "userAction"
ex stream fiet event {
    scribe event
}

// Using figendum (equivalent, more natural Latin)
figendum stream = ausculta "userAction"
ex stream fiet event {
    scribe event
}
```

Or inline:
```
ex ausculta "data" fiet chunk {
    processChunk(chunk)
}
```

## Stdlib: Eventus

```typescript
// TypeScript stdlib (conceptual)
type Handler = (data?: unknown) => void;
const listeners = new Map<string, Set<Handler>>();

export const Eventus = {
    // Push: emit event to all listeners
    emitte(event: string, data?: unknown): void {
        listeners.get(event)?.forEach(fn => fn(data));
    },

    // Pull: create async iterator for event stream
    async *ausculta(event: string): AsyncGenerator<unknown> {
        const queue: unknown[] = [];
        let resolve: (() => void) | null = null;

        const handler = (data: unknown) => {
            queue.push(data);
            resolve?.();
        };

        listeners.get(event)?.add(handler) ??
            listeners.set(event, new Set([handler]));

        try {
            while (true) {
                if (queue.length === 0) {
                    await new Promise<void>(r => resolve = r);
                }
                yield queue.shift();
            }
        } finally {
            listeners.get(event)?.delete(handler);
        }
    },

    // Callback-style subscription (alternative to ausculta)
    audi(event: string, handler: Handler): () => void {
        if (!listeners.has(event)) listeners.set(event, new Set());
        listeners.get(event)!.add(handler);
        return () => listeners.get(event)?.delete(handler);
    }
};
```

## Usage Patterns

### Fire and Forget

```
functio saveUser(user) {
    database.save(user)
    emitte "user:saved", user
}
```

### Event-Driven Loop

```
futura functio eventLoop() {
    ex ausculta "command" fiet cmd {
        elige cmd.type {
            si "start" { start() }
            si "stop" { rumpe }
            aliter { scribe "Unknown:", cmd }
        }
    }
}
```

### Multiple Streams

```
futura functio monitor() {
    fixum errors = ausculta "error"
    fixum warnings = ausculta "warning"

    // Process errors with priority
    ex errors fiet err {
        handleError(err)
    }
}
```

## Target Mappings

### TypeScript

```typescript
// emitte "userLogin", { userId: 42 }
Eventus.emitte("userLogin", { userId: 42 });

// ausculta "userAction"
Eventus.ausculta("userAction")
```

### Zig (Future)

Zig has no async generators. Would need manual iterator struct:
```zig
const stream = Eventus.ausculta("userAction");
while (stream.next()) |event| {
    // handle event
}
```

### Rust (Future)

Could use async-stream or channels:
```rust
let stream = Eventus::ausculta("userAction");
while let Some(event) = stream.next().await {
    // handle event
}
```

## Etymology

- `emitte` - "send out!" (imperative of `emittere`)
- `ausculta` - "listen!" (imperative of `auscultare` - to listen attentively)
- `Eventus` - "outcome, event" (noun)
- `audi` - "listen!" (imperative of `audire`)

Note: `ausculta` is the root of medical "auscultation" (listening to body sounds).

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `emitte` keyword | Done | Statement, parser + TS codegen |
| `ausculta` keyword | Done | Expression, parser + TS codegen |
| Event data payload | Done | Optional second argument on emitte |
| Async iteration | Done | `ex ausculta "x" fiet y { }` |
| Callback subscription (`audi`) | Not Done | Stdlib method only |
| Typed events | Not Done | Future: compile-time type checking |
| Zig target | Not Done | Needs iterator struct pattern |
| Python target | Not Done | Needs stdlib |

## Future Considerations

1. **Typed events** - Enforce event name + payload type pairs at compile time
2. **Event filtering** - `ausculta "user:*"` for wildcard patterns
3. **Buffering** - `ausculta "data" in buffer(10)` for bounded queues
4. **Timeout** - `ausculta "data" per timeout(5000)` for bounded waiting
