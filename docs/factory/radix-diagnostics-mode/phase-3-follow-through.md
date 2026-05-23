# Phase 3 Follow-Through: File-Mode `faber check --diagnostics`

## Interpreted Problem

The diagnostics plan asks for `faber check --diagnostics <file>` if wiring through `faber` is cheap. `faber` already reuses `radix::tool::CheckArgs` and delegates non-package inputs to `radix::tool::cmd_check`, so file-mode support is a command wiring follow-through rather than a separate package diagnostics design.

## Normalized Spec

Support `faber check --diagnostics <file>` for non-package inputs by forwarding the parsed diagnostics flag into the shared `CheckCommand`.

Out of scope:

- Package-mode expanded diagnostics.
- Package-wide grouping or batch summaries.
- Reworking `crates/faber/src/package.rs` reporting.

## Evidence

Implementation uses the shared `CheckArgs` flag and `DiagnosticMode` mapping in `crates/faber/src/main.rs`.

Manual smoke:

```text
cargo run -q -p faber -- check --diagnostics <bad-file>
```

produced an expanded `SEM001` analysis record with phase, file, span, source, and help.

