# Zig Target Notes

Zig is a systems programming target that presents interesting challenges due to its explicit, low-level nature. It serves as an educational bridge between high-level Latin syntax and systems programming concepts.

Zig and Rust share similar memory management concerns. Faber uses a **unified approach** for both targets: Latin prepositions (`de`, `in`) for borrowing semantics, and arena allocation for memory management. See rust.md for the full ownership design; this document covers Zig-specific details.

## Implementation Status Summary

| Category               | Status      | Notes                                                  |
| ---------------------- | ----------- | ------------------------------------------------------ |
| Variables              | Done        | `var`/`const` with type inference                      |
| Functions              | Done        | Parameters, return types, async stubs                  |
| Control flow           | Done        | `if`, `while`, `for`, `switch`                         |
| `genus`/struct         | Done        | Fields, methods, `init()` with `@hasField`             |
| `pactum`/interface     | Stub        | Emits doc comment only                                 |
| `ego` → `self`         | Done        | Explicit self parameter                                |
| `novum .. de`          | Done        | `@hasField` pattern with `.{}` default                 |
| Lambdas                | Done        | Anonymous struct `.call` pattern                       |
| Error handling         | Done        | `mori` → `@panic`, `iace` → `return error.X` with `!T` |
| Allocators             | Partial     | Arena preamble auto-emitted for collections            |
| `de`/`in` prepositions | Done        | `de` = borrowed/const, `in` = mutable pointer          |
| Collections            | Partial     | Core methods implemented, functional methods stubbed   |
| Comptime               | Done        | `prae typus` → `comptime T: type`, `praefixum` blocks  |
| Slices                 | Partial     | Type mapping works, runtime building needs allocators  |
| Tuples                 | Not started | `series<A,B>` designed but not implemented             |
| Tagged unions          | Done        | `discretio` → `union(enum)` with pattern matching      |

**Exempla Status: 25/45 passing (56%)**

## What Makes Zig Easier

### 1. `varia`/`fixum` → `var`/`const`

Direct 1:1 mapping. Faber's explicit mutability declaration is exactly what Zig wants.

### 2. `ego` (this) as Explicit

Faber treats `ego` as a real expression, not magic syntax sugar. Zig requires explicit `self` parameters in methods - so `ego.nomen` → `self.nomen` is a clean transformation.

### 3. `novum X { overrides }`

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

**Key insight:** Interface enforcement happens in Faber's semantic analyzer, not Zig. By the time Zig receives the code, it's already validated.

**Generated Zig:** Just the struct with methods. The `pactum` becomes a documentation comment.

### 2. Generators (`cursor` Functions)

Faber supports generator functions with `cede` yielding values. Zig has no generators - you'd need to manually build an iterator struct with state.

**Status:** Not implemented. Would require significant design work.

### 3. Async Model

Faber's `futura`/`cede` assumes JS-style Promise-based async. Zig's async is frame-based with explicit suspend/resume points.

**Current approach:** Fake it with error unions (`!T` return type). Not real async.

### 4. Generics

Faber has `lista<T>` style generics. Zig uses comptime type parameters: `fn ArrayList(comptime T: type)`. Runtime generic instantiation doesn't exist.

**Current approach:** Type annotations map (`lista<T>` → `[]T`), but no runtime generic support.

### 5. String Concatenation

Faber allows `"a" + "b"`. In Zig, you can only do this at comptime with `++`. Runtime string building needs an allocator.

**Current status:** Only works at comptime. Runtime concatenation would need arena allocator (not implemented).

### 6. Nullable Types

Faber's nullable types (`textus?`) map to Zig's `?[]const u8`. The mapping works, but Zig's optional handling requires explicit unwrapping (`.?`, `orelse`, `if (x) |val|`).

## Type Mappings

| Faber      | Zig          | Nullable      | Notes                                       |
| ---------- | ------------ | ------------- | ------------------------------------------- |
| `textus`   | `[]const u8` | `?[]const u8` | String slice                                |
| `numerus`  | `i64`        | `?i64`        | Integer                                     |
| `fractus`  | `f64`        | `?f64`        | Float                                       |
| `decimus`  | `f128`       | `?f128`       | Wide float                                  |
| `magnus`   | `i128`       | `?i128`       | Big integer                                 |
| `bivalens` | `bool`       | `?bool`       | Boolean                                     |
| `nihil`    | `void`       | —             | Unit type                                   |
| `vacuum`   | `void`       | —             | Void return                                 |
| `octeti`   | `[]u8`       | `?[]u8`       | Byte slice                                  |
| `objectum` | —            | —             | **No equivalent** - generates compile error |
| `ignotum`  | —            | —             | **No equivalent** - generates compile error |
| `numquam`  | `noreturn`   | —             | Never returns                               |

### Generic Types

| Faber          | Zig                        | Notes                                 |
| -------------- | -------------------------- | ------------------------------------- |
| `lista<T>`     | `[]T`                      | Slice (no allocator)                  |
| `tabula<K,V>`  | `std.StringHashMap(V)`     | Type only, unusable without allocator |
| `copia<T>`     | `std.AutoHashMap(T, void)` | Type only, unusable without allocator |
| `promissum<T>` | `!T`                       | Error union                           |
| `series<A,B>`  | —                          | **Not implemented**                   |

## Ownership Design: Latin Prepositions

> **Status:** Implemented for parameter types.

Faber uses Latin prepositions to annotate borrowing semantics. This design is shared with the Rust target.

| Preposition | Meaning             | Zig Output                          |
| ----------- | ------------------- | ----------------------------------- |
| (none)      | Owned, may allocate | Base type (uses module-level arena) |
| `de`        | Borrowed, read-only | `[]const T`, `*const T`             |
| `in`        | Mutable borrow      | `*T`, `*std.ArrayList(T)`           |

### Type Transformations

| Faber Type    | No Preposition         | `de` (borrowed)               | `in` (mutable)          |
| ------------- | ---------------------- | ----------------------------- | ----------------------- |
| `textus`      | `[]const u8`           | `[]const u8`                  | `*[]u8`                 |
| `numerus`     | `i64`                  | `*const i64`                  | `*i64`                  |
| `lista<T>`    | `[]T`                  | `[]const T`                   | `*std.ArrayList(T)`     |
| `tabula<K,V>` | `std.StringHashMap(V)` | `*const std.StringHashMap(V)` | `*std.StringHashMap(V)` |
| User type     | `T`                    | `*const T`                    | `*T`                    |

### Examples

```fab
// No preposition = owned value
functio process(numerus x) -> numerus {
    redde x * 2
}

// "de" = borrowed, read-only
functio length(de textus source) -> numerus {
    redde source.longitudo
}

// "in" = mutable pointer, will be modified
functio append(in lista<numerus> items, numerus value) {
    items.adde(value)
}
```

**Generates:**

```zig
fn process(x: i64) i64 {
    return x * 2;
}

fn length(source: []const u8) i64 {
    return @intCast(source.len);
}

fn append(items: *std.ArrayList(i64), value: i64) void {
    items.append(alloc, value) catch @panic("OOM");
}
```

### Not Yet Implemented

- Allocator threading to call sites (owned parameters that allocate)
- Return type prepositions (`-> de textus` for borrowed returns)
- Automatic `try` insertion at call sites for error-returning functions with prepositions

## Memory Management: Arena Allocator

> **Status:** Implemented for collections.

Faber uses arena allocation as the default memory strategy for systems targets.

### Why Arena?

1. **Simple mental model** - Allocate freely, everything freed at scope exit
2. **No explicit frees** - Generated code doesn't need `defer alloc.free(...)`
3. **Zero memory leaks** - Arena deinit handles everything
4. **Standard library** - Uses `std.heap.ArenaAllocator`, no external deps

### Generated Code Pattern

When a program uses collections (`lista`, `tabula`, or `copia`), the codegen automatically emits the arena preamble:

```zig
const std = @import("std");

pub fn main() void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const alloc = arena.allocator();

    // User code - allocations use arena
    var items = std.ArrayList(i64).init(alloc);
    items.append(alloc, 42) catch @panic("OOM");
}
```

The preamble is only emitted when the AST contains collection type annotations or `novum` expressions for collections.

## Collection Methods

> **Status:** Core methods implemented. Functional methods stubbed with `@compileError`.

### Implementation Files

- `fons/codegen/zig/norma/lista.ts` — ArrayList methods
- `fons/codegen/zig/norma/tabula.ts` — HashMap methods
- `fons/codegen/zig/norma/copia.ts` — HashSet (as HashMap with void values)

### lista<T> Methods

| Faber       | Zig Output                         | Status |
| ----------- | ---------------------------------- | ------ |
| `adde`      | `items.append(alloc, x)`           | [x]    |
| `remove`    | `items.pop()`                      | [x]    |
| `praepone`  | `items.insert(alloc, 0, x)`        | [x]    |
| `decapita`  | `items.orderedRemove(0)`           | [x]    |
| `purga`     | `items.clearRetainingCapacity()`   | [x]    |
| `primus`    | `items.items[0]`                   | [x]    |
| `ultimus`   | `items.items[items.items.len - 1]` | [x]    |
| `accipe`    | `items.items[i]`                   | [x]    |
| `longitudo` | `items.items.len`                  | [x]    |
| `vacua`     | `items.items.len == 0`             | [x]    |
| `continet`  | inline loop                        | [x]    |
| `indiceDe`  | inline loop                        | [x]    |
| `filtrata`  | `@compileError`                    | [-]    |
| `mappata`   | `@compileError`                    | [-]    |
| `reducta`   | `@compileError`                    | [-]    |

Functional methods (`filtrata`, `mappata`, `reducta`, etc.) emit `@compileError` with guidance to use `ex...pro` loops instead.

### tabula<K,V> Methods

| Faber       | Zig Output                     | Status |
| ----------- | ------------------------------ | ------ |
| `pone`      | `map.put(alloc, k, v)`         | [x]    |
| `accipe`    | `map.get(k)`                   | [x]    |
| `habet`     | `map.contains(k)`              | [x]    |
| `dele`      | `map.remove(k)`                | [x]    |
| `longitudo` | `map.count()`                  | [x]    |
| `vacua`     | `map.count() == 0`             | [x]    |
| `purga`     | `map.clearRetainingCapacity()` | [x]    |
| `claves`    | `map.keyIterator()`            | [x]    |
| `valores`   | `map.valueIterator()`          | [x]    |
| `paria`     | `map.iterator()`               | [x]    |
| `accipeAut` | `map.get(k) orelse default`    | [x]    |

### copia<T> Methods

Copia uses `std.AutoHashMap(T, void)` — a HashMap with void values as a set.

| Faber         | Zig Output                     | Status |
| ------------- | ------------------------------ | ------ |
| `adde`        | `set.put(alloc, x, {})`        | [x]    |
| `habet`       | `set.contains(x)`              | [x]    |
| `dele`        | `set.remove(x)`                | [x]    |
| `longitudo`   | `set.count()`                  | [x]    |
| `vacua`       | `set.count() == 0`             | [x]    |
| `purga`       | `set.clearRetainingCapacity()` | [x]    |
| `valores`     | `set.keyIterator()`            | [x]    |
| `unio`        | `@compileError`                | [-]    |
| `intersectio` | `@compileError`                | [-]    |
| `differentia` | `@compileError`                | [-]    |

### Design Notes

1. **Allocator threading** — Methods that allocate (`adde`, `praepone`, `pone`) use the module-level `alloc` from the arena preamble.

2. **Error handling** — Allocation failures panic with `catch @panic("OOM")`. This is intentional for simplicity; recoverable allocation failures would require error union propagation throughout.

3. **Functional methods** — Deliberately stubbed. Zig's philosophy favors explicit iteration over hidden allocation. Users should use `ex...pro` loops:

```fab
// Instead of: items.filtrata(pro x redde x > 0)
// Use:
varia result = novum lista<numerus>()
ex items pro x {
    si x > 0 { result.adde(x) }
}
```

## Error Handling Design

> **Status:** Implemented. `iace` generates proper error unions, `mori` generates panic.

| Keyword      | Meaning           | Output           | Status |
| ------------ | ----------------- | ---------------- | ------ |
| `iace`       | Recoverable error | `return error.X` | [x]    |
| `mori`       | Fatal/panic       | `@panic("msg")`  | [x]    |
| `fac`/`cape` | Error boundary    | Comment stub     | [ ]    |

### iace (Recoverable Error)

Functions containing `iace` automatically get error union return types (`!T`).
Error messages are converted to PascalCase error names.

```fab
functio fetch(textus url) -> textus {
    si timeout { iace "connection timeout" }
    redde data
}
```

**Generates:**

```zig
fn fetch(url: []const u8) ![]const u8 {
    if (timeout) { return error.ConnectionTimeout; }
    return data;
}
```

### mori (Fatal Panic)

Fatal errors generate `@panic` without changing the function signature.

```fab
functio initialize() {
    si nihil config { mori "config required" }
}
```

**Generates:**

```zig
fn initialize() void {
    if (config == null) { @panic("config required"); }
}
```

### What's Missing

1. Call sites don't automatically use `try` for error-returning functions
2. `fac`/`cape` blocks need proper `catch |err|` codegen

## Lambda Syntax

Faber uses `pro` for lambdas. Zig doesn't have closures - lambdas compile to anonymous struct functions.

```fab
pro x redde x * 2
```

**Generates:**

```zig
struct { fn call(x: i64) i64 { return x * 2; } }.call
```

**Capturing lambda (context struct):**

```fab
fixum multiplier = 2
pro x redde x * multiplier
```

**Generates:**

```zig
const Context = struct { multiplier: i64 };
const ctx = Context{ .multiplier = 2 };
fn lambda(ctx: *const Context, x: i64) i64 {
    return x * ctx.multiplier;
}
```

## Zig-Specific Features NOT in Faber

### Comptime

> **Status:** Implemented via `prae` and `praefixum` keywords. See `consilia/prae.md`.

Faber exposes Zig's comptime through two constructs:

**Type parameters:** `prae typus T` → `comptime T: type`

```fab
functio max(prae typus T, T a, T b) -> T {
    redde a > b sic a secus b
}
```

```zig
fn max(comptime T: type, a: T, b: T) T {
    return if (a > b) a else b;
}
```

**Compile-time blocks:** `praefixum { ... }` → `comptime blk: { ... }`

```fab
fixum table = praefixum {
    varia result = []
    ex 0..10 pro i { result.adde(i * i) }
    redde result
}
```

````zig
const table = comptime blk: {
    var result: [10]i64 = undefined;
    // ...
    break :blk result;
};

### Slices vs Arrays

Zig distinguishes fixed arrays `[N]T` from slices `[]T`. Faber's `lista<T>` maps to slice, but:

- Array literals `.{1,2,3}` don't coerce to slices
- Runtime slice building needs allocators

### Tagged Unions

> **Status:** Implemented via `discretio` keyword. See `consilia/unio.md`.

Faber's `discretio` maps directly to Zig's `union(enum)`:

```fab
discretio Event {
    Click { numerus x, numerus y }
    Keypress { textus key }
    Quit
}
```

```zig
const Event = union(enum) {
    click: struct { x: i64, y: i64 },
    keypress: struct { key: []const u8 },
    quit,
};
```

Pattern matching with `elige`/`ex` generates Zig switch statements:

```fab
elige event {
    ex Click pro x, y { scribe x, y }
    ex Quit { mori "goodbye" }
}
```

```zig
switch (event) {
    .click => |payload| {
        const x = payload.x;
        const y = payload.y;
        std.debug.print("{} {}\n", .{ x, y });
    },
    .quit => @panic("goodbye"),
}
```

## Exempla Test Results

**Current: 25/45 passing (56%)**

### Passing (25)

| Category   | Files                                                             |
| ---------- | ----------------------------------------------------------------- |
| Fundamenta | fixum, litterae, salve, scribe, varia (5/5)                       |
| Functiones | async, basic, praepositiones, typed, verba (5/5)                  |
| Regimen    | adfirma, custodi, elige, iace-mori, si-ergo (5/9)                 |
| Structurae | ego, genus/basic, genus/creo, genus/defaults, genus/methods (5/7) |
| Typi       | bigint, collectiones, nullable (3/4)                              |
| Operatores | ternarius, vel (2/7)                                              |

### Failure Categories

#### 1. Lambda Type Inference (9 files)

Lambdas without explicit return types generate `@compileError`. Zig requires explicit return types.

**Example:** `regimen/clausura.fab`

**Fix needed:** Infer lambda return types from context or require annotations.

#### 2. Unused Function Parameters (6 files)

Zig strictly enforces parameter usage. Functions with `anytype` parameters that aren't used fail.

**Files:** fundamenta/primitivi, operatores/logici, operatores/nulla, regimen/si-aliter, structurae/pactum, errores/tempta-cape

**Fix needed:** Prefix unused params with `_` or update exempla.

#### 3. `objectum` Return Type (3 files)

Functions returning `objectum` generate compile error since Zig has no equivalent.

**Files:** structurae/in, structurae/objecta, structurae/destructuring

**Fix needed:** Use concrete struct types.

#### 4. For-In Object Iteration (1 file)

`de obj pro key` generates invalid Zig - `for` only works on arrays/slices.

**File:** regimen/de-pro

**Fix needed:** Different codegen pattern for object key iteration.

#### 5. Loop Variable Redeclaration (1 file)

Multiple loops reusing same variable name.

**File:** operatores/intervalla

**Fix needed:** Generate unique loop variable names.

#### 6. Spread Operator (1 file)

Known limitation - generates `@compileError`.

**File:** operatores/sparge

## Future Work

### High Priority (Blocking Self-Hosting)

1. **Arena allocator expansion** - Runtime string/collection operations beyond preamble
2. **Return type prepositions** - `-> de textus` for borrowed returns with lifetime semantics

### Medium Priority

3. **Lambda type inference** - Reduce `@compileError` on lambdas without return types
4. **Slice literals** - Proper `[]T` from array literals
5. **`fac`/`cape` error handling** - Generate proper `catch |err|` blocks

### Lower Priority

6. **Build integration** - Generate `build.zig` for projects
7. **Functional collection methods** - `filtrata`, `mappata`, `reducta` via explicit loops

### Completed

- ~~Error unions~~ - `iace` generates `return error.X` with `!T` return types
- ~~Tagged unions~~ - `discretio` → `union(enum)` with pattern matching
- ~~Comptime~~ - `prae typus` and `praefixum` blocks
- ~~Collection methods~~ - Core methods for `lista`, `tabula`, `copia`
- ~~`de`/`in` prepositions~~ - Parameter ownership semantics

## Design Tensions

The core tension: **Faber leans toward dynamic/high-level semantics** while **Zig is explicitly low-level**.

The ownership prepositions and arena allocator bridge this gap, but neither is implemented yet. Current Zig codegen produces valid Zig for simple cases but fails on anything requiring runtime memory management.
````
