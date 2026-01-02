# Endpoint Annotation: `@ ad`

## Summary

Add `@ ad` as an annotation to register functions as addressable endpoints in the nucleus dispatch table.

## Motivation

The `ad` statement dispatches to named endpoints:

```fab
ad "fasciculus:lege" ("config.json") fiet textus pro content { }
```

But there's no mechanism to _define_ what responds to these addresses. The `@ ad` annotation provides that:

```fab
@ ad "fasciculus:lege"
functio lege(textus path) fiet textus { ... }
```

## Syntax

### Exact Match

```fab
@ ad "module:method"
functio handler(...) fit|fiet|fiunt|fient T { ... }
```

The string follows the `module:method` namespace convention.

### Pattern Match

```fab
@ ad sed "module:\w+"
functio handler(textus method) fiet Responsum { ... }
```

Uses the existing `sed` regex literal syntax for wildcard routing.

## Examples

### Stdlib Registration

```fab
# norma/fasciculus.fab
@ ad "fasciculus:lege"
functio lege(textus path) fiet textus { ... }

@ ad "fasciculus:scribe"
functio scribe(textus path, textus content) fiet vacuum { ... }

# norma/aleator.fab
@ ad "aleator:inter"
functio inter(numerus min, numerus max) fit numerus { ... }

@ ad "aleator:uuid"
functio uuid() fit textus { ... }

# norma/tempus.fab
@ ad "tempus:nunc"
functio nunc() fit numerus { ... }

@ ad "tempus:dormi"
functio dormi(numerus ms) fiet vacuum { ... }
```

### Pattern Routing

```fab
@ ad sed "api/v\d+/users"
functio usersEndpoint(textus path) fiet Responsum { ... }

@ ad sed "events:\w+"
functio handleEvent(textus eventType) fiet vacuum { ... }
```

## Semantics

### Function Signature Determines Async

The `@ ad` annotation is purely routing metadata. The function's own return verb determines sync/async behavior:

| Return Verb | Behavior            |
| ----------- | ------------------- |
| `fit T`     | sync, single value  |
| `fiet T`    | async, single value |
| `fiunt T`   | sync, generator     |
| `fient T`   | async, generator    |

The nucleus routes to the function; no implicit transformation occurs.

### Caller Must Match

The caller's binding verb must match the target's signature:

```fab
# Definition
@ ad "aleator:inter"
functio inter(numerus min, numerus max) fit numerus { ... }

# Caller
ad "aleator:inter" (1, 100) fit numerus pro n { }   # ✓ matches
ad "aleator:inter" (1, 100) fiet numerus pro n { }  # ✗ type error
```

## Match Resolution

### Priority

1. **Exact match** — always wins over pattern match
2. **Longest match** — among patterns, longest matched string wins
3. **Ambiguous** — same-length matches are a compile error

### Examples

```fab
@ ad "users:list"              # exact
@ ad sed "users:\w+"           # pattern
@ ad sed "users:\w+:\w+"       # longer pattern
```

| Call                 | Winner          | Reason                      |
| -------------------- | --------------- | --------------------------- |
| `ad "users:list"`    | exact           | exact beats pattern         |
| `ad "users:get"`     | `users:\w+`     | only pattern matches        |
| `ad "users:get:all"` | `users:\w+:\w+` | longer match (13 > 9 chars) |

### Conflict Detection

```fab
@ ad sed "users/\d+"      # matches "users/123" (9 chars)
@ ad sed "users/\d\d\d"   # matches "users/123" (9 chars)
```

Same length on same input → compile error. Developer must disambiguate.

## Dispatch Table

The compiler collects all `@ ad` annotations and builds a dispatch table:

```
fasciculus:lege    → norma/fasciculus.lege
fasciculus:scribe  → norma/fasciculus.scribe
aleator:inter      → norma/aleator.inter
aleator:uuid       → norma/aleator.uuid
users:\w+          → app/handlers.handleUser (pattern)
```

At runtime (or compile-time for static dispatch), `ad` statements resolve against this table.

## Relationship to Nucleus

The `@ ad` annotation feeds into the nucleus micro-kernel design (see `nucleus.md`). The nucleus:

1. Receives `ad` calls as syscall requests
2. Looks up the handler in the dispatch table
3. Routes to the registered function
4. Returns the `Responsum` to the caller

## Future Considerations

### Named Capture Groups

```fab
@ ad sed "users/(?<id>\w+)"
functio getUser(textus id) fiet User { ... }
```

Pattern groups could bind directly to function parameters. Requires runtime or compile-time extraction.

### Multiple Registrations

```fab
@ ad "users:get"
@ ad "users:fetch"
functio getUser(textus id) fiet User { ... }
```

Allow a function to respond to multiple addresses.

### HTTP Sugar

For web frameworks, HTTP-specific annotations might layer on top:

```fab
@ ad "/api/users"
@ http "GET"
functio listUsers() fiet lista<User> { ... }
```

This is out of scope for the core `@ ad` mechanism but compatible with it.
