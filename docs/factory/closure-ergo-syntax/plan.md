# Closure Ergo Syntax Factory Plan

**Status**: planned, pending design review
**Created**: 2026-05-22
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/closure-ergo-syntax/`
**Mode**: language surface redesign / parser and type inference follow-up
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Current closure syntax is too heavy for the place closures are normally used:

```fab
users.filtrata(clausura User user: non user.activus)
```

The long `clausura` keyword makes inline functional code feel bulky, and the `:` body separator is visually awkward in a language that intentionally uses type-first declarations instead of colon-based type annotations.

The redesign should make short closures short while preserving Faber's larger grammar principles:

- type-first syntax remains canonical,
- `→` and `⇥` continue to describe callable result and error channels,
- single-statement expression bodies use a compact marker,
- multi-line bodies require braces,
- no new closure-only body marker is introduced if an existing language concept can carry the meaning.

## Current Reality

The current grammar describes closure expressions as:

```ebnf
clausuraExpr   := 'clausura' clausuraParams? ('→' typeAnnotation)? (':' expression | blockStmt)
clausuraParams := clausuraParam (',' clausuraParam)*
clausuraParam  := typeAnnotation IDENTIFIER
```

The grammar also allows `_` as a type annotation generally, so `clausura _ user: ...` parses today. Simple receiver-method cases such as list `map` and `filtrata` already use expected callable context for `_` closure parameters; implementation should inventory any remaining inference gaps before assuming explicit parameter types are still required.

The stdlib list filter signature is:

```fab
functio filtrata((T) → bivalens pred) → lista<T>
```

That target shape is the right semantic model: a predicate closure over `T` returns `bivalens`.

## Proposed Design

### Ergo Symbol

Treat `∴` as the symbolic spelling of `ergo`.

```text
ergo-token := 'ergo' | '∴'
```

The meaning is language-wide rather than closure-specific: "therefore, the following tail applies."

Existing single-tail constructs may accept either spelling:

```fab
si cond ergo redde x
si cond ∴ redde x
```

The symbolic form is not a block opener.

The existing `⇒` glyph remains reserved for runtime conversion (`conversio`) and is not part of this redesign.

### Tail Scope

`∴` follows the existing `ergo` behavior: it scopes to one following statement, not one physical source line.

Valid:

```fab
_ user ∴ non user.activus
si user.activus ∴ redde user
dum i < 10 ∴ i ← i + 1
```

Also valid, because newlines are whitespace in the current parser:

```fab
_ user ∴
    non user.activus
```

Once a body needs vertical space, it must use braces:

```fab
User user {
    nota user.nomen
    redde non user.activus
}
```

Formatters should prefer keeping short `∴` tails on one line. If the tail is complex enough to read poorly, they should convert or preserve a braced block form.

### Closure Forms

Short closures should not need the `clausura` keyword.

Inferred parameter type:

```fab
users.filtrata(_ user ∴ non user.activus)
```

Explicit parameter type:

```fab
users.filtrata(User user ∴ non user.activus)
```

Multiple parameters should use parentheses to avoid ambiguity:

```fab
numeri.compone((_ a, _ b) ∴ a + b)
numeri.compone((numerus a, numerus b) ∴ a + b)
```

Bare identifiers are not closure parameters. An inferred closure parameter must still
have an explicit type slot, written `_`, so the parser can recognize parameter
syntax without reinterpreting an arbitrary expression before `∴`.

Explicit callable signatures should use the existing result and error arrows, with a braced body:

```fab
users.filtrata((User user) → bivalens {
    redde non user.activus
})

texts.mappata((textus s) → numerus ⇥ ParseError {
    redde parse(s)
})
```

This keeps the meaning of `→` and `⇥` stable: they describe callable result and error channels, not the body boundary.

### Error Handling

Closures handle errors through the same callable type shape as named functions:

```fab
(textus) → numerus ⇥ ParseError
```

An expression closure may be accepted where the expected type is fallible if the body expression can produce that error channel:

```fab
texts.mappata(_ s ∴ parse(s))
```

When there is no expected fallible callable type, the closure body's error behavior should be inferred from the body and then checked normally against the call site.

Explicit fallible closures should prefer braced signatures over crowded expression forms:

```fab
(textus s) → numerus ⇥ ParseError {
    redde parse(s)
}
```

Do not introduce a separate closure-specific error syntax.

## Break Boundary

### In Scope

- Add `∴` as a symbolic alias or canonical spelling for `ergo`.
- Add compact closure expression syntax without the `clausura` keyword.
- Preserve `→` and `⇥` as callable result and error type markers.
- Require braces for multi-line closure bodies.
- Support typed and context-inferred closure parameters, with `_ name` as the minimum inferred-parameter spelling.
- Improve expected-type propagation so `lista<User>.filtrata(_ user ∴ ...)` can infer `user: User`.
- Update docs, examples, Faber pretty-printing, and tests.

### Out of Scope

- Changing function declaration syntax.
- Changing callable type syntax.
- Inventing a new error-channel operator.
- Removing `ergo` text spelling.
- Deciding broader keyword contextualization beyond syntax required here.
- Reworking stdlib method names or signatures except tests/examples that exercise closures.

## Compatibility Stance

The current `clausura ... : ...` syntax should remain accepted during the first implementation phase unless the design review explicitly chooses a hard break.

The preferred migration shape is:

```fab
clausura User user: non user.activus
```

to:

```fab
User user ∴ non user.activus
```

and inferred parameters migrate as:

```fab
clausura _ user: non user.activus
```

to:

```fab
_ user ∴ non user.activus
```

and:

```fab
clausura User user {
    redde non user.activus
}
```

to:

```fab
User user {
    redde non user.activus
}
```

If `clausura` remains as a compatibility spelling, generated Faber output should eventually prefer the new syntax after the migration period.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
|-------|------|------|------------|
| 0 | Design review | Confirm `∴`/`ergo`, single-statement tails, closure forms, and compatibility stance. | Plan approved or revised before implementation. |
| 1 | Grammar inventory | Inspect parser sites for `ergo`, closure parsing, and closure codegen. | Ledger records exact edit sites and ambiguity risks. |
| 2 | Ergo symbol | Lex and parse `∴` anywhere `ergo` is accepted today, with matching single-statement behavior. | Existing `ergo` tests pass; new `∴` tests cover valid symbolic tails. |
| 3 | Compact closure parser | Parse inferred, typed, multi-param, and signature-braced closure forms. | Parser tests prove the new forms and preserve old compatibility forms. |
| 4 | Contextual closure typing | Push expected callable signatures into closure checking, including method-call arguments. | `users.filtrata(_ user ∴ non user.activus)` typechecks as `lista<User>`. |
| 5 | Error-channel closures | Validate fallible closure signatures and expected fallible callable contexts. | Positive and negative tests cover `→ ... ⇥ ...` closure signatures. |
| 6 | Codegen and printer | Update target codegen as needed and make Faber output prefer the new syntax. | Rust/TS/Go/Faber closure tests pass for expression and block forms. |
| 7 | Docs and examples | Update EBNF, grammatica docs, stdlib examples, and migration notes. | Docs no longer teach `clausura ... : ...` as preferred inline syntax. |
| 8 | Validation | Run full repository checks. | `./scripta/ci` passes. |

## Review Questions

- Should `∴` be an exact alias for `ergo`, or should generated Faber prefer `∴` as canonical?
- Should old `clausura` syntax remain indefinitely, warn, or be removed after a migration window?
- Are unparenthesized typed closures with multiple parameters ever allowed, or should multi-param closures always require parentheses?
- Should braced closure bodies without explicit result type be allowed for typed single-parameter closures?
- Should `_ user { ... }` be allowed for inferred block closures, or should block closures require either an exact type or a parenthesized parameter list to avoid object/block ambiguity?
- Does `si cond ∴ redde x` become preferred over `si cond ergo redde x`, or is `∴` mainly for closure-heavy code?

## Validation Targets

Positive syntax and typing:

```fab
fixum _ inactive ← users.filtrata(_ user ∴ non user.activus)
fixum _ inactive2 ← users.filtrata(User user ∴ non user.activus)
fixum _ sum ← nums.compone((_ a, _ b) ∴ a + b)
fixum _ sum ← nums.compone((numerus a, numerus b) ∴ a + b)
```

Positive braced closure:

```fab
fixum _ inactive ← users.filtrata((User user) → bivalens {
    redde non user.activus
})
```

Positive fallible closure:

```fab
fixum _ parsed ← texts.mappata((textus s) → numerus ⇥ ParseError {
    redde parse(s)
})
```

Negative:

```fab
fixum _ inactive ← users.filtrata(_ user ∴ {
    redde non user.activus
})
```

This keeps the invariant that `∴` is a single-statement tail marker, not a block introducer.

---

*This plan records the current design direction only. Implementation should not begin until the review questions are resolved enough to prevent grammar churn.*
