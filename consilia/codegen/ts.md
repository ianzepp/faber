# TypeScript Target Notes

TypeScript is the primary compilation target for Faber. The language design was heavily influenced by TypeScript/JavaScript semantics, making this the most natural and complete target.

## Why TypeScript is the Primary Target

1. **Web-first philosophy** - Faber aims to be accessible; browsers are ubiquitous
2. **Semantic alignment** - Faber's runtime model assumes GC, dynamic dispatch, exceptions
3. **Rapid iteration** - No compilation step, immediate feedback
4. **Rich ecosystem** - npm packages, tooling, IDE support

## Direct Mappings

| Faber | TypeScript | Notes |
|-------|------------|-------|
| `varia` | `let` | Mutable binding |
| `fixum` | `const` | Immutable binding |
| `functio` | `function` | Function declaration |
| `futura` | `async` | Async function |
| `cede` | `await`/`yield` | Context-dependent |
| `genus` | `class` | With auto-merge constructor |
| `pactum` | `interface` | Direct mapping |
| `implet` | `implements` | Direct mapping |
| `ego` | `this` | Self reference |
| `novum` | `new` | Object construction |
| `novum X cum {}` | `new X({})` | Constructor with overrides |
| `si`/`aliter` | `if`/`else` | Conditionals |
| `dum` | `while` | While loop |
| `ex...pro` | `for...of` | Iteration |
| `elige` | `if/else` chain | See `principles.md` — always if/else, not switch |
| `tempta`/`cape`/`demum` | `try`/`catch`/`finally` | Exception handling |
| `iace` | `throw` | Throw exception |
| `redde` | `return` | Return statement |
| `textus` | `string` | String type |
| `numerus` | `number` | Integer (JS has single number type) |
| `fractus` | `number` | Floating point (JS has single number type) |
| `decimus` | `Decimal` | Arbitrary precision (requires library) |
| `bivalens` | `boolean` | Boolean type |
| `nihil` | `null` | Null value |
| `lista<T>` | `T[]` | Array type |
| `tabula<K,V>` | `Map<K,V>` | Map type |
| `copia<T>` | `Set<T>` | Set type |

## Implementation Status

**Complete:**
- All control flow constructs
- All expression types
- Full OOP support (genus, pactum, methods, constructors)
- Async/generators
- Generic types
- Collection methods (lista, tabula, copia)
- Import/export

**Not applicable:**
- Memory management (GC handles it)
- Ownership/borrowing (not a concept in TS)

## Design Decisions

### Auto-merge Constructor

Faber's `genus` generates a constructor that merges field defaults with `cum` overrides:

```typescript
class persona {
    nomen: string = "anonymous";
    aetas: number = 0;

    constructor(overrides: { nomen?: string, aetas?: number } = {}) {
        if (overrides.nomen !== undefined) this.nomen = overrides.nomen;
        if (overrides.aetas !== undefined) this.aetas = overrides.aetas;
        this.creo(); // if defined
    }
}
```

This allows: `novum persona cum { nomen: "Marcus" }` → `new persona({ nomen: "Marcus" })`

### `cede` Context Sensitivity

`cede` maps to `yield` inside generator functions, `await` inside async functions. The codegen tracks context via `inGenerator` flag.

## Future Considerations

- **ESM vs CommonJS** - Currently generates ESM-style imports
- **Source maps** - For debugging Latin source from TS output
- **Type narrowing** - Better handling of nullable types
