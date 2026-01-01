---
status: implemented
targets: [ts]
updated: 2024-12
---

# TypeScript Target Notes

TypeScript is the primary compilation target for Faber. The language design was heavily influenced by TypeScript/JavaScript semantics, making this the most natural and complete target.

## Why TypeScript is the Primary Target

1. **Web-first philosophy** - Faber aims to be accessible; browsers are ubiquitous
2. **Semantic alignment** - Faber's runtime model assumes GC, dynamic dispatch, exceptions
3. **Rapid iteration** - No compilation step, immediate feedback
4. **Rich ecosystem** - npm packages, tooling, IDE support

## Direct Mappings

| Faber                   | TypeScript              | Notes                                            |
| ----------------------- | ----------------------- | ------------------------------------------------ |
| `varia`                 | `let`                   | Mutable binding                                  |
| `fixum`                 | `const`                 | Immutable binding                                |
| `functio`               | `function`              | Function declaration                             |
| `futura`                | `async`                 | Async function                                   |
| `cede`                  | `await`/`yield`         | Context-dependent                                |
| `genus`                 | `class`                 | With auto-merge constructor                      |
| `pactum`                | `interface`             | Direct mapping                                   |
| `implet`                | `implements`            | Direct mapping                                   |
| `ego`                   | `this`                  | Self reference                                   |
| `novum`                 | `new`                   | Object construction                              |
| `novum X {}`            | `new X({})`             | Constructor with overrides                       |
| `si`/`secus`           | `if`/`else`             | Conditionals                                     |
| `dum`                   | `while`                 | While loop                                       |
| `ex...pro`              | `for...of`              | Iteration                                        |
| `elige`                 | `if/else` chain         | See `principles.md` — always if/else, not switch |
| `tempta`/`cape`/`demum` | `try`/`catch`/`finally` | Exception handling                               |
| `iace`                  | `throw`                 | Throw exception                                  |
| `redde`                 | `return`                | Return statement                                 |
| `textus`                | `string`                | String type                                      |
| `numerus`               | `number`                | Integer (JS has single number type)              |
| `fractus`               | `number`                | Floating point (JS has single number type)       |
| `decimus`               | `Decimal`               | Arbitrary precision (requires library)           |
| `bivalens`              | `boolean`               | Boolean type                                     |
| `nihil`                 | `null`                  | Null value                                       |
| `T[]`              | `T[]`                   | Array type                                       |
| `tabula<K,V>`           | `Map<K,V>`              | Map type                                         |
| `copia<T>`              | `Set<T>`                | Set type                                         |

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

Faber's `genus` generates a constructor that merges field defaults with overrides:

```typescript
class persona {
    nomen: string = 'anonymous';
    aetas: number = 0;

    constructor(overrides: { nomen?: string; aetas?: number } = {}) {
        if (overrides.nomen !== undefined) this.nomen = overrides.nomen;
        if (overrides.aetas !== undefined) this.aetas = overrides.aetas;
        this.creo(); // if defined
    }
}
```

This allows: `novum persona { nomen: "Marcus" }` → `new persona({ nomen: "Marcus" })`

### `cede` Context Sensitivity

`cede` maps to `yield` inside generator functions, `await` inside async functions. The codegen tracks context via `inGenerator` flag.

## Unsupported TypeScript Features

Features that would need new Faber syntax to support.

### High Priority (commonly used)

| TS Feature          | Syntax           | Status   | Latin Candidate       | Notes                            |
| ------------------- | ---------------- | -------- | --------------------- | -------------------------------- |
| Spread (array)      | `...arr`         | Planned  | `sparge arr`          | Array spreading                  |
| Spread (object)     | `...obj`         | Planned  | `sparge obj`          | Object spreading                 |
| Rest parameters     | `...args`        | Planned  | `ceteri args`         | Collect remaining args           |
| Optional chaining   | `?.`             | Planned  | `?.`                  | Keep as punctuation              |
| Non-null assertion  | `!.`             | Planned  | `!.`                  | Keep as punctuation              |
| Nullish coalescing  | `??`             | Partial  | `vel`                 | Extending to general expressions |
| Array destructuring | `[a, b] = arr`   | Planned  | `ex arr fixum [a, b]` | Enabled by `ceteri` rest syntax  |
| typeof (runtime)    | `typeof x`       | Planned  | `x est numerus`       | Via `est` with type RHS          |
| instanceof          | `x instanceof T` | Planned  | `x est T`             | Via `est` with type RHS          |
| Regex literals      | `/pattern/`      | Planned  | `sed /pattern/`       | Keyword disambiguates tokenizer  |
| Getters             | `get prop()`     | Excluded | —                     | Use methods instead              |
| Setters             | `set prop(v)`    | Excluded | —                     | Use `nexum` or methods           |

### Medium Priority (useful)

| TS Feature              | Syntax         | Status   | Latin Candidate | Notes                                       |
| ----------------------- | -------------- | -------- | --------------- | ------------------------------------------- |
| Type assertions         | `x as T`       | Planned  | `x qua T`       | Cast/narrow type                            |
| Computed property names | `{ [key]: v }` | Excluded | —               | Runtime key construction; use tabula        |
| Shorthand properties    | `{ x }`        | Excluded | —               | Rust-only; not portable                     |
| Tagged templates        | `` fn`text` `` | Excluded | —               | JS-specific                                 |
| BigInt                  | `123n`         | Designed | `magnus`        | Constructed via preamble, no literal suffix |
| Decorators              | `@foo`         | Excluded | —               | Semantics differ across targets             |
| Static blocks           | `static { }`   | Excluded | —               | Semantics vary across targets               |

### Lower Priority (advanced TS)

| TS Feature          | Syntax                | Status   | Latin             | Notes                                  |
| ------------------- | --------------------- | -------- | ----------------- | -------------------------------------- |
| keyof               | `keyof T`             | Excluded | —                 | TS-specific type machinery             |
| typeof (type-level) | `typeof x`            | Designed | `typus x`         | Same keyword as type alias declaration |
| Mapped types        | `{ [K in T]: V }`     | Excluded | —                 | TS-specific                            |
| Conditional types   | `T extends U ? A : B` | Excluded | —                 | TS-specific                            |
| Tuple types         | `[A, B, C]`           | Designed | `series<A, B, C>` | Generic type, bracket literals         |
| Index signatures    | `{ [k: string]: T }`  | Designed | `aperit T`        | TS-only; use tabula elsewhere          |
| satisfies           | `x satisfies T`       | Excluded | —                 | TS-specific                            |
| infer               | `infer R`             | Excluded | —                 | TS-specific                            |
| never type          | `never`               | Designed | `numquam`         | "never" — function doesn't return      |
| unknown type        | `unknown`             | Done     | `ignotum`         | Already implemented                    |

### Design Notes

**Index Signatures (`aperit`)**: Declared on `genus` or `pactum` to indicate dynamic string key access.

```
// Just aperit
genus config aperit textus { }

// With implet (order: implet before aperit)
genus persona implet Nominabilis aperit textus {
    textus id
}

// On pactum
pactum DynamicRecord aperit textus { }
```

Generates:

```typescript
class config {
    [key: string]: string;
}

interface DynamicRecord {
    [key: string]: string;
}
```

Grammar:

```
genus_decl := 'genus' identifier type_params? implet_clause? aperit_clause? '{' members '}'
implet_clause := 'implet' identifier (',' identifier)*
aperit_clause := 'aperit' type
pactum_decl := 'pactum' identifier type_params? aperit_clause? '{' members '}'
```

**typeof (type-level) (`typus`)**: Same keyword as type alias declaration. Context disambiguates.

```
fixum config = { port: 3000 }
typus Config = typus config   // type Config = typeof config
```

**Tuple types (`series`)**: Variadic generic type. Values use bracket literals.

```
typus Point = series<numerus, numerus>
fixum p: Point = [10, 20]
fixum series<textus, numerus> pair = ["answer", 42]
```

**never type (`numquam`)**: Function never returns (throws, loops forever).

```
functio moritur() -> numquam {
    iace novum Error { message: "fatal" }
}
```

Distinct from `vacuum` (void — returns nothing) vs `numquam` (never — doesn't return).

**BigInt (`magnus`)**: Arbitrary precision integer. Constructed via preamble, no literal suffix.

```
fixum big: magnus = magnus(123)
fixum huge: magnus = magnus("99999999999999999999")
```

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
