# Side Quest: Type-First `ad` Channel Syntax

**Status**: implemented, validation passed
**Created**: 2026-05-24
**Scope**: syntax cleanup after Epic 3

## Interpreted Problem

Epic 3 made non-strict `ad` capability calls compile in the Rust path, but the binding syntax still uses the historical endpoint form:

```fab
ad "parser:parse" (input) → numerus pro value { ... }
```

That is inconsistent with Faber's type-first declarations and hides the fact that `ad` has success and error channels, like functions.

## Target Syntax

Canonical success channel:

```fab
ad "parser:parse" (input) → numerus value { ... }
```

Canonical success plus declared error channel:

```fab
ad "http:get" (url) → HttpResponse res ⇥ HttpError { ... } cape err { ... }
```

Semantics:

- `→ Type name` declares the success value type and success binding.
- `⇥ ErrorType` declares the error channel type.
- `cape err` binds the error value using the declared error channel type.
- `pro` is no longer part of `ad` syntax.
- Unresolved non-strict providers still compile when channel types are explicit.

## Implementation Steps

1. Update parser AST and grammar for `ad` success/error channels. Done.
2. Lower the success binding and error type into HIR. Done.
3. Typecheck catch blocks against the declared error type. Done.
4. Update Rust/Faber codegen, docs, examples, and focused tests. Done.
5. Validate with focused `ad`, Rust codegen, and Rust exempla e2e tests. Done.

## Validation Results

- `cargo test -p radix ad -- --nocapture`: passed.
- `cargo test -p radix codegen::rust -- --nocapture`: passed.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`: passed, `100/100` Rust exempla.

## Stop Line

Do not implement provider metadata, strict verification, host routing, or MIR/Wasm lowering here. This is only the source syntax and current Rust/Faber surface migration.
