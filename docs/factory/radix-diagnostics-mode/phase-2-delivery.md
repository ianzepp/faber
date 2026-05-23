# Phase 2 Delivery: `radix emit --diagnostics`

## Interpreted Problem

`radix emit` already compiles through the public compiler API and receives normalized `Diagnostic` values, but the command prints only terse severity/message lines. The diagnostics plan includes `radix emit --diagnostics <file>` as part of the first deliverable, so the existing diagnostics mode should be carried through this command without changing normal emit output.

## Normalized Spec

Implement `radix emit --diagnostics <file>` for single-file inputs.

Required behavior:

- Add the existing diagnostics mode flag to `emit`.
- Preserve current `radix emit <file>` behavior when the flag is absent.
- In diagnostics mode, print normalized diagnostics through the expanded renderer.
- Keep successful source emission on stdout unchanged.
- Keep package routing policy unchanged.

Out of scope:

- JSON diagnostics.
- Package-wide diagnostics.
- Build command diagnostics mode.
- Backend diagnostic wording changes.

## Repo-Aware Baseline

Relevant current files:

- `crates/radix/src/tool.rs` defines `EmitArgs`, `EmitCommand`, and `cmd_emit`.
- `crates/radix/src/bin/radix.rs` maps parsed `EmitArgs` into `EmitCommand`.
- `crates/faber/src/main.rs` delegates file-mode `faber emit` to `radix::tool::cmd_emit`.
- `crates/radix/src/diagnostics/render.rs` now has the expanded renderer from Phase 1.

## Stage Graph

1. Add `--diagnostics` to `EmitArgs`.
2. Add `diagnostic_mode` to `EmitCommand` construction sites.
3. Switch only diagnostics-mode diagnostic printing in `cmd_emit` to expanded records.
4. Add focused CLI parsing and helper-output tests.

## Epic Candidates And Scopable Issues

- Command surface: `EmitArgs` flag and `EmitCommand` mode.
- Emit reporting: branch diagnostic printing without touching normal stdout.
- Tests: flag parsing plus expanded semantic diagnostic output from `compile_cli_source`.

## Checkpoints

Phase checkpoint passes when:

- `radix emit --diagnostics <file>` parses.
- Normal `radix emit <file>` command construction is unchanged except for explicit normal mode.
- Expanded emit diagnostics include phase/code/span/source/help for semantic failures.
- Focused radix tool tests pass.

## Companion Skill Plan

- `factory`: supervise phase boundary, verification, poker-face gate, and commit.
- `delivery`: this saved artifact is the single-phase delivery plan.

## Gate Plan

Run:

- `cargo test -p radix tool::tests::`
- A direct CLI smoke command for `radix emit --diagnostics`.

## Open Questions

None blocking.

