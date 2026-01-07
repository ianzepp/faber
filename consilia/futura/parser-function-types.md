# Parser: Function Types in Type Annotations

## Status: IMPLEMENTED

Function types are now supported in type annotation positions.

## Syntax

```fab
# Predicate parameter
functio filtrata((T) -> bivalens pred) -> lista<T>

# Multi-param function
functio reducta((U, T) -> U fn, U init) -> U

# Nested function types
functio compose((A) -> B f, (B) -> C g) -> (A) -> C

# Callback with no return
functio forEach((T) -> vacuum callback) -> vacuum
```

## Grammar

```
functionType := '(' typeList? ')' '->' typeAnnotation
typeList := typeAnnotation (',' typeAnnotation)*
```

## Codegen Output

| Faber | TypeScript | Python | Rust | C++ | Zig |
|-------|------------|--------|------|-----|-----|
| `(T) -> U` | `(arg0: T) => U` | `Callable[[T], U]` | `impl Fn(T) -> U` | `std::function<U(T)>` | `*const fn(T) U` |

## Implementation

- `TypeAnnotation` extended with `parameterTypes` and `returnType` fields
- `parseTypeAnnotation()` checks for `(` to parse function types
- `parseParameter()` recognizes `(` as start of type annotation
- All 6 codegen targets updated to emit function type syntax

## Related

- lexer-nested-generics.md - both needed for full lista.fab type signatures
- Enables generating semantic exports from .fab files
