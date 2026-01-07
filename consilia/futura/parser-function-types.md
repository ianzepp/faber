# Parser: Function Types in Type Annotations

## Problem

The parser does not support function types as type annotations in parameter positions.

```fab
# This fails to parse:
functio filtrata((T) -> bivalens pred) -> lista<T>

# Error: Expected identifier, got '('
```

## Current Behavior

Type annotations only accept simple identifiers and generics:
- `textus`
- `numerus`
- `lista<T>`
- `tabula<K, V>`

## Required Behavior

Support function type syntax in any type position:

```fab
# Predicate parameter
functio filtrata((T) -> bivalens pred) -> lista<T>

# Multi-param function
functio reducta((U, T) -> U fn, U init) -> U

# Nested function types
functio compose((A) -> B f, (B) -> C g) -> (A) -> C
```

## Syntax

```
functionType := '(' typeList? ')' '->' typeAnnotation
typeList := typeAnnotation (',' typeAnnotation)*
```

## Use Cases

1. **Collection methods** - `filtrata`, `mappata`, `reducta` all take callbacks
2. **Higher-order functions** - compose, curry, pipe
3. **Event handlers** - `(Event) -> vacuum`

## Implementation Notes

- Parser needs to recognize `(` as start of function type in type annotation context
- Must handle nested function types: `((A) -> B) -> C`
- Codegen already handles function types in other contexts

## Related

- lista.fab methods need function type parameters for full type signatures
- Enables generating semantic exports from .fab files
