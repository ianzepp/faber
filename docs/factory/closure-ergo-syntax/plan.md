# Closure Ergo Syntax Factory Plan

**Status**: complete
**Created**: 2026-05-22
**Completed**: 2026-05-23
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/closure-ergo-syntax/`
**Mode**: language surface redesign / parser and type inference cleanup
**Commit Policy**: Commit after each completed phase and validation gate pass

## Outcome

This factory is done. The implemented language surface is:

- `∴` is the canonical symbolic spelling for compact single-statement tails, while `ergo` remains accepted as the Latin alias.
- Compact closure expressions no longer require `clausura`; examples use `_ item ∴ expr`, `Type item ∴ expr`, or parenthesized multi-parameter forms.
- Multi-statement compact closures use `∴ fac { ... }`, with optional `cape`; postfix `fac ... dum` remains rejected in closure bodies.
- Legacy `clausura ... : ...` remains accepted as compatibility syntax, but generated Faber output prefers compact closure syntax.
- Closure syntax parses explicit `⇥` error-channel annotations. Full closure error-channel semantics remain a separate follow-up; this factory deliberately stopped at syntax, lowering compatibility, contextual typing, docs, examples, and validation.

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

## Implemented Reality

The grammar now describes closure expressions as compact or legacy forms:

```ebnf
clausuraExpr   := compactClausuraExpr | legacyClausuraExpr
compactClausuraExpr := clausuraSignature ergoToken (expression | closureFacBlock)
clausuraSignature := (clausuraParam | '(' clausuraParams? ')') ('→' typeAnnotation alternateExitClause?)?
closureFacBlock := 'fac' blockStmt catchClause?
legacyClausuraExpr := 'clausura' clausuraParams? ('→' typeAnnotation)? (':' expression | blockStmt)
clausuraParams := clausuraParam (',' clausuraParam)*
clausuraParam  := typeAnnotation IDENTIFIER
```

The grammar allows `_` as a type annotation generally, so compact inferred
parameters use `_ user ∴ ...`. Simple receiver-method cases such as list `map`
and `filtrata` use expected callable context for `_` closure parameters.

The stdlib list filter signature is:

```fab
functio filtrata((T) → bivalens pred) → lista<T>
```

That target shape is the right semantic model: a predicate closure over `T` returns `bivalens`.

## Implemented Design

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

Short closures do not need the `clausura` keyword.

Inferred parameter type:

```fab
users.filtrata(_ user ∴ non user.activus)
```

Explicit parameter type:

```fab
users.filtrata(User user ∴ non user.activus)
```

Multiple parameters use parentheses to avoid ambiguity:

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

## Delivered Boundary

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
- Completing closure error-channel semantics beyond parsed `⇥` syntax.
- Removing `ergo` text spelling.
- Deciding broader keyword contextualization beyond syntax required here.
- Reworking stdlib method names or signatures except tests/examples that exercise closures.

## Compatibility Stance

The current `clausura ... : ...` syntax remains accepted as compatibility
syntax. The Faber printer prefers the compact closure form.

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

Generated Faber output prefers the new syntax while preserving input
compatibility for `clausura`.

## Stage Graph

| Phase | Name | Status | Result |
|-------|------|--------|--------|
| 0 | Design review | Done | `∴` is canonical, `ergo` remains accepted, compact closures use type-first parameters, and legacy `clausura` remains compatible. |
| 1 | Grammar inventory | Done | Ledger records lexer, parser, AST, lowering, typecheck, printer, docs, and example edit sites. |
| 2 | Ergo symbol | Done | `∴` lexes to `Ergo` and works anywhere `ergo` is accepted. |
| 3 | Compact closure parser | Done | Parser tests cover inferred, typed, multi-param, explicit-signature, expression-body, and `∴ fac { ... }` forms. |
| 4 | Contextual closure typing audit | Done | Method-call closure arguments keep expected callable signatures for `_` parameters. |
| 5 | Codegen and printer | Done | Faber output prefers compact closure syntax. |
| 6 | Docs and examples | Done | `EBNF.md`, explain docs, and examples teach compact syntax as canonical. |
| 7 | Error-channel closure follow-up | Done for this factory | `⇥` syntax is parsed and recorded; full error-channel typing remains a separate semantic follow-up. |
| 8 | Validation | Done | Validation commands are recorded in the ledger. |

## Resolved Review Questions

- `∴` is canonical in generated Faber output; `ergo` is accepted as an alias.
- Old `clausura` syntax remains accepted as compatibility syntax.
- Multi-parameter compact closures require parentheses.
- `si cond ∴ redde x` is preferred in canonical examples, while `ergo` remains valid.

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

*This plan is retained as the completed factory record. Future work on full closure error-channel typing should start from a new semantic follow-up plan, not by reopening this syntax factory.*
