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
- multi-statement closure bodies use an explicit `fac { ... }` block,
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

Once a closure body needs multiple statements, it must use an explicit `fac` block:

```fab
User user ∴ fac {
    nota user.nomen
    redde non user.activus
}
```

Formatters should prefer keeping short `∴` tails on one line. If the tail is complex enough to read poorly, they should convert or preserve a `fac` block form.

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

Block closures use `∴ fac { ... }` rather than a bare braced body:

```fab
users.filtrata(_ user ∴ fac {
    nota user.nomen
    redde non user.activus
})

users.filtrata(User user ∴ fac {
    nota user.nomen
    redde non user.activus
})

numeri.compone((_ a, _ b) ∴ fac {
    redde a + b
})

users.filtrata(_ user → bivalens ∴ fac {
    redde parseFlag(user.nomen)
} cape err {
    redde falsum
})
```

Closure-body `fac` may use `cape` for local recovery, matching the existing `fac { ... } cape err { ... }` statement shape. Closure-body `fac ... dum` is rejected for now; do-while closure bodies are too surprising to admit as part of the compact syntax pass.

Bare identifiers are not closure parameters. An inferred closure parameter must still
have an explicit type slot, written `_`, so the parser can recognize parameter
syntax without reinterpreting an arbitrary expression before `∴`.

Explicit closure signatures are part of this redesign. They reuse the existing result and error-channel arrows, but they are a closure-expression form rather than the existing callable type syntax because they include parameter names.

```fab
users.filtrata(User user → bivalens ∴ fac {
    redde non user.activus
})

texts.mappata(textus s → numerus ⇥ ParseError ∴ fac {
    redde parse(s)
})
```

This keeps the meaning of `→` and `⇥` stable: they describe callable result and error channels, not the body boundary.

### Error Handling Follow-Up

Closure syntax should leave room for closures to handle errors through the same callable type shape as named functions:

```fab
(textus) → numerus ⇥ ParseError
```

Full closure error-channel support is a late implementation phase. The initial syntax work should not require adding closure `⇥` support beyond parsing enough structure to avoid grammar churn.

Eventually, an expression closure may be accepted where the expected type is fallible if the body expression can produce that error channel:

```fab
texts.mappata(_ s ∴ parse(s))
```

When there is no expected fallible callable type, the closure body's error behavior should be inferred from the body and then checked normally against the call site.

Explicit fallible closures should prefer `fac` block signatures over crowded expression forms:

```fab
textus s → numerus ⇥ ParseError ∴ fac {
    redde parse(s)
}
```

Do not introduce a separate closure-specific error syntax.

## Break Boundary

### In Scope

- Add `∴` as a symbolic alias or canonical spelling for `ergo`.
- Add compact closure expression syntax without the `clausura` keyword.
- Preserve `→` and `⇥` as callable result and error type markers.
- Add explicit closure signature syntax with parameter names before `∴`.
- Require `∴ fac { ... }` for multi-statement closure bodies.
- Support typed and context-inferred closure parameters, with `_ name` as the minimum inferred-parameter spelling.
- Audit expected-type propagation and fill gaps so contextual `_` closure parameters continue to work in method-call and callable-argument positions.
- Update docs, examples, Faber pretty-printing, and tests.

### Out of Scope

- Changing function declaration syntax.
- Changing callable type syntax.
- Inventing a new error-channel operator.
- Completing closure error-channel semantics before the syntax phases are stable.
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
User user ∴ fac {
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
| 3 | Compact closure parser | Parse inferred, typed, multi-param, explicit-signature, and `∴ fac { ... }` block closure forms. | Parser tests prove the new forms and preserve old compatibility forms. |
| 4 | Contextual closure typing audit | Verify expected callable signatures still flow into closure checking, especially method-call arguments. | Existing `_` closure inference remains green; any discovered gaps are fixed or recorded. |
| 5 | Codegen and printer | Update target codegen as needed and make Faber output prefer the new syntax. | Rust/TS/Go/Faber closure tests pass for expression and `fac` block forms. |
| 6 | Docs and examples | Update EBNF, explain docs, stdlib examples, and migration notes. | Docs no longer teach `clausura ... : ...` as preferred inline syntax. |
| 7 | Error-channel closure follow-up | Validate fallible closure signatures and expected fallible callable contexts after syntax is stable. | Positive and negative tests cover `→ ... ⇥ ...` closure signatures, or a follow-up plan records remaining semantic work. |
| 8 | Validation | Run full repository checks. | `./scripta/ci` passes. |

## Review Questions

- Should `∴` be an exact alias for `ergo`, or should generated Faber prefer `∴` as canonical?
- Should old `clausura` syntax remain indefinitely, warn, or be removed after a migration window?
- Are unparenthesized typed closures with multiple parameters ever allowed, or should multi-param closures always require parentheses?
- Does `si cond ∴ redde x` become preferred over `si cond ergo redde x`, or is `∴` mainly for closure-heavy code?

## Validation Targets

Positive syntax and typing:

```fab
fixum _ inactive ← users.filtrata(_ user ∴ non user.activus)
fixum _ inactive2 ← users.filtrata(User user ∴ non user.activus)
fixum _ sum ← nums.compone((_ a, _ b) ∴ a + b)
fixum _ sum ← nums.compone((numerus a, numerus b) ∴ a + b)
```

Positive block closure:

```fab
fixum _ inactive ← users.filtrata(User user ∴ fac {
    redde non user.activus
})
```

Positive fallible closure:

```fab
fixum _ parsed ← texts.mappata(textus s → numerus ⇥ ParseError ∴ fac {
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

```fab
fixum _ inactive ← users.filtrata(_ user ∴ fac {
    redde non user.activus
} dum user.activus)
```

This keeps `fac ... dum` out of closure bodies unless a later design review explicitly admits do-while closure bodies.

---

*This plan records the current design direction only. Implementation should not begin until the review questions are resolved enough to prevent grammar churn.*
