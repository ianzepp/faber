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
bindingClause := bindingKeyword typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
bindingKeyword := '->' | 'fit' | 'fiet' | 'fiunt' | 'fient'
target := STRING
```

The `pro` preposition introduces the binding (consistent with iteration/lambda bindings). Optional `ut` provides an alias.

## Binding Keywords

| Keyword | Meaning             | Number   | Protocol | Target Support |
| ------- | ------------------- | -------- | -------- | -------------- |
| `->`    | native return       | singular | No       | TS/Python only |
| `fit`   | becomes (sync)      | singular | Yes      | All            |
| `fiet`  | will become (async) | singular | Yes      | All            |
| `fiunt` | become (sync)       | plural   | Yes      | All            |
| `fient` | will become (async) | plural   | Yes      | All            |

### Arrow vs Verb Binding

The binding keyword determines whether the call uses the Responsum protocol:

```fab
// Arrow: direct native call, no protocol wrapping
ad "fasciculus:lege" ("file.txt") -> textus pro content { ... }

// Verb: protocol-wrapped, consistent across all targets
ad "fasciculus:lege" ("file.txt") fiet textus pro content { ... }
```

**Arrow (`->`)** bypasses the Responsum protocol and emits target-native code:
- TypeScript: `await fs.readFile('file.txt', 'utf-8')`
- Python: `await aiofiles.open('file.txt').read()`
- Zig/Rust/C++: **Compile error** (no native async)

**Verbs (`fit`/`fiet`/`fiunt`/`fient`)** use the Responsum protocol:
- Uniform dispatch via syscall table
- Explicit terminal semantics (`.ok`, `.done`, `.err`)
- Cross-target consistency
- Observable/traceable

This mirrors `functio` declarations, where arrow syntax uses native async and verb syntax uses protocol.

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

**Decision: Protocol by default, direct as escape hatch.**

The binding keyword determines codegen strategy:

### Verb Binding (Protocol)

```fab
ad "https://api.example.com/users" ("GET") fiet Response pro r { }
```

Becomes (TypeScript):

```ts
const r = await run(dispatch('caelum:request', 'https://api.example.com/users', 'GET'));
```

Becomes (Zig):

```zig
var future = caelum.request("https://api.example.com/users", .GET);
const r = try executor.run(&future);
```

Uses Responsum protocol for:
- Uniform dispatch via syscall table
- Cross-target consistency
- Observable/traceable execution
- Explicit error handling (`.err` variant)

### Arrow Binding (Direct)

```fab
ad "https://api.example.com/users" ("GET") -> Response pro r { }
```

Becomes (TypeScript):

```ts
const r = await fetch('https://api.example.com/users', { method: 'GET' });
```

Becomes (Zig):

```
Error P192: Arrow binding not supported for Zig target. Use `fiet` instead.
```

Bypasses protocol for:
- Performance-critical hot paths
- Interop with native libraries
- Targets with native async (TS, Python)

### Comparison

| Aspect          | Arrow (`->`)          | Verb (`fiet`)              |
| --------------- | --------------------- | -------------------------- |
| Performance     | Native, zero overhead | Protocol overhead          |
| Observability   | None                  | Logging, metrics, tracing  |
| Cross-target    | TS/Python only        | All targets                |
| Error handling  | Native exceptions     | Responsum `.err` values    |
| Mocking/testing | Target-specific       | Swap dispatcher handler    |

### Per-Target Support

| Target     | Arrow (`->`) | Verb (`fiet`) |
| ---------- | ------------ | ------------- |
| TypeScript | ✓ Native     | ✓ Protocol    |
| Python     | ✓ Native     | ✓ Protocol    |
| Zig        | ✗ Error      | ✓ Protocol    |
| Rust       | ✗ Error      | ✓ Protocol    |
| C++        | ✗ Error      | ✓ Protocol    |

**Recommendation:** Use verb binding (`fiet`/`fiunt`/`fient`) by default for consistency and observability. Use arrow binding (`->`) only for performance-critical paths on TS/Python targets.

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
