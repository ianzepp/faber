---
status: superseded
updated: 2026-01-06
note: Implemented but architecture evolved. Runtime at fons/subsidia/zig/ (not runtime/). Uses per-method allocators and norma-faber annotations instead of simplified TypeScript registries.
see: consilia/completa/norma-faber.md, consilia/archived/stdlib-refactor.md
---

# Zig Native Runtime Library

Replace inline Zig code generation with a proper Zig runtime library that gets copied alongside compiled output.

## Current State (Wasteful)

```
Faber source → lista.ts (maps names → inline Zig snippets) → ugly inline code
```

```typescript
// fons/codegen/zig/norma/lista.ts
summa: {
    zig: obj => `blk: { var sum: i64 = 0; for (${obj}.items) |v| { sum += v; } break :blk sum; }`;
}
```

Problems:

- Zig code written inside TypeScript template strings
- No syntax highlighting, type checking, or testability
- Verbose, fragile inline blocks with `@TypeOf()` hacks
- Same logic duplicated across every method call site

## Proposed Architecture

```
Faber source → emit method call → faber/lista.zig (real Zig)
```

### Directory Structure

```
fons/codegen/zig/
├── runtime/
│   ├── faber.zig          # pub const lista = @import("lista.zig"); etc
│   ├── lista.zig          # Lista(T) wrapper with methods
│   ├── tabula.zig         # Tabula(K,V) wrapper
│   ├── copia.zig          # Copia(T) wrapper
│   ├── aleator.zig        # random functions
│   └── tempus.zig         # time functions
├── norma/
│   ├── lista.ts           # trivial: { summa: { zig: (obj) => `${obj}.summa()` } }
│   └── ...
└── index.ts               # copies runtime to output, generates preamble
```

### Output Structure

```
opus/                          # output directory
├── faber.zig                  # runtime library (copied from fons/codegen/zig/runtime/)
└── main.zig                   # user's compiled code, imports faber.zig
```

## Implementation

### runtime/faber.zig

```zig
pub const lista = @import("lista.zig");
pub const tabula = @import("tabula.zig");
pub const copia = @import("copia.zig");
pub const aleator = @import("aleator.zig");
pub const tempus = @import("tempus.zig");
```

### runtime/lista.zig

```zig
const std = @import("std");

pub fn Lista(comptime T: type) type {
    return struct {
        items: std.ArrayList(T),
        alloc: std.mem.Allocator,

        const Self = @This();

        pub fn init(alloc: std.mem.Allocator) Self {
            return .{ .items = std.ArrayList(T).init(alloc), .alloc = alloc };
        }

        // Light wrapper - just delegates
        pub fn adde(self: *Self, value: T) void {
            self.items.append(self.alloc, value) catch @panic("OOM");
        }

        pub fn longitudo(self: Self) usize {
            return self.items.items.len;
        }

        pub fn vacua(self: Self) bool {
            return self.items.items.len == 0;
        }

        // Has logic
        pub fn summa(self: Self) T {
            var sum: T = 0;
            for (self.items.items) |v| sum += v;
            return sum;
        }

        pub fn minimus(self: Self) ?T {
            return std.mem.min(T, self.items.items);
        }

        pub fn maximus(self: Self) ?T {
            return std.mem.max(T, self.items.items);
        }

        // Predicate methods
        pub fn omnes(self: Self, predicate: fn (T) bool) bool {
            for (self.items.items) |v| {
                if (!predicate(v)) return false;
            }
            return true;
        }

        pub fn aliquis(self: Self, predicate: fn (T) bool) bool {
            for (self.items.items) |v| {
                if (predicate(v)) return true;
            }
            return false;
        }

        // Allocator-aware, returns new list
        pub fn filtrata(self: Self, predicate: fn (T) bool) Self {
            var result = Self.init(self.alloc);
            for (self.items.items) |v| {
                if (predicate(v)) result.adde(v);
            }
            return result;
        }

        pub fn mappata(self: Self, comptime R: type, transform: fn (T) R) Lista(R) {
            var result = Lista(R).init(self.alloc);
            for (self.items.items) |v| {
                result.adde(transform(v));
            }
            return result;
        }
    };
}
```

### norma/lista.ts (Simplified)

```typescript
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // Adding
    adde: { zig: (obj, args) => `${obj}.adde(${args[0]})` },
    praepone: { zig: (obj, args) => `${obj}.praepone(${args[0]})` },

    // Removing
    remove: { zig: obj => `${obj}.remove()` },
    decapita: { zig: obj => `${obj}.decapita()` },
    purga: { zig: obj => `${obj}.purga()` },

    // Accessing
    primus: { zig: obj => `${obj}.primus()` },
    ultimus: { zig: obj => `${obj}.ultimus()` },
    accipe: { zig: (obj, args) => `${obj}.accipe(${args[0]})` },
    longitudo: { zig: obj => `${obj}.longitudo()` },
    vacua: { zig: obj => `${obj}.vacua()` },

    // Searching
    continet: { zig: (obj, args) => `${obj}.continet(${args[0]})` },
    indiceDe: { zig: (obj, args) => `${obj}.indiceDe(${args[0]})` },

    // Functional
    filtrata: { zig: (obj, args) => `${obj}.filtrata(${args[0]})` },
    mappata: { zig: (obj, args) => `${obj}.mappata(${args[0]})` },
    reducta: { zig: (obj, args) => `${obj}.reducta(${args[0]}, ${args[1]})` },

    // Aggregation
    summa: { zig: obj => `${obj}.summa()` },
    minimus: { zig: obj => `${obj}.minimus()` },
    maximus: { zig: obj => `${obj}.maximus()` },

    // Predicates
    omnes: { zig: (obj, args) => `${obj}.omnes(${args[0]})` },
    aliquis: { zig: (obj, args) => `${obj}.aliquis(${args[0]})` },
};
```

### Generated Output

```zig
const std = @import("std");
const faber = @import("faber.zig");

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const alloc = arena.allocator();

    var nums = faber.lista.Lista(i64).init(alloc);
    nums.adde(1);
    nums.adde(2);
    nums.adde(3);

    const total = nums.summa();           // clean!
    const hasPositive = nums.aliquis(isPositive);
    const positives = nums.filtrata(isPositive);
}

fn isPositive(x: i64) bool {
    return x > 0;
}
```

## Benefits

1. **Real Zig** - syntax highlighting, `zig test`, proper type errors
2. **Allocator baked in** - `Lista` stores its allocator, no curator threading
3. **Testable** - unit test the runtime independently with `zig test runtime/lista.zig`
4. **Readable output** - generated code looks like idiomatic Zig
5. **Maintainable** - Zig experts can contribute without knowing TypeScript

## Migration Path

1. Create `runtime/` directory with `faber.zig` and `lista.zig`
2. Update `index.ts` to copy runtime files to output directory
3. Update preamble to emit `const faber = @import("faber.zig");`
4. Simplify `norma/lista.ts` to trivial method call mappings
5. Update type codegen: `lista<numerus>` → `faber.lista.Lista(i64)`
6. Repeat for tabula, copia, aleator, tempus

## Open Questions

1. **Lambda handling** - Zig doesn't have closures. Methods like `filtrata` take function pointers. How do we handle Faber lambdas that capture variables?

2. **Type inference** - When user writes `varia items = []`, we infer type later. Need to track the element type to emit `Lista(T)`.

3. **Interop with raw std.ArrayList** - Should `Lista(T)` expose `.items` for escape hatch, or fully encapsulate?

4. **Error handling** - Currently using `catch @panic("OOM")`. Should we propagate errors instead?
