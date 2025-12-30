# Stdlib Refactor

Restructure standard library code generation to use external runtime libraries and a unified method registry.

## Problem

Current stdlib implementation has several issues:

1. **Zig allocator threading**: Lista(T) stores allocator at construction, but `cura...fit` pushes new allocators onto curatorStack that Lista doesn't see
2. **Scattered registries**: Each target has separate norma/\*.ts files (N methods x M targets = many files)
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

Phase 1 includes only Zig entries. Other targets will be added in Phase 3.

```typescript
interface MethodDef {
    mutates: boolean;
    needsAlloc: boolean; // Determines if codegen passes curator

    // Per-target: string (delegate to stdlib) | function (inline) | null (unsupported)
    // Other targets added in Phase 3
    zig?: string | ((obj: string, args: string[], alloc: string) => string);
}

export const LISTA: Record<string, MethodDef> = {
    adde: {
        mutates: true,
        needsAlloc: true, // Growing operation
        zig: 'adde', // String = delegate to stdlib
    },
    addita: {
        mutates: false,
        needsAlloc: true, // Returns new Lista
        zig: 'addita', // Delegate to stdlib
    },
    primus: {
        mutates: false,
        needsAlloc: false, // Read-only
        zig: 'primus',
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

| Category               | Needs Alloc | Examples                                   |
| ---------------------- | ----------- | ------------------------------------------ |
| Construction           | Yes         | `init`, `fromItems`, `clone`               |
| Destruction            | Yes         | `deinit`                                   |
| Growing                | Yes         | `adde`, `praepone`                         |
| Shrinking              | No          | `remove`, `decapita`, `purga`              |
| Reading                | No          | `primus`, `longitudo`, `continet`          |
| Returns new collection | Yes         | `addita`, `filtrata`, `mappata`, `inversa` |
| In-place (no resize)   | No          | `ordina`, `inverte`                        |
| Aggregation            | No          | `summa`, `reducta`, `minimus`              |

## Philosophy

**Prefer stdlib when inline code is moderately complex.**

| Target     | Approach      | Rationale                                    |
| ---------- | ------------- | -------------------------------------------- |
| TypeScript | Mostly inline | Native features are clean                    |
| Python     | Mostly inline | Native features are clean                    |
| Zig        | Mostly stdlib | Allocators, error handling make inline messy |
| Rust       | Mixed         | Ownership makes some transforms verbose      |
| C++        | Mixed         | Lambdas/RAII patterns get verbose            |

## Implementation Plan

### Phase 1: Zig Lista Refactor

1. Create `subsidia/zig/lista.zig` with explicit allocator API
2. Create unified `fons/codegen/lista.ts` registry (Zig only initially)
3. Update `fons/codegen/zig/preamble/index.ts` to read from `subsidia/zig/lista.zig`
4. Update `fons/codegen/zig/expressions/call.ts` to use unified registry, pass curator when `needsAlloc: true`
5. Delete old `fons/codegen/zig/norma/lista.ts` and update imports
6. Fix failing tests in `proba/norma/lista.yaml`

**Deferred from Phase 1:**

- Default allocator auto-generation in main() — until core refactor works
- README.md status updates — after tests pass

### Phase 2: Zig Tabula/Copia

1. Create `subsidia/zig/tabula.zig`
2. Create `subsidia/zig/copia.zig`
3. Add Zig entries to unified `fons/codegen/tabula.ts` and `copia.ts`

### Phase 3: Other Targets (per-language)

Wire up each language one at a time to unified registries:

1. Add target entries to unified registry files
2. Update target's `expressions/call.ts` to use unified registry
3. Delete target's `norma/*.ts` files

### Phase 4: Cleanup

1. Remove any remaining per-target `norma/*.ts` files
2. Final validation across all targets

## File Changes Summary

### Phase 1 File Changes

**New Files:**

```
subsidia/zig/lista.zig              # Lista(T) with explicit allocator API
fons/codegen/lista.ts               # Unified registry (Zig only initially)
```

**Modified Files:**

```
fons/codegen/zig/preamble/index.ts  # Read from subsidia/zig/lista.zig
fons/codegen/zig/expressions/call.ts # Use unified registry
proba/norma/lista.yaml              # Update Zig expectations
```

**Deleted Files:**

```
fons/codegen/zig/norma/lista.ts     # Replaced by unified registry
fons/codegen/zig/preamble/lista.txt # Replaced by subsidia/zig/lista.zig
```

### Future Phase File Changes

**Phase 2 (Zig Tabula/Copia):**

```
subsidia/zig/tabula.zig
subsidia/zig/copia.zig
fons/codegen/tabula.ts
fons/codegen/copia.ts
```

**Phase 3 (Other Targets):**

- Add entries to unified registries per target
- Delete `fons/codegen/*/norma/lista.ts` as each target migrates

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

### Phase 1 (completed 2025-12-30)

- [x] Create `subsidia/zig/` directory
- [x] Write `subsidia/zig/lista.zig` (no stored allocator, methods accept allocator)
- [x] Create unified `fons/codegen/lista.ts` (Zig entries only)
- [x] Update `fons/codegen/zig/preamble/index.ts` to read from `subsidia/zig/lista.zig`
- [x] Update `fons/codegen/zig/expressions/call.ts` to use unified registry
- [x] Delete `fons/codegen/zig/norma/lista.ts` and update imports
- [x] Fix failing tests in `proba/norma/lista.yaml` and `proba/curator.yaml`
- [x] Run `bun test -t "@zig"` to verify (469 tests pass)

### Phase 2a: Zig Tabula (completed 2025-12-30)

- [x] Create `subsidia/zig/tabula.zig` (StringHashMap/AutoHashMap wrappers)
- [x] Create unified `fons/codegen/tabula.ts` (Zig entries only)
- [x] Update `fons/codegen/zig/preamble/index.ts` to include tabula
- [x] Update `fons/codegen/zig/expressions/call.ts` to use unified registry
- [x] Delete `fons/codegen/zig/norma/tabula.ts`
- [x] Add Zig expectations to `proba/norma/tabula.yaml`
- [x] Run tests (479 Zig tests pass, 3557 total)

### Phase 2b: Zig Copia (completed 2025-12-30)

- [x] Create `subsidia/zig/copia.zig` (HashSet via HashMap(T, void) wrapper)
- [x] Create unified `fons/codegen/copia.ts` (Zig entries only)
- [x] Update `fons/codegen/zig/preamble/index.ts` to include copia
- [x] Update `fons/codegen/zig/expressions/call.ts` to use unified registry
- [x] Delete `fons/codegen/zig/norma/copia.ts`
- [x] Add Zig expectations to `proba/norma/copia.yaml`
- [x] Run tests (3563 total pass)

### Phase 3: Other Targets (completed 2025-12-30)

- [x] Add ts, py, rs, cpp entries to `fons/codegen/lista.ts`
- [x] Add ts, py, rs, cpp entries to `fons/codegen/tabula.ts`
- [x] Add ts, py, rs, cpp entries to `fons/codegen/copia.ts`
- [x] Update each target's `expressions/call.ts` to import from unified registries
- [x] Add standard headers to C++ preamble (algorithm, numeric, ranges, vector)
- [x] Delete old per-target `norma/{lista,tabula,copia}.ts` files (12 files)
- [x] Run tests (3563 pass)

### Deferred

- [x] Fix default allocator bug (auto-generate arena in main) — 2025-12-30: Always emit arena allocator in main() unconditionally. Simpler than detecting collection usage, and `_ = alloc` silences unused warning. Also set features flags in call.ts when collection methods are called (still needed for preamble Lista/Tabula/Copia inclusion).
- [ ] Update README.md status table
- [ ] Phase 4: Cleanup remaining norma files (aleator, mathesis, tempus)

### Future: Package-based Import (Phase 5)

Currently, subsidia/\*.zig files are injected into the preamble of every generated file. This works but bloats output. The goal is to allow importing from a package instead:

```zig
// Current (preamble injection - 300+ lines per file)
const std = @import("std");
pub fn Lista(comptime T: type) type { ... }  // Entire definition inline

// Goal (package import - 2 lines)
const std = @import("std");
const faber = @import("faber");
```

**Package files already created:**

```
subsidia/zig/
  build.zig    # Exposes "faber" module for build.zig projects
  mod.zig      # Re-exports Lista, Tabula, Copia
  lista.zig
  tabula.zig
  copia.zig
```

**Two usage modes:**

1. **Single-file compilation** (test runner, quick scripts):
   Use Zig's `-M` module flag:

    ```bash
    zig build-exe output.zig --mod faber::/path/to/subsidia/zig/mod.zig -femit-bin=output
    ```

2. **Build.zig projects** (real applications):
   Add faber as a dependency in `build.zig.zon`:
    ```zig
    .dependencies = .{
        .faber = .{
            .url = "https://github.com/ianzepp/faber-romanus/archive/v1.0.0.tar.gz",
            .hash = "...",
        },
    },
    ```
    Then in code: `const faber = @import("faber");`

**Implementation steps:**

- [ ] Update `scripta/exempla.ts` to use `--mod` flag instead of preamble injection:
    ```typescript
    const SUBSIDIA = join(ROOT, 'subsidia', 'zig', 'mod.zig');
    await $`zig build-exe ${file} --mod faber::${SUBSIDIA} -femit-bin=${output}`.quiet();
    ```
- [ ] Update preamble generator to emit `@import("faber")` instead of inline definitions
- [ ] Add CLI flag `--emit-single` to preserve current inline behavior if needed
- [ ] Test with Zig 0.11+ (module flag syntax may vary)
- [ ] Document package installation for end users

**Note:** Zig packages are decentralized (URL-based, no registry account needed). The faber-romanus repo itself can serve as the package source.

## Related Documents

- `consilia/futura/preamble-rework.md` - Related preamble restructuring
- `consilia/futura/zig-norma.md` - Previous Zig stdlib notes
