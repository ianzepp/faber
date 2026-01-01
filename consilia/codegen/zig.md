---
status: partial
targets: [zig]
note: Core language working; collections partial; allocators (arena/page/curator) implemented
updated: 2025-12
---

# Zig Target

Zig is a systems programming target. Faber uses Latin prepositions (`de`, `in`) for borrowing semantics and arena allocation for memory management. See `praepositiones.md` for the unified preposition system.

## Status

| Category               | Status  | Notes                                                  |
| ---------------------- | ------- | ------------------------------------------------------ |
| Variables              | Done    | `var`/`const` with type inference                      |
| Functions              | Done    | Parameters, return types                               |
| Control flow           | Done    | `if`, `while`, `for`, `switch`                         |
| `genus`/struct         | Done    | Fields, methods, `init()` with `@hasField`             |
| `pactum`/interface     | Stub    | Emits doc comment only                                 |
| `ego` → `self`         | Done    | Explicit self parameter                                |
| `novum .. de`          | Done    | `@hasField` pattern with `.{}` default                 |
| Lambdas                | Done    | Anonymous struct `.call` pattern                       |
| Error handling         | Done    | `mori` → `@panic`, `iace` → `return error.X` with `!T` |
| Allocators             | Done    | `cura arena/page`, `curator` type, curatorStack        |
| `de`/`in` prepositions | Done    | `de` = borrowed/const, `in` = mutable pointer          |
| Collections            | Partial | Core methods implemented, functional methods stubbed   |
| Comptime               | Done    | `prae typus` → `comptime T: type`, `praefixum` blocks  |
| Tagged unions          | Done    | `discretio` → `union(enum)` with pattern matching      |

## Type Mappings

### Primitives

| Faber      | Zig          | Nullable      | Notes                        |
| ---------- | ------------ | ------------- | ---------------------------- |
| `textus`   | `[]const u8` | `?[]const u8` | String slice                 |
| `numerus`  | `i64`        | `?i64`        | Integer                      |
| `fractus`  | `f64`        | `?f64`        | Float                        |
| `decimus`  | `f128`       | `?f128`       | Wide float                   |
| `magnus`   | `i128`       | `?i128`       | Big integer                  |
| `bivalens` | `bool`       | `?bool`       | Boolean                      |
| `nihil`    | `void`       | -             | Unit type                    |
| `vacuum`   | `void`       | -             | Void return                  |
| `octeti`   | `[]u8`       | `?[]u8`       | Byte slice                   |
| `objectum` | -            | -             | No equivalent, compile error |
| `ignotum`  | -            | -             | No equivalent, compile error |
| `numquam`  | `noreturn`   | -             | Never returns                |

### Collections

| Faber          | Zig                        | Notes                       |
| -------------- | -------------------------- | --------------------------- |
| `T[]`          | `std.ArrayList(T)`         | Requires allocator          |
| `tabula<K,V>`  | `std.StringHashMap(V)`     | String keys; requires alloc |
| `copia<T>`     | `std.AutoHashMap(T, void)` | Set via HashMap; req alloc  |
| `promissum<T>` | `!T`                       | Error union                 |
| `curator`      | `std.mem.Allocator`        | Allocator interface         |

## Ownership: Prepositions

| Preposition | Meaning             | Zig Output              |
| ----------- | ------------------- | ----------------------- |
| (none)      | Owned               | Base type               |
| `de`        | Borrowed, read-only | `[]const T`, `*const T` |
| `in`        | Mutable borrow      | `*T`, `*std.ArrayList`  |

### Examples

```fab
functio process(numerus x) -> numerus { redde x * 2 }
functio length(de textus source) -> numerus { redde source.longitudo }
functio append(in numerus[] items, numerus value) { items.adde(value) }
```

```zig
fn process(x: i64) i64 { return x * 2; }
fn length(source: []const u8) i64 { return @intCast(source.len); }
fn append(items: *std.ArrayList(i64), value: i64) void {
    items.append(alloc, value) catch @panic("OOM");
}
```

## Memory Management

### Implicit Main Scope

Top-level `main()` wraps runtime code in an arena:

```zig
pub fn main() void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const alloc = arena.allocator();
    // user code
}
```

### Explicit Scopes

`cura arena fit X` creates nested allocator scope:

```fab
cura arena fit temp {
    varia scratch: numerus[] = []
    scratch.adde(42)
}
```

```zig
{
    var temp_arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer temp_arena.deinit();
    const temp = temp_arena.allocator();

    var scratch = std.ArrayList(i64).init(temp);
    scratch.append(temp, 42) catch @panic("OOM");
}
```

### The `curator` Type

Functions needing an allocator declare a `curator` parameter:

```fab
functio buildList(curator memoria, textus prefix) -> textus[] {
    varia items: textus[] = []
    items.adde(prefix)
    redde items
}
```

```zig
fn buildList(memoria: std.mem.Allocator, prefix: []const u8) [][]const u8 {
    var items = std.ArrayList([]const u8).init(memoria);
    items.append(memoria, prefix) catch @panic("OOM");
    return items.toOwnedSlice();
}
```

### Codegen State

The `ZigGenerator` class tracks the active allocator name via a stack:

```ts
class ZigGenerator {
    depth = 0;
    curatorStack: string[] = ['alloc']; // default allocator

    getCurator(): string {
        return this.curatorStack[this.curatorStack.length - 1] ?? 'alloc';
    }

    pushCurator(name: string): void {
        this.curatorStack.push(name);
    }

    popCurator(): void {
        if (this.curatorStack.length > 1) {
            this.curatorStack.pop();
        }
    }
}
```

Push on:

- `cura arena/page fit X` block entry
- Function entry when a `curator` param exists

Method registry uses curator:

```ts
type ZigGenerator = (obj: string, args: string, curator?: string) => string;

adde: {
    zig: (obj, args, curator) => `${obj}.append(${curator}, ${args}) catch @panic("OOM")`,
},
```

## Collection Methods

### T[]

| Faber       | Zig Output                       | Status |
| ----------- | -------------------------------- | ------ |
| `adde`      | `list.append(alloc, x)`          | Done   |
| `remove`    | `list.pop()`                     | Done   |
| `praepone`  | `list.insert(alloc, 0, x)`       | Done   |
| `decapita`  | `list.orderedRemove(0)`          | Done   |
| `purga`     | `list.clearRetainingCapacity()`  | Done   |
| `primus`    | `list.items[0]`                  | Done   |
| `ultimus`   | `list.items[list.items.len - 1]` | Done   |
| `longitudo` | `list.items.len`                 | Done   |
| `vacua`     | `list.items.len == 0`            | Done   |
| `filtrata`  | `@compileError`                  | Stub   |
| `mappata`   | `@compileError`                  | Stub   |
| `reducta`   | `@compileError`                  | Stub   |

Functional methods emit `@compileError` - use `ex...pro` loops instead.

### tabula<K,V>

| Faber       | Zig Output                     | Status |
| ----------- | ------------------------------ | ------ |
| `pone`      | `map.put(alloc, k, v)`         | Done   |
| `accipe`    | `map.get(k)`                   | Done   |
| `habet`     | `map.contains(k)`              | Done   |
| `dele`      | `map.remove(k)`                | Done   |
| `longitudo` | `map.count()`                  | Done   |
| `purga`     | `map.clearRetainingCapacity()` | Done   |

### copia<T>

Uses `std.AutoHashMap(T, void)` as a set.

| Faber       | Zig Output              | Status |
| ----------- | ----------------------- | ------ |
| `adde`      | `set.put(alloc, x, {})` | Done   |
| `habet`     | `set.contains(x)`       | Done   |
| `dele`      | `set.remove(x)`         | Done   |
| `longitudo` | `set.count()`           | Done   |

## Error Handling

| Faber  | Meaning           | Zig Output       |
| ------ | ----------------- | ---------------- |
| `iace` | Recoverable error | `return error.X` |
| `mori` | Fatal panic       | `@panic("msg")`  |

Functions containing `iace` get error union return types (`!T`). Error messages convert to PascalCase:

```fab
functio fetch(textus url) -> textus {
    si timeout { iace "connection timeout" }
    redde data
}
```

```zig
fn fetch(url: []const u8) ![]const u8 {
    if (timeout) { return error.ConnectionTimeout; }
    return data;
}
```

## Lambdas

Lambdas compile to anonymous struct functions:

```fab
pro x redde x * 2
```

```zig
struct { fn call(x: i64) i64 { return x * 2; } }.call
```

## Comptime

### Type Parameters

`prae typus T` becomes `comptime T: type`:

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

### Compile-time Blocks

`praefixum { }` becomes `comptime blk: { }`:

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
    // ...
    break :blk result;
};
```

## Tagged Unions

`discretio` maps to `union(enum)`:

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

Pattern matching:

```fab
discerne event {
    si Click pro x, y { scribe x, y }
    si Quit { mori "goodbye" }
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

## What Works Well

1. **`varia`/`fixum`** maps directly to `var`/`const`
2. **`ego`** as explicit expression matches Zig's explicit `self`
3. **`novum X { overrides }`** uses `@hasField` pattern naturally
4. **No inheritance** - Faber's `genus` and Zig's structs align
5. **Explicit types** - Faber encourages what Zig requires

## Limitations

1. **`pactum`** - Zig has no interfaces; emit as doc comment
2. **Generators** - Zig has no generators; not implemented
3. **Async** - Zig's async differs from Promise model; uses error unions
4. **String concat** - Runtime `+` needs allocator; only comptime works
5. **Nullable unwrap** - Requires explicit `.?`, `orelse`, `if (x) |val|`

## Current Blockers

| Blocker                    | Files | Fix                                                | Priority |
| -------------------------- | ----- | -------------------------------------------------- | -------- |
| Unused function parameters | ~8    | Prefix with `_` or add `_ = param;`                | Medium   |
| Lambda return type         | ~2    | Emit `@compileError` or infer from context         | Low      |
| Generic `discretio`        | ~1    | Needs comptime function wrapper; non-generic works | Deferred |

### Unused Parameters

**Problem:** Zig errors on unused function parameters.

```zig
fn foo(x: i64, y: i64) i64 { return x; }  // error: unused parameter 'y'
```

**Fix:** Either prefix unused params with `_` in codegen, or add `_ = param;` statements. Requires tracking which params are actually used in the body.

## Future Work

**High priority:**

- Return type prepositions (`-> de textus`)
- Generic resource `cura X fit Y` with `defer Y.solve()`

**Medium priority:**

- Lambda type inference
- `fac`/`cape` error handling
- Slice literals from array literals

**Lower priority:**

- Build.zig generation
- Functional collection methods via loops

## Recently Completed

- Division/modulo operators → `@divTrunc`, `@mod` (Zig requires explicit signed ops)
- Range iteration → native `for (0..n) |i|` syntax (Zig 0.11+)
- Stepped ranges → block-scoped `while` to avoid redeclaration
- `curator` type → `std.mem.Allocator`
- `cura arena/page fit X` → ArenaAllocator/page_allocator
- `curatorStack` for tracking active allocator in nested scopes
