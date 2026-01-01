---
status: analysis
note: Research document identifying gaps for self-hosting; not a feature spec
updated: 2024-12
---

# Rivus: Self-Hosted Compiler Analysis

Analysis of TypeScript features used in `fons/` that are NOT covered by the current Faber grammar. This identifies what must be added to Faber before the compiler can be self-hosted.

## Critical Gaps

These features are heavily used and would be difficult to work around:

### 1. Type Casting (`as` operator)

**Usage:** 200+ occurrences

```typescript
// fons/semantic/types.ts:257
b as PrimitiveType;
```

Used extensively for type narrowing in pattern matching. Faber would need an equivalent like `qua` (as):

```fab
(b qua typus_primitivus)
```

### 2. Interface Declarations

**Usage:** 50+ interfaces

```typescript
// fons/semantic/types.ts
export interface BaseType {
    kind: string;
    nullable?: boolean;
}
```

Faber has `pactum` for method contracts but not for pure data shapes. Options:

- Extend `genus` to work as interfaces
- Add `forma` keyword for structural types
- Use `pactum` for data shapes too

### 3. Discriminated Unions

**Usage:** 50+ type unions

```typescript
// fons/parser/ast.ts:81-101
export type Statement =
    | VarDeclaration
    | FuncDeclaration
    | IfStatement
    | ...;
```

Core to AST safety. The `kind` field pattern enables exhaustive `switch` matching. Faber's `typus` supports unions but not discrimination.

### 4. Generic Type Parameters

**Usage:** Extensive

```typescript
// fons/semantic/types.ts
interface GenericType extends BaseType {
    typeParameters: SemanticType[];
}
```

Faber has `T[]` syntax but not generic interfaces/classes.

### 5. Optional Properties (`?:`)

**Usage:** 100+ fields

```typescript
// fons/parser/ast.ts
resolvedType?: SemanticType;
```

Faber has `T?` for nullable types but the `:` field syntax with `?` isn't in the grammar for genus fields.

### 6. Template Literals

**Usage:** 100+ occurrences

```typescript
// fons/codegen/ts/index.ts
`const ${name} = ${value};`;
```

Essential for code generation. Faber needs backtick templates with `${}` interpolation.

## Moderate Gaps

These are used but workarounds exist:

### 7. Spread Operator

**Usage:** 20+ occurrences

```typescript
[...items, newItem]
{ ...obj, newField: value }
```

Faber has `sparge` keyword. Verify it works in all contexts (arrays, objects, function calls).

### 8. Arrow Functions in Expressions

**Usage:** 40+ occurrences

```typescript
items.map(x => x * 2);
```

Faber has `pro x: x * 2` lambda syntax. Verify it works in method call contexts.

### 9. Array Method Chains

**Usage:** 30+ occurrences

```typescript
// fons/lexicon/types-builtin.ts:467
builtinTypes.map(t => [t.stem, t]).filter(...)
```

Faber has `lista` methods. May need to verify `.mappa()`, `.filtra()`, `.reduce()` equivalents.

### 10. Object.entries/keys/values

**Usage:** 10+ occurrences

```typescript
Object.entries(exports);
```

Would need `tabula` or `objectum` methods: `.ingressus()`, `.claves()`, `.valores()`.

### 11. Regular Expressions

**Usage:** 10+ occurrences

Used in tokenizer for pattern matching. Could be replaced with character-by-character parsing, but adds complexity.

### 12. Destructuring

**Usage:** 20+ occurrences

```typescript
const { tokens, errors } = tokenize(source);
```

Faber has `ex response fixum { data }` syntax. Verify nested destructuring works.

## Low Priority Gaps

These can be easily worked around:

### 13. `import type` Syntax

Tree-shaking optimization only. Use regular imports.

### 14. Enum Declarations

```typescript
export enum SemanticErrorCode { ... }
```

Faber has `ordo`. Direct equivalent.

### 15. Default Parameters

```typescript
function foo(x: number = 0) { ... }
```

Handle at call sites or add to Faber grammar.

### 16. `as const` Assertions

```typescript
const TARGETS = ['ts', 'py'] as const;
```

Use explicit type annotations instead.

### 17. Class Declarations

Only 5 uses (error classes). Faber's `genus` covers this.

## Features NOT Used in fons/

Good news — these TypeScript features are NOT needed:

- Mapped types (`{ [K in keyof T]: ... }`)
- Conditional types (`T extends U ? X : Y`)
- Utility types (Partial, Pick, Omit)
- Function overloading
- Namespaces
- Abstract classes
- Decorators
- Symbols

## Recommended Language Additions

Priority order for enabling self-hosting:

1. **Template literals** — `"hello ${nomen}"` syntax
2. **Type casting** — `expr qua typus` syntax
3. **Optional fields in genus** — `textus? nomen` syntax
4. **Interface/shape types** — extend `pactum` or add `forma`
5. **Generic constraints** — `genus T[]` declarations
6. **Discriminated union support** — exhaustiveness checking

## Bootstrap Strategy

Given these gaps, recommended approach:

1. **Phase 1**: Compile `rivus/` to Zig (not TypeScript)
    - Zig doesn't need TS type features
    - Focus on runtime correctness

2. **Phase 2**: Add missing Faber features incrementally
    - Template literals first (essential for codegen)
    - Type casting second (needed for AST manipulation)

3. **Phase 3**: Rewrite using new features
    - Update `rivus/` to use new syntax
    - Maintain parallel TS/Faber implementations

## Current rivus/ Status

### Converted Files

| fons/                      | rivus/                        | Size | Status         |
| -------------------------- | ----------------------------- | ---- | -------------- |
| `lexicon/types.ts`         | `lexicon/typi.fab`            | 3.8K | ✓ Full compile |
| `lexicon/types-builtin.ts` | `lexicon/typi_constructi.fab` | 13K  | Parser OK      |
| `lexicon/keywords.ts`      | `lexicon/verba_clavium.fab`   | 9.4K | Parser OK      |
| `lexicon/nouns.ts`         | `lexicon/nomina.fab`          | 11K  | Parser OK      |
| `lexicon/verbs.ts`         | `lexicon/verba.fab`           | 13K  | Parser OK      |

**Not yet converted:** `lexicon/index.ts` (module re-exports — depends on import resolution)

### Blocking Issues

#### 1. Cross-File Import Resolution

The semantic analyzer doesn't resolve imports from other files.

```fab
# rivus/lexicon/nomina.fab
ex "./typi" importa declinatio, genera, casus, numerositas

# Later:
si decl est declinatio.prima { ... }
#              ^^^^^^^^^^^ "Undefined variable 'declinatio'"
```

The parser creates an `ImportDeclaration` AST node, but the semantic analyzer doesn't:

1. Resolve the path (`"./typi"` → `./typi.fab`)
2. Parse the imported file
3. Extract its exports
4. Add them to the current file's scope

**Required work:**

- Path resolution logic
- File I/O during semantic analysis
- Export tracking per file
- Import caching (don't re-parse same file)
- Cycle detection (A imports B imports A)

#### 2. Generic Type Inference for Literals

Object/array literals don't infer types from annotations:

```fab
fixum verbum_clavis[] verba = [
    { latinum: "si", ... }
]
# Error: Type 'objectum[]' is not assignable to 'verbum_clavis[]'
```

The analyzer infers `objectum[]` from the literal instead of using the declared type.

#### 3. `novum tabula de {...}` Type Inference

Same issue with Map initialization:

```fab
fixum tabula<textus, casus_numerus[]> terminationes = novum tabula de {
    "a": [...]
}
# Error: Type 'tabula' is not assignable to 'tabula<textus, casus_numerus[]>'
```

### Naming Conventions

Type names are lowercase (Latin had no title case):

- `casus`, `numerositas`, `genera`, `declinatio`
- `nomen_ingressus`, `verbum_resolutum`

Field names avoid reserved keywords:

- `genus` (keyword) → `forma` (for grammatical gender field)
- `typus` (keyword) → use different variable names in loops

### Directory Structure

```
rivus/
└── lexicon/
    ├── typi.fab              — grammar type definitions
    ├── typi_constructi.fab   — built-in type vocabulary
    ├── verba_clavium.fab     — reserved keywords
    ├── nomina.fab            — noun declension tables
    └── verba.fab             — verb conjugation tables
```

Mirrors `fons/lexicon/` with Latin file names.
