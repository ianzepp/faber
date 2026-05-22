# Phase 5C Delivery: Structured `cape` Handling Surface

## Interpreted Problem

Phase 5A introduced typed alternate exits with `→ Success ⇥ Error`, and Phase 5B lowers function-level `iace` into MIR `return_error`. The remaining local handling surface still carries older `tempta` / `cape` / `demum` try-catch shape, which makes recoverable failure look like ambient exceptions rather than lexical alternate-exit control flow.

Phase 5C should remove `tempta` from the canonical error-handling model and make `cape` attach directly to structured statements. The primary local boundary is `fac { ... } cape err { ... }`, with the same handler model available on supported control-flow statements and conditional arms.

## Normalized Spec

- Define `cape` as an optional handler attached to a structured statement or conditional arm.
- Keep `cape` ignored for normal flow, including fallthrough and normal `redde`.
- Run `cape` only for recoverable alternate-exit flow from the associated construct.
- Do not define `cape` as attaching to the nearest arbitrary `{}` block.
- Use `fac { ... } cape err { ... }` as the canonical one-shot local error boundary.
- Support `cape` on `dum` as statement-scoped loop handling.
- Support `cape` on `si`, `sin`, and `secus` arms as arm-scoped handling.
- Permit failable calls inside an active lexical `cape` boundary and route their alternate exits to that handler.
- Remove or fail hard on `tempta` in the canonical source surface.
- Keep `demum` cleanup/finally semantics deferred until resource cleanup rules are designed.
- Keep caller-side propagation syntax separate; this phase handles local consumption, not propagation.

## Attachment Contract

`cape` attaches by grammar production, not by nearest block token.

Valid attachment targets:

```text
fac block cape?
fac block cape? dum expr
dum expr body cape?
si expr arm (sin expr arm)* (secus else_arm)?
```

Where:

```text
arm      := body cape?
else_arm := body cape?
cape     := 'cape' IDENTIFIER block
body     := block | 'ergo' statement
```

A bare block remains only a lexical block:

```fab
{
    process()
} cape err {
    mone err
}
```

That form should be rejected. If a programmer wants a local boundary, they must write:

```fab
fac {
    process()
} cape err {
    mone err
}
```

## Flow Contract

For `fac`:

```fab
fac {
    process()
    save()
} cape err {
    mone err
}
```

- normal completion skips the handler,
- normal `redde` skips the handler and exits the enclosing function after its operand evaluates successfully,
- `iace` or a handled failable operation enters the handler,
- handler fallthrough rejoins after the `fac`,
- handler `redde`, `iace`, or `mori` exits according to its own terminator.
- failable calls inside the body continue normally on their success path and enter the handler on their alternate path.

The handler binding type is inferred from the alternate exits caught by the associated construct. Phase 5C should start conservatively: all caught alternate-exit values for one handler must have a single assignable error type. Full least-upper-bound or synthetic union inference can be deferred if it is not already available.

For `dum`:

```fab
dum ready {
    process()
} cape err {
    mone err
}
after()
```

- alternate exit from the loop condition or loop body enters the handler,
- handler fallthrough continues after the entire `dum`,
- the loop is not resumed after handler fallthrough.

Per-iteration recovery uses an inner `fac`:

```fab
dum ready {
    fac {
        process()
    } cape err {
        mone err
    }
}
```

For conditionals:

```fab
si ready {
    process()
} cape err {
    mone err
} sin waiting {
    pause()
} cape err {
    mone err
} secus {
    done()
} cape err {
    mone err
}
```

Each handler is scoped to the immediately preceding arm. The `si` handler covers the `si` condition and body, the `sin` handler covers that `sin` condition and body, and the `secus` handler covers only the `secus` body.

## MIR Contract

Phase 5C should lower local handling to explicit control-flow edges in MIR. It should not model `cape` as exceptions, stack unwinding, or dynamic search.

The expected lowering shape is:

```text
body_entry:
  ...
  branch/goto/return/return_error

error_handler:
  bind err
  ...
  goto after

after:
  ...
```

Open details for implementation:

- whether the first MIR representation uses a local handler stack in the lowerer or introduces an explicit temporary handler target in block state,
- whether handler binding is represented as a local initialized from the alternate-exit operand or as a dedicated MIR edge payload,
- whether failable expression calls need a dedicated MIR statement form immediately or can lower through existing call-plus-branch machinery.

The invariant is stronger than the shape: a handled `iace` or handled failable-call alternate exit inside a local boundary must not become the enclosing function's `return_error` unless the handler itself re-raises with `iace`.

## Repo-Aware Baseline

- `EBNF.md` already documents `dum ... cape` and `fac ... cape`.
- The parser already has `parse_dum_stmt` and `parse_fac_stmt` support for optional `cape`.
- The parser already accepts `cape` on `si` and `sin` arms before `secus` / chained `sin`.
- The parser does not currently accept `cape` after a `secus` body.
- `tempta` parsing still exists and should be removed or converted into a diagnostic in this phase.
- HIR lowering already has handler-related surfaces that must be audited before MIR handling is added.
- Phase 5B deliberately leaves `HirExprKind::Tempta` fail-closed for MIR.

## Stage Graph

1. Update `EBNF.md` and language docs to describe structured `cape` attachment.
2. Remove `tempta` from the canonical grammar or make it a hard diagnostic with a migration note to `fac { ... } cape`.
3. Keep `demum` out of the canonical grammar until cleanup semantics are designed.
4. Extend `secus` parsing to allow trailing arm-scoped `cape`.
5. Add parser tests for `fac`, `dum`, `si`, `sin`, and `secus` handler attachment.
6. Add negative parser tests proving bare `{}` cannot be followed by `cape`.
7. Audit AST/HIR handler carriers and remove `Tempta` dependence from local handling.
8. Typecheck handled bodies with a lexical local alternate-exit sink.
9. Infer the handler binding type from caught alternate exits and reject incompatible mixed error types.
10. Allow failable calls in handled bodies and route their alternate exits to the associated handler.
11. Lower handled local `iace` to the associated handler edge instead of the enclosing function's `return_error`.
12. Lower handled failable calls with explicit success continuation and handler error edge.
13. Lower handler fallthrough to the construct's post-handler join block.
14. Preserve `mori` as fatal and not catchable.

## Checkpoints

- `fac { iace ... } cape err { ... }` is accepted without requiring the enclosing function to declare `⇥`.
- failable calls inside `fac { ... } cape err { ... }` are accepted and locally handled.
- The same `iace` without a local handler and without a function `⇥` remains a compiler error.
- The same failable call outside a local handler and without propagation syntax remains a compiler error.
- `dum ... cape` handles alternate exits from the loop condition/body and exits the loop after handler fallthrough.
- `si` / `sin` / `secus` arm handlers parse and scope independently.
- `secus { ... } cape err { ... }` parses.
- Bare `{ ... } cape err { ... }` is rejected.
- `tempta` no longer parses as the canonical source construct or fails with a direct migration diagnostic.
- `demum` does not acquire accidental semantics.
- MIR dump shows local handler edges or equivalent explicit control flow for supported handled `iace` fixtures.

## Fixture Candidates

One-shot local handler:

```fab
functio handled() → numerus {
    fac {
        iace "bad"
    } cape err {
        redde 0
    }
    redde 1
}
```

Handled failable call:

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b = 0 ergo iace "zero"
    redde a / b
}

functio safe_divide(numerus a, numerus b) → numerus {
    fac {
        redde divide(a, b)
    } cape err {
        redde 0
    }
}
```

Loop-level handler:

```fab
functio work(bivalens ready) → vacuum {
    dum ready {
        process()
    } cape err {
        mone err
    }
}
```

Arm-level handlers:

```fab
functio choose(bivalens ready, bivalens waiting) → vacuum {
    si ready {
        process()
    } cape err {
        mone err
    } sin waiting {
        pause()
    } cape err {
        mone err
    } secus {
        done()
    } cape err {
        mone err
    }
}
```

Rejected bare block:

```fab
functio invalid() → vacuum {
    {
        process()
    } cape err {
        mone err
    }
}
```

## Out Of Scope

- `demum` / finally cleanup semantics.
- Dynamic exception handling or stack search.
- Retrying loops after handler fallthrough.
- Broad failable-call propagation syntax outside lexical handlers.
- Automatic union synthesis for mixed handler error types, unless the existing type system already provides it cheaply.
- Rust backend support for the new handler surface.
- WASM or native output.

## Validation

- Focused parser tests for every supported attachment target.
- Focused parser negative tests for bare-block `cape` and removed `tempta`.
- Focused semantic tests for local handler consumption of `iace`.
- Focused semantic tests for local handler consumption of failable calls.
- Focused MIR tests for local handler control flow.
- Regression tests proving function-level `iace` from Phase 5B still lowers to `return_error`.
- `cargo test -p radix cape`.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 5C complete.

## Completion Gate

Phase 5C is complete when `fac`, `dum`, and conditional arms provide the canonical lexical `cape` handling surface, `tempta` is removed or rejected with a migration diagnostic, local handled `iace` and handled failable-call alternate exits no longer escape through the enclosing function's `return_error`, and existing alternate-exit MIR behavior from Phase 5B remains intact.
