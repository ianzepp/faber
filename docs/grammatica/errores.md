# Error Handling

Faber separates recoverable alternate exits from fatal failure.

- `iace expr` exits through a recoverable alternate path.
- `mori expr` is fatal and is not caught by local handlers.
- `cape err { ... }` handles recoverable alternate exits for the structured statement or conditional arm it is attached to.

Functions declare recoverable alternate exits with `⇥`:

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b ≡ 0 ergo iace "division by zero"
    redde a / b
}
```

Without a local handler or a declared function-level `⇥`, `iace` is a compile-time error.

## Structured Cape

`cape` attaches by grammar production, not to the nearest arbitrary block token.

The canonical one-shot local boundary is `fac { ... } cape err { ... }`:

```fab
functio safe_divide(numerus a, numerus b) → numerus {
    fac {
        redde divide(a, b)
    } cape err {
        redde 0
    }
}
```

Normal fallthrough and normal `redde` skip the handler. A recoverable `iace` or handled failable call inside the `fac` body enters the handler. If the handler falls through, execution rejoins after the `fac`.

Bare blocks do not accept `cape`:

```fab
{
    iace "bad"
} cape err {
    mone err
}
```

Use `fac` when a lexical local boundary is intended.

## Loop And Arm Handlers

`dum` can carry a statement-scoped handler:

```fab
dum ready {
    process()
} cape err {
    mone err
}
```

Handler fallthrough exits the loop; it does not retry the failed iteration. Put an inner `fac` inside the loop for per-iteration recovery.

Conditional arms can carry independent handlers:

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

The `si` and `sin` handlers cover that arm's condition and body. The `secus` handler covers only the `secus` body.

## Legacy Surface

`tempta` is no longer canonical and is rejected with a migration diagnostic. Use `fac { ... } cape err { ... }` for local recoverable-error handling.

`demum` cleanup/finally semantics are deferred until resource cleanup rules are designed.
