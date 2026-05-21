# Faber Test Runner Evolution Factory Plan

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-test-runner-evolution/`
**Depends On**: Faber build tool evolution, generated Rust crate layout, `proba`/`probandum` lowering
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber already has language-level test constructs:

- `proba "case" { ... }`
- `probandum "suite" { ... }`
- suite setup/teardown via `praepara`, `praeparabit`, `postpara`, and `postparabit`
- test modifiers such as `omitte`, `futurum`, `solum`, `tag`, `temporis`, `metior`, `repete`, `fragilis`, `requirit`, and `solum_in`

The Rust backend already lowers `proba` cases into Rust `#[test]` functions. `omitte` and `futurum` currently lower to ignored Rust tests.

The missing piece is the package-tool command: `faber test` is still a stub. The first useful version should be minimal and Cargo-backed, while the design leaves clear extension points for Faber-specific test semantics over later phases.

## Non-Negotiable Directory Contract

`faber test` must reuse the build-tool layout:

```text
<package>/
├── faber.toml
├── src/
│   └── main.fab
└── target/
    ├── faber/
    │   ├── Cargo.toml
    │   └── src/
    │       └── main.rs
    └── debug/
        ├── deps/
        └── incremental/
```

`target/faber/` is the generated Rust crate. Cargo test artifacts belong under package-level `target/debug/`, not under `target/faber/target/`.

The minimal command should be equivalent to:

```bash
cargo test \
  --manifest-path target/faber/Cargo.toml \
  --target-dir target
```

Do not create or rely on `target/faber/target/...`.

## Normalized Spec

### Current Baseline

- `faber build` emits a generated Rust crate at `target/faber/`.
- `faber build` invokes Cargo with `--target-dir target`.
- `faber run` builds and executes Rust package binaries.
- `faber test` currently exits with a not-implemented message.
- Radix parses and lowers `proba` and `probandum`.
- Rust codegen emits `#[test]` for synthetic `proba` functions.
- `omitte` and `futurum` currently lower to Rust `#[ignore]`.
- Some richer test modifiers are parsed but are not yet fully honored by the Rust test harness.

### Phase-One Target Behavior

`faber test [path]` should:

- discover the package root and manifest,
- compile the package to generated Rust,
- write the generated Rust crate under `target/faber/`,
- invoke `cargo test --manifest-path target/faber/Cargo.toml --target-dir target`,
- forward Cargo test output to the user,
- exit with Cargo test's status code,
- preserve the existing sibling layout.

This is intentionally not a custom Faber test runner yet.

### Later Target Behavior

Later phases should progressively expose Faber-specific test behavior:

- filter by test name or suite,
- pass through common Rust harness flags when they make sense,
- run ignored `omitte` / `futurum` cases when requested,
- honor `solum` as "run only this test" in Faber terms,
- honor tags and environment requirements,
- report Faber source names and locations rather than only generated Rust test names,
- document test syntax and command behavior together.

## Repo-Aware Baseline

Relevant files:

- `crates/faber/src/main.rs`
- `crates/faber/src/package.rs`
- `crates/faber/src/package_test.rs`
- `crates/radix/src/parser/decl.rs`
- `crates/radix/src/hir/lower/mod.rs`
- `crates/radix/src/codegen/rust/decl.rs`
- `crates/radix/src/driver/mod_test.rs`
- `docs/grammatica/manifest.md`
- `docs/grammatica/verba.md`
- `examples/exempla/proba/proba.fab`
- `examples/exempla/proba/modificatores.fab`

Current implementation details to preserve:

- `proba` lowers to synthetic Rust test functions.
- `probandum` lowers nested cases into synthetic test functions.
- suite-level `praepara omnia` and `postpara omnia` are inherited into nested tests.
- ignored test lowering currently uses `HirFunction::is_generator` as a test metadata carrier.

That metadata carrier is a local implementation detail and should not be expanded casually. A later phase should introduce explicit HIR test metadata if richer runner behavior needs it.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Preflight and baseline capture | Record current `faber test` stub behavior and existing proba/probandum lowering. | Ledger created; no behavior changed. |
| 1 | Minimal Cargo-backed `faber test` | Reuse generated crate emission and invoke Cargo test with package `target/` as target dir. | `faber test <pkg>` runs Rust-lowered Faber tests. |
| 2 | Test command ergonomics | Add filter/pass-through flags that map cleanly to Cargo's harness. | Common workflows work without invoking Cargo manually. |
| 3 | Ignored and future tests | Expose ignored-test execution for `omitte` and `futurum` cases. | Users can list/run ignored tests deliberately. |
| 4 | Faber test metadata model | Stop overloading generic HIR fields for test metadata; preserve source-level test names and modifiers. | Rust codegen and Faber tooling can inspect structured test metadata. |
| 5 | Faber-specific selection | Honor `solum`, `tag`, `requirit`, and `solum_in` at the Faber tool layer. | Faber selection semantics work before Cargo is invoked or via generated harness config. |
| 6 | Reporting and docs | Document test syntax, command behavior, and current limits; improve output where feasible. | README/grammar/docs match live command behavior. |
| 7 | Validation and release readiness | Run full Rust validation and smoke tests for passing/failing/ignored cases. | fmt/test/clippy/build and Faber test smoke pass. |

## Phase Details

### Phase 0: Preflight and Baseline Capture

Steps:

- Inspect `git status --short`.
- Capture `faber test --help` and `faber test <sample>` behavior before changes.
- Capture existing Radix evidence that `proba` lowers to Rust `#[test]`.
- Create a ledger in this artifact directory if implementation spans multiple commits.
- Pick or create a package fixture containing:
  - one passing `proba`,
  - one failing `proba`,
  - one `proba omitte`,
  - optionally one nested `probandum`.

Checkpoint:

- Baseline is recorded truthfully.
- No source behavior changes.

### Phase 1: Minimal Cargo-Backed `faber test`

Steps:

- Replace the `faber test` stub with a real command.
- Reuse the package compile and generated crate writer used by `faber build`.
- Add a Cargo test adapter that invokes:

```bash
cargo test --manifest-path <package-root>/target/faber/Cargo.toml --target-dir <package-root>/target
```

- Forward Cargo's stdout/stderr.
- Exit with Cargo's status code.
- Keep CLI shape minimal:

```bash
faber test [path]
```

- Default `path` to `.`.
- Do not implement Faber-specific filtering in this phase.

Checkpoint:

- A package containing passing `proba` tests passes with `faber test`.
- A package containing a failing `proba` exits nonzero.
- `proba omitte` / `proba futurum` are ignored by default through Rust `#[ignore]`.
- No package test creates `target/faber/target`.

### Phase 2: Test Command Ergonomics

Steps:

- Add a narrow set of Cargo-compatible flags:
  - optional test name filter,
  - `--exact`,
  - `--nocapture`,
  - possibly `--test-threads <n>`.
- Decide whether trailing args after `--` are passed directly to the Rust test harness.
- Keep the Faber CLI documented as a wrapper over generated Rust tests for now.

Checkpoint:

- `faber test <path> <filter>` works.
- `faber test <path> -- --nocapture` or the chosen explicit flag shape works.
- Unsupported harness flags produce clear errors or are deliberately passed through.

### Phase 3: Ignored and Future Tests

Steps:

- Add command support for ignored tests:
  - likely `faber test --ignored`,
  - possibly `faber test --include-ignored`.
- Document current mapping:
  - `omitte` -> ignored,
  - `futurum` -> ignored until a better todo/expected-failure model exists.
- Decide whether `futurum` should remain ignored or become an expected-fail/todo status in a later custom runner.

Checkpoint:

- `faber test` skips ignored tests by default.
- `faber test --ignored` runs ignored tests.
- Docs explain the temporary `futurum` behavior.

### Phase 4: Faber Test Metadata Model

Steps:

- Add explicit test metadata to HIR rather than overloading `HirFunction::is_generator`.
- Preserve:
  - original test name,
  - suite path,
  - source span,
  - modifiers and modifier payloads.
- Update Rust codegen to emit the same Rust output initially.
- Keep this phase behavior-preserving except for metadata availability.

Checkpoint:

- Existing proba/probandum compiler tests still pass.
- New unit tests prove metadata survives parsing, lowering, and codegen inspection.
- No CLI behavior changes are required in this phase.

### Phase 5: Faber-Specific Selection

Steps:

- Implement `solum` semantics.
- Add tag selection, likely:

```bash
faber test --tag smoke
```

- Add environment/capability selection for:
  - `requirit "capability"`,
  - `solum_in "env"`.
- Decide whether selection happens by:
  - filtering before generated Rust is written,
  - generating `#[ignore]` / test-name filters,
  - or generating a custom Rust test harness.

Checkpoint:

- `solum` runs only focused tests.
- `--tag` selection works for tagged tests.
- required capability/environment behavior is explicit and tested.

### Phase 6: Reporting and Docs

Steps:

- Add or update a test grammar doc if one does not exist.
- Update `README.md`.
- Update `docs/grammatica/manifest.md` with `faber test` and test artifact behavior.
- Update `docs/grammatica/verba.md`; `solum` is parsed today and should not be documented as merely planned once behavior is settled.
- Document current limits honestly:
  - Rust-only package test execution,
  - generated Rust test names may still appear until reporting improves,
  - richer modifiers may be parsed before fully enforced.

Checkpoint:

- Docs match live CLI help and `faber targets`.
- Examples under `examples/exempla/proba/` still parse and make sense.

### Phase 7: Validation and Release Readiness

Steps:

- Run:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
cargo build --release -p radix
```

- Run smoke tests:

```bash
cargo run -p faber -- init /tmp/faber-test-smoke
# add or use a fixture with proba/probandum cases
cargo run -p faber -- test /tmp/faber-test-smoke
cargo run -p faber -- test /tmp/faber-test-smoke --ignored
```

- Confirm no `target/faber/target` exists.
- Confirm working tree contains no generated target output.

Checkpoint:

- Validation passes.
- Work is committed.

## Epic Candidates And Scopable Issues

### Issue A: Cargo Test Adapter

Implement a small adapter next to the existing Cargo build adapter.

Acceptance:

- command uses `--manifest-path <root>/target/faber/Cargo.toml`,
- command uses `--target-dir <root>/target`,
- nonzero test failures propagate as command failures,
- no nested target directory appears.

### Issue B: `faber test` Command

Replace the stub with the minimal package test flow.

Acceptance:

- default path is `.`,
- package compilation and generated crate emission are reused,
- Cargo test output is visible,
- exit status is forwarded.

### Issue C: Test Fixture Coverage

Add focused tests or smoke fixtures for test execution.

Acceptance:

- passing test package,
- failing test package,
- ignored test package,
- nested `probandum` package.

### Issue D: Metadata Cleanup

Create explicit HIR metadata for Faber tests.

Acceptance:

- `omitte`, `futurum`, `solum`, tags, requirements, and suite path are represented directly,
- Rust codegen no longer relies on generator metadata to mean ignored test.

### Issue E: Documentation

Document the Faber test model.

Acceptance:

- language docs explain `proba` and `probandum`,
- tool docs explain `faber test`,
- current limits are explicit.

## Checkpoints

The first implementation slice should stop after Phase 1 unless the session is explicitly asked to continue. That gives the project a working `faber test` quickly without prematurely designing a custom Faber test runner.

Later phases should be committed independently:

- after minimal Cargo-backed runner,
- after CLI ergonomics,
- after ignored/future semantics,
- after HIR metadata cleanup,
- after Faber-specific selection,
- after docs and validation.

## Companion Skill Plan

- Use `factory` for implementation execution.
- Use `poker-face` after Phase 1 because the minimal runner is intentionally narrow and easy to overclaim.
- Use `zombie-docs` after Phase 6 to catch drift in keyword and command docs.
- Use `carmack-linus` before a custom test runner or custom harness is introduced.

## Gate Plan

Required gate before final commit of any implementation phase:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
```

Additional smoke for Phase 1:

- `faber test` runs a package with a passing `proba`.
- `faber test` exits nonzero for a package with a failing `proba`.
- ignored `proba omitte` / `proba futurum` tests are skipped by default.
- no `target/faber/target` is created.

## Open Questions

- Should `faber test <filter>` use positional filtering, or should Faber require an explicit `--filter` flag?
- Should `faber test -- <args>` pass raw args to the Rust test harness, or should Faber expose only curated flags?
- Should `futurum` eventually mean expected failure, todo, ignored, or a separate report status?
- Should `solum` be honored by filtering before codegen or by generated Rust harness behavior?
- Should Faber eventually generate a custom test harness to preserve suite names, tags, and source locations?

