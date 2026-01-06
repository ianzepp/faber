---
status: completed
updated: 2026-01-06
note: innatum keyword fully implemented for all targets with comprehensive test coverage
implemented:
  - innatum keyword and syntax
  - Empty collection construction ({} innatum tabula, [] innatum lista)
  - Non-empty collection construction
  - All targets (ts, py, rs, cpp, zig)
tests: 8 tests covering empty/non-empty lista and tabula
---

# innatum - Native Type Construction

## Summary

The `innatum` keyword constructs native/builtin types with target-appropriate initialization.

## Motivation

Faber has builtin generic types (`tabula<K,V>`, `lista<T>`) that map to different native types per target. Currently there's no way to construct an empty instance:

```fab
# This is a cast, not a constructor - {} has no Map methods
varia agri = {} qua tabula<textus, Typus>
agri[key] = value  # TypeError: agri.set is not a function
```

## Syntax

```fab
literal 'innatum' typeAnnotation
```

The `innatum` keyword (Latin: "inborn, innate") indicates the literal should be constructed as the native representation of the specified type.

## Examples

```fab
# Empty tabula (map)
varia agri = {} innatum tabula<textus, Typus>

# Empty lista (array/vector)
varia items = [] innatum lista<numerus>

# With initial values
varia scores = [1, 2, 3] innatum lista<numerus>
```

## Grammar

```ebnf
innatumExpression := literalExpression 'innatum' typeAnnotation
literalExpression := objectLiteral | arrayLiteral
```

## Codegen

| Expression | TypeScript | Python | Rust | Zig | C++ |
|------------|------------|--------|------|-----|-----|
| `{} innatum tabula<K,V>` | `new Map<K,V>()` | `{}` | `HashMap::new()` | `std.AutoHashMap(K,V).init(alloc)` | `std::map<K,V>{}` |
| `[] innatum lista<T>` | `[]` | `[]` | `Vec::new()` | `std.ArrayList(T).init(alloc)` | `std::vector<T>{}` |

## Semantics

1. The literal must be compatible with the target type:
   - `{}` (empty object) for `tabula<K,V>`
   - `[]` (empty/non-empty array) for `lista<T>`
2. The type annotation must be a builtin generic type
3. Type parameters are required (no inference from empty literal)

## Comparison with `qua`

| Operator | Meaning | Runtime effect |
|----------|---------|----------------|
| `x qua T` | Cast/reinterpret x as type T | None - type assertion only |
| `x innatum T` | Construct x as native T | Creates proper native instance |

## Error Cases

```fab
# Error: {} is not valid for lista
varia x = {} innatum lista<numerus>

# Error: [] is not valid for tabula
varia y = [] innatum tabula<textus, numerus>

# Error: innatum requires builtin type
varia z = {} innatum MeumGenus
```

## Implementation

1. **Lexer**: Add `innatum` to keyword list
2. **Parser**: Parse as postfix operator with type annotation, create `InnatumExpression` node
3. **Semantic**: Validate literal/type compatibility, resolve type parameters
4. **Codegen**: Emit target-specific constructor

## Related

- `qua` - type casting (no runtime effect)
- `novum` - genus instance construction
