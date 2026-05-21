# Faber Test Runner Evolution Factory Plan

**Status**: complete
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
| 2 | Test command ergonomics | Add a curated filter and common harness flags that map cleanly to Cargo's harness. | Common workflows work without invoking Cargo manually. |
| 3 | Ignored and future tests | Expose ignored-test execution for `omitte` and `futurum` cases. | Users can list/run ignored tests deliberately. |
| 4 | Faber test metadata model | Stop overloading generic HIR fields for test metadata; preserve source-level test names and modifiers. | Rust codegen and Faber tooling can inspect structured test metadata. |
| 5 | Faber-specific selection | Honor `solum`, `tag`, `requirit`, and `solum_in` at the Faber tool layer. | Faber selection semantics work before Cargo is invoked or via generated harness config. |
| 6 | Reporting and docs | Document test syntax, command behavior, and current limits; improve output where feasible. | README/grammar/docs match live command behavior. |
| 7 | Validation and release readiness | Run full Rust validation and smoke tests for passing/failing/ignored cases. | fmt/test/clippy/build and Faber test smoke pass. |

## Implementation Pipeline Contract

Every implementation phase must be executed through the full `delivery` skill pipeline before code changes begin. Each phase should create or update a phase-local delivery artifact that records:

1. `Intake And Interpret`
2. `Normalize Spec`
3. `Resolve Against Repo Context`
4. `Lower To Stage Graph`
5. `Assign Parallel Workstreams`
6. `Define Checkpoints And Gates`

The phase implementer should then use that delivery artifact as the execution contract for the phase. This prevents a future session from treating the phase bullets as a loose checklist and skipping baseline capture, repo-aware constraints, workstream boundaries, or gate design.

Minimum per-phase delivery outputs:

- confirmed command surface and non-goals,
- affected files and modules,
- test fixtures or smoke commands required for the phase,
- gate commands,
- commit boundary,
- any decisions intentionally deferred to later phases.

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

Goal:

Make the Cargo-backed Phase 1 runner usable for the common day-to-day cases without exposing arbitrary Cargo internals as the Faber interface.

Command shape:

```bash
faber test [path] [filter]
faber test [path] [filter] --exact
faber test [path] [filter] --nocapture
faber test [path] [filter] --test-threads <n>
```

Rules:

- The optional `filter` is passed to Cargo test as the Rust harness filter.
- `--exact`, `--nocapture`, and `--test-threads <n>` are passed after Cargo's `--` separator.
- Do not implement raw trailing pass-through in Phase 2. Raw pass-through is too easy to turn into an undocumented second CLI.
- Do not implement Faber-specific filtering yet. Filtering is by generated Rust test names for now, which usually means `proba_...`.
- Keep `path` defaulting to `.`.
- Reject invalid flag combinations through Clap or a clear manual error.
- Preserve Phase 1 behavior when no filter or flags are provided.

Steps:

- Extend `TestArgs` with:
  - optional positional `filter`,
  - `--exact`,
  - `--nocapture`,
  - `--test-threads <n>`.
- Extend the Cargo test adapter so it can pass:
  - pre-harness test filter before `--`,
  - harness flags after `--`.
- Update `faber test --help` so the supported shape is visible.
- Add unit tests for command construction if the adapter can be made testable without spawning Cargo; otherwise rely on fixture smoke plus code review.

Checkpoint:

- `faber test <path> <filter>` works.
- `faber test <path> <filter> --exact` works.
- `faber test <path> --nocapture` shows test stdout when a fixture prints.
- `faber test <path> --test-threads 1` passes the flag to the harness.
- Unsupported harness flags fail clearly instead of being silently ignored.
- No `target/faber/target` directory is created.

Phase 2 completion metrics:

- Help output documents every supported flag.
- Passing fixture still exits `0` with no flags.
- A known generated test filter runs a subset of the passing fixture.
- `--nocapture` visibly changes output for a fixture with test stdout, or a dedicated fixture is added to prove it.
- `--exact` is forwarded and does not break ordinary runs.
- `--test-threads 1` is forwarded and exits `0` on passing fixtures.
- Failing fixture still propagates nonzero status.
- No raw `-- <args>` pass-through is added in this phase.

### Phase 3: Ignored and Future Tests

Goal:

Expose ignored-test execution deliberately while keeping `omitte` and `futurum` semantics honest about their current implementation.

Command shape:

```bash
faber test [path] --ignored
faber test [path] --include-ignored
```

Rules:

- `--ignored` maps to Cargo/Rust harness `--ignored` and runs only ignored tests.
- `--include-ignored` maps to Cargo/Rust harness `--include-ignored` and runs both normal and ignored tests.
- `--ignored` and `--include-ignored` are mutually exclusive.
- `omitte` and `futurum` both remain Rust `#[ignore]` for this phase.
- Do not introduce expected-failure/todo semantics for `futurum` in this phase.
- Do not add Faber-specific `omitte`/`futurum` reports beyond what the Rust harness prints.

Steps:

- Extend `TestArgs` with `--ignored` and `--include-ignored`.
- Pass the selected harness flag after Cargo's `--`.
- Add conflict validation for `--ignored` plus `--include-ignored`.
- Update help text.
- Update docs to say:
  - `omitte` is skipped by default,
  - `futurum` is currently skipped by default,
  - both can be run through ignored-test flags until a later Faber-native reporter exists.

Checkpoint:

- `faber test` skips ignored tests by default.
- `faber test --ignored` runs ignored tests.
- `faber test --include-ignored` runs normal and ignored tests.
- Conflicting ignored flags fail clearly.
- Docs explain the temporary `futurum` behavior.

Phase 3 completion metrics:

- Ignored fixture default run exits `0` and reports two ignored tests.
- Ignored fixture with `--ignored` exits nonzero because the ignored failing tests are actually executed.
- Ignored fixture with `--include-ignored` exits nonzero for the same reason while also running normal tests.
- Help output shows both flags and their distinction.
- `--ignored --include-ignored` is rejected before invoking Cargo.
- No `target/faber/target` directory is created.

### Phase 4: Faber Test Metadata Model

Goal:

Make Faber tests first-class in HIR so later tool phases can reason about source-level test names, suite membership, and modifiers without reverse-engineering synthetic Rust functions.

Current problem:

- `lower_proba_item` currently lowers each `proba` to a synthetic `HirFunction`.
- Ignored-test state for `omitte` and `futurum` is carried by setting `HirFunction::is_generator`.
- Rust codegen identifies tests by synthetic `def_id >= 1_000_000`.
- That works for Cargo-backed Phase 1-3 behavior, but it overloads unrelated function metadata and gives `faber test` no honest source-level model for Phase 5 selection.

Data model target:

Add explicit HIR test metadata. The exact names can follow local style, but the shape should be equivalent to:

```rust
pub struct HirTestMetadata {
    pub name: Symbol,
    pub suite_path: Vec<Symbol>,
    pub modifiers: Vec<HirTestModifier>,
    pub span: Span,
}

pub enum HirTestModifier {
    Omitte(Symbol),
    Futurum(Symbol),
    Solum,
    Tag(Symbol),
    Temporis(i64),
    Metior,
    Repete(i64),
    Fragilis(i64),
    Requirit(Symbol),
    SolumIn(Symbol),
}
```

Attach the metadata to synthetic test functions, likely as:

```rust
pub test: Option<HirTestMetadata>
```

on `HirFunction`. A normal user function has `test: None`; every lowered `proba` has `test: Some(...)`.

Rules:

- Add explicit test metadata to HIR rather than overloading `HirFunction::is_generator`.
- Preserve:
  - original test name,
  - suite path,
  - source span,
  - modifiers and modifier payloads.
- Keep `HirFunction::is_generator` only for real generator/cursor semantics. Do not use it for ignored tests.
- Keep this phase behavior-preserving except for metadata availability. Existing `faber test` CLI behavior should not change.
- Rust codegen should still emit:
  - `#[test]` for lowered `proba` functions,
  - `#[ignore]` for metadata containing `Omitte(_)` or `Futurum(_)`,
  - the same generated Rust test function naming scheme unless a small internal helper rename is necessary.
- Do not implement Faber-specific selection in this phase.
- Do not implement custom reporting in this phase.
- Do not change `futurum` semantics yet; it remains ignored for now.
- Do not change the parser syntax.

Likely implementation steps:

- Add `HirTestMetadata` and `HirTestModifier` to the HIR node model and exports.
- Add `test: Option<HirTestMetadata>` to `HirFunction`.
- Update all manual `HirFunction` construction sites and tests with `test: None`.
- Convert parser-level `ProbaModifier` values into `HirTestModifier` values during lowering.
- Thread suite path information through `lower_probandum_items` so nested tests retain their full source suite path.
- Set `test: Some(...)` in `lower_proba_item`.
- Set `is_generator: false` for all lowered `proba` functions.
- Update Rust codegen test detection to use `func.test.is_some()` instead of synthetic `def_id` heuristics where practical.
- Update Rust codegen ignore detection to inspect `func.test.modifiers`.
- Update Faber codegen's synthetic proba round-trip path to use `func.test` instead of `is_generator` for ignored-test output.
- Keep any remaining `def_id >= 1_000_000` checks only as a temporary naming fallback if removing them would broaden the phase.

Checkpoint:

- Existing proba/probandum compiler tests still pass.
- New unit tests prove metadata survives lowering for:
  - top-level `proba`,
  - nested `probandum`,
  - `omitte`,
  - `futurum`,
  - `solum`,
  - `tag`,
  - `requirit`,
  - `solum_in`.
- Rust codegen tests prove `#[test]` and `#[ignore]` are emitted from `HirTestMetadata`, not `is_generator`.
- Faber codegen tests prove ignored proba cases round-trip from test metadata.
- No CLI behavior changes are required in this phase.
- `rg "is_generator" crates/radix/src/hir/lower crates/radix/src/codegen` shows no use of `is_generator` as test metadata.
- `faber test` Phase 1-3 fixture smokes still behave exactly the same.

Phase 4 completion metrics:

- `HirFunction::is_generator` is never set from a `proba` modifier.
- `HirFunction::is_generator` is not read to decide whether a Faber test is ignored.
- Every lowered `proba` function has `test: Some(HirTestMetadata)`.
- Every non-test function has `test: None`.
- Original source test names and suite paths are inspectable from HIR.
- All parsed proba modifiers are represented in HIR metadata, including modifiers not yet enforced by the runner.
- Existing generated Rust behavior for passing, failing, ignored, and suite fixtures is unchanged.
- No `faber test` command-line flags are added or removed.

### Phase 5: Faber-Specific Selection

Goal:

Make `faber test` understand source-level Faber test selection while preserving the compiler invariant that every test is still generated and compiled.

Core invariant:

Selection affects execution, not compilation.

```text
Faber source
  -> parse/lower/check all tests
  -> generate Rust for all tests
  -> compile all generated Rust tests
  -> execute only the selected tests
```

Non-selected tests must still appear in the generated Rust crate so the Rust compiler catches broken test code even during a focused run.

Selection mechanism:

Use generated Rust `#[ignore = "..."]` reasons for deselected tests in the Cargo-backed runner. Rust supports reason strings and prints them in standard harness output:

```rust
#[test]
#[ignore = "faber: not selected by solum"]
fn proba_1000000() {
    // test body still compiles
}
```

This is intentionally an execution mechanism, not the source of truth. Faber HIR test metadata from Phase 4 remains authoritative.

Initial command surface:

```bash
faber test [path]
faber test [path] --name <name>
faber test [path] --suite <suite-path>
faber test [path] --tag <tag>
```

Rules:

- Implement `solum` semantics.
- Add source-level selection flags:
  - `--name <name>` selects by Faber `proba` name, not generated Rust function name,
  - `--suite <suite-path>` selects by Faber `probandum` suite path,
  - `--tag <tag>` selects by Faber `tag` modifier.
- If one or more tests are marked `solum`, default `faber test` runs only the `solum` tests.
- Explicit selectors narrow the execution set. If selectors are combined, use AND semantics unless the delivery artifact for this phase deliberately chooses another rule.
- All tests are still generated and compiled.
- Tests outside the selected execution set are emitted with generated ignore reasons such as:
  - `#[ignore = "faber: not selected by solum"]`,
  - `#[ignore = "faber: not selected by name arithmetic passes"]`,
  - `#[ignore = "faber: not selected by suite math suite"]`,
  - `#[ignore = "faber: not selected by tag smoke"]`.
- Source-level `omitte` and `futurum` still emit ignore reasons distinct from selection reasons:
  - `#[ignore = "faber: omitte - <reason>"]`,
  - `#[ignore = "faber: futurum - <reason>"]`.
- A selected test with `omitte` or `futurum` remains ignored unless `--ignored` or `--include-ignored` is used.
- Phase 5 may accept that Rust `--ignored` / `--include-ignored` runs both source-level ignored tests and selection-ignored tests, because both compile to Rust `#[ignore]` in the Cargo-backed implementation. This limitation must be documented clearly.
- Do not implement `requirit`, `solum_in`, `temporis`, `repete`, `fragilis`, or custom reporting in this phase. They remain represented in metadata for later phases.
- Do not generate only selected tests.
- Do not introduce a custom Rust test harness in this phase.

Likely implementation steps:

- Add test selection options to `TestArgs`: `--name`, `--suite`, and `--tag`.
- Add a Faber test selection model in the build/test path that can inspect Phase 4 HIR metadata.
- Detect implicit `solum` selection after lowering and before Rust codegen.
- Pass selection context into Rust codegen or a HIR-to-codegen preparation step.
- Generate all test functions regardless of selection.
- For each test, compute:
  - whether it is selected for execution,
  - whether it is source-level ignored/future,
  - the generated Rust ignore reason, if any.
- Emit at most one Rust `#[ignore = "..."]` attribute per generated test. Prefer the most specific reason:
  - source-level `omitte` / `futurum` reason when the test is selected but ignored,
  - selection reason when the test is not selected.
- Add fixtures or extend existing fixtures for:
  - `solum`,
  - `tag`,
  - nested `probandum` suite selection,
  - a deselected test whose body still must compile.
- Update help text for the new flags.

Checkpoint:

- `solum` runs only focused tests while non-focused tests still compile.
- `--name` selects by original Faber test name.
- `--suite` selects by original Faber suite path.
- `--tag` selects by Faber tag metadata.
- Deselected tests show readable Rust ignore reasons in Cargo output.
- Source-level `omitte` and `futurum` still show distinct ignore reasons.
- A compile error in a deselected test still fails `faber test`.
- Existing Phase 1-3 flags still work.
- `--ignored` / `--include-ignored` behavior with selection-ignored tests is documented in the phase delivery artifact and user docs.
- No `target/faber/target` directory is created.

Phase 5 completion metrics:

- All generated Rust tests are present under `target/faber/src/main.rs` even when selection is active.
- At least one smoke proves a deselected test compiles by making it reference valid code and then checking it appears in generated Rust.
- At least one negative smoke proves a compile error in a deselected test still fails before execution.
- `faber test <path>` with `solum` executes only `solum` tests and reports the others ignored with `faber: not selected by solum`.
- `faber test <path> --name <name>` executes only matching source-name tests.
- `faber test <path> --suite <suite-path>` executes only matching suite-path tests.
- `faber test <path> --tag <tag>` executes only matching tagged tests.
- Combining selectors is tested and follows the chosen semantics.
- Phase 3 ignored-test controls still pass their existing smokes.
- Full gate passes: fmt, test, clippy, and fixture smoke matrix.

### Phase 6: Reporting and Docs

Goal:

Make the user-facing test documentation and command descriptions match the implemented Phases 1-5 behavior. This phase is about truthfulness, discoverability, and small reporting polish, not new test-runner semantics.

Scope:

- Add or update a test grammar doc if one does not exist.
- Update `README.md`.
- Update `docs/grammatica/manifest.md` with `faber test` and test artifact behavior.
- Update `docs/grammatica/verba.md`; `solum` is parsed today and should not be documented as merely planned once behavior is settled.
- Update `faber test --help` text only if it is inaccurate or omits Phase 5 behavior that users need to understand.
- Document current limits honestly:
  - Rust-only package test execution,
  - Cargo-backed standard Rust test harness output,
  - generated Rust `proba_*` names may still appear until reporting improves,
  - `--ignored` / `--include-ignored` also affect selection-ignored tests in the current Cargo-backed implementation,
  - richer modifiers may be parsed and represented in HIR before fully enforced.

Required docs content:

- `proba` syntax:
  - basic test case,
  - `adfirma`,
  - source-level test names.
- `probandum` syntax:
  - nested suites,
  - suite path format used by `--suite`, joined with `/`.
- setup/teardown:
  - `praepara`,
  - `praeparabit`,
  - `postpara`,
  - `postparabit`,
  - `omnia` behavior where currently supported.
- ignored/future tests:
  - `omitte "reason"` is skipped by default,
  - `futurum "reason"` is skipped by default for now,
  - both can be run with `--ignored` / `--include-ignored`.
- selection:
  - `solum`,
  - `tag`,
  - `--name`,
  - `--suite`,
  - `--tag`,
  - selector combination semantics.
- compilation invariant:
  - selection affects execution, not compilation,
  - all tests are generated and compiled,
  - non-selected tests are temporarily emitted as Rust ignored tests with `faber: not selected by ...` reasons.
- artifact layout:
  - generated Rust crate under `target/faber/`,
  - Cargo test artifacts under package-level `target/debug/`,
  - no `target/faber/target`.
- currently parsed but not fully enforced modifiers:
  - `requirit`,
  - `solum_in`,
  - `temporis`,
  - `metior`,
  - `repete`,
  - `fragilis`.
- coverage status:
  - Faber line coverage is not implemented in Phase 6,
  - generated Rust coverage is not the same as Faber source coverage,
  - future coverage work needs Faber source mapping or Faber-native instrumentation.

Reporting polish allowed:

- Improve wording in help text or error messages for `faber test` selectors if the implementation is already present and the change is low risk.
- Do not introduce a custom reporter.
- Do not change pass/fail/ignored semantics.
- Do not add line coverage, coverage flags, coverage reports, or source-map machinery.

Checkpoint:

- Docs match live CLI help and `faber targets`.
- Docs explain `proba`, `probandum`, setup/teardown, ignored/future tests, `solum`, tags, and selector flags.
- Docs explain the generated Rust ignore-reason model for selection.
- Docs explain the `--ignored` / `--include-ignored` caveat.
- Docs explicitly say Faber line coverage is not yet implemented.
- Examples under `examples/exempla/proba/` still parse and make sense.
- New or updated examples use current single-glyph syntax and documented Faber grammar.

Phase 6 completion metrics:

- There is a stable user-facing doc page for Faber tests.
- `README.md` points users at `faber test` or the test docs if README currently describes package workflows.
- `docs/grammatica/manifest.md` describes test artifact layout accurately.
- `docs/grammatica/verba.md` no longer misclassifies implemented test behavior as planned.
- `faber test --help` and docs do not contradict each other.
- Documentation states that coverage is future work, not part of the current runner.
- Phase 1-5 fixture smoke commands still behave the same after docs/reporting edits.

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

- Use the full `delivery` skill pipeline at the start of every implementation phase.
- Use `factory` for implementation execution after the phase delivery artifact is written or refreshed.
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

Additional smoke for Phase 2:

- `faber test examples/exempla/proba/packages/passing` still exits `0`.
- `faber test examples/exempla/proba/packages/passing <known-generated-test-name>` runs a subset of the passing fixture.
- `faber test examples/exempla/proba/packages/passing <known-generated-test-name> --exact` exits `0`.
- `faber test examples/exempla/proba/packages/passing --nocapture` shows captured stdout when the fixture includes a printed test.
- `faber test examples/exempla/proba/packages/passing --test-threads 1` exits `0`.
- an unsupported harness flag fails at the Faber CLI boundary with a clear error.
- raw `-- <args>` pass-through is not accepted in this phase.
- no `target/faber/target` is created.

Additional smoke for Phase 3:

- `faber test examples/exempla/proba/packages/ignored` exits `0` and reports ignored tests.
- `faber test examples/exempla/proba/packages/ignored --ignored` runs only ignored tests and exits nonzero when ignored failing tests execute.
- `faber test examples/exempla/proba/packages/ignored --include-ignored` runs normal and ignored tests and exits nonzero when ignored failing tests execute.
- `faber test examples/exempla/proba/packages/ignored --ignored --include-ignored` is rejected before invoking Cargo.
- no `target/faber/target` is created.

## Open Questions

- Should `futurum` eventually mean expected failure, todo, ignored, or a separate report status?
- Should `solum` be honored by filtering before codegen or by generated Rust harness behavior?
- Should Faber eventually generate a custom test harness to preserve suite names, tags, and source locations?
