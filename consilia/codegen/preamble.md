---
status: partial
targets: [ts]
note: Infrastructure done; `panic` tracking for TS done; other features pending
updated: 2024-12
---

# Preamble / Prologue System

Each codegen target needs to emit setup code (imports, includes, class definitions) based on which language features are actually used in the source.

## Implementation Status

| Feature              | TypeScript | Zig | Python | Notes                                |
| -------------------- | :--------: | :-: | :----: | ------------------------------------ |
| Infrastructure       |    Done    |  -  |   -    | `RequiredFeatures` in types.ts       |
| `panic` tracking     |    Done    |  -  |   -    | Emits `Panic` class when `mori` used |
| `lista/tabula/copia` |     -      |  -  |   -    | TS doesn't need imports              |
| `async`              |     -      |  -  |   -    | TS doesn't need imports              |
| `events`             |     -      |  -  |   -    | May need EventEmitter polyfill       |

## Problem

Different targets require different setup:

| Feature Used   | TypeScript       | Python                    | C++23                        | Rust                            | Zig                          |
| -------------- | ---------------- | ------------------------- | ---------------------------- | ------------------------------- | ---------------------------- |
| `mori` (panic) | `class Panic...` | `import sys`              | `#include <stdexcept>`       | — (built-in)                    | — (built-in)                 |
| `T[]`     | —                | `from typing import List` | `#include <vector>`          | `use std::vec::Vec`             | `const std = @import("std")` |
| `tabula<K,V>`  | —                | `from typing import Dict` | `#include <unordered_map>`   | `use std::collections::HashMap` | —                            |
| `promissum<T>` | —                | `import asyncio`          | `#include <future>`          | —                               | —                            |
| `scribe`       | —                | —                         | `#include <print>`           | —                               | —                            |
| Arena alloc    | —                | —                         | `#include <memory_resource>` | `use bumpalo::Bump`             | —                            |

## Design

### Feature Tracking

During codegen, track which features are used:

```typescript
interface RequiredFeatures {
    // Error handling
    panic: boolean; // mori used

    // Collections
    lista: boolean; // T[] or array methods
    tabula: boolean; // tabula<K,V>
    copia: boolean; // copia<T>

    // Async
    async: boolean; // futura, cede, promissum
    asyncIterator: boolean; // fiet, async for

    // Generators
    generator: boolean; // cursor, fiunt

    // I/O (future stdlib)
    fileIO: boolean; // lege, inscribe
    networkIO: boolean; // pete, mitte

    // Other
    arena: boolean; // arena allocator (systems targets)
}
```

### Preamble Generation

Each target implements its own preamble generator:

```typescript
interface PreambleGenerator {
    generate(features: RequiredFeatures): string;
}
```

### Output Structure

Generated code follows this structure:

```
[preamble]      <- imports, includes, class definitions
[blank line]
[program body]  <- actual compiled statements
```

## Target-Specific Details

### TypeScript

Inline definitions (no external dependencies):

```typescript
// When panic: true
class Panic extends Error {
    name = 'Panic';
}

// When asyncIterator: true (for ausculta)
// May need EventEmitter polyfill for browsers
```

TypeScript needs minimal preamble since most features are built-in.

### Python

Import statements:

```python
# When panic: true
import sys

# When async: true
import asyncio

# When lista/tabula used with type hints
from typing import List, Dict, Set, Optional
```

### C++23

Include directives:

```cpp
// Always needed
#include <string>
#include <print>

// When lista: true
#include <vector>

// When tabula: true
#include <unordered_map>

// When copia: true
#include <unordered_set>

// When panic: true (for std::terminate or custom)
#include <exception>

// When async: true
#include <future>
#include <coroutine>

// When arena: true
#include <memory_resource>
```

### Rust

Use statements:

```rust
// When arena: true
use bumpalo::Bump;

// When lista methods used extensively
use std::vec::Vec;

// When tabula: true
use std::collections::HashMap;

// When async: true (runtime-dependent)
use tokio; // or async-std
```

### Zig

Import statements:

```zig
// Almost always needed
const std = @import("std");

// When arena: true
const Allocator = std.mem.Allocator;
```

## Implementation Strategy

### Phase 1: Infrastructure

1. Define `RequiredFeatures` interface in `fons/codegen/types.ts`
2. Add feature tracking to base codegen
3. Add preamble generation hook

### Phase 2: TypeScript

1. Track `panic` usage
2. Emit `Panic` class when needed
3. Update tests

### Phase 3: Other Targets

Roll out to Python, then systems targets as they mature.

## Open Questions

1. **Deduplication** — If multiple files are compiled, should preamble be in each or in a shared module?

2. **Runtime library** — Should frequently-used definitions live in `@faber/runtime` (TS) or `faber::prelude` (Rust) instead of inline?

3. **Tree shaking** — For bundlers, inline definitions may be better than imports for dead code elimination.

4. **Version compatibility** — Python type hints vary by version (3.9 vs 3.10+). How to handle?

## Examples

### Input (Faber)

```
si x < 0 { mori "negative value" }

ex items pro item {
    scribe item
}
```

### Output (TypeScript)

```typescript
class Panic extends Error {
    name = 'Panic';
}

if (x < 0) {
    throw new Panic('negative value');
}
for (const item of items) {
    console.log(item);
}
```

### Output (C++23)

```cpp
#include <string>
#include <print>
#include <stdexcept>
#include <vector>

if (x < 0) {
  throw std::runtime_error("negative value");
}
for (const auto& item : items) {
  std::print("{}\n", item);
}
```

### Output (Python)

```python
import sys

if x < 0:
    raise SystemExit("negative value")

for item in items:
    print(item)
```
