# Ad

Universal dispatch for stdlib, external packages, and remote services.

## Status

| Feature              | Status      | Notes                        |
| -------------------- | ----------- | ---------------------------- |
| Stdlib dispatch      | Not started | `ad "fasciculus:lege" (...)` |
| External packages    | Not started | `ad "hono/Hono" (...)`       |
| Remote/RPC           | Not started | `ad "https://..." (...)`     |
| Sync binding (`fit`) | Not started | `fit Type qua name`          |
| Async binding        | Not started | `fiet`/`fiunt`/`fient`       |

## Overview

`ad` ("to/toward") dispatches a call to a named endpoint and optionally binds the result. It provides a uniform syntax for:

- Stdlib syscalls (`"fasciculus:lege"`)
- External package methods (`"hono/app:serve"`)
- Remote services (`"https://api.example.com/users"`)

## Syntax

```ebnf
adStmt := 'ad' target '(' args ')' bindingClause? block?
bindingClause := fitKeyword typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
fitKeyword := 'fit' | 'fiet' | 'fiunt' | 'fient'
target := STRING
```

The `pro` preposition introduces the binding (consistent with iteration/lambda bindings). Optional `ut` provides an alias.

## Binding Keywords

| Keyword | Meaning             | Number   |
| ------- | ------------------- | -------- |
| `fit`   | becomes (sync)      | singular |
| `fiet`  | will become (async) | singular |
| `fiunt` | become (sync)       | plural   |
| `fient` | will become (async) | plural   |

## Examples

### Stdlib Syscall

```fab
ad "fasciculus:lege" ("file.txt") fit textus pro content {
    scribe content
}
```

Reads as: "to fasciculus:lege with 'file.txt', becomes textus, for content"

### Async Call

```fab
ad "http:get" (url) fiet Response pro response {
    scribe response.body
}
```

### Batch/Plural

```fab
ad "http:batch" (urls) fient Response[] pro responses {
    ex responses pro r {
        scribe r.status
    }
}
```

### External Package

```fab
ad "hono/Hono" () fit App pro app {
    app.get("/", handler)
}

ad "hono/app:serve" (app, 3000) fiet Server pro server {
    scribe "Listening on " + server.port
}
```

### Fire and Forget

No binding clause needed for side-effect-only calls:

```fab
ad "log:info" ("Application started")
```

### Type Inference

If the syscall table defines the return type, the type annotation is optional:

```fab
// Explicit type
ad "fasciculus:lege" ("file.txt") fit textus pro content { ... }

// Inferred from syscall table
ad "fasciculus:lege" ("file.txt") pro content { ... }
```

When type is omitted, `fit`/`fiet` can also be omitted — sync is assumed.

## Target Resolution

The target string is matched against a syscall table with pattern registration. Patterns route to stdlib handlers:

| Pattern                 | Handler                | First Arg |
| ----------------------- | ---------------------- | --------- |
| `http://*`, `https://*` | `caelum:request`       | URL       |
| `file://*`              | `fasciculus:lege`      | path      |
| `ws://*`, `wss://*`     | `caelum:websocket`     | URL       |
| `module:method`         | direct stdlib dispatch | —         |
| `package/export`        | external package       | —         |

### Protocol Sugar

URLs are syntactic sugar. The compiler prepends the URL to the args and rewrites to the registered handler:

```fab
// What you write
ad "https://api.example.com/users" ("GET") fiet Response pro r { }
ad "https://api.example.com/users" ("POST", body) fiet Response pro r { }

// What the compiler rewrites to
ad "caelum:request" ("https://api.example.com/users", "GET") fiet Response pro r { }
ad "caelum:request" ("https://api.example.com/users", "POST", body) fiet Response pro r { }
```

The args pass through unchanged with the URL prepended. The stdlib handler defines its signature:

```fab
// HTTP - args are (method, body?, headers?)
ad "https://api.example.com/users" ("GET") fiet Response pro r { }
ad "https://api.example.com/users" ("POST", body, headers) fiet Response pro r { }

// File - args are (mode, content?)
ad "file:///etc/hosts" ("r") fit textus pro content { }
ad "file:///tmp/out" ("w", content) fit pro ok { }

// WebSocket - args are (options?)
ad "wss://stream.example.com" () fiet Socket pro ws { }

// Explicit stdlib call (equivalent)
ad "caelum:request" (url, "GET") fiet Response pro r { }
```

### Namespace Conventions

| Pattern               | Meaning                      |
| --------------------- | ---------------------------- |
| `"module:method"`     | stdlib module + method       |
| `"package/export"`    | npm/external package         |
| `"package/mod:fn"`    | package + method             |
| `https://`, `http://` | routed to `caelum:request`   |
| `file://`             | routed to `fasciculus:lege`  |
| `ws://`, `wss://`     | routed to `caelum:websocket` |

## Comparison to `functio`

The `ad` binding mirrors function declaration syntax:

```fab
// Declaration: defines return type
functio fetch(textus url) fiet Response

// Dispatch: binds result with same keywords
ad "http:get" (url) fiet Response pro response { ... }
```

| Aspect        | `functio`              | `ad`                          |
| ------------- | ---------------------- | ----------------------------- |
| Return type   | `fiet Type` after args | `fiet Type` before `pro`      |
| Async marker  | `fiet` vs `fit`        | `fiet` vs `fit`               |
| Result access | caller binds with `=`  | `pro name` binds in statement |

## Codegen Strategy

**Decision: Direct codegen, no runtime proxy.**

Two approaches were considered:

### Option A: Direct Codegen

```fab
ad "https://api.example.com/users" ("GET") fiet Response pro r { }
```

Becomes (TypeScript):

```ts
const r = await fetch('https://api.example.com/users', { method: 'GET' });
```

Becomes (Zig):

```zig
const r = try std.http.Client.fetch(allocator, "https://api.example.com/users", .{ .method = .GET });
```

### Option B: Runtime Syscall Proxy

```fab
ad "https://api.example.com/users" ("GET") fiet Response pro r { }
```

Becomes (TypeScript):

```ts
const r = await __fab_syscall('caelum:request', 'https://api.example.com/users', 'GET');
```

### Analysis

| Aspect          | Direct codegen        | Syscall proxy        |
| --------------- | --------------------- | -------------------- |
| Performance     | Native, zero overhead | One indirection      |
| Bundle size     | Only what you use     | Runtime included     |
| Debugging       | Clear stack traces    | Extra frame          |
| Extensibility   | Compile-time only     | Runtime registration |
| Mocking/testing | Harder                | Swap the handler     |

### Per-Target Fit

| Target     | Direct codegen | Syscall proxy         |
| ---------- | -------------- | --------------------- |
| TypeScript | Natural        | Fine (dynamic)        |
| Python     | Natural        | Fine (dynamic)        |
| Zig        | Natural        | Fighting the language |
| Rust       | Natural        | Awkward type erasure  |
| C++        | Natural        | Verbose               |

**Zig problems with proxy:** Return type would need `anytype` at runtime; string dispatch prevents comptime type checking; args tuple needs type erasure; allocator passing becomes awkward. Zig's philosophy is "no hidden runtime."

**Rust problems with proxy:** Needs turbofish or trait object for return type; args as tuple requires `Box<dyn Any>` or macro magic; lifetime annotations get messy.

**Conclusion:** Direct codegen always. Runtime interception (logging, mocking) is a target-specific debug feature, not a language primitive.

## Relation to Imports

`importa` brings in types and interfaces. `ad` dispatches to implementations:

```fab
importa { App, Context } de "hono"  // types only

ad "hono/Hono" () fit App pro app {
    app.get("/") fit Context pro c {
        c.text("Salve")
    }
}
```

## Open Questions

1. Should the syscall table be user-extensible (define your own syscalls)?
2. How do errors propagate? `ad ... cape err { }`?
3. Streaming results — does `pro` bind each item as it arrives?

```fab
ad "wss://stream.example.com/events" () pro Event pro event {
    scribe event.data
}
```
