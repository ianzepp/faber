# Radix Diagnostics Mode Completion Audit

## Requirements Checked

- `radix check --diagnostics <file>` selects diagnostics mode.
- `radix emit --diagnostics <file>` selects diagnostics mode.
- File-mode `faber check --diagnostics <file>` works if cheap to wire.
- Normal command behavior remains the default when `--diagnostics` is absent.
- Expanded records include severity, code, phase, message, file, byte span, source line, and help when available.
- `Diagnostic` carries explicit phase attribution.
- Focused tests cover parse and semantic diagnostics in diagnostics mode.

## Evidence

- `crates/radix/src/tool_test.rs` covers `check --diagnostics`, `emit --diagnostics`, parse diagnostics, semantic diagnostics, and emit diagnostics rendering.
- `cargo test -p radix tool::tests::` passed.
- `cargo test -p faber` passed.
- `./scripta/test` passed.
- Manual smoke commands for `radix check --diagnostics`, `radix emit --diagnostics`, and `faber check --diagnostics` produced expanded records and failed with status 1 for intentionally invalid inputs.

## Poker Face

- Self estimate: 95%
- Evaluator mode: self-contained independent pass. Subagents were not used because the user requested factory, not explicit sub-agent delegation, and the available subagent tool requires explicit delegation authorization.
- Evaluator estimate: 92%
- Overclaim: 3%
- Miss: 8%

## Verdict

CLEARED. CALIBRATION HELD.

The largest residual gap is that package-mode `faber check --diagnostics` is intentionally not implemented; the source plan only asked for `faber` wiring if cheap, and package-wide expanded reporting would require a separate output policy. The first deliverable is satisfied for the named single-file paths, with package diagnostics left as a documented follow-up.

