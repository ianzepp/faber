---
status: completed
updated: 2026-01-06
note: @ externa annotation fully implemented for TypeScript and Zig targets. No tests yet.
implemented:
  - @ externa annotation parsing
  - TypeScript codegen (declare const/function)
  - Zig codegen (extern var/fn)
  - Semantic validation (skip missing init/body errors)
not_implemented:
  - Python, Rust, C++ targets
  - Test coverage
  - Conditional compilation (@ externa(ts) vs @ externa(zig))
  - C FFI annotations (@ c_string)
---

# Externa Declarations

External declarations tell the compiler "this symbol exists but is provided elsewhere" - by the runtime, FFI, or linker.

## Syntax

```fab
@ externa
fixum errno: numerus

@ externa
functio printf(textus format, ...args) -> numerus
```

The `@ externa` annotation marks a declaration as externally provided. No initializer or body is required.

## Target Output

### TypeScript

Emits `declare` statements (ambient declarations):

```fab
@ externa
fixum Bun: ignotum

@ externa
fixum process: ignotum

@ externa
functio require(textus path) -> ignotum
```

```ts
declare const Bun: unknown;
declare const process: unknown;
declare function require(path: string): unknown;
```

These tell TypeScript "trust me, this exists at runtime" without emitting any code.

### Zig

Emits `extern` declarations for linker-resolved symbols:

```fab
@ externa
fixum errno: numerus

@ externa
functio printf(textus format) -> numerus

@ externa
functio malloc(numerus size) -> octeti
```

```zig
extern var errno: i64;
extern fn printf(format: [*:0]const u8) i64;
extern fn malloc(size: usize) [*]u8;
```

These tell Zig "this symbol will be provided at link time" - typically from libc or other native libraries.

## Use Cases

### Runtime Globals (TS/JS)

```fab
@ externa
fixum Bun: ignotum

@ externa
fixum process: ignotum

@ externa
fixum globalThis: ignotum

incipit {
    fixum args = (process.argv qua lista<textus>)
    scribe args[0]
}
```

### C FFI (Zig)

```fab
@ externa
functio puts(textus s) -> numerus

@ externa
functio getenv(textus name) -> textus?

incipit {
    fixum home = getenv("HOME")
    si nonnihil home {
        puts(home)
    }
}
```

### Platform-Specific APIs

```fab
# Browser only
@ externa
fixum window: ignotum

@ externa
fixum document: ignotum

# Node only
@ externa
fixum __dirname: textus

@ externa
fixum __filename: textus
```

## Semantic Rules

1. **No initializer required** - `@ externa fixum x: T` is valid without `= value`
2. **Type annotation required** - the compiler needs to know the type for downstream usage
3. **No body for functions** - `@ externa functio f()` has no `{ ... }` block
4. **Scope registration** - the name is added to scope so references don't error
5. **No duplicate externals** - redeclaring the same external is an error

## Validation

### Cross-Target Externals

Some externals only make sense on specific targets:

```fab
@ externa
fixum Bun: ignotum  # JS runtime - no Zig equivalent
```

Options for handling:
1. **Error on incompatible target** - `Bun` errors when compiling to Zig
2. **Conditional compilation** - `@ externa(ts)` vs `@ externa(zig)`
3. **Stub generation** - emit a panic/undefined for incompatible targets

Recommended: Error on incompatible target with clear message.

### Type Mapping

External types need careful mapping:

| Faber | TypeScript | Zig |
|-------|------------|-----|
| `textus` | `string` | `[*:0]const u8` (C string) |
| `numerus` | `number` | `i64` or `c_int` depending on context |
| `octeti` | `Uint8Array` | `[*]u8` |
| `ignotum` | `unknown` | `anyopaque` |

For C FFI, additional annotations may be needed:

```fab
@ externa
@ c_string  # textus → [*:0]const u8 instead of []const u8
functio puts(textus s) -> numerus
```

## Implementation

### Parser

Recognize `@ externa` annotation on:
- `VariaSententia` (fixum/varia declarations)
- `FunctioDeclaratio` (function declarations)

Allow missing initializer/body when `externa` is present.

### AST

Add `externa: boolean` field to relevant AST nodes, or track via annotation list.

### Semantic Analyzer

1. Check for `@ externa` annotation
2. Register name in scope with declared type
3. Skip "missing initializer" / "missing body" errors for externa
4. Validate type annotation is present

### TypeScript Codegen

```fab
@ externa
fixum X: T
```

Emit:
```ts
declare const X: T;
```

For functions:
```ts
declare function name(params): ReturnType;
```

### Zig Codegen

```fab
@ externa
fixum X: T
```

Emit:
```zig
extern var X: ZigType;
```

For functions:
```zig
extern fn name(params) ReturnType;
```

## Future Extensions

### Typed External Modules

```fab
@ externa
modulum "libc" {
    functio printf(textus format) -> numerus
    functio malloc(numerus size) -> octeti
    functio free(octeti ptr)
}
```

### Link Attributes

```fab
@ externa
@ link("libssl")
functio SSL_new() -> ignotum
```

### Calling Conventions

```fab
@ externa
@ callconv("c")
functio callback(numerus x) -> numerus
```

## References

- TypeScript: [Ambient Declarations](https://www.typescriptlang.org/docs/handbook/2/type-declarations.html)
- Zig: [extern](https://ziglang.org/documentation/master/#extern)
