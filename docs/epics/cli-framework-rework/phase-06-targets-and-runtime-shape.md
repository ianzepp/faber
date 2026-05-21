# Phase 06: Targets and Runtime Shape

## Status

Deferred out of the initial CLI framework rework.

Phases 1-5 proved the Rust CLI path: syntax, CLI IR, single-command codegen, subcommand dispatch, and package-local
module mounts. Target strategy and runtime extraction are broader follow-on work and should be handled as a separate
future effort.

## Current Decision

- Keep generated Rust CLI support self-contained.
- Do not add `norma::cli` in this epic.
- Do not create a separate public CLI runtime crate in this epic.
- Keep runnable CLI support Rust-only for now.
- Keep TypeScript, Go, and Faber runnable CLI codegen explicitly gated.
- Treat conformance tests for future targets as future target/runtime work.

## Rationale

The generated parser and dispatcher are still settling. Recent phases changed option grammar, subcommand dispatch,
module mounts, mount-local aliases, global merging, and package codegen. Extracting a runtime now would freeze an API
before the shape has earned that stability.

Runtime extraction should be reconsidered only when at least one of these becomes true:

- generated Rust CLI helper code becomes meaningfully duplicated or hard to maintain,
- a second runnable target needs shared behavior,
- packaging generated programs with helper code becomes cleaner than emitting helpers inline.

## Future Epic Scope

A future target/runtime epic should decide:

- whether CLI support belongs in `radix/crates/norma` as an internal `norma::cli` module,
- whether a separate public CLI runtime crate is justified,
- which non-Rust target should become the next runnable CLI backend,
- how target capabilities are reported by `radix targets`,
- which shared conformance fixtures every runnable CLI target must pass.

## Closeout Acceptance

- Current docs state that Rust is the only runnable CLI target.
- Non-Rust runnable CLI targets remain explicitly gated.
- Runtime extraction is documented as deferred, not silently forgotten.
