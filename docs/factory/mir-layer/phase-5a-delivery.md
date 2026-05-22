# Phase 5A Delivery: Alternate-Exit Surface

## Interpreted Problem

Phase 4 gives MIR an explicit normal control-flow graph, but recoverable failure is still only a loose source-level idea. Before MIR should lower failure flow, the language needs a visible function contract for the alternate exit path.

The selected working syntax is:

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus
```

`→ numerus` is the normal exit path. `⇥ textus` is the recoverable alternate exit path. The `⇥` glyph remains a working choice; if the glyph changes, Phase 5A should rename the token and docs before implementation proceeds.

## Normalized Spec

- Add a lexer token for `⇥`.
- Extend function declarations to support `→ Success ⇥ Error`.
- Extend pactum method signatures to support `→ Success ⇥ Error`.
- Extend function types to support `(A, B) → Success ⇥ Error`.
- Carry the optional alternate-exit type through AST, HIR, semantic function signatures, and relevant inspection output.
- Typecheck `iace expr` against the current function's alternate-exit type.
- Reject `iace` when no alternate-exit type is declared, unless a later local handler explicitly consumes it.
- Reject failable calls used as ordinary values until caller-side handling/propagation syntax is defined.
- Keep `redde` tied to the `→` path and `iace` tied to the `⇥` path.
- Keep `mori` outside the typed recoverable path; it remains fatal and not catchable.

## Non-Propagation Contract

`iace` does not search up the dynamic call stack for the nearest function with a `⇥` exit. That would make recoverable failure an ambient exception mechanism.

Instead:

- `iace` in a function with `⇥ Error` exits through that function's own alternate-exit path.
- `iace` in a function without `⇥ Error` is a compiler error unless a later local handler construct explicitly consumes it inside the same function.
- Calling a failable function does not silently infect the caller; the caller must handle, propagate, or fatally convert the alternate path through syntax defined by a later phase.

## Surface Contract

Valid:

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b = 0 ergo iace "division by zero"
    redde a / b
}
```

Invalid:

```fab
functio divide(numerus a, numerus b) → numerus {
    si b = 0 ergo iace "division by zero"
    redde a / b
}
```

The invalid example should fail because `iace` has no declared alternate-exit path.

## Repo-Aware Baseline

- `EBNF.md` currently defines `returnClause := '→' typeAnnotation`.
- `crates/radix/src/lexer/token.rs` has `TokenKind::Arrow` for `→`.
- `crates/radix/src/parser/decl.rs` parses function return types after `TokenKind::Arrow`.
- `crates/radix/src/parser/types.rs` parses function types as `(args) → ret`.
- `crates/radix/src/hir/lower/stmt.rs` already lowers `iace` to `HirExprKind::Throw`.
- `crates/radix/src/semantic/passes/typecheck/expr.rs` currently treats `Throw` as `vacuum`, without checking a function-level error type.
- `crates/radix/src/semantic/types.rs` has `FuncSig` as the function type carrier.

## Stage Graph

1. Name the token and glyph contract consistently, likely `TokenKind::ExitArrow` for `⇥`.
2. Add lexer coverage for `⇥`.
3. Extend AST function declarations and method signatures with `err: Option<TypeAnnotation>`.
4. Extend parser support for optional `⇥ Error` after `→ Success`.
5. Extend function type parsing for `(A) → B ⇥ E`.
6. Extend HIR function and function type carriers with optional error type.
7. Extend semantic `FuncSig` with optional error type.
8. Track the current function's alternate-exit type during typechecking.
9. Typecheck `HirExprKind::Throw` against the current alternate-exit type.
10. Add diagnostics for `iace` without `⇥ Error`.
11. Add diagnostics for failable calls used as plain values until caller handling is defined.
12. Update `EBNF.md` and `docs/grammatica/functiones.md`.

## Checkpoints

- Lexer tests prove `⇥` is tokenized distinctly.
- Parser tests cover function declarations, pactum method signatures, and function types with `⇥`.
- Semantic tests prove `iace` is accepted only when the enclosing function declares a compatible alternate-exit type.
- Semantic tests prove `redde` still checks against the normal `→` type.
- Failable calls in ordinary expression position fail clearly.
- Existing function syntax remains accepted unchanged.
- Existing target codegen behavior for old programs remains unchanged.

## Fixture Candidates

Failable function:

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b = 0 ergo iace "division by zero"
    redde a / b
}
```

Non-failable function with invalid `iace`:

```fab
functio broken() → numerus {
    iace "no alternate exit"
    redde 0
}
```

Failable function type:

```fab
functio apply((numerus) → numerus ⇥ textus op, numerus n) → numerus ⇥ textus {
    iace "caller handling deferred"
}
```

## Out Of Scope

- MIR lowering of `iace`.
- `tempta`, `cape`, and `demum` semantics.
- Caller-side propagation syntax.
- Caller-side local handling syntax.
- Rust backend support for failable signatures.
- WASM or native lowering.

## Validation

- Focused lexer/parser tests for `⇥`.
- Focused semantic tests for `iace` and function signatures.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 5A complete.

## Completion Gate

Phase 5A is complete when the compiler understands `→ Success ⇥ Error` as a typed function contract, typechecks `iace` against that contract, rejects unhandled failable calls and undeclared `iace`, and leaves MIR/backend behavior unchanged.
