# Faber Build Tool Evolution Factory Plan

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-build-tool-evolution/`
**Depends On**: Faber/Radix split and `faber.toml` manifest support
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber is now split from Radix, but the `faber` binary is not yet a useful Cargo-like build tool. It can discover and check a package manifest, and it can produce Rust code, but it does not yet own the full package build lifecycle.

The next evolution is to make `faber build` behave like the front door for package builds:

1. Resolve a Faber package from `faber.toml`.
2. Compile Faber source into generated Rust source.
3. Write that generated Rust source as a Rust crate under `target/faber/`.
4. Invoke Rust/Cargo on that generated crate.
5. Place Rust build artifacts under package-level `target/debug/` or `target/release/`.
6. Let `faber run` build when needed and execute the resulting binary.

## Non-Negotiable Directory Contract

The build layout must preserve this shape:

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
    ├── debug/
    │   ├── <package-binary>
    │   ├── deps/
    │   └── incremental/
    └── release/
        ├── <package-binary>
        └── deps/
```

`target/faber/` is the generated Rust crate. It is not the final artifact directory.

`target/debug/` and `target/release/` are the Rust backend artifact directories. They are package-level peers of `target/faber/`, not nested under it.

The intended Cargo invocation is equivalent to:

```bash
cargo build \
  --manifest-path target/faber/Cargo.toml \
  --target-dir target
```

For release builds:

```bash
cargo build \
  --manifest-path target/faber/Cargo.toml \
  --target-dir target \
  --release
```

This produces:

```text
target/faber/          # generated Rust source crate
target/debug/...       # debug binary and Rust artifacts
target/release/...     # release binary and Rust artifacts
```

Do not implement a layout that produces `target/faber/target/debug/...`.

## Normalized Spec

### Current Baseline

- `crates/faber` owns the user-facing `faber` binary.
- `faber.toml` exists and is documented.
- `faber init` creates `faber.toml` and `src/main.fab`.
- `faber check` can analyze packages.
- `faber build` still writes generated backend output through the older single-output-file path.
- `faber run` and `faber test` are stubs.
- Package compilation is Rust-only.
- Radix remains responsible for compiler internals and compiler inspection commands.

### Target Behavior

`faber build` with a package input should:

- discover the package root and manifest,
- compile the package to Rust,
- write a generated Rust crate into `<package-root>/target/faber/`,
- write a generated `Cargo.toml` with package metadata derived from `faber.toml`,
- invoke Cargo with `--target-dir <package-root>/target`,
- print the final binary path or a concise build success line,
- preserve useful diagnostics from both Faber/Radix and Cargo.

`faber run` should:

- discover the package root and manifest,
- call the same build pipeline,
- execute `<package-root>/target/debug/<package-name>` by default,
- support release execution once `--release` exists,
- pass trailing user arguments to the compiled binary.

`faber emit` should remain source-emission oriented and should not be confused with package build artifacts. It may print generated Rust to stdout for inspection.

## Repo-Aware Baseline

Relevant files:

- `crates/faber/src/main.rs`
- `crates/faber/src/package.rs`
- `crates/faber/src/package_test.rs`
- `crates/radix/src/tool.rs`
- `docs/grammatica/manifest.md`
- `docs/grammatica/targets.md`
- `README.md`

Current package compilation assembles a single Rust crate string in memory. The implementation may either:

- initially write that assembled crate string to `target/faber/src/main.rs`, or
- broaden codegen to emit multiple Rust source files that mirror the package module tree.

The first implementation should prefer the smaller step: one generated Rust crate with `src/main.rs`, plus generated `Cargo.toml`. Multi-file generated Rust can follow after the crate lifecycle is correct.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Preflight and baseline capture | Record current command behavior, dirty tree, and validation status. | Ledger created; no behavior changed. |
| 1 | Build pipeline model | Add Faber-owned build structs for package root, generated crate paths, profile, and binary path calculation. | Unit tests prove path layout without invoking Cargo. |
| 2 | Generated Rust crate emission | Write `target/faber/Cargo.toml` and `target/faber/src/main.rs` from package compilation output. | A package build helper can produce the expected crate tree before Cargo is invoked. |
| 3 | Cargo backend invocation | Invoke Cargo against `target/faber/Cargo.toml` with `--target-dir target`. | `faber build` creates `target/debug/<package-name>` for a sample package. |
| 4 | Build modes and command shape | Add debug/release controls and clarify output messages. | `faber build` and `faber build --release` route artifacts to the correct peer directories. |
| 5 | `faber run` | Build if needed, then execute the compiled binary with trailing args. | `faber run -- <args>` runs the generated executable. |
| 6 | Docs and manifest updates | Document build layout, command behavior, and manifest build fields. | README and grammar docs match current implementation. |
| 7 | Validation and release readiness | Run full Rust validation and package smoke tests. | fmt/test/clippy/build plus package build/run smoke pass. |

## Phase Details

### Phase 0: Preflight and Baseline Capture

Steps:

- Inspect `git status --short`.
- Capture `cargo run -p faber -- build <sample-package>` behavior before changes.
- Capture `cargo run -p faber -- run <sample-package>` stub behavior.
- Pick or create a small package fixture for smoke testing.
- Create a ledger in this artifact directory if the implementing session needs multi-turn tracking.

Checkpoint:

- Baseline is recorded truthfully.
- No generated `target/` contents are committed.

### Phase 1: Build Pipeline Model

Steps:

- Add a Faber-owned package build model, likely in `crates/faber/src/package.rs` or a new focused module.
- Represent:
  - package root,
  - manifest path,
  - generated Rust crate root: `<package-root>/target/faber`,
  - generated Rust manifest: `<package-root>/target/faber/Cargo.toml`,
  - generated Rust entry: `<package-root>/target/faber/src/main.rs`,
  - Cargo target dir: `<package-root>/target`,
  - debug binary path: `<package-root>/target/debug/<package-name>`,
  - release binary path: `<package-root>/target/release/<package-name>`.
- Sanitize package names for generated Rust crate names if needed.
- Keep path calculation testable without running Cargo.

Checkpoint:

- Tests prove `target/faber`, `target/debug`, and `target/release` are siblings.
- Tests explicitly reject or avoid `target/faber/target`.

### Phase 2: Generated Rust Crate Emission

Steps:

- Change package build output from a single ad hoc output file to a generated crate tree.
- Write:

```text
target/faber/Cargo.toml
target/faber/src/main.rs
```

- Generate `Cargo.toml` from `faber.toml`:
  - package name,
  - version,
  - edition appropriate for Rust, probably `2021` unless the implementation proves a newer edition is required,
  - dependencies needed by generated code.
- Keep the generated crate deterministic enough for tests.
- Remove stale generated files before writing, or overwrite the known generated files deliberately.

Checkpoint:

- A package build can produce an inspectable generated Rust crate.
- Generated crate files are not committed.
- Generated Rust remains readable enough for debugging.

### Phase 3: Cargo Backend Invocation

Steps:

- Invoke Cargo after successful generated crate emission:

```bash
cargo build --manifest-path <package-root>/target/faber/Cargo.toml --target-dir <package-root>/target
```

- Preserve Cargo stdout/stderr in a user-usable way.
- Return a clear failure if Cargo is missing or exits nonzero.
- Do not run Cargo from `target/faber` in a way that makes Cargo create `target/faber/target`.
- Ensure the generated crate builds with the current `norma` runtime support if needed.

Checkpoint:

- `faber build <package>` creates `<package>/target/debug/<package-name>`.
- No `<package>/target/faber/target/` directory is created.

### Phase 4: Build Modes and Command Shape

Steps:

- Add a `--release` flag to `faber build`.
- Decide whether `--target rust` stays visible or becomes manifest-driven for package builds.
- Preserve direct file build behavior only if it remains useful and does not confuse package behavior.
- Print the final binary path on success, for example:

```text
target/debug/hello
```

- Keep `faber emit -t rust --package` as the inspection path for stdout source emission.

Checkpoint:

- `faber build` writes debug artifacts under `target/debug`.
- `faber build --release` writes release artifacts under `target/release`.
- Help text distinguishes generated source output from compiled binary output.

### Phase 5: `faber run`

Steps:

- Add `faber run [path] [--release] -- [args...]`.
- Reuse the build pipeline.
- Execute the computed binary path after a successful build.
- Forward exit status, stdout, and stderr from the child process.
- Define behavior for missing package path, defaulting to `.`.

Checkpoint:

- `faber run` builds and runs a package from the current directory.
- `faber run path/to/package -- arg1 arg2` forwards arguments.
- Child exit codes are preserved.

### Phase 6: Docs and Manifest Updates

Steps:

- Update `README.md` command examples.
- Update `docs/grammatica/manifest.md` with the generated Rust crate layout.
- Update `docs/grammatica/targets.md` so Rust package build/run capability is accurate.
- Add a short generated-artifacts policy:
  - `target/faber/` is generated source,
  - `target/debug/` and `target/release/` are backend artifacts,
  - all are ignored build output.

Checkpoint:

- Docs describe the implemented layout exactly.
- No docs claim that `faber run` is still a stub after it is implemented.

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

- Run package smoke tests:

```bash
cargo run -p faber -- init /tmp/faber-smoke
cargo run -p faber -- check /tmp/faber-smoke
cargo run -p faber -- build /tmp/faber-smoke
cargo run -p faber -- run /tmp/faber-smoke
cargo run -p faber -- build --release /tmp/faber-smoke
```

- Confirm no accidental generated files are tracked.
- Confirm no `target/faber/target` exists in smoke output.

Checkpoint:

- Validation passes.
- Work is committed.

## Epic Candidates And Scopable Issues

### Issue A: Path Model and Package Metadata

Implement package root discovery that returns enough metadata for build/run commands without duplicating manifest parsing.

Acceptance:

- path model tests cover manifest path, directory input, and entry file input,
- binary path calculation is deterministic,
- package name validation and Rust crate name conversion are explicit.

### Issue B: Generated Rust Crate Writer

Write the generated Rust crate under `target/faber`.

Acceptance:

- generated `Cargo.toml` and `src/main.rs` are created,
- stale generated source is handled deliberately,
- no Cargo invocation happens in writer-only tests.

### Issue C: Cargo Build Adapter

Invoke Cargo with the correct manifest and target directory.

Acceptance:

- command includes `--manifest-path target/faber/Cargo.toml`,
- command includes `--target-dir target`,
- tests or smoke checks prove artifact output is `target/debug` or `target/release`.

### Issue D: Run Command

Implement `faber run`.

Acceptance:

- default path is `.`,
- run reuses build behavior,
- trailing args are passed to the compiled binary,
- exit status is forwarded.

### Issue E: Docs and Examples

Update docs after implementation.

Acceptance:

- generated source and backend artifacts are documented as separate sibling directories,
- command examples match live CLI help,
- target capability docs are current.

## Checkpoints

Every phase should be independently commit-ready. At minimum:

- after path model,
- after generated crate writer,
- after Cargo build invocation,
- after `faber run`,
- after docs and validation.

Generated files under package `target/` must never be committed.

## Companion Skill Plan

- Use `factory` for implementation execution if this plan is handed to an autonomous implementation session.
- Use `poker-face` after each implementation phase to compare actual behavior against this document.
- Use `zombie-docs` after `faber run` lands to catch stale command references.
- Use `carmack-linus` only if the build API starts growing abstractions that are hard to justify.

## Gate Plan

Required gate before final commit:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
cargo build --release -p radix
```

Required smoke behavior:

- `faber init` creates a buildable package.
- `faber check` passes on the package.
- `faber build` creates `target/faber/Cargo.toml`, `target/faber/src/main.rs`, and `target/debug/<package-name>`.
- `faber build --release` creates `target/release/<package-name>`.
- `faber run` executes the package binary.
- No package build creates `target/faber/target`.

## Open Questions

- Should `faber build` default to package mode only, or should direct single-file output remain part of the Faber surface for now?
- Should generated Rust use one `src/main.rs` file initially, or should it immediately mirror Faber modules as separate Rust files?
- Should generated Rust depend on the local workspace `crates/norma` by path, or should generated code remain self-contained until runtime support requires Norma?
- Should `faber.toml [build]` grow profile-specific fields now, or should `--release` be the only profile control in this phase?
- What exact output should successful `faber build` print: generated crate root, binary path, or both?
