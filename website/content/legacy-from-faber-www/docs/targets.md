---
title: Target Compatibility
section: docs
order: 50
---

# Target Compatibility

Faber compiles to multiple target languages, each with different capabilities. The compiler validates that your Faber code only uses features supported by your chosen target.

## Support Matrix

**Legend:** ✓ = supported, ~ = emulated, ✗ = unsupported

| Feature                     | ts  | py  | rs  | zig | cpp |
| --------------------------- | --- | --- | --- | --- | --- |
| async functions (`fiet`)    | ✓   | ✓   | ✓   | ✗   | ✗   |
| generators (`fiunt`)        | ✓   | ✓   | ✗   | ✗   | ✗   |
| async generators (`fient`)  | ✓   | ✓   | ✗   | ✗   | ✗   |
| try-catch (`tempta...cape`) | ✓   | ✓   | ~   | ~   | ✓   |
| throw (`iace`)              | ✓   | ✓   | ~   | ~   | ✓   |
| object destructuring        | ✓   | ✗   | ~   | ~   | ~   |
| array destructuring         | ✓   | ✓   | ✓   | ✓   | ✓   |
| default parameters (`vel`)  | ✓   | ✓   | ✗   | ✗   | ✓   |

## Support Levels

The compiler uses three support levels:

- **supported** (✓): Native implementation with correct semantics. Code compiles without errors.
- **emulated** (~): Systematic transform (e.g., `Result<T,E>` for exceptions, field-by-field extraction for destructuring). May have performance/ergonomic costs but semantics are preserved.
- **unsupported** (✗): Cannot be emitted; compilation fails with actionable error.

Emulated features work correctly but may be less idiomatic in the target language. For example, Rust's `tempta...cape` becomes `Result<T,E>` patterns, and Zig's `iace` becomes `return error.X`.

## Error Format

When you use an unsupported feature, the compiler reports all incompatibilities before codegen:

```
Target compatibility errors for 'zig':

  file.fab:12:1 - Target 'zig' does not support async functions (@ futura)
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

**Emulated targets:** Rust, Zig

**Unsupported targets:** None

**Emulation details:**

- **Rust**: `tempta...cape` transforms to `Result<T, E>` patterns. `iace` becomes `return Err("msg")`.
- **Zig**: `tempta...cape` transforms to error union handling. `iace` becomes `return error.X`.

**Performance notes:**
Emulated error handling preserves semantics but may be less idiomatic than native patterns in the target language.

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

**Supported targets:** TypeScript

**Emulated targets:** Rust, Zig, C++

**Unsupported targets:** Python

**Emulation details:**
Object destructuring transforms to field-by-field extraction:

```fab
fixum { x, y } = p
```

becomes:

```
const x = p.x;
const y = p.y;
```

**Performance notes:**
No performance cost - the emulated code is semantically identical to native destructuring.

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
    itera ex items fixum item {
        total = total + item
    }
    redde total
}
```

This code compiles to TypeScript, Python, Rust, Zig, and C++ without modification.

## Future Work

### Mismatched Support Level (Not Yet Implemented)

Features with semantic differences may be marked `mismatched`:

- Go goroutines are not async/await (different execution model)
- Zig async is actually for async I/O, not general coroutines
- Language-specific coalesce operators with different null semantics

When implemented, these will require opt-in via CLI flag: `--allow-mismatched`

### Policy Control (Future)

Potential CLI flags for fine-grained control:

```bash
faber compile program.fab -t zig --warn-emulated    # Warn on emulated features
faber compile program.fab -t go --allow-mismatched  # Accept semantic differences (when implemented)
faber compile program.fab -t rs --strict            # Fail on warnings
```

## Target-Specific Notes

### TypeScript

TypeScript supports all Faber features. Use it for prototyping and as a reference implementation.

### Python

Python supports most features except object destructuring. Use tuple unpacking or explicit field access instead.

### Rust

Rust is a systems language with explicit error handling. Exception handling (`tempta...cape`, `iace`) is emulated via `Result<T, E>` transforms. Object destructuring is emulated via field-by-field extraction. Generators are not available in stable Rust.

### Zig

Zig is a minimal systems language. Exception handling (`tempta...cape`, `iace`) is emulated via error union transforms. Object destructuring is emulated via field-by-field extraction. Async/await is not supported (Zig uses explicit event loops).

### C++

C++ supports native exception handling (`try`/`catch`/`throw`). Object destructuring is emulated via field-by-field extraction. Async/await and generators are not supported (coroutines are experimental).

## Checking Compatibility

To verify your code is compatible with a target before compiling:

```bash
bun run faber compile program.fab -t zig
```

The compiler will report all incompatibilities before attempting codegen. Fix the issues and recompile.

## Related

- Design document: `consilia/capabilities.md`
- Implementation: `fons/rivus/codegen/` (target-specific modules)
- Tests: `fons/proba/capabilities/`

