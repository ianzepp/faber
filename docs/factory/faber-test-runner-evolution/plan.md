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

### Phase-One Completion Metrics

Phase 1 should not be called complete unless these user-visible checks are true:

- `faber test --help` describes an implemented command, not a planned/stub command.
- `faber test` with no path defaults to the current directory and uses the same package discovery rules as `faber build`.
- Passing, failing, ignored, and suite fixtures are committed under `examples/exempla/proba/packages/`.
- Passing, ignored, and suite fixtures exit `0`.
- The failing fixture exits nonzero because a generated Rust test fails, not because Faber parse/check/build failed.
- Cargo test output is visible enough to see pass/fail/ignored counts.
- Exit status matches Cargo test status.
- No smoke run creates `target/faber/target`.
- The implementation does not add Faber-specific filtering, tags, custom reporting, or a custom harness in Phase 1.
- `docs/factory/faber-test-runner-evolution/ledger.md` is created or updated, and the plan status is updated when the full plan is complete.

### Later Target Behavior

Later phases should progressively expose Faber-specific test behavior:

- filter by test name or suite,
- pass through common Rust harness flags when they make sense,
- run ignored `omitte` / `futurum` cases when requested,
- honor `solum` as "run only this test" in Faber terms,
- honor tags and environment requirements,
- report Faber source names and locations rather than only generated Rust test names,
- document test syntax and command behavior together.

## Test Fixture Contract

The implementation must add concrete package fixtures before claiming `faber test` works. These fixtures should live in the repository, not only in `/tmp`, so future sessions can rerun the same cases.

Recommended fixture root:

```text
examples/exempla/proba/packages/
├── passing/
├── failing/
├── ignored/
└── suite/
```

Each fixture is a normal Faber package:

```text
<fixture>/
├── faber.toml
└── src/
    └── main.fab
```

The `faber.toml` shape should be identical except for package name:

```toml
[package]
name = "proba-passing"
version = "0.1.0"
edition = "2026"

[paths]
source = "src"
entry = "main.fab"

[build]
target = "rust"
kind = "bin"
```

### Fixture: Passing

Path:

```text
examples/exempla/proba/packages/passing/
```

`src/main.fab`:

```fab
proba "arithmetic passes" {
    adfirma 1 + 1 ≡ 2
}

proba "text passes" {
    fixum greeting = "salve"
    adfirma greeting ≡ "salve"
}

incipit {
    scribe "test fixture"
}
```

Expected Phase 1 behavior:

- `faber test examples/exempla/proba/packages/passing` exits `0`.
- Cargo reports two passing tests.
- Generated Rust contains at least two `#[test]` functions.
- No `target/faber/target` directory is created.

### Fixture: Failing

Path:

```text
examples/exempla/proba/packages/failing/
```

`src/main.fab`:

```fab
proba "intentional failure" {
    adfirma 1 + 1 ≡ 3
}

incipit {}
```

Expected Phase 1 behavior:

- `faber test examples/exempla/proba/packages/failing` exits nonzero.
- Cargo reports a failed test.
- The command failure comes from the Rust test harness, not from Faber parse/check/build failure.

### Fixture: Ignored

Path:

```text
examples/exempla/proba/packages/ignored/
```

`src/main.fab`:

```fab
proba omitte "blocked by external service" "skipped failing case" {
    adfirma falsum
}

proba futurum "not implemented yet" "future case" {
    adfirma falsum
}

proba "normal passing case" {
    adfirma verum
}

incipit {}
```

Expected Phase 1 behavior:

- `faber test examples/exempla/proba/packages/ignored` exits `0`.
- Cargo reports one passing test and two ignored tests.
- Generated Rust contains `#[ignore]` for the `omitte` and `futurum` cases.
- Running ignored tests is not required until Phase 3.

### Fixture: Suite

Path:

```text
examples/exempla/proba/packages/suite/
```

`src/main.fab`:

```fab
probandum "math suite" {
    praepara omnia {
        fixum setup_value = 1
        adfirma setup_value ≡ 1
    }

    proba "top level suite case" {
        adfirma 2 + 2 ≡ 4
    }

    probandum "nested suite" {
        proba "nested case" {
            adfirma verum
        }
    }
}

incipit {}
```

Expected Phase 1 behavior:

- `faber test examples/exempla/proba/packages/suite` exits `0`.
- Cargo reports two passing tests.
- Nested `probandum` cases are lowered into runnable Rust tests.
- Suite setup marked `praepara omnia` is included in generated test bodies.

### Fixture Policy

- Keep fixtures intentionally small. They are command smoke fixtures, not exhaustive compiler tests.
- Use canonical Faber syntax in fixtures. After the glyph clean break, examples must use glyph forms such as `≡`, `→`, `≤`, `≥`, `⊕`, `⊖`, `⊛`, and `⊘` where those operators are needed.
- Do not rely on generated Rust function names being stable beyond the `proba_` prefix in Phase 1.
- Do not commit generated `target/` contents from fixture runs.

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
- `examples/exempla/proba/packages/` once Phase 1 fixture work lands

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
- Confirm the repository either already has the fixtures from the Test Fixture Contract or create them before implementing the runner.

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
- The suite fixture proves nested `probandum` tests execute.
- Cargo test output remains visible to the user.
- The command exits with Cargo test's exit status.
- No package test creates `target/faber/target`.
- `faber test --help` no longer advertises the command as planned or stubbed.

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
cargo run -p faber -- test examples/exempla/proba/packages/passing
cargo run -p faber -- test examples/exempla/proba/packages/ignored
cargo run -p faber -- test examples/exempla/proba/packages/suite
! cargo run -p faber -- test examples/exempla/proba/packages/failing
```

- Confirm no `target/faber/target` exists.
- Confirm working tree contains no generated target output.
- Review the human command surface:
  - `faber test --help`,
  - `faber test examples/exempla/proba/packages/passing`,
  - `faber test examples/exempla/proba/packages/failing`,
  - `faber test examples/exempla/proba/packages/ignored`,
  - `faber test examples/exempla/proba/packages/suite`.
- Update `docs/factory/faber-test-runner-evolution/plan.md` status and ledger state when the full plan is actually complete.

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

- `examples/exempla/proba/packages/passing` exists and exits `0` under `faber test`,
- `examples/exempla/proba/packages/failing` exists and exits nonzero under `faber test`,
- `examples/exempla/proba/packages/ignored` exists and exits `0` with ignored cases skipped,
- `examples/exempla/proba/packages/suite` exists and proves nested `probandum` execution,
- generated fixture output stays untracked.

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

For Phase 1, "done" means the command works end-to-end for the four committed fixtures and the output/exit-status/layout checks in the Phase-One Completion Metrics all pass. Do not treat a compiled adapter or unit tests alone as sufficient.

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

- `faber test examples/exempla/proba/packages/passing` exits `0`.
- `faber test examples/exempla/proba/packages/failing` exits nonzero.
- `faber test examples/exempla/proba/packages/ignored` exits `0` and skips ignored tests by default.
- `faber test examples/exempla/proba/packages/suite` exits `0`.
- `faber test --help` describes real behavior.
- failing fixture output shows a Rust test failure.
- ignored fixture output shows ignored tests.
- no `target/faber/target` is created.

## Open Questions

- Should `faber test <filter>` use positional filtering, or should Faber require an explicit `--filter` flag?
- Should `faber test -- <args>` pass raw args to the Rust test harness, or should Faber expose only curated flags?
- Should `futurum` eventually mean expected failure, todo, ignored, or a separate report status?
- Should `solum` be honored by filtering before codegen or by generated Rust harness behavior?
- Should Faber eventually generate a custom test harness to preserve suite names, tags, and source locations?
