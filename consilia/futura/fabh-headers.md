# Faber Header Files (`.fabh`)

Contract-based compilation via header files and `ad` dispatch.

## Status

| Feature                     | Status      | Notes                                    |
| --------------------------- | ----------- | ---------------------------------------- |
| `.fabh` file format         | Not started | Header syntax with `@ externa`           |
| Header-only compilation     | Not started | Type-check against headers, no impl      |
| `@ ad` in headers           | Not started | Endpoint registration in contract        |
| `@ optio` in headers        | Not started | Argument metadata in contract            |
| Dispatch table from headers | Not started | Build syscall table from `.fabh` files   |
| Target-specific linking     | Not started | Connect headers to implementations       |

## Motivation

Currently, compiling a Faber program that uses stdlib requires parsing the full stdlib implementation. This has several drawbacks:

1. **Compilation overhead** - Parser/semantic analyzer processes implementation code that isn't needed for type-checking
2. **Tight coupling** - Source files implicitly depend on implementation details
3. **No clear contract** - The "API" is whatever the implementation exposes
4. **LLM unfriendly** - Models need to read implementation code to understand available functions

The header file approach separates contract from implementation:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  your-app.fab   │────▶│   norma.fabh    │────▶│  norma/*.fab    │
│  (source)       │     │   (contract)    │     │  (impl)         │
└─────────────────┘     └─────────────────┘     └─────────────────┘
     compiles              type-checks             links/runs
     against               against
```

## Overview

A `.fabh` (Faber Header) file declares function signatures with `@ ad` endpoint registration and `@ optio` argument metadata, without implementation bodies:

```fab
# norma/solum.fabh

@ ad "solum:lege"
@ optio textus via descriptio "File path to read"
@ externa
functio lege(textus via) fiet textus

@ ad "solum:scribe"
@ optio textus via descriptio "File path to write"
@ optio textus contentum descriptio "Content to write"
@ externa
functio scribe(textus via, textus contentum) fiet vacuum

@ ad "solum:exstat"
@ optio textus via descriptio "Path to check"
@ externa
functio exstat(textus via) fit bivalens
```

The `@ externa` annotation marks "signature only, implementation provided elsewhere."

Programs call these via `ad` dispatch:

```fab
ad "solum:lege" ("config.json") fiet textus pro content {
    scribe content
}

ad "solum:exstat" ("/tmp/file") fit bivalens pro exists {
    si exists {
        scribe "File exists"
    }
}
```

The compiler:
1. Parses `.fabh` files to build the dispatch table
2. Type-checks `ad` calls against the declared signatures
3. Emits dispatch calls (resolved at link time or runtime)

## Syntax

### Header File (`.fabh`)

```ebnf
headerFile   := headerDecl*
headerDecl   := annotation* 'functio' IDENTIFIER '(' paramList ')' returnClause
annotation   := adAnnotation | optioAnnotation | externaAnnotation
adAnnotation := '@' 'ad' STRING
optioAnnotation := '@' 'optio' typeAnnotation IDENTIFIER optioModifier*
optioModifier := 'brevis' STRING | 'longum' STRING | 'descriptio' STRING
externaAnnotation := '@' 'externa'
```

### Annotations

| Annotation | Purpose |
|------------|---------|
| `@ ad "namespace:method"` | Registers function as dispatch endpoint |
| `@ optio <type> <name> ...` | Declares argument with metadata |
| `@ externa` | Marks function as implementation-less (required in `.fabh`) |

### `@ optio` Modifiers

| Modifier | Purpose | Example |
|----------|---------|---------|
| `brevis "x"` | Short flag form | `brevis "o"` → `-o` |
| `longum "name"` | Long flag form | `longum "output"` → `--output` |
| `descriptio "..."` | Human-readable description | For docs/help |

The `brevis`/`longum` modifiers enable CLI-style named arguments at call sites.

## Call Site Syntax

With headers providing argument metadata, `ad` calls can use named parameters:

```fab
# Positional (always works)
ad "solum:scribe" ("/tmp/out.txt", "hello") fiet pro ok { }

# Named (when @ optio declares the names)
ad "solum:scribe" (via: "/tmp/out.txt", contentum: "hello") fiet pro ok { }

# Mixed (positional first, then named)
ad "processus:exsequi" (["git", "clone", url], cwd: workDir) fiet pro r { }
```

Named parameters map to `@ optio` bindings. The compiler validates:
- All required parameters are provided
- Named parameters match declared names
- Types match declared types

## Complete Header Example

```fab
# norma/processus.fabh - Process execution contract

@ ad "processus:exsequi"
@ optio lista<textus> mandatum descriptio "Command and arguments as array"
@ optio textus cwd longum "cwd" descriptio "Working directory"
@ optio bivalens tace longum "quiet" descriptio "Suppress stdout capture"
@ optio tabula<textus, textus> env longum "env" descriptio "Environment variables"
@ externa
functio exsequi(
    lista<textus> mandatum,
    si textus cwd,
    si bivalens tace,
    si tabula<textus, textus> env
) fiet Responsum<ExitusProcessus>

@ ad "processus:genera"
@ optio lista<textus> mandatum descriptio "Command and arguments"
@ optio textus cwd longum "cwd" descriptio "Working directory"
@ externa
functio genera(lista<textus> mandatum, si textus cwd) fiet Processus

@ ad "processus:env"
@ optio textus nomen descriptio "Environment variable name"
@ externa
functio env(textus nomen) fit si textus
```

## Usage Example

```fab
# app.fab - Uses stdlib via ad dispatch only

@ cli "deployer"
@ versio "1.0.0"
incipit argumenta args {
    # Check if config exists
    ad "solum:exstat" ("deploy.json") fit bivalens pro exists {
        si non exists {
            mone "No deploy.json found"
            processus.expirat(1)
        }
    }

    # Read config
    ad "solum:lege" ("deploy.json") fiet textus pro content {
        fixum config = json.pange(content)
    }

    # Run deployment
    ad "processus:exsequi" (
        ["docker", "compose", "up", "-d"],
        cwd: config.directory,
        env: { "ENV": "production" }
    ) fiet Responsum pro result {
        si result.ok {
            scribe "Deployed successfully"
        }
        secus {
            mone result.err
        }
    }
}
```

## Compilation Model

### Phase 1: Header Loading

The compiler loads all `.fabh` files and builds the dispatch table:

```
Dispatch Table:
  "solum:lege"        → { params: [via: textus], returns: fiet textus }
  "solum:scribe"      → { params: [via: textus, contentum: textus], returns: fiet vacuum }
  "solum:exstat"      → { params: [via: textus], returns: fit bivalens }
  "processus:exsequi" → { params: [mandatum: lista<textus>, cwd?: textus, ...], returns: fiet Responsum }
  ...
```

### Phase 2: Type Checking

`ad` calls are validated against the dispatch table:

```fab
ad "solum:lege" (42) fiet textus pro x { }
#               ^^
# Error: Expected textus for parameter 'via', got numerus

ad "solum:lege" ("file.txt") fit textus pro x { }
#                            ^^^
# Error: Endpoint "solum:lege" returns fiet (async), caller uses fit (sync)
```

### Phase 3: Code Generation

The codegen phase emits dispatch calls. Strategy depends on target:

**TypeScript (inline):**
```ts
// ad "solum:lege" ("config.json") fiet textus pro content { }
const content = await solum.lege("config.json");
```

**Zig (dispatch table):**
```zig
// ad "solum:lege" ("config.json") fiet textus pro content { }
const content = try dispatch.call("solum:lege", .{"config.json"});
```

### Phase 4: Linking

Implementations are connected to headers:

| Target | Linking Strategy |
|--------|------------------|
| TypeScript | Import actual `norma/*.ts` modules, inline calls |
| Zig | Link against `libnorma.a`, dispatch via function pointers |
| Rust | Link against `norma` crate, dispatch via trait objects |

## Benefits

### For Compilation

- **Faster** - Only parse headers, not full implementations
- **Incremental** - Implementation changes don't force recompilation of dependents
- **Parallel** - Compile multiple targets from same source against same headers

### For LLMs

- **Minimal context** - Header file is the complete API reference
- **Self-documenting** - `@ optio descriptio` provides inline documentation
- **Uniform pattern** - All stdlib calls use `ad`, one syntax to learn
- **Predictable** - No need to understand implementation to use correctly

An LLM given only the header file can:
1. List all available endpoints (`@ ad` annotations)
2. Understand each endpoint's parameters (`@ optio` annotations)
3. Know sync vs async (`fit` vs `fiet` return)
4. Generate correct `ad` calls

### For Testing

- **Easy mocking** - Replace dispatch table entries with test doubles
- **No implementation dependency** - Tests compile against headers only
- **Behavior verification** - Assert on dispatch calls rather than side effects

```fab
# test setup
dispatch.mock("solum:lege", fit pro via { redde "mock content" })

# test runs against mock
ad "solum:lege" ("any-file") fiet textus pro content {
    adfirma content == "mock content"
}
```

### For Multi-Target

- **Single source** - Write once, compile to any target
- **Target-agnostic contracts** - Headers don't mention target-specific types
- **Implementation freedom** - Each target implements endpoints optimally

## Trade-offs

### Loss of Direct Calls

With headers, you lose:

```fab
# This requires importing the full module
ex "norma/solum" importa solum
fixum content = cede solum.lege("file.txt")
```

Everything becomes:

```fab
# Dispatch only
ad "solum:lege" ("file.txt") fiet textus pro content { }
```

**Mitigation:** The `ad` syntax is consistent and learnable. For simple cases, it's slightly more verbose. For complex cases (optional params, error handling), it's comparable.

### IDE Experience

Go-to-definition on `ad "solum:lege"` needs tooling support to jump from header to implementation.

**Mitigation:** Language server can map dispatch strings to implementations.

### Potential Runtime Overhead

Dispatch table lookup vs direct function call.

**Mitigation:**
- TypeScript/Python: Compiler inlines at codegen time, no runtime dispatch
- Zig/Rust: Link-time optimization can inline known endpoints
- Dynamic dispatch only needed for truly dynamic endpoint strings

## Relationship to Existing Proposals

### `ad` (ad.md)

This proposal extends `ad` by making it the **primary** (potentially only) way to call stdlib. The `ad.md` proposal treats it as one option among many; this proposal makes it central.

### `@ ad` Annotation (ad-annotatio.md)

The `@ ad` annotation is required for this proposal. Headers are essentially collections of `@ ad`-annotated function signatures.

### CLI Framework (grammatica/cli.md)

The `@ optio` syntax is borrowed from CLI argument declarations. This proposal extends it to all `ad` endpoints, not just CLI commands.

## Open Questions

1. **File discovery** - How does the compiler find `.fabh` files? Explicit import, convention-based, or config?

2. **Versioning** - How do headers indicate API versions? Can a program require `norma >= 2.0`?

3. **Extension** - Can user code define new endpoints via `@ ad` in `.fab` files, or only in `.fabh`?

4. **Escape hatch** - Is there a way to make direct calls when needed, or is `ad` strictly enforced?

5. **Generated headers** - Should the compiler generate `.fabh` from annotated `.fab` files, or are they hand-written?

6. **Namespace collisions** - What happens if two headers declare `@ ad "foo:bar"`?

## Implementation Path

1. **Define `.fabh` syntax** - Subset of `.fab` with `@ externa` requirement
2. **Header parser** - Load headers into dispatch table
3. **Type checker integration** - Validate `ad` calls against dispatch table
4. **Named parameter support** - Parse `ad` calls with `name: value` syntax
5. **Codegen per target** - Emit appropriate dispatch mechanism
6. **Stdlib headers** - Write `.fabh` for all `norma` modules
7. **Stdlib implementations** - Ensure implementations match headers

## References

- [ad.md](./ad.md) - Universal dispatch syntax
- [ad-annotatio.md](./ad-annotatio.md) - Endpoint registration annotation
- [grammatica/cli.md](../grammatica/cli.md) - CLI framework with `@ optio`
