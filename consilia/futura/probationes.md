# Probationes: Execution Test Framework

Runtime execution tests for Faber, using Faber's native `proba` syntax to verify compiled code behavior.

## Motivation

The existing `fons/proba/` YAML tests verify **codegen output**—they check that Faber source compiles to expected target code. They don't verify **runtime behavior**—that executing the compiled code produces correct results.

For Nucleus and the Responsum protocol, we need execution tests that:
1. Compile Faber source to target code
2. Execute the generated code
3. Capture actual output/behavior
4. Assert correctness

## Directory Structure

```
fons/probationes/
├── README.md
├── nucleus/                    # Responsum protocol tests
│   ├── responsum.fab           # Protocol conformance
│   ├── streaming.fab           # fient/fiunt behavior
│   └── errors.fab              # iace/malum behavior
├── norma/                      # Stdlib execution tests
│   ├── solum.fab               # File I/O
│   ├── caelum.fab              # Network I/O
│   └── ...
└── integra/                    # Integration tests
    └── ...
```

## Relationship to `proba` Syntax

Execution tests use Faber's existing `proba`/`probandum`/`adfirma` syntax (see `consilia/completa/proba.md`). This provides:

- Dogfooding: test the language with the language
- Single syntax for all targets
- Native async support via `cede`

## Protocol Conformance Testing

The key use case is verifying Responsum protocol behavior across targets.

### Example: Single-Value Return

```fab
probandum "Responsum protocol" {
    proba "fiet returns single bene" {
        functio greet() fiet textus { redde "hello" }

        adfirma greet() est "hello"
    }

    proba "fiet with iace returns malum" {
        functio fail() fiet textus { iace "boom" }

        # Assertion syntax for expected errors TBD
        adfirma fail() iacit "boom"
    }
}
```

### Example: Streaming

```fab
probandum "Streaming protocol" {
    proba "fiunt yields multiple res then factum" {
        functio range(numerus n) fiunt numerus {
            varia i = 0
            dum i < n { cede i; i = i + 1 }
        }

        # Collect stream to list for assertion
        adfirma range(3) est [0, 1, 2]
    }

    proba "fient collects stream to single value" {
        functio sum(numerus n) fient numerus {
            varia total = 0
            ex range(n) pro i { total = total + i }
            redde total
        }

        adfirma sum(4) est 6  # 0+1+2+3
    }
}
```

## Cross-Target Execution

Tests should run on all supported targets. The test harness:

1. Compiles `.fab` file to each target
2. Executes compiled code (Bun for TS, `zig run` for Zig, etc.)
3. Collects results
4. Compares across targets

### Target-Specific Assertions

Some tests may have target-specific expectations:

```fab
proba "memory allocation" {
    # Zig/Rust: explicit allocator behavior
    # TS/Python: GC behavior

    @ si target = "zig" {
        # Zig-specific assertions
    }
    @ si target = "ts" {
        # TS-specific assertions
    }
}
```

## Protocol Shape Assertions

For Responsum conformance, we may need assertions that inspect protocol shape, not just final values:

```fab
proba "protocol shape" {
    functio range(numerus n) fiunt numerus { ... }

    # Hypothetical syntax for protocol inspection
    adfirma range(2).responsa est [
        { op: "res", data: 0 },
        { op: "res", data: 1 },
        { op: "factum" }
    ]
}
```

This would require:
- A way to capture raw Responsum stream
- Assertion syntax for protocol values
- Cross-target representation of protocol (JSON-like?)

## Open Questions

### Test Runner Architecture

**Option A: Faber-native runner**
- Test runner written in Faber
- Compiles to each target
- Self-hosting advantage

**Option B: External harness**
- Test runner in TS (like current `faber.test.ts`)
- Invokes compiler, executes targets
- Simpler initial implementation

Recommendation: Start with Option B, migrate to Option A when Faber is more mature.

### Async Test Semantics

Tests using `cede` are async. How does the harness handle:
- Timeouts for hung tests
- Concurrent test execution
- Resource cleanup on failure

### Protocol Inspection API

Should `proba` tests have access to raw Responsum protocol, or only unwrapped values?

**Arguments for protocol access:**
- Needed for conformance testing
- Can verify exact protocol shape

**Arguments against:**
- Protocol is internal implementation detail
- Tests should verify behavior, not implementation

Recommendation: Provide protocol access for `probationes/nucleus/` tests specifically, not general user tests.

### Error Assertion Syntax

Current `adfirma` doesn't handle expected errors. Options:

```fab
# Option 1: iacit keyword
adfirma fail() iacit "error message"

# Option 2: cape block in test
tempta { fail() } cape err { adfirma err.message est "..." }

# Option 3: Dedicated error test
proba iacit "expected error" "test name" {
    fail()
}
```

## Implementation Plan

1. **Directory structure** — Create `fons/probationes/` with initial test files
2. **Harness** — Extend existing test infrastructure to compile+execute
3. **TS execution** — Run compiled TS via Bun, capture output
4. **Assertions** — Verify `adfirma` works for execution results
5. **Zig execution** — Add `zig run` support when Zig codegen matures
6. **Protocol tests** — Add Responsum conformance tests

## References

- `consilia/completa/proba.md` — Existing `proba` syntax implementation
- `fons/proba/README.md` — Current YAML test framework
- `consilia/nucleus/responsum.md` — Responsum protocol specification
