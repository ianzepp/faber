---
status: completed
updated: 2026-01-06
note: Multi-phase semantic analysis fully implemented. Forward references and type alias resolution working. Throwability propagation remains future work.
implemented:
  - Phase 1a: Predeclaration (functions, genus, pactum, ordo, discretio, type aliases)
  - Phase 1b: Signature resolution
  - Phase 1c: Type alias fixed-point iteration
  - Phase 1d: Circular type alias detection
  - Phase 2: Body analysis with complete symbol table
  - Forward function references
  - Mutual recursion
  - Forward type references
tests: 3 tests in fundamenta.yaml (forward function, mutual recursion, forward type)
future:
  - Phase 3: Effect analysis (throwability propagation for Zig/Rust)
  - Type inference across functions
---

# Two-Pass Semantic Analysis

Faber's semantic analyzer uses a **multi-phase approach** to enable forward references within a file. This document describes the current implementation and remaining work.

## Current Implementation

### Phase Structure

The analyzer in `fons/semantic/index.ts` processes each file in five phases:

```
Phase 1a: Predeclare all top-level names with placeholder types
Phase 1b: Resolve signatures with real types
Phase 1c: Iteratively resolve type aliases to fixed point
Phase 1d: Detect circular type aliases
Phase 2:  Analyze bodies with complete symbol table
```

### Phase 1a: Predeclaration

Walks the entire file and registers top-level names _without analyzing bodies_:

- **Functions:** name + placeholder function type (UNKNOWN params/return)
- **Genus:** name + empty shell (no fields/methods yet)
- **Pactum:** name + empty shell
- **Ordo:** name + fully resolved members (enums don't forward-reference)
- **Discretio:** name as user type
- **Type aliases:** name + UNKNOWN placeholder

This mirrors the cross-file export extraction in `modules.ts`, but for within-file declarations.

### Phase 1b: Signature Resolution

Now that all names exist in the symbol table:

- Resolve function parameter and return type annotations
- Resolve genus fields and method signatures
- Resolve pactum method signatures
- Resolve type alias definitions

Uses `updateSymbolType()` to refine placeholder types to real types.

### Phase 1c: Type Alias Fixed Point

Type alias chains like `A = B; B = C; C = numerus` require multiple passes when defined in forward-reference order. This phase iteratively re-resolves any type alias still at UNKNOWN until no progress is made.

### Phase 1d: Cycle Detection

Any type alias remaining at UNKNOWN after iteration is flagged as circular:

```fab
typus A = B
typus B = A  # Both flagged: "Circular type alias: 'X' cannot be resolved"
```

Direct self-references (`typus A = A`) are caught earlier via `resolvingTypeAliases` tracking.

### Phase 2: Body Analysis

Performs existing analysis with a complete symbol table:

- Variable resolution and assignment checks
- Expression typing
- Return checking
- Async/generator checks

The `analyze*` functions check `lookupSymbolLocal()` and skip `define()` if the symbol was predeclared.

## What This Enables

### Forward Function References

```fab
functio b() { a() }
functio a() { scribe "hello" }
```

### Mutual Recursion

```fab
functio isEven(numerus n) fit bivalens {
  si n == 0 { redde verum }
  redde isOdd(n - 1)
}
functio isOdd(numerus n) fit bivalens {
  si n == 0 { redde falsum }
  redde isEven(n - 1)
}
```

### Forward Type References

```fab
functio process(User u) fit textus {
  redde u.name
}
genus User {
  textus name
}
```

### Type Alias Chains

```fab
typus A = B
typus B = C
typus C = numerus
fixum A x = 42  # Works: A resolves to numerus
```

## Remaining Work: Throwability Propagation

### The Problem

```fab
functio a() { iace "error" }
functio b() { a() }  # b also throws, but semantics doesn't know this
```

For Zig (and potentially Rust/C++), the compiler needs transitive throwability information.

### Phase 3: Effect Analysis (Planned)

#### 3a: Build Call Graph

During Phase 2, record static call edges:

- Identifier calls: `a()`
- Method calls on genus instances: `ego.method()` when resolved

Dynamic calls (function values) should be treated conservatively.

#### 3b: Compute `canThrow` With Catch-Awareness

Compute **uncaught-throw** inside each function body:

- When analyzing a call to a throwing callee, check if it's in a caught context (`tempta` with handler)
- Only record edges representing uncaught propagation

Fixed-point iteration over the call graph:

- If `a` contains an uncaught `iace`, mark `a` throws
- If `b` has an uncaught call to `a`, mark `b` throws
- Repeat until stable

### Impact on Codegen

**Zig:** Currently scans the AST during codegen to decide if a function needs `!T`. Should move to semantic `canThrow` property. `genCallExpression` can insert `try` when calling a throwing callee.

**Rust:** Currently emits `return Err(expr)` but doesn't wrap return types. Options: require explicit `Result<T, E>` or auto-rewrite when `canThrow` is true.

**C++:** Currently uses exceptions. Migration to `std::expected` would require significant codegen changes.

## Open Questions

### Top-Level Variable Forward References

Variable initializers are analyzed immediately and will fail if they reference later bindings. This is intentional - forward references in initializers remain disallowed.

### Generics and Inference

The semantic analyzer does not infer function return types from `redde` expressions; unannotated functions default to `vacuum`. Type inference across functions would require:

- Another fixed-point analysis (possibly over SCCs)
- Conservative widening
- Explicit restrictions for recursion

## Migration Plan

| Step | Description | Status |
|------|-------------|--------|
| 0 | Predeclare top-level functions | Done |
| 1 | Add predeclare pass in `fons/semantic/index.ts` | Done |
| 2 | Split signature resolution from body analysis | Done |
| 3 | Add iterative type alias resolution | Done |
| 4 | Add type alias cycle detection | Done |
| 5 | Add call graph collection during Phase 2 | Planned |
| 6 | Add throwability computation (Phase 3) | Planned |
| 7 | Teach Zig codegen to use semantic `canThrow` | Planned |

## Test Coverage

Tests in `proba/fundamenta.yaml`:

| Test | Description | Status |
|------|-------------|--------|
| Forward function reference | `b()` calls `a()` defined later | Passing |
| Mutual recursion | `isEven`/`isOdd` call each other | Passing |
| Forward type reference | Function param uses genus defined later | Passing |

Cycle detection verified via `faber check`:

```bash
echo 'typus A = B; typus B = A' | bun run faber check -
# Errors: Circular type alias: 'A' cannot be resolved
#         Circular type alias: 'B' cannot be resolved
```

## Key Files

| File | Purpose |
|------|---------|
| `fons/semantic/index.ts` | Main analyzer with phase loops |
| `fons/semantic/scope.ts` | `updateSymbolType()` for refining predeclared types |
| `fons/semantic/modules.ts` | Cross-file export extraction (similar pattern) |

Opus nondum perfectum est, sed via est clara.
