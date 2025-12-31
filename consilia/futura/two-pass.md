---
status: partial
note: Phase 0/1 (forward references) implemented 2025-12; Phase 3 (throwability) remains planned
updated: 2025-12
---

# Two-Pass Semantic Analysis (Revised)

Faber’s semantic analyzer currently performs a **single pass** over the AST. This is sufficient for many local checks, but it breaks down when analysis of one construct depends on information that appears later in the same file.

This document updates the original proposal with **findings from the current codebase** and a revised implementation plan that fits existing architecture.

## Current State (Verified In Repo)

### Semantic Analyzer is Single-Pass (Within a File)

The main entry point `fons/semantic/index.ts` walks statements in source order:

- `for (const stmt of program.body) analyzeStatement(stmt)`

Declarations are registered only when their statement is visited:

- `analyzeFunctioDeclaration` defines the function symbol, then analyzes its body.

This means **within-file forward references are currently invalid** for functions (and anything else that depends on a symbol being defined earlier).

### Cross-File Imports Already Do a “Pre-Pass”

Local import resolution (`fons/semantic/modules.ts`) parses imported modules and extracts exports by scanning all top-level statements.

Important detail: export extraction intentionally creates **placeholder types** (e.g. a genus export uses a `genusType` with empty maps). This is a big reason cross-file references are more forgiving than within-file references.

### Type Resolution is Permissive for Unknown Types

`resolveTypeAnnotation` behaves as follows:

- Built-ins map via `LATIN_TYPE_MAP`
- Known generics map via `GENERIC_TYPES`
- If the name resolves to a `kind === 'type'` alias in scope, it expands
- Otherwise it returns `userType(name)` (no error)

So _spelling a type that is not yet defined_ typically does not error immediately; it becomes an opaque user type.

This permissive behavior helps parsing/analysis continue, but it also means that without a later “linking” phase, code like:

```fab
functio getName(B user) { redde user.nomen }

genus B { textus nomen }
```

cannot be validated properly when `B` isn’t yet a `genusType` at the time `getName` is analyzed.

### `tempta/cape` Exists as Syntax and Scoping, Not as Effect Masking

The parser supports `tempta/cape` and also `cape` clauses on several control-flow constructs. The semantic analyzer:

- Analyzes the try block and catch block in their own scopes
- Defines the catch variable as `Error`

There is **no semantic notion of “caught vs propagated”** today.

### `iace` Codegen Exists, But “Throwability” is Not a Semantic Property Yet

This repo already has meaningful `iace` codegen, but it is target-specific and not driven by semantic throwability.

**Rust**

- `iace expr` becomes `return Err(expr);`.
- Function signatures are **not automatically rewritten to `Result<..., ...>`**; they only emit the explicit return annotation (or none).
- `incipit` generates `fn main() { ... }` (not `-> Result<...>`).

So Rust currently has _Err-returning statements_, but not an end-to-end “throwability type” story.

**Zig**

- A function return type becomes `!T` if its body contains a non-fatal `iace` (a recursive AST scan during codegen).
- This is **local-only**: callers do not become throwing just because they call a throwing callee.
- Call expressions do not insert `try` based on callee throwability.
- `tempta` codegen for Zig is explicitly a stub/comment.

**C++**

- The current target note says it uses exceptions “for simplicity”, not `std::expected`.

Conclusion: “throwability propagation” remains a semantic gap (and Zig’s current behavior is a syntactic approximation limited to direct `iace` within a function).

## Why Two-Pass (Still) Matters

Two-pass is still the cleanest approach for Faber’s goals:

- Natural ordering within files
- Mutual recursion
- Enabling transitive analyses (throwability, async, allocator needs, purity)

But the repo realities suggest that “two-pass” should be implemented as **two passes plus a couple micro-phases**.

## Problems to Solve (Reframed)

### 1) Within-File Forward References (Functions)

```fab
functio b() { a() }
functio a() { }
```

Current semantics fails because `a` is undefined at the time `b` is analyzed.

### 2) Within-File Forward References (Types)

```fab
genus A { B campo }
genus B { A campo }
```

Today this typically becomes `userType('B')` when `A` is analyzed first, which prevents accurate field/member validation later.

### 3) Throwability (as a Transitive Property)

```fab
functio a() { iace "error" }
functio b() { a() }
```

For Zig (and potentially Rust/C++ in the future), the compiler needs to know that `b` also throws.

### 4) `tempta/cape` Masking

```fab
functio safe() {
    tempta { mayThrow() } cape e { scribe(e) }
}
```

A naive call-graph propagation would mark `safe` as throwing; semantically it should not.

## Minimum Viable: Function Hoisting Only

If the immediate goal is just to avoid within-file function ordering constraints (bootstrap-friendly), implement a minimal two-pass semantic flow:

- **Pass 0:** Predeclare all top-level `functio` names and signatures in the file scope.
- **Pass 1:** Analyze statements and function bodies as today, but skip redefining functions (only analyze bodies).

This solves:

```fab
functio b() { a() }
functio a() { }
```

It intentionally does **not** solve forward references for genus/pactum/type aliases, nor does it add throwability propagation.

## Recommended Architecture

### Phase 1a: Predeclare (Names Only)

Walk the entire file and predeclare top-level names _without analyzing bodies_.

Predeclare:

- Functions: name + (possibly unresolved) function type shell
- Type aliases: name (and later expansion)
- Ordo (enum): name shell
- Genus (struct/class): name shell
- Pactum (interface): name shell

**Key design choice:** for genus/pactum/ordo, create a nominal entry up front (a “shell”) so later references can be linked to the same semantic identity.

This phase should be conceptually similar to cross-file export extraction, but for the current file.

### Phase 1b: Resolve Signatures and Type Shapes (No Bodies)

Now that all names exist:

- Resolve function parameter/return annotations
- Resolve genus fields + method signatures
- Resolve pactum methods
- Resolve type aliases (potentially with cycle checks)

Important: this is what turns `userType("B")` references into a resolvable nominal `genusType("B", ...)` when `B` is in the same file.

### Phase 2: Analyze Bodies (Typecheck + Scopes)

Perform existing body analysis using a stable, complete symbol table.

This phase includes:

- Variable resolution and assignment checks
- Expression typing
- Return checking
- Existing async/generator checks

### Phase 3: Effect Analysis (Throwability, Async Propagation, …)

This repo already has a precedent for “semantic properties driving codegen”:

- `CallExpression.needsCurator` is set semantically based on function type metadata.

Throwability should follow that approach:

- Semantic analysis computes per-function properties
- Codegen consumes those properties (rather than re-scanning the AST in codegen)

#### 3a: Build a Call Graph (Static Edges Only)

During body analysis, record static call edges:

- identifier calls: `a()`
- method calls on genus instances: `ego.connect()` when resolved to a known method symbol

Dynamic calls (function values) should be treated conservatively:

- Either: record as “unknown call” and assume `mayThrow`
- Or: skip propagation from them (unsound, not recommended)

#### 3b: Compute `canThrow` With Catch-Awareness

Instead of a simple propagation pass that ignores `cape`, compute **uncaught-throw** inside each function body.

A minimal viable approach:

- When analyzing a call to a potentially-throwing callee, determine whether it occurs in a “caught context” (inside `tempta` try region that has a handler, inside a `cape`-capable statement region, etc.).
- Only record edges that represent _uncaught propagation_.

Then, compute a fixed point (or SCC propagation) over the uncaught call graph:

- If `a` contains an uncaught `iace`, mark `a` throws.
- If `b` has an uncaught call edge to `a`, mark `b` throws.
- Repeat to stability (mutual recursion works).

SCC propagation is a nice optimization, but a simple fixed-point loop is acceptable to start.

## Impact on Codegen (Revised)

### Zig

Today Zig codegen scans the body to decide whether a function needs `!T`. That should move to semantic.

- `FunctioDeclaration.canThrow` (or an external semantic side-table keyed by node) becomes the source of truth.
- `genCallExpression` can insert `try` when calling a throwing callee.

Also note: Zig `tempta` codegen is currently a stub. If Zig is meant to use error unions, then `tempta/cape` semantics likely need to be expressed as:

- `expr catch |err| { ... }` patterns for expressions
- or a structured lowering strategy for statement blocks

This is non-trivial, but semantic throwability should still be implemented because it is useful even without full Zig `tempta` support.

### Rust

Rust currently emits `return Err(expr);` for `iace`, but does not wrap function return types.

Two realistic near-term options:

- Keep Rust “manual”: require explicit `-> Result<T, E>` when using `iace`.
- Or: add codegen that rewrites return types to `Result<...>` when `canThrow` is true (requires choosing an error type and updating `incipit`/`main`).

### C++

C++ currently uses exceptions. If the long-term goal is `std::expected`, throwability analysis still helps, but codegen needs a much larger design (expected propagation, early returns, etc.).

## Open Questions (Updated)

### Top-Level Variable Forward References

Currently, variable initializers are analyzed immediately and will fail if they reference later bindings.

This design suggests continuing to **disallow forward references in initializers** for now.

### Generics and Inference

The current semantic implementation does not infer function return types from `redde` expressions; unannotated functions default to `vacuum`.

So “type inference across functions” is not solved by two-pass alone.

If function return inference becomes a goal later, it likely requires:

- Another fixed-point analysis (possibly over SCCs)
- Conservative widening
- Explicit restrictions for recursion

## Migration Plan (Revised)

0. ~~MVP: Predeclare top-level functions only (hoisting)~~ ✓ Done
1. ~~Add an internal "predeclare" pass in `fons/semantic/index.ts`~~ ✓ Done
2. ~~Split type-shape resolution into a signatures-only phase (no bodies)~~ ✓ Done
3. ~~Run existing body analysis as Phase 2~~ ✓ Done
4. Add call graph collection during Phase 2
5. Add throwability computation as Phase 3 (uncaught edges)
6. Teach Zig codegen to rely on semantic `canThrow` instead of AST scanning
7. ~~Add tests mirroring the cases below~~ ✓ Done (forward refs, mutual recursion, forward types)

## Testing (Revised)

1. Within-file forward function reference
2. Mutual recursion
3. Within-file genus forward type reference + member access validity
4. Direct `iace` marks function throwing
5. Transitive propagation (a calls b, b throws)
6. Mutual recursion propagation (SCC)
7. `tempta/cape` masks throwability for caught calls
8. Zig: `try` insertion at call sites (when enabled)

Opus nondum perfectum est, sed via est clara.
