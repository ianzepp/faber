# Eventus - Event System

## Overview

Faber provides a minimal event emission primitive via the `emitte` keyword. Like `scribe` for output, `emitte` is a statement-level construct that compiles to a stdlib call.

## Design Philosophy

Events in Faber follow the same pattern as other output statements:

| Statement | Purpose | Compiles to (TS) |
|-----------|---------|------------------|
| `scribe x` | Log output | `console.log(x)` |
| `vide x` | Debug output | `console.debug(x)` |
| `mone x` | Warning output | `console.warn(x)` |
| `emitte "name", data` | Event emission | `Eventus.emitte("name", data)` |

The language provides the emission primitive. The stdlib provides the subscription mechanism.

## Syntax

```
emitte <event-name>
emitte <event-name>, <data>
```

### Examples

```
// Simple event
emitte "userLogin"

// Event with data payload
emitte "userLogin", { userId: 42, timestamp: nunc() }

// Event name from variable
fixum eventName = "order:created"
emitte eventName, order
```

## Stdlib: Eventus

The `Eventus` module provides event infrastructure:

```typescript
// TypeScript stdlib (conceptual)
type Handler = (data?: unknown) => void;
const listeners = new Map<string, Set<Handler>>();

export const Eventus = {
    emitte(event: string, data?: unknown): void {
        listeners.get(event)?.forEach(fn => fn(data));
    },

    audi(event: string, handler: Handler): () => void {
        if (!listeners.has(event)) listeners.set(event, new Set());
        listeners.get(event)!.add(handler);
        return () => listeners.get(event)?.delete(handler);
    }
};
```

### Subscription (Library-Level)

Subscribing to events uses method calls, not keywords:

```
// Subscribe to an event
fixum unsubscribe = Eventus.audi("userLogin", fac data fit {
    scribe "User logged in:", data.userId
})

// Later: unsubscribe
unsubscribe()
```

## Target Mappings

### TypeScript

```typescript
// emitte "userLogin", { userId: 42 }
Eventus.emitte("userLogin", { userId: 42 });
```

### Zig (Future)

```zig
// Event system would use callback registry
const Eventus = @import("eventus");
Eventus.emitte("userLogin", .{ .user_id = 42 });
```

### Rust (Future)

```rust
// Could use channels or a signal crate
Eventus::emitte("userLogin", json!({ "userId": 42 }));
```

## Why a Keyword?

`emitte` could have been a function call (`Eventus.emitte(...)`), but making it a keyword:

1. **Signals intent** - "Event emission is fundamental, not incidental"
2. **Consistent with `scribe`** - Output primitives are statements, not calls
3. **Enables future optimization** - Compiler can inline, tree-shake unused events
4. **Latin consistency** - Imperative verb form matches other statements

## Etymology

- `emitte` - "send out!" (imperative of `emittere`)
- `Eventus` - "outcome, event" (noun)
- `audi` - "listen!" (imperative of `audire`)

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `emitte` keyword | Done | Parser + TS codegen |
| Basic emission | Done | `Eventus.emitte()` call |
| Event data payload | Done | Optional second argument |
| Subscription (`audi`) | Not Done | Stdlib method only |
| Typed events | Not Done | Future: compile-time event type checking |
| Zig target | Not Done | Needs callback registry |
| Python target | Not Done | Needs stdlib |

## Future Considerations

1. **Typed events** - Could enforce event name + payload type pairs at compile time
2. **Async events** - `emitte futura` for events that wait for handlers
3. **Event namespacing** - `emitte "user:login"` with namespace conventions
4. **Wildcard subscription** - `Eventus.audi("user:*", handler)`
