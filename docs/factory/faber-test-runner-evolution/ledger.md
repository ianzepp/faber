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

### Phase 2 Implementation
- TestArgs + invoke_cargo_test + cmd_test extended.
- Smokes: filter + --exact narrows (1 filtered out), --test-threads accepted, --nocapture emits the scribe line from inside a proba body.
- Gate: fmt/test/clippy clean.
- Commit: independent after Phase 1.

### Phase 3 Implementation
- --ignored / --include-ignored added with conflicts_with.
- Clap rejects the pair early with "cannot be used with".
- ignored fixture:
  * default → 2 ignored, exit 0
  * --ignored → executes falsum cases, 2 failed, nonzero
  * --include-ignored → 1 passed + 2 failed, nonzero
- Gate + commit independent.

### Final Matrix (post-Phase 3)
- All 4 fixtures + representative flag combos pass their expected exit/output counts.
- No target/faber/target anywhere.
- Help surface complete and accurate.
- Two independent commits after the Phase 1 baseline.

## Current Overall Status
- Phases 0–6 are now delivered in this repository, with phases 0–3 committed earlier and phases 4–6 implemented in the current sequence.
- `faber test` now has structured test metadata, source-level selection behavior, and docs that describe the live surface.
- Remaining work is Phase 7 release validation.
- The plan remains open until phase 7 lands and the plan status is updated truthfully.

---

## Phase 4: Structured Test Metadata

**Date**: 2026-05-21

### Implementation
- Added explicit HIR test metadata for `proba` functions instead of overloading `is_generator`.
- Preserved source test names, suite paths, and modifiers in `HirTestMetadata`.
- Lowering now records the metadata directly; Rust and Faber codegen consume it without reverse-engineering synthetic function state.

### Validation
- `cargo test --all`
- `cargo clippy --all-targets --all-features -- -D warnings`

## Phase 5: Faber-Specific Selection

**Date**: 2026-05-21

### Implementation
- Added Faber-level selector flags to `faber test`: `--name`, `--suite`, and `--tag`.
- Added generated Rust ignore reasons for deselected tests, including `solum` default selection and explicit selector AND semantics.
- Kept compilation honest: all tests are still generated and compiled even when execution is narrowed.
- Added fixture packages for:
  - `examples/exempla/proba/packages/solum/`
  - `examples/exempla/proba/packages/selectors/`
  - `examples/exempla/proba/packages/selection-failure/`

### Validation
- `cargo test --all`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run -p faber -- test --help`
- `cargo run -p faber -- test examples/exempla/proba/packages/passing`
- `cargo run -p faber -- test examples/exempla/proba/packages/ignored`
- `cargo run -p faber -- test examples/exempla/proba/packages/ignored --ignored`
- `cargo run -p faber -- test examples/exempla/proba/packages/ignored --include-ignored`
- `cargo run -p faber -- test examples/exempla/proba/packages/suite`
- `cargo run -p faber -- test examples/exempla/proba/packages/solum`
- `cargo run -p faber -- test examples/exempla/proba/packages/selectors --name "name match"`
- `cargo run -p faber -- test examples/exempla/proba/packages/selectors --suite "outer suite/inner suite"`
- `cargo run -p faber -- test examples/exempla/proba/packages/selectors --tag smoke`
- `cargo run -p faber -- test examples/exempla/proba/packages/selectors --name "combined match" --suite "outer suite/inner suite" --tag focus`
- `cargo run -p faber -- test examples/exempla/proba/packages/selection-failure --name "selected case"`

## Current Overall Status
- Phases 0–5 are now delivered in this repository, with phases 0–3 committed earlier and phases 4–5 implemented in the current slice.
- `faber test` now has structured test metadata and source-level selection behavior.
- Remaining work is Phase 6 documentation/reporting polish and Phase 7 release validation.
- The plan remains open until those later phases land and the plan status is updated truthfully.

---

## Phase 6: Documentation and Reporting

**Date**: 2026-05-21

### Implementation
- Added `docs/grammatica/test.md` to cover:
  - `proba` and `probandum`
  - test modifiers and current limits
  - `faber test` selector flags and ignored-test behavior
  - the reusable smoke fixtures under `examples/exempla/proba/packages/`
- Updated `docs/grammatica/manifest.md` with the Cargo-backed `faber test` layout and selector behavior.
- Updated `docs/grammatica/verba.md` so `solum` is described as active behavior instead of planned vocabulary.
- Updated `README.md` so `faber test` is presented as implemented and the test docs are discoverable from the main project entrypoint.

### Validation
- Readback inspection of the updated docs and README against the live `faber test --help` surface from phase 5.
- `git diff --check`
