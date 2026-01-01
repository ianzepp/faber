---
status: implemented
targets: [ts, py]
updated: 2024-12
---

# Import System

## Overview

Faber's import system handles two categories differently:

| Category                 | Example                           | Behavior                                        |
| ------------------------ | --------------------------------- | ----------------------------------------------- |
| **External packages**    | `ex "@hono/hono" importa Hono`    | Pass-through: emits native import               |
| **Faber stdlib (norma)** | `ex "norma/tempus" importa dormi` | Compiler-handled: intrinsics, no import emitted |

## External Package Imports

External packages are passed through to the target language unchanged:

```fab
ex "@hono/hono" importa Hono, Context
ex "pg" importa Pool
```

Compiles to (TypeScript):

```typescript
import { Hono, Context } from '@hono/hono';
import { Pool } from 'pg';
```

The compiler does not validate external imports. Type checking happens in the target language.

## Faber Stdlib (norma)

The `norma` stdlib is compiler-handled. No runtime files are shipped.

```fab
ex "norma/tempus" importa dormi, SECUNDUM
cede dormi(5 * SECUNDUM)
```

Compiles to (TypeScript):

```typescript
// No import emitted
await new Promise(r => setTimeout(r, 5 * 1000));
```

### How It Works

1. **Semantic analyzer** recognizes `norma/*` paths, validates exports, adds symbols to scope with types
2. **Codegen** suppresses import statements for `norma/*` paths
3. **Codegen** translates function calls to intrinsics (inline code)

### Intrinsic Naming Convention

Internal intrinsic functions use double-underscore prefix: `__`

```
__tempus_nunc      → Date.now()
__tempus_dormi     → new Promise(r => setTimeout(r, ms))
```

The `__` prefix signals "compiler internal" (vs `_` which conventionally means "unused").

For namespacing, intrinsics follow: `__<module>_<function>`

## Architecture

The norma stdlib is distributed across the compiler:

```
fons/semantic/index.ts       # Type signatures (NORMA_*_EXPORTS)
fons/codegen/ts/index.ts     # Intrinsic implementations (TS_INTRINSICS)
fons/codegen/ts/norma/*.ts   # Method mappings (lista, tabula, copia)
arca/norma/*.fab             # Documentation/specification
```

### Semantic Layer

`fons/semantic/index.ts` defines exports for each norma module:

```typescript
const NORMA_TEMPUS_EXPORTS = {
    nunc: { type: functionType([], NUMERUS), kind: 'function' },
    dormi: { type: functionType([NUMERUS], genericType('promissum', [VACUUM])), kind: 'function' },
    SECUNDUM: { type: NUMERUS, kind: 'variable' },
    // ...
};

const NORMA_SUBMODULES = {
    'norma/tempus': NORMA_TEMPUS_EXPORTS,
};
```

### Codegen Layer

`fons/codegen/<target>/index.ts` defines intrinsic implementations:

```typescript
const TS_INTRINSICS = {
    __tempus_nunc: () => `Date.now()`,
    __tempus_dormi: args => `new Promise(r => setTimeout(r, ${args}))`,
    // ...
};
```

### Documentation Layer

`arca/norma/*.fab` documents the stdlib API in Faber syntax:

```fab
# arca/norma/tempus.fab

# Get current time in milliseconds since epoch
functio nunc() -> numerus

# Sleep for specified milliseconds
futura functio dormi(numerus ms) -> vacuum

# Duration constants
fixum SECUNDUM = 1000
fixum MINUTUM = 60000
```

These files are documentation only - not compiled or shipped.

## Module Resolution

| Import path        | Resolution                         |
| ------------------ | ---------------------------------- |
| `"norma"`          | Base stdlib (I/O, math intrinsics) |
| `"norma/tempus"`   | Time module                        |
| `"norma/crypto"`   | Crypto module (future)             |
| `"@scope/package"` | External npm package               |
| `"package"`        | External package                   |
| `"./local"`        | Relative import (pass-through)     |

## Adding New Stdlib Modules

To add a new norma module (e.g., `norma/crypto`):

1. **Semantic types** - Add `NORMA_CRYPTO_EXPORTS` to `fons/semantic/index.ts`
2. **Register submodule** - Add to `NORMA_SUBMODULES` map
3. **Intrinsics** - Add implementations to each target's codegen
4. **Documentation** - Create `arca/norma/crypto.fab`
5. **Tests** - Add semantic and codegen tests

## Design Decisions

### Why No Runtime Files?

1. **Zero dependencies** - Compiled output has no Faber-specific imports
2. **Simpler deployment** - No npm/pip package to install
3. **Target flexibility** - Each target implements intrinsics natively
4. **Performance** - Inline code, no function call overhead

### Why Distinguish norma From External?

1. **Validation** - Compiler validates norma imports at compile time
2. **Optimization** - Intrinsics can be inlined
3. **Portability** - Same Faber code works across all targets
4. **Clarity** - User knows norma is "batteries included"

### When to Use Intrinsics vs Preamble

| Complexity    | Approach                                     |
| ------------- | -------------------------------------------- |
| One-liner     | Intrinsic (inline substitution)              |
| Few lines     | Intrinsic with IIFE                          |
| Complex logic | Preamble (emit once at top of file)          |
| External deps | Not suitable for norma (use external import) |
