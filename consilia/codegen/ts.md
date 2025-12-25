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

## Unsupported TypeScript Features

Features that would need new Faber syntax to support.

### High Priority (commonly used)

| TS Feature | Syntax | Status | Latin Candidate | Notes |
|------------|--------|--------|-----------------|-------|
| Spread (array) | `...arr` | Not done | `sparge`? | Array/object spreading |
| Spread (object) | `...obj` | Not done | `sparge`? | Object spreading |
| Rest parameters | `...args` | Not done | `ceteri`? | Collect remaining args |
| Optional chaining | `?.` | Not done | TBD | Safe property access |
| Nullish coalescing | `??` | Partial | `vel` | Only in destructuring |
| Array destructuring | `[a, b] = arr` | Not done | — | Explicitly excluded |
| typeof (runtime) | `typeof x` | Not done | `typus`? | Runtime type inspection |
| instanceof | `x instanceof T` | Not done | `exemplar`? | Type checking |
| Regex literals | `/pattern/` | Not done | — | No tokenizer support |
| Setters | `set prop(v)` | Not done | — | Computed props read-only |

### Medium Priority (useful)

| TS Feature | Syntax | Status | Latin Candidate | Notes |
|------------|--------|--------|-----------------|-------|
| Type assertions | `x as T` | Not done | `ut`? | Cast/narrow type |
| Computed property names | `{ [key]: v }` | Not done | — | Dynamic keys |
| Shorthand properties | `{ x }` | Not done | — | For `{ x: x }` |
| Tagged templates | `` fn`text` `` | Not done | — | Template processors |
| BigInt | `123n` | Not done | `magnus`? | Arbitrary precision int |
| Decorators | `@foo` | Not done | — | Metaprogramming |
| Static blocks | `static { }` | Not done | `generis { }` | Class-level init |

### Lower Priority (advanced TS)

| TS Feature | Syntax | Status | Notes |
|------------|--------|--------|-------|
| keyof | `keyof T` | Not done | Type-level key extraction |
| typeof (type-level) | `typeof x` | Not done | Type from value |
| Mapped types | `{ [K in T]: V }` | Not done | Type transformation |
| Conditional types | `T extends U ? A : B` | Not done | Type-level conditionals |
| Tuple types | `[A, B, C]` | Not done | Fixed-length arrays |
| Index signatures | `{ [k: string]: T }` | Not done | Dynamic property types |
| satisfies | `x satisfies T` | Not done | Type check without widening |
| infer | `infer R` | Not done | Type inference in conditionals |
| never type | `never` | Not done | Bottom type |
| unknown type | `unknown` | Not done | Safe any |

### Design Notes

**Spread/Rest**: Latin `spargere` (to scatter) → `sparge` for spread, `ceteri` (the rest) for rest params.

```
// Possible syntax
fixum combined = [sparge arr1, sparge arr2]
fixum merged = { sparge obj1, sparge obj2 }
functio f(a, ceteri args) { }
```

**Optional chaining**: Could use `?.` directly (punctuation, not Latin) or a keyword like `forsitan` (perhaps).

```
// Option 1: Direct
user?.address?.city

// Option 2: Latin (verbose)
user forsitan address forsitan city
```

**Nullish coalescing**: `vel` (or) currently used in destructuring. Could extend to general expressions.

```
fixum name = user.name vel "Anonymous"
```

## Future Considerations

- **ESM vs CommonJS** - Currently generates ESM-style imports
- **Source maps** - For debugging Latin source from TS output
- **Type narrowing** - Better handling of nullable types
