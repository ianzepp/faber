# Faber Test Runner Evolution Ledger

**Plan**: docs/factory/faber-test-runner-evolution/plan.md
**Started**: 2026-05-21

## Phase 0: Preflight and Baseline Capture

**Date**: 2026-05-21 (this session)

### Git Status (start of session)
```
On branch main
Your branch is ahead of 'origin/main' by 40 commits.

nothing to commit, working tree clean
```

### Current `faber test` Behavior (stub)
```
$ cargo run -p faber -- test --help
Run package tests (planned)

Usage: faber test [PATH]

Arguments:
  [PATH]  Package path to test [default: .]

Options:
  -h, --help  Print help
```

When run on a path:
```
error: `faber test` is not implemented yet; use `faber check` on <path>
```
(exit 1)

### Existing Proba Lowering Evidence
- `proba "name" { adfirma ... }` parses via `StmtKind::Proba` in radix parser.
- Lowers in `crates/radix/src/hir/lower/mod.rs:lower_proba_item` to `HirItem::Function` with synthetic `def_id >= 1_000_000`.
- `proba omitte ...` and `proba futurum ...` set `is_generator: true` (repurposed as ignore flag).
- Codegen in `crates/radix/src/codegen/rust/decl.rs:generate_function_with_cli_args_type`:
  - emits `#[test]`
  - emits `#[ignore]` if `is_generator` (for omitte/futurum)
  - names the fn `proba_<def_id>`
- `probandum` nests and inherits `praepara omnia`/`postpara omnia` setups.
- Existing examples: `examples/exempla/proba/proba.fab`, `modificatores.fab`
- Compiler tests in `driver/mod_test.rs`, `parser/mod_test.rs`, `lexer/scan_test.rs` cover proba lowering (post glyph clean-break use glyphs).

### Fixture Status (pre-creation)
- `examples/exempla/proba/packages/` did not exist.
- Per plan, fixtures created as part of Phase 0 before any runner changes (committed artifacts for repeatable smoke tests).

### Directory Contract Verified
- BuildLayout in `crates/faber/src/package.rs` already implements:
  - `package_root/target/faber/` for generated crate
  - `package_root/target/` as cargo --target-dir
- `invoke_cargo_build` uses exactly `--manifest-path .../target/faber/Cargo.toml --target-dir .../target`
- No `target/faber/target` is created by current build/run paths.

### Next
- Create the four fixture packages (passing/failing/ignored/suite).
- Update todo, then implement Phase 1: replace cmd_test stub with real Cargo-test-backed logic.
- Only Phase 1 for first slice; later phases in subsequent work.

### Fixture Creation Complete
- Created `examples/exempla/proba/packages/{passing,failing,ignored,suite}/` with `faber.toml` (proba-* names) and `src/main.fab`.
- Used canonical post-glyph syntax: `←` for bindings, `≡`, `adfirma`, `proba omitte "reason" "name"`, `proba futurum`, `probandum`, `praepara omnia`, `verum`/`falsum`.
- All four pass `faber check` (verified).
- Fixtures are small smoke cases per contract; no target/ contents committed.

## Phase 1 Implementation and Validation

**Date**: 2026-05-21

### Changes Made
- Added `invoke_cargo_test(layout)` in `crates/faber/src/package.rs` (modeled on `invoke_cargo_build`, returns `ExitStatus` directly so test failures become command failures with correct code).
- Replaced stub `cmd_test` in `crates/faber/src/main.rs` with full package compile + emit + cargo-test flow (reuses all existing helpers; always treats input as package per `cmd_run` precedent).
- Updated CLI help text from "(planned)" to "Run package tests via the generated Rust test harness (Cargo-backed)".
- **Root-cause fix**: Added empty `[workspace]` table to generated `Cargo.toml` (in both `generate_cargo_toml` and fallback path) so that in-repo packages (under examples/) do not trigger "believes it's in a workspace" errors from the parent `Cargo.toml`. This was required for the committed fixtures to work and also makes `faber build` on in-tree packages functional. (Build evolution had not yet exercised in-repo package paths.)

### Smoke Results (all match Phase-One Completion Metrics)
- `faber test --help`: now describes implemented command.
- `faber test examples/.../passing`: exit 0, "2 passed; 0 failed; 0 ignored", two `proba_*` tests.
- `faber test .../failing`: nonzero exit (101 from cargo test), "FAILED", panic from assertion in lowered test (not Faber error).
- `faber test .../ignored`: exit 0, "1 passed; ... 2 ignored", the omitte/futurum show as ignored via #[ignore].
- `faber test .../suite`: exit 0, "2 passed", nested probandum cases execute; praepara omnia setup bodies (incl. their adfirma) run inside tests.
- Cargo output visible with counts, names, timing.
- Exit status exactly matches cargo test status (forwarded).
- No `.../target/faber/target` created in any run (layout contract honored; --target-dir sibling used).
- All fixtures pass `faber check` and produce runnable tests.
- Default path (.) uses package discovery (same as build).

### Layout Proof
- target/ contents under fixtures: only `debug/` + `faber/` siblings; cleaned before commit.
- Generated crate includes `[workspace]` + comment.

### Gate Status (pre-full)
- Compiles cleanly (`cargo build -p faber`).
- `faber test` on all four fixtures behaves per spec.

### Open for This Slice
- Phase 2+ (flags, --ignored, metadata, docs) deferred.
- No new unit tests added to package_test.rs in Phase 1 (user-visible + smoke per plan priority).
- The `[workspace]` addition is a compatibility fix surfaced by this work.

## Status
- Phase 0 + Phase 1 complete.
- First implementation slice done. Later phases will be separate commits.
- `plan.md` status left as "planned" (per "when the full plan is actually complete").
- Ready to clean targets, run full fmt/test/clippy gate, commit.

---

## Phase 2 + 3 Continuation (document update received)

**Date**: 2026-05-21 (continuation session)

User notified that `plan.md` was updated with concrete specs for:
- Phase 2: Test Command Ergonomics (positional `[filter]`, `--exact`, `--nocapture`, `--test-threads <n>`)
- Phase 3: Ignored and Future Tests (`--ignored`, `--include-ignored`, mutual exclusion, docs for `omitte`/`futurum`)

Plan rules observed:
- Phases committed independently.
- Filter before `--`, harness flags after `--`.
- No raw trailing pass-through (`-- <raw>`) in Phase 2.
- Clap `conflicts_with` for ignored flags in Phase 3.
- `--nocapture` proof via test-body stdout (scribe inside a proba) or dedicated behavior.

Work will proceed Phase 2 (full gate + commit) then Phase 3 (full gate + commit), then final ledger + smoke matrix.

No changes to HIR/metadata (Phase 4+) or docs (Phase 6) in this slice.
