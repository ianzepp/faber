# Zig Target Notes

Zig is a systems programming target that presents interesting challenges due to its explicit, low-level nature. It serves as an educational bridge between high-level Latin syntax and systems programming concepts.

Zig and Rust share similar memory management concerns. Faber uses a **unified approach** for both targets: Latin prepositions (`de`, `in`) for borrowing semantics, and arena allocation for memory management. See rust.md for the full ownership design; this document covers Zig-specific details.

## What Makes Zig Easier

### 1. `varia`/`fixum` → `var`/`const`

Direct 1:1 mapping. Faber's explicit mutability declaration is exactly what Zig wants.

### 2. `ego` (this) as Explicit

Faber treats `ego` as a real expression, not magic syntax sugar. Zig requires explicit `self` parameters in methods - so `ego.nomen` → `self.nomen` is a clean transformation.

### 3. `novum X cum { overrides }`

Maps beautifully to Zig's comptime capabilities:

```zig
pub fn init(overrides: anytype) Self {
    var self = Self{
        .field = if (@hasField(@TypeOf(overrides), "field")) overrides.field else default,
    };
    return self;
}
```

The `anytype` + `@hasField` pattern is idiomatic Zig and gives auto-merge behavior without runtime overhead.

### 4. No Inheritance

Faber's `genus` doesn't support class inheritance - only `implet` (interface implementation). Zig also has no inheritance. Good alignment.

### 5. Explicit Types

Faber encourages type annotations (`numerus x = 5`). Zig requires them for `var`. The language nudges users toward what Zig needs.

## What Makes Zig Harder

### 1. `pactum` (Interfaces)

Faber has nominal interfaces (`pactum`). Zig uses structural/duck typing with no interface declarations.

**Key insight:** Interface enforcement happens in Faber's semantic analyzer, not Zig. This is analogous to TypeScript → JavaScript: TypeScript enforces types at compile time, then emits untyped JavaScript. By the time JS runs, the types are already validated.

Similarly:
1. Faber checks that `genus X implet Y` has all methods declared in `pactum Y`
2. Errors are raised during Faber compilation if methods are missing
3. Zig receives already-validated code — no interface info needed

```
// Faber - semantic analyzer catches this
pactum Nominatus {
    functio nomen() -> textus
}

genus Persona implet Nominatus {
    // error: missing method 'nomen' required by Nominatus
}
```

**Generated Zig:** Just the struct with methods. The `pactum` becomes a documentation comment for human readers. Zig's duck typing "just works" because Faber already verified the methods exist.

### 2. Generators (`cursor` Functions)

Faber supports generator functions with `cede` yielding values. Zig has no generators - you'd need to manually build an iterator struct with state. This is a significant impedance mismatch.

**Current approach:** Not implemented.

### 3. Async Model

Faber's `futura`/`cede` assumes JS-style Promise-based async. Zig's async is frame-based with explicit suspend/resume points and allocator concerns.

**Current approach:** Fake it with error unions (`!T` return type, `try` for cede).

### 4. Generics

Faber has `lista<T>` style generics. Zig uses comptime type parameters: `fn ArrayList(comptime T: type)`. Runtime generic instantiation doesn't exist - everything is monomorphized at compile time.

**Current approach:** Not implemented for genus. Collection types not supported.

### 5. String Concatenation

Faber allows `"a" + "b"`. In Zig, you can only do this at comptime with `++`. Runtime string building needs an allocator and `std.mem.concat` or similar.

**Solution:** Arena allocator (see Memory Management section below). Runtime concatenation uses `std.fmt.allocPrint` with arena allocator.

### 6. Nullable Types

Faber's nullable types (`textus?`) map to Zig's `?[]const u8`. The mapping works, but Zig's optional handling requires explicit unwrapping (`.?`, `orelse`, `if (x) |val|`) that Faber doesn't surface.

## Current Implementation

| Feature | Status | Notes |
|---------|--------|-------|
| Variables | Done | `var`/`const` |
| Functions | Done | `fn` with params and return type |
| Control flow | Done | `if`, `while`, `for`, `switch` |
| `genus` | Done | `struct` with `init()` |
| `pactum` | Done | Enforced in semantic analyzer; emits doc comment |
| `ego` | Done | → `self` |
| `novum cum` | Done | `@hasField` pattern |
| Async | Partial | Error unions, not real async |
| Generators | No | Would need iterator struct |
| Generics | No | Would need comptime params |
| Collections | No | `lista` methods not mapped |

## Type Mappings

| Faber | Zig | Notes |
|-------|-----|-------|
| `textus` | `[]const u8` | String slice |
| `numerus` | `i64` | Target default (integer) |
| `bivalens` | `bool` | Boolean |
| `nihil` | `void` | Unit type |
| `vacuum` | `void` | Void return |
| `textus?` | `?[]const u8` | Optional |

## Ownership Design: Latin Prepositions

> **Note:** The `de` and `in` keywords are **systems-target-specific**. They are only valid in Faber projects configured to compile to Zig or Rust. TypeScript or Python targets would reject these as syntax errors.

Faber uses Latin prepositions to annotate borrowing semantics. This design is shared with the Rust target (see rust.md for full rationale).

| Preposition | Meaning | Zig Output |
|-------------|---------|------------|
| (none) | Owned, may allocate | Allocator-managed value |
| `de` | Borrowed, read-only | `[]const u8`, `*const T` |
| `in` | Mutable borrow | `*T`, `*[]u8` |

### Examples

```
// No preposition = owned, allocator-managed
functio greet(textus name) -> textus {
    redde "Hello, " + name + "!"
}

// "de" = borrowed slice, read-only
functio length(de textus source) -> numerus {
    redde source.longitudo
}

// "in" = mutable pointer, will be modified
functio append(in lista<textus> items, textus value) {
    items.adde(value)
}
```

**Zig output:**
```zig
fn greet(alloc: Allocator, name: []const u8) []const u8 {
    return std.fmt.allocPrint(alloc, "Hello, {s}!", .{name}) catch @panic("OOM");
}

fn length(source: []const u8) i64 {
    return @intCast(source.len);
}

fn append(alloc: Allocator, items: *std.ArrayList([]const u8), value: []const u8) void {
    items.append(alloc, value) catch @panic("OOM");
}
```

### Type Mappings with Prepositions

| Faber | Zig | Notes |
|-------|-----|-------|
| `textus` | arena-allocated `[]const u8` | Owned string |
| `de textus` | `[]const u8` | Borrowed slice (no alloc needed) |
| `in textus` | `*std.ArrayList(u8)` | Mutable string buffer |
| `lista<T>` | `std.ArrayList(T)` | Arena-managed list |
| `de lista<T>` | `[]const T` | Borrowed slice view |
| `in lista<T>` | `*std.ArrayList(T)` | Mutable list pointer |

## Memory Management: Arena Allocator

Faber uses arena allocation as the default memory strategy for systems targets. This provides GC-like ergonomics without runtime overhead.

### Why Arena?

1. **Simple mental model** - Allocate freely, everything freed at scope exit
2. **No explicit frees** - Generated code doesn't need `defer alloc.free(...)`
3. **Zero memory leaks** - Arena deinit handles everything
4. **Standard library** - Uses `std.heap.ArenaAllocator`, no external deps
5. **Unified approach** - Same strategy works for Rust (`bumpalo`)

### Generated Code Pattern

```zig
const std = @import("std");

pub fn main() void {
    // Arena wraps all allocations
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const alloc = arena.allocator();

    // User code - allocations use arena
    const greeting = greet(alloc, "World");
    std.debug.print("{s}\n", .{greeting});

    // Arena freed on function exit - no manual cleanup
}
```

### Allocator Threading

Functions that may allocate receive `alloc: std.mem.Allocator` as a hidden first parameter. The codegen inserts this automatically for:
- Functions returning owned values (no `de` on return type)
- Functions that concatenate strings
- Functions that build collections

Functions with only borrowed parameters (`de`) and borrowed returns don't need allocator.

```
// Needs allocator (returns owned)
functio build() -> textus { ... }
// Zig: fn build(alloc: Allocator) []const u8

// No allocator needed (all borrowed)
functio first(de lista<textus> items) -> de textus { ... }
// Zig: fn first(items: []const []const u8) []const u8
```

### Scope-Based Arenas (Future)

For long-running programs, nested arenas can bound memory lifetime:

```
// Faber (future syntax)
fac arena {
    // allocations here freed when block exits
}
```

WHY `fac arena` not `cum arena`: The `cum` keyword is already used for context objects (`cum user { nomen = "Marcus" }`). Using `fac arena` extends the existing block syntax with a modifier, avoiding semantic confusion. The word "arena" is genuinely Latin (sand, amphitheater floor) and familiar to systems programmers.

This maps to Zig's pattern of creating child arenas for bounded scopes.

## Lambda Syntax

Faber uses `fac` for lambdas/closures, with `fit`/`fiet` as the delimiter between parameters and body.

### Syntax

```
fac <params> fit <expr>   // sync lambda
fac <params> fiet <expr>  // async lambda
```

Parameters are comma-separated identifiers, terminated by `fit` (sync) or `fiet` (async).

### Examples

```
fixum double = fac x fit x * 2
fixum add = fac x, y fit x + y
fixum fetcher = fac url fiet getData(url)

// With higher-order functions
lista.mappata(fac item fit item.nomen)
lista.reducta(fac acc, x fit acc + x, 0)
lista.filtrata(fac x fit x > 0)
```

Reads as: "do x, becomes x times 2"

### Zig Output

Zig doesn't have closures. Lambdas compile to function pointers with explicit context.

**Non-capturing lambda:**
```
// Faber
fac x fit x * 2

// Zig
struct { fn call(x: i64) i64 { return x * 2; } }.call
```

**Capturing lambda:**
```
// Faber
fixum multiplier = 2
fac x fit x * multiplier

// Zig - context struct
const Context = struct { multiplier: i64 };
const ctx = Context{ .multiplier = 2 };
fn lambda(ctx: *const Context, x: i64) i64 {
    return x * ctx.multiplier;
}
```

The compiler identifies captures (variables referenced but not in param list) and generates context structs automatically.

### Capture Semantics

For Zig, all captures are by value (copied into context struct). This matches Zig's explicit ownership model. If mutable capture is needed, use `in` preposition on the captured variable.

## Error Handling Design

Faber uses a simplified error model that maps cleanly to Zig's error unions. This design is shared with the Rust target.

### Keywords

| Keyword | Meaning | Zig Output |
|---------|---------|------------|
| `iace` | Expected failure, recoverable | `return error.X` |
| `mori` | Fatal, unrecoverable | `@panic("msg")` |
| `fac` | Block scope (like `{}`) | scoped block |
| `cape` | Catch errors on block | `catch \|err\|` |

**Target-specific keywords:**
- `tempta`/`demum` - Available for TS/Python (maps to try/finally), but **not valid for Zig** because `demum` has no equivalent. Use `fac`/`cape` instead.

### `fac` vs `tempta`

`fac` (do!) is a simple block scope. `tempta` (try) implies the full try/catch/finally pattern.

```
// Systems targets (Zig/Rust): use fac
fac {
    riskyCall()
} cape err {
    handleError(err)
}

// TS/Python: tempta with demum works
tempta {
    riskyCall()
} cape err {
    handleError(err)
} demum {
    cleanup()  // always runs
}
```

For Zig, `tempta` is rejected because it implies `demum` which cannot be implemented. Use `fac` for explicit block scoping with error handling.

### `iace` - Recoverable Errors

`iace` (throw) becomes an error return. The compiler automatically:
1. Marks functions containing `iace` as failable
2. Changes return type from `T` to `!T`
3. Inserts `try` at call sites of failable functions
4. Wraps successful returns appropriately

```
// Faber
functio fetch(url: textus) -> textus {
    si timeout { iace "timeout" }
    redde data
}

fixum result = fetch(url)
```

**Zig output:**
```zig
fn fetch(alloc: Allocator, url: []const u8) ![]const u8 {
    if (timeout) { return error.Timeout; }
    return data;
}

const result = try fetch(alloc, url);
```

### `mori` - Fatal Errors

`mori` (to die) indicates an unrecoverable error - a bug or impossible state. Maps directly to `@panic`.

```
// Faber
si index < 0 { mori "negative index" }

// Zig
if (index < 0) { @panic("negative index"); }
```

### `cape` - Error Boundaries

`cape` attaches to `fac` blocks or control flow statements to catch errors.

```
// Faber - fac block with cape
fac {
    riskyCall()
} cape err {
    handleError(err)
}

// cape on control flow
si needsData {
    fetchData()
} cape err {
    useFallback()
}
```

**Zig output:**
```zig
if (riskyCall()) |_| {
    // success path
} else |err| {
    handleError(err);
}

if (needsData) {
    if (fetchData()) |_| {
        // success
    } else |err| {
        useFallback();
    }
}
```

### Error Type Inference

The compiler infers error types from `iace` usage. String literals become error enum members:

```
iace "timeout"     // -> error.Timeout
iace "not_found"   // -> error.NotFound
```

Functions that can fail get an inferred error set based on all possible errors in their call graph.

## Design Tensions

The core tension: **Faber leans toward dynamic/high-level semantics** while **Zig is explicitly low-level**. The ownership prepositions and arena allocator bridge this gap, giving Zig users familiar semantics without sacrificing Faber's accessibility.

Remaining tensions:
- **Comptime vs runtime** - Faber doesn't distinguish; may generate runtime code where comptime would work
- **Generics** - Faber's runtime-style generics vs Zig's comptime monomorphization

## Current Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| Variables | Done | `var`/`const` |
| Functions | Done | `fn` with params and return type |
| Control flow | Done | `if`, `while`, `for`, `switch` |
| `genus` | Done | `struct` with `init()` |
| `pactum` | Done | Enforced in semantic analyzer; emits doc comment |
| `ego` | Done | → `self` |
| `novum cum` | Done | `@hasField` pattern |
| Async | Partial | Error unions, not real async |
| Generators | No | Would need iterator struct |
| Generics | No | Would need comptime params |
| Collections | No | `lista` methods not mapped |
| `de`/`in` prepositions | No | Design complete, not implemented |
| Arena allocator | No | Design complete, not implemented |
| `iace`/`mori` errors | No | Design complete, not implemented |
| `fac` lambdas | No | Design complete, not implemented |

## Future Considerations

1. ~~**Allocator threading**~~ - Solved: arena allocator with implicit threading
2. ~~**Ownership annotations**~~ - Solved: `de`/`in` prepositions (shared with Rust)
3. ~~**Error handling**~~ - Solved: `iace`/`mori` split, `fac`/`cape` for blocks
4. ~~**Lambda syntax**~~ - Solved: `fac x fit expr` with context struct for captures
5. **Iterator pattern** - Manual struct for generators
6. **Comptime generics** - `fn(comptime T: type)` for generic types
7. **Build integration** - Generate `build.zig` for projects
8. ~~**Scope-based arenas**~~ - Decided: `fac arena { }` for bounded allocation lifetimes
