# Target Compatibility

Faber compiles to multiple target languages, each with different capabilities. The compiler validates that your Faber code only uses features supported by your chosen target.

## Support Matrix

| Feature | ts | py | rs | zig | cpp |
|---------|----|----|----|----|-----|
| async functions (`fiet`) | ✓ | ✓ | ✓ | ✗ | ✗ |
| generators (`fiunt`) | ✓ | ✓ | ✗ | ✗ | ✗ |
| async generators (`fient`) | ✓ | ✓ | ✗ | ✗ | ✗ |
| try-catch (`tempta...cape`) | ✓ | ✓ | ✗ | ✗ | ✓ |
| throw (`iace`) | ✓ | ✓ | ✗ | ✗ | ✓ |
| object destructuring | ✓ | ✗ | ✗ | ✗ | ✗ |
| array destructuring | ✓ | ✓ | ✓ | ✓ | ✓ |
| default parameters (`vel`) | ✓ | ✓ | ✗ | ✗ | ✓ |

## Support Levels

The compiler uses four support levels:

- **supported**: Native implementation with correct semantics. Code will compile without errors.
- **unsupported**: Cannot be emitted; compilation fails with actionable error.
- **emulated**: Can be implemented with systematic transform, but may have performance/ergonomic costs. (Not yet implemented - currently treated as unsupported)
- **mismatched**: Can be "made to work" but semantics differ in important ways. (Not yet implemented - currently treated as unsupported)

Currently, only `supported` and `unsupported` levels are active. Features marked unsupported will cause compilation to fail.

## Error Format

When you use an unsupported feature, the compiler reports all incompatibilities before codegen:

```
Target compatibility errors for 'zig':

  file.fab:12:1 - Target 'zig' does not support async functions (futura)
    context: function fetch
    hint: Refactor to synchronous code; consider explicit callbacks/event loop

  file.fab:15:5 - Target 'zig' does not support try-catch (tempta...cape)
    context: try-catch block
    hint: Use error unions (!T) and handle errors explicitly
```

## Feature Details

### Async Functions (`fiet`)

**Faber syntax:**
```fab
functio fetch(textus url) fiet textus {
    redde "data"
}
```

**Supported targets:** TypeScript, Python, Rust

**Unsupported targets:** Zig, C++

**Why unsupported:**
- Zig uses explicit error unions and event loops, not async/await
- C++ has no standard async/await (coroutines require careful design)

**Alternatives:**
- Refactor to synchronous code
- Use target-specific concurrency primitives (goroutines, event loops)
- Consider splitting code by target if async is essential

### Generators (`fiunt`)

**Faber syntax:**
```fab
functio count(numerus n) fiunt numerus {
    varia i = 0
    dum i < n {
        cede i
        i = i + 1
    }
}
```

**Supported targets:** TypeScript, Python

**Unsupported targets:** Rust, Zig, C++

**Why unsupported:**
- Rust: Generators are unstable (nightly only)
- Zig/C++: No native generator support

**Alternatives:**
- Return an array/collection instead
- Use iterators with explicit state
- Use `while` loops at call site

### Exception Handling (`tempta...cape`, `iace`)

**Faber syntax:**
```fab
functio divide(numerus a, numerus b) fit numerus {
    tempta {
        si b == 0 {
            iace error("division by zero")
        }
        redde a / b
    } cape err {
        scribe err
        redde 0
    }
}
```

**Supported targets:** TypeScript, Python, C++

**Unsupported targets:** Rust, Zig

**Why unsupported:**
- Rust uses `Result<T, E>` for recoverable errors
- Zig uses error unions (`!T`) and explicit error handling
- Both reject exceptions for explicitness and zero-cost error handling

**Alternatives:**
- Use explicit return value checks
- Return optional types (`ignotum`)
- Restructure to avoid exceptional conditions

### Object Destructuring

**Faber syntax:**
```fab
genus Punto {
    numerus x
    numerus y
}

functio distance(Punto p) fit numerus {
    fixum { x, y } = p
    redde x * x + y * y
}
```

**Supported targets:** TypeScript only

**Unsupported targets:** Python, Rust, Zig, C++

**Why unsupported:**
- Python: No native object destructuring (only tuple/list unpacking)
- Rust: Pattern matching works differently (requires match expressions)
- Zig/C++: No destructuring syntax

**Alternatives:**
- Use explicit field access: `fixum x = p.x`
- Access fields inline: `redde p.x * p.x + p.y * p.y`

### Default Parameters (`vel`)

**Faber syntax:**
```fab
functio greet(textus name vel "World") fit textus {
    redde "Salve, " + name
}
```

**Supported targets:** TypeScript, Python, C++

**Unsupported targets:** Rust, Zig

**Why unsupported:**
- Rust: Use Option<T> or separate functions
- Zig: Use optional types (`?T`) with explicit null handling

**Alternatives:**
- Use function overloading (separate signatures)
- Accept optional type and check for null
- Provide separate convenience functions

## Writing Portable Code

To maximize portability across targets:

1. **Avoid async/generators unless necessary** - Most code doesn't need them
2. **Use explicit error handling** - Return optional types instead of throwing
3. **Access fields explicitly** - Don't rely on destructuring
4. **Provide all parameters** - Don't rely on defaults
5. **Use collections and loops** - These work everywhere

Example of portable code:

```fab
genus Punto {
    numerus x
    numerus y
}

functio distance(Punto a, Punto b) fit numerus {
    fixum dx = a.x - b.x
    fixum dy = a.y - b.y
    redde dx * dx + dy * dy
}

functio sum(numerus[] items) fit numerus {
    varia total = 0
    ex items pro item {
        total = total + item
    }
    redde total
}
```

This code compiles to TypeScript, Python, Rust, Zig, and C++ without modification.

## Future Work

### Emulated Support Level

Features with runtime cost but systematic implementations may be marked `emulated`:

- Default parameters → function overloads
- Simple generators → manual iterator classes
- Object destructuring → multiple statements

When implemented, these will require opt-in via CLI flag: `--allow-emulated`

### Mismatched Support Level

Features with semantic differences may be marked `mismatched`:

- Go goroutines are not async/await (different execution model)
- Zig async is actually for async I/O, not general coroutines
- Language-specific coalesce operators with different null semantics

When implemented, these will require opt-in via CLI flag: `--allow-mismatched`

### Policy Control

Future CLI flags for fine-grained control:

```bash
faber compile program.fab -t zig --allow-emulated   # Accept emulation
faber compile program.fab -t go --allow-mismatched  # Accept semantic differences
faber compile program.fab -t rs --strict            # Fail on warnings
```

## Target-Specific Notes

### TypeScript

TypeScript supports all Faber features. Use it for prototyping and as a reference implementation.

### Python

Python supports most features except object destructuring. Use tuple unpacking or explicit field access instead.

### Rust

Rust is a systems language with explicit error handling. Use `Result<T, E>` patterns instead of exceptions. Generators are not available in stable Rust.

### Zig

Zig is a minimal systems language. Use error unions (`!T`) for errors, explicit state for iteration, and synchronous code. Zig values explicitness over convenience.

### C++

C++ supports exceptions but not async/await or generators. Keep code synchronous unless using a specific async runtime library.

## Checking Compatibility

To verify your code is compatible with a target before compiling:

```bash
bun run faber compile program.fab -t zig
```

The compiler will report all incompatibilities before attempting codegen. Fix the issues and recompile.

## Related

- Design document: `consilia/capabilities.md`
- Implementation: `fons/faber/codegen/capabilities.ts`, `validator.ts`, `feature-detector.ts`
- Tests: `fons/proba/capabilities/`
