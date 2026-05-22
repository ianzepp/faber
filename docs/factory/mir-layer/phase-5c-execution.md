# Phase 5C Execution Spec

## Target

Implement `docs/factory/mir-layer/phase-5c-delivery.md`: structured lexical `cape` handling for `fac`, `dum`, and conditional arms, with `tempta` rejected as a legacy construct.

## Repo Baseline

- `parser/stmt.rs` parses `cape` on `si`, `sin`, `dum`, and `fac`.
- `parser/stmt.rs` does not parse `cape` after `secus`.
- `parser/stmt.rs` still parses `tempta`.
- `hir/lower/stmt.rs` lowers `fac ... cape` through `HirExprKind::Tempta` and ignores `si` / `dum` catches.
- Typechecking uses one `current_error` field for both function-level `⇥` and old local handler tolerance, while failable calls are still rejected everywhere.
- MIR lowers unhandled `iace` to the function `return_error`; local handler edges and failable-call error edges do not exist yet.

## Implementation Shape

- Add explicit HIR cape metadata for the handler binding and body.
- Represent structured local handling as a HIR handled expression, not as `Tempta`.
- Carry `si` arm catches in HIR so a `si` / `sin` catch covers both the arm condition and body.
- Wrap `secus` bodies with a handled expression when `secus ... cape` is present.
- Distinguish function-level and local error sinks in typechecking.
- Permit failable calls only under a local sink and unify their error type into the handler binding type.
- Lower handled `iace` by assigning the payload to the handler binding local and jumping to the handler block.
- Lower handled failable direct calls with an explicit MIR success/error terminator.
- Keep `mori` fatal and not catchable.

## Validation Gates

- Parser tests for `fac`, `dum`, `si`, `sin`, and `secus` attachment.
- Parser negative tests for bare-block `cape` and `tempta` migration diagnostics.
- Semantic tests for handled `iace`, handled failable direct calls, and unhandled regressions.
- MIR tests for handled `iace`, handled failable direct calls, loop handler fallthrough, and Phase 5B unhandled `return_error`.
- `cargo test -p radix cape`
- `cargo test -p radix mir`
- `cargo test -p radix`
- `./scripta/ci`
