---
status: completed
updated: 2026-01-06
note: Core goals achieved. All targets have preamble/ directories and use RequiredFeatures. Preambles organized as constants in index.ts (not individual files per snippet).
implemented:
  - preamble/ directories for all targets (ts, py, rs, cpp, zig)
  - RequiredFeatures standardized across all targets
  - Zig uses subsidia/zig/ external files (not inline snippets)
  - Feature-driven preamble generation
not_as_designed:
  - Preambles are constants in preamble/index.ts, not separate files (panic.ts, flumina.ts)
  - C++ uses RequiredFeatures but also maintains includes Set for headers
---

# Preamble Rework

Restructure preamble generation to use external files and standardize detection across all targets.

## Current State

| Target | Detection Method | Preamble Location | Issues |
|--------|------------------|-------------------|--------|
| **TS** | RequiredFeatures | Inline in `index.ts` | Flumina is ~60 lines inline |
| **Py** | RequiredFeatures | Inline in `index.ts` | Missing Panic class |
| **Rs** | RequiredFeatures | Inline in `index.ts` | Only regex; missing collections, async |
| **C++** | Dynamic `includes` Set | Inline in `index.ts` | Inconsistent with RequiredFeatures |
| **Zig** | JSON.stringify hack | Hardcoded in `index.ts` | No RequiredFeatures, hacky detection |

### Current Preamble Items

| Feature | TS | Py | Rs | C++ | Zig |
|---------|----|----|-----|-----|-----|
| Panic class | `class Panic extends Error` | Missing | N/A (built-in) | N/A | N/A |
| Decimal import | `import type Decimal` | `from decimal import Decimal` | N/A | N/A | N/A |
| Enum import | N/A | `from enum import Enum, auto` | N/A | N/A | N/A |
| Regex import | N/A | `import re` | `use regex::Regex;` | N/A | N/A |
| Math import | N/A | `import math` | Missing | N/A | N/A |
| Random import | N/A | `import random` | Missing | N/A | N/A |
| UUID import | N/A | `import uuid` | Missing | N/A | N/A |
| Secrets import | N/A | `import secrets` | Missing | N/A | N/A |
| Praefixum helper | N/A | `def __praefixum__` | N/A | N/A | N/A |
| Flumina/Responsum | Type + 4 helpers (~60 lines) | Missing | Missing | Missing | Missing |
| ScopeGuard | N/A | N/A | N/A | `_ScopeGuard` template | N/A |
| Arena allocator | N/A | N/A | Missing | N/A | Inline in main() |
| Std import | N/A | N/A | N/A | Always: `<print>`, `<string>`, `<cstdint>` | Always: `const std = @import("std");` |

## Proposed Structure

Extract preambles to target-local directories:

```
fons/codegen/
├── ts/
│   ├── preamble/
│   │   ├── index.ts       # Re-exports, genPreamble() logic
│   │   ├── panic.ts       # export const PANIC_CLASS = `...`
│   │   └── flumina.ts     # export const FLUMINA_PROTOCOL = `...`
│   ├── index.ts           # Imports from ./preamble
│   └── ...
├── py/
│   ├── preamble/
│   │   ├── index.ts
│   │   ├── panic.ts       # (new) PanicException class
│   │   └── praefixum.ts
│   └── ...
├── rs/
│   ├── preamble/
│   │   ├── index.ts
│   │   └── collections.ts # use std::collections::*
│   └── ...
├── cpp/
│   ├── preamble/
│   │   ├── index.ts
│   │   └── scopeguard.ts
│   └── ...
├── zig/
│   ├── preamble/
│   │   ├── index.ts
│   │   └── arena.ts
│   └── ...
```

## Implementation Plan

### Phase 1: Standardize Detection

1. **C++**: Replace `g.includes` Set with RequiredFeatures flags
   - Map current dynamic includes to feature flags
   - Keep `includes` Set for header tracking, but drive it from features

2. **Zig**: Replace JSON.stringify hack with RequiredFeatures
   - Add `g.features` to ZigGenerator
   - Set flags during AST traversal like other targets

### Phase 2: Extract Preambles (TypeScript first)

1. Create `ts/preamble/` directory
2. Extract `PANIC_CLASS` to `panic.ts`
3. Extract `FLUMINA_PROTOCOL` to `flumina.ts`
4. Create `index.ts` with `genPreamble()` that imports and concatenates
5. Update `ts/index.ts` to import from `./preamble`

### Phase 3: Propagate Pattern

Apply same structure to other targets:
- **Py**: Extract praefixum helper, add Panic class
- **Rs**: Add collections, async runtime preambles
- **C++**: Extract ScopeGuard
- **Zig**: Extract arena setup, convert to feature-driven

### Phase 4: Fill Gaps

| Target | Missing Items |
|--------|---------------|
| Py | Panic class |
| Rs | Collections (`use std::collections::*`), async runtime |
| C++ | Nullable types preamble |
| Zig | Proper RequiredFeatures tracking |
| All non-TS | Flumina/Responsum protocol |

## Benefits

1. **Maintainability**: Preamble snippets have syntax highlighting, are testable
2. **Consistency**: All targets use RequiredFeatures for detection
3. **Discoverability**: Easy to see what each target emits
4. **Extensibility**: Adding new preamble items is straightforward

## Related Issues

- `rs/norma/lista.ts:47` has `args: string` instead of `args: string[]` (separate fix)
