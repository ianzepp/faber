# Stdlib Refactor

Restructure standard library code generation to use external runtime libraries and a unified method registry.

## Problem

Current stdlib implementation has several issues:

1. **Zig allocator threading**: Lista(T) stores allocator at construction, but `cura...fit` pushes new allocators onto curatorStack that Lista doesn't see
2. **Scattered registries**: Each target has separate norma/*.ts files (N methods x M targets = many files)
3. **Inconsistent complexity**: Some targets inline complex multi-statement transforms, others delegate to preamble
4. **Location confusion**: Runtime library code lives in `fons/codegen/*/preamble/` but isn't codegen logic

## Current State

### Zig Lista(T) Allocator Issue

```zig
// Current: stores allocator at construction
pub fn Lista(comptime T: type) type {
    return struct {
        items: std.ArrayList(T),
        alloc: std.mem.Allocator,  // Problem: fixed at construction

        pub fn addita(self: Self, value: T) Self {
            var result = self.clone();  // Uses self.alloc, ignores curatorStack
            // ...
        }
    };
}
```

When Faber code does:
```fab
cura arena fit outer {
    fixum lista<numerus> items = [1, 2, 3]
    cura arena fit inner {
        // inner allocator on curatorStack, but items.addita() uses outer
        fixum newItems = items.addita(4)
    }
}
```

The `addita` call should use `inner` allocator but uses `outer` instead.

### Registry Fragmentation

```
fons/codegen/
  ts/norma/lista.ts      # TS lista methods
  ts/norma/tabula.ts
  py/norma/lista.ts      # Python lista methods (different file)
  py/norma/tabula.ts
  zig/norma/lista.ts     # Zig lista methods (another file)
  zig/norma/tabula.ts
  ...
```

Each file has slightly different structure. Adding a method requires editing 5+ files.

## Proposed Design

### 1. Runtime Library Location

Move runtime libraries to project root under `subsidia/` (Latin: "supports, aids"):

```
subsidia/
  zig/
    lista.zig      # Lista(T) wrapper
    tabula.zig     # Tabula(K,V) wrapper
    copia.zig      # Copia(T) wrapper
  rs/
    lista.rs       # (future)
  cpp/
    lista.hpp      # (future)
```

These are actual library files in the target language, not TypeScript codegen.

### 2. Unified Method Registry

Consolidate per-target registries into single files per stdlib module:

```
fons/codegen/
  lista.ts         # All targets in one file
  tabula.ts
  copia.ts
  mathesis.ts
  aleator.ts
```

### 3. Explicit Allocator Threading (Zig)

Remove stored allocator from Lista. Pass allocator explicitly to methods that need it:

```zig
pub fn Lista(comptime T: type) type {
    return struct {
        items: std.ArrayList(T),
        // No stored allocator

        const Self = @This();

        // Growing methods need allocator
        pub fn adde(self: *Self, alloc: Allocator, value: T) void {
            self.items.append(alloc, value) catch @panic("OOM");
        }

        // Methods returning new Lista need allocator
        pub fn addita(self: Self, alloc: Allocator, value: T) Self {
            var result = Self.init(alloc);
            result.items.appendSlice(alloc, self.items.items) catch @panic("OOM");
            result.adde(alloc, value);
            return result;
        }

        // Read-only methods don't need allocator
        pub fn primus(self: Self) ?T {
            if (self.items.items.len == 0) return null;
            return self.items.items[0];
        }

        // In-place mutation without resize doesn't need allocator
        pub fn ordina(self: *Self) void {
            std.mem.sort(T, self.items.items, {}, std.sort.asc(T));
        }
    };
}
```

Codegen passes `_alloc` (from curatorStack) to methods that need it.

### 4. Registry Schema

```typescript
interface MethodDef {
    mutates: boolean;
    needsAlloc: boolean;  // Determines if codegen passes curator

    // Per-target: string (delegate to stdlib) | function (inline) | null (unsupported)
    ts?: string | ((obj: string, args: string[]) => string);
    py?: string | ((obj: string, args: string[]) => string);
    zig?: string | ((obj: string, args: string[], alloc: string) => string);
    rs?: string | ((obj: string, args: string[]) => string);
    cpp?: string | ((obj: string, args: string[]) => string);
}

export const LISTA: Record<string, MethodDef> = {
    adde: {
        mutates: true,
        needsAlloc: true,  // Growing operation
        ts: 'push',
        py: 'append',
        zig: 'adde',  // String = delegate to stdlib
        rs: 'push',
        cpp: 'push_back',
    },
    addita: {
        mutates: false,
        needsAlloc: true,  // Returns new Lista
        ts: (obj, args) => `[...${obj}, ${args.join(', ')}]`,
        py: (obj, args) => `[*${obj}, ${args.join(', ')}]`,
        zig: 'addita',  // Delegate to stdlib
        rs: (obj, args) => `{ let mut v = ${obj}.clone(); v.push(${args[0]}); v }`,
        cpp: null,  // Use stdlib when ready
    },
    primus: {
        mutates: false,
        needsAlloc: false,  // Read-only
        ts: (obj) => `${obj}[0]`,
        py: (obj) => `${obj}[0]`,
        zig: 'primus',
        rs: (obj) => `${obj}.first().cloned()`,
        cpp: (obj) => `${obj}.front()`,
    },
    // ...
};
```

When `zig` is a string and `needsAlloc` is true, codegen generates:
```zig
obj.methodName(_alloc, args...)
```

When `zig` is a string and `needsAlloc` is false:
```zig
obj.methodName(args...)
```

## Allocator Categories

| Category | Needs Alloc | Examples |
|----------|-------------|----------|
| Construction | Yes | `init`, `fromItems`, `clone` |
| Destruction | Yes | `deinit` |
| Growing | Yes | `adde`, `praepone` |
| Shrinking | No | `remove`, `decapita`, `purga` |
| Reading | No | `primus`, `longitudo`, `continet` |
| Returns new collection | Yes | `addita`, `filtrata`, `mappata`, `inversa` |
| In-place (no resize) | No | `ordina`, `inverte` |
| Aggregation | No | `summa`, `reducta`, `minimus` |

## Philosophy

**Prefer stdlib when inline code is moderately complex.**

| Target | Approach | Rationale |
|--------|----------|-----------|
| TypeScript | Mostly inline | Native features are clean |
| Python | Mostly inline | Native features are clean |
| Zig | Mostly stdlib | Allocators, error handling make inline messy |
| Rust | Mixed | Ownership makes some transforms verbose |
| C++ | Mixed | Lambdas/RAII patterns get verbose |

## Implementation Plan

### Phase 1: Zig Lista Refactor

1. Create `subsidia/zig/lista.zig` with explicit allocator API
2. Create unified `fons/codegen/lista.ts` registry
3. Update Zig codegen to:
   - Include subsidia files in output
   - Pass `_alloc` based on `needsAlloc` flag
4. Update README.md to flip `[-]` to `[x]` for working methods

### Phase 2: Zig Tabula/Copia

1. Create `subsidia/zig/tabula.zig`
2. Create `subsidia/zig/copia.zig`
3. Create unified `fons/codegen/tabula.ts` and `copia.ts`

### Phase 3: Other Targets

Apply same pattern to Rust and C++ where beneficial.

### Phase 4: Migrate Existing Registries

1. Migrate `fons/codegen/*/norma/*.ts` content into unified registries
2. Remove old per-target registry files
3. Update imports throughout codegen

## File Changes Summary

### New Files

```
subsidia/
  zig/
    lista.zig
    tabula.zig
    copia.zig

fons/codegen/
  lista.ts       # Unified registry (replaces */norma/lista.ts)
  tabula.ts
  copia.ts
  mathesis.ts
  aleator.ts
```

### Deleted Files (after migration)

```
fons/codegen/ts/norma/lista.ts
fons/codegen/py/norma/lista.ts
fons/codegen/zig/norma/lista.ts
fons/codegen/rs/norma/lista.ts
fons/codegen/cpp/norma/lista.ts
(and corresponding tabula.ts, copia.ts, etc.)
```

### Modified Files

```
fons/codegen/zig/preamble/index.ts  # Include subsidia files
fons/codegen/*/expressions/call.ts  # Use unified registry
README.md                            # Update implementation status
```

## Validation Findings

Design validated against codebase on 2025-12-30. Key findings:

### Infrastructure Already Exists

The curatorStack and method dispatch plumbing is complete. Only the final step (using the curator in method handlers) is missing.

**`fons/codegen/zig/generator.ts`** (lines 88-120):
```typescript
curatorStack: string[] = ['alloc'];  // Default allocator name

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
```

**`fons/codegen/zig/statements/cura.ts`**:
- `cura arena fit name` correctly pushes/pops curator stack
- Generates proper ArenaAllocator setup with defer

**`fons/codegen/zig/expressions/call.ts`** (lines 119-157):
- Already calls `g.getCurator()` before method dispatch
- Already passes curator to method handlers: `method.zig(obj, argsArray, curator)`
- Handlers receive curator but ignore it

**`fons/codegen/zig/norma/lista.ts`**:
- Handlers have signature `(obj, args, curator)` but don't use curator
- Example: `adde: { zig: (obj, args) => \`${obj}.adde(${args[0]})\` }` — curator unused

### The Only Change Needed

Update lista.ts handlers to use the curator parameter they already receive:

```typescript
// Before
adde: { zig: (obj, args) => `${obj}.adde(${args[0]})` },

// After
adde: { zig: (obj, args, curator) => `${obj}.adde(${curator}, ${args[0]})` },
```

And update `subsidia/zig/lista.zig` to accept allocator per-method instead of storing it.

### Test Files to Update

**`proba/norma/lista.yaml`**:
- Current Zig expectations: `.adde(1)`, `.addita(1)`, etc.
- After refactor: `.adde(alloc, 1)`, `.addita(alloc, 1)`, etc.

**`proba/codegen/statements/cura.yaml`**:
- Has good coverage of `cura arena fit name` blocks
- Tests nested arena blocks (outer/inner)
- No tests yet for lista methods inside cura blocks

## Known Issues / Bugs

### BUG: Zig main() doesn't create a default allocator

The curatorStack defaults to `['alloc']`, meaning codegen emits code like `items.adde(alloc, 1)`. But there's no guarantee `alloc` exists in scope.

**Current behavior**: Code compiles only if:
- Inside a `cura arena fit alloc { }` block, OR
- User manually defines `alloc` variable

**Problem**: Raw code without cura block generates invalid Zig:
```zig
pub fn main() void {
    var items = Lista(i64).init(alloc);  // Error: alloc undefined
    items.adde(alloc, 1);                 // Error: alloc undefined
}
```

**Possible fixes**:
1. **Require cura blocks** — Enforce that collection-using code is inside `cura arena fit`
2. **Auto-generate arena in main()** — When `features.lista` is true, emit arena setup
3. **Use page_allocator fallback** — Default to `std.heap.page_allocator` (leaks memory)

**Note**: `fons/codegen/zig/preamble/index.ts` has `usesCollections()` helper but doesn't use it to emit arena setup. The function exists at line 42-44:
```typescript
export function usesCollections(features: RequiredFeatures): boolean {
    return features.lista || features.tabula || features.copia;
}
```

This could be wired up to auto-generate arena in main().

### Current Preamble Location

**`fons/codegen/zig/preamble/lista.txt`** — 366-line Lista(T) with stored allocator. This is the file to replace with `subsidia/zig/lista.zig`.

**`fons/codegen/zig/preamble/index.ts`** — Reads lista.txt and includes it when `features.lista` is true.

## Implementation Checklist

- [ ] Create `subsidia/` directory structure
- [ ] Write `subsidia/zig/lista.zig` (no stored allocator)
- [ ] Update `fons/codegen/zig/norma/lista.ts` to pass curator
- [ ] Update `fons/codegen/zig/preamble/index.ts` to read from subsidia/
- [ ] Fix default allocator bug (auto-generate arena or require cura blocks)
- [ ] Update `proba/norma/lista.yaml` Zig expectations
- [ ] Add tests for lista methods inside cura blocks
- [ ] Run `bun test -t "@zig"` to verify
- [ ] Update README.md status table (flip `[-]` to `[x]`)

## Related Documents

- `consilia/futura/preamble-rework.md` - Related preamble restructuring
- `consilia/futura/zig-norma.md` - Previous Zig stdlib notes
