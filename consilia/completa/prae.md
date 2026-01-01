---
status: implemented
targets: [ts, py, zig]
updated: 2024-12
---

# Prae: Compile-Time Evaluation

Compile-time computation for Faber, enabling static evaluation and type-level programming.

## Implementation Status

**Status: Implemented** (December 2024)

### What's Working

| Component          | Status | Notes                                                       |
| ------------------ | ------ | ----------------------------------------------------------- |
| Lexicon            | Done   | `prae`, `praefixum` keywords added                          |
| Parser             | Done   | `TypeParameterDeclaration`, `PraefixumExpression` AST nodes |
| Zig codegen        | Done   | `comptime T: type`, `comptime blk:`                         |
| TypeScript codegen | Done   | `<T>` generics; IIFE for `praefixum`                        |
| Python codegen     | Done   | `TypeVar('T')`; graceful degradation for `praefixum`        |
| Tests              | Done   | Parser tests, Zig codegen tests                             |
| Example            | Done   | `exempla/functiones/prae.fab`                               |
| Grammar docs       | Done   | Auto-generated in GRAMMAR.md                                |

### Known Issues

1. **Hex literals not tokenized** - `0xFF` parses incorrectly (tokenizer limitation, not prae-specific)
2. **Semantic check timing** - `praefixum` target check happens at codegen, not semantic phase
3. **Type literals as arguments** - `max(numerus, 10, 20)` fails semantic check (types not recognized as values)
4. **Mutable params in swap** - `in T a` still flagged as immutable by semantic analyzer

### Not Yet Implemented

- C++ codegen (`constexpr`, `template<typename T>`)
- Rust codegen (`const`, `<T>`)
- Semantic-phase target language validation for `praefixum`
- Compile-time constraint checking (I/O, runtime values)

## Keywords

| Context          | Keyword     | Meaning                               |
| ---------------- | ----------- | ------------------------------------- |
| Block/expression | `praefixum` | "pre-fixed" - compute at compile time |
| Type parameter   | `prae`      | "before" - type known at compile time |

## Etymology

- **praefixum** - past participle of `praefigere` (to fix beforehand)
    - `prae` (before) + `fixum` (fixed)
    - Extends `fixum` vocabulary: "fixed" → "pre-fixed"
- **prae** - preposition meaning "before" (i.e., before runtime)

## Syntax

### Compile-Time Blocks

Evaluate a block at compile time, embed result as constant:

```fab
fixum lookup = praefixum {
    varia table = []
    ex 0..256 pro i {
        table.adde(i ^ 0xFF)
    }
    redde table
}
```

The block executes during compilation. The result (`table`) is baked into the binary as a static constant.

### Compile-Time Expressions

Short form for simple expressions:

```fab
fixum size = praefixum(256 * 4)
fixum mask = praefixum(0xFF ^ 0xAA)
fixum greeting = praefixum("Hello, " + "World!")
```

### Type Parameters

Use `prae` (shorter) for function type parameters:

```fab
functio max(prae typus T, T a, T b) -> T {
    redde a > b sic a secus b
}

functio swap(prae typus T, in T a, in T b) {
    fixum temp = a
    a = b
    b = temp
}

functio create(prae typus T) -> T {
    redde novum T
}
```

Type parameters are inherently compile-time - the shorter `prae` reflects their common usage.

## Constraints

Everything inside `praefixum` must be evaluable at compile time:

- No runtime values (function arguments, mutable state from outer scope)
- No I/O operations (`scribe`, `lege`, network, filesystem)
- No allocators (arena, heap)
- Only literals, constants, and pure computation

```fab
# Valid - pure computation
fixum table = praefixum {
    varia result = []
    ex 0..10 pro i {
        result.adde(i * i)
    }
    redde result
}

# Invalid - runtime value
functio compute(numerus n) -> numerus {
    redde praefixum(n * 2)  # ERROR: n is runtime
}

# Invalid - I/O
fixum data = praefixum {
    scribe "computing..."  # ERROR: I/O not allowed
    redde 42
}
```

## Use Cases

### Lookup Tables

```fab
fixum sinTable = praefixum {
    varia table = []
    ex 0..360 pro deg {
        fixum rad = deg * 3.14159 / 180
        table.adde(sin(rad))
    }
    redde table
}

functio fastSin(numerus degrees) -> fractus {
    redde sinTable[degrees % 360]
}
```

### Type-Safe Generics

```fab
functio clamp(prae typus T, T value, T min, T max) -> T {
    si value < min { redde min }
    si value > max { redde max }
    redde value
}

fixum x = clamp(numerus, 150, 0, 100)  # 100
fixum y = clamp(fractus, 3.5, 0.0, 1.0)  # 1.0
```

### Conditional Compilation

```fab
fixum debug = praefixum(env("DEBUG") est "1")

functio log(textus msg) {
    si debug {
        scribe msg
    }
}
```

### Compile-Time Assertions

```fab
praefixum {
    adfirma sizeof(Packet) == 64, "Packet must be 64 bytes"
}
```

## Relationship to Test Unrolling

The `proba ex` syntax for table-driven tests is implicitly `praefixum`:

```fab
# This table is evaluated at compile time
proba "parse" ex [
    { input: "42", expect: 42 },
    { input: "-7", expect: -7 },
] pro { input, expect } {
    adfirma parse(input) est expect
}
```

The compiler unrolls the table into N separate test functions - compile-time evaluation producing static output.

For computed test cases, `praefixum` can be explicit:

```fab
proba "generated" ex praefixum(generateCases(100)) pro c {
    adfirma validate(c)
}
```

## Grammar

```ebnf
praefixumExpr := 'praefixum' (blockStmt | '(' expression ')')
praeTypeParam := 'prae' 'typus' IDENTIFIER
```

## Code Generation

### Zig

```fab
fixum mask = praefixum(0xFF ^ 0xAA)
```

```zig
const mask = comptime (0xFF ^ 0xAA);
```

```fab
fixum table = praefixum {
    varia result = []
    ex 0..10 pro i { result.adde(i * i) }
    redde result
}
```

```zig
const table = comptime blk: {
    var result: [10]i64 = undefined;
    for (0..10) |i| {
        result[i] = i * i;
    }
    break :blk result;
};
```

```fab
functio max(prae typus T, T a, T b) -> T { ... }
```

```zig
fn max(comptime T: type, a: T, b: T) T { ... }
```

### C++

```fab
fixum mask = praefixum(0xFF ^ 0xAA)
```

```cpp
constexpr auto mask = (0xFF ^ 0xAA);
```

```fab
functio max(prae typus T, T a, T b) -> T { ... }
```

```cpp
template<typename T>
T max(T a, T b) { ... }
```

### Rust

```fab
fixum mask = praefixum(0xFF ^ 0xAA)
```

```rust
const MASK: u8 = 0xFF ^ 0xAA;
```

```fab
functio max(prae typus T, T a, T b) -> T { ... }
```

```rust
fn max<T: Ord>(a: T, b: T) -> T { ... }
```

### TypeScript / Python

TypeScript and Python lack native compile-time evaluation. Rather than failing the build, `praefixum` degrades gracefully to runtime evaluation:

**TypeScript:**

```fab
fixum size = praefixum(256 * 4)
```

```typescript
const size = 256 * 4;
```

```fab
fixum table = praefixum { varia t = []; ex 0..10 pro i { t.adde(i) }; redde t }
```

```typescript
const table = (() => {
    let t = [];
    for (let i = 0; i < 10; i++) {
        t.push(i);
    }
    return t;
})();
```

**Python:**

```fab
fixum size = praefixum(256 * 4)
```

```python
SIZE = (256 * 4)
```

```fab
fixum table = praefixum { varia t = []; ex 0..10 pro i { t.adde(i) }; redde t }
```

```python
def __praefixum__(code):
    __globals__ = {"range": range, "len": len, "list": list, ...}
    __locals__ = {}
    exec(code, __globals__, __locals__)
    return __locals__.get('__result__')

table = __praefixum__('''t = []
for i in range(0, 10):
    t.append(i)
__result__ = t''')
```

Python lacks IIFEs, so complex blocks use a `__praefixum__` helper that executes code via `exec()` with restricted builtins (mimicking compile-time constraints: no I/O, limited stdlib).

**Future consideration:** A Faber interpreter could enable true `praefixum` for all targets by evaluating blocks at compile time and emitting the result as a literal. This is deferred until there's concrete demand.

## Implementation Notes

### Target Language Support

| Target     | `praefixum`          | `prae typus`           | Notes                          |
| ---------- | -------------------- | ---------------------- | ------------------------------ |
| Zig        | `comptime`           | `comptime T: type`     | Full support                   |
| C++        | `constexpr`          | `template<typename T>` | Full support                   |
| Rust       | `const`              | `<T>`                  | Full support                   |
| TypeScript | IIFE `(() => {})()`  | `<T>`                  | Runtime eval, not compile-time |
| Python     | `__praefixum__('''`) | `TypeVar`              | exec() with restricted globals |

### Compiler Requirements

For `praefixum`:

1. **Parse** - Recognize `praefixum` blocks/expressions as AST nodes
2. **Semantic check** - Verify target supports compile-time evaluation
3. **Codegen** - Emit target's native compile-time keyword (e.g., `comptime`)

For `prae typus`:

1. **Parse** - Recognize type parameters in function signatures
2. **Type inference** - Track generic type usage through function body
3. **Codegen** - Emit target's generic syntax

### Phases

1. Parse `praefixum` blocks as special AST nodes
2. During semantic analysis, check target language support
3. Emit with target's native compile-time keyword (no interpreter needed)

### Error Handling

```fab
fixum bad = praefixum {
    varia x = 1 / 0  # ERROR: Division by zero at compile time
    redde x
}
```

Compile-time errors during `praefixum` evaluation are reported as compiler errors with full context.

## Open Questions

### Recursion Limits

Should `praefixum` blocks have a recursion/iteration limit to prevent infinite compile times?

```fab
fixum infinite = praefixum {
    dum verum { }  # Infinite loop at compile time
    redde 0
}
```

Recommendation: Yes, configurable limit (default 10,000 iterations).

### Memory Limits

Large compile-time computations could exhaust memory. Should there be a limit on result size?

### Cross-Module Praefixum

Can `praefixum` blocks reference constants from other modules?

```fab
ex "./config" importa BASE_SIZE

fixum table = praefixum {
    # Can we use BASE_SIZE here?
}
```

Recommendation: Yes, if the imported value is itself `praefixum` or a literal.

### Praefixum Functions

Should entire functions be markable as compile-time only?

```fab
praefixum functio factorial(numerus n) -> numerus {
    si n <= 1 { redde 1 }
    redde n * factorial(n - 1)
}

fixum fact10 = factorial(10)  # Evaluated at compile time
```

Recommendation: Consider for v2. For now, inline `praefixum` blocks suffice.
