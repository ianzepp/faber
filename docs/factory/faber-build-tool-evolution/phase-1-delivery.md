# Phase 1 Delivery Spec: Build Pipeline Model

**Parent Plan**: `docs/factory/faber-build-tool-evolution/plan.md`
**Phase**: 1 - Build pipeline model
**Status**: ready for implementation
**Created**: 2026-05-21 (from ledger)

## Interpreted Phase Problem

The current package handling in `crates/faber/src/package.rs` discovers packages and compiles them to a single in-memory Rust crate string, but there is no owned model for the full package build lifecycle paths: where the generated Rust crate lives (`target/faber/`), where Cargo will place artifacts (`target/debug|release/` as siblings), and how to compute binary paths from the manifest/package name.

Phase 1 introduces a pure, testable build model (paths + metadata) **before** any file writing or Cargo invocation. This keeps the contract explicit and prevents layout bugs (e.g. accidental `target/faber/target`).

## Normalized Phase Spec

### Inputs
- Package root (dir containing `faber.toml` or `src/main.fab` etc.)
- Manifest (already parsed as `FaberManifest` in package.rs)
- Package name from manifest (sanitized for Rust crate name if needed)
- Build profile (debug vs release) - initially just represented, `--release` flag comes in phase 4

### Required Model Types (new or extended in `package.rs`)
- `PackageBuild` or `BuildLayout` struct holding:
  - `package_root: PathBuf`
  - `manifest_path: PathBuf`
  - `generated_crate_root: PathBuf`  (= `<root>/target/faber`)
  - `generated_cargo_manifest: PathBuf` (= `<root>/target/faber/Cargo.toml`)
  - `generated_rust_entry: PathBuf` (= `<root>/target/faber/src/main.rs`)
  - `cargo_target_dir: PathBuf` (= `<root>/target`)
  - `debug_binary: PathBuf` (= `<root>/target/debug/<sanitized-pkg-name>`)
  - `release_binary: PathBuf` (= `<root>/target/release/<sanitized-pkg-name>`)
- Helper constructors: `from_package_root`, `from_manifest_path`, `discover_and_build` (thin wrapper)
- Name sanitization: map Faber package name (e.g. "my-pkg", "FaberTool") to valid Rust crate/binary name (lowercase, -, _, alphanum; Cargo rules)
- Pure path methods, no I/O except for discovery that reuses existing `discover_package`

### Outputs
- The model types + unit tests that assert sibling layout:
  - `target/faber/Cargo.toml` and `target/debug/foo` are peers under package root
  - `target/faber/target` does not appear in any computed path
- No changes to compilation, emission, or CLI yet.

### Out Of Scope (for this phase)
- Writing any files under target/faber/
- Invoking Cargo
- `--release` flag handling / CLI changes
- Multi-file Rust module emission (still single assembled crate string for now)
- Changing how `compile_package` produces the code string

## Repo-Aware Phase Baseline

Relevant code (current state post-phase-0):
- `crates/faber/src/package.rs`:
  - `FaberManifest`, `ManifestPackage`, `PackageSpec`, `discover_package`, `parse_manifest`
  - `compile_package` returns `CompileResult` with assembled `crate_code`
  - `cmd_build` still uses old `build_output_path` -> single `main.rs`
  - `read_manifest` exists (used by tests)
- `crates/faber/src/package_test.rs`: temp_dir helper + compile/check tests; no layout tests yet
- `crates/faber/src/main.rs`: thin dispatch, Run/Test are still stubs
- `crates/radix/src/tool.rs`: `BuildArgs` (has out_dir), `build_output_path` (package always "main")
- No existing `BuildLayout` or equivalent.

Key invariants to enforce in model:
- `cargo_target_dir` must be the package's `target/`, **not** nested under `target/faber`
- Generated crate is always `target/faber/` (hard-coded per Non-Negotiable Directory Contract in plan)
- Binary stem = sanitized package name (from `[package].name`)

Package name sanitization notes (Cargo rules):
- Must be ASCII lowercase alphanum + `-` `_` (usually replace invalid with `-` or `_`)
- Binary name for `[[bin]]` or default is the package name sanitized

Existing manifest parsing already gives us the name; reuse it.

## Phase Stage Graph (single-phase)

| Step | Task | Verification |
|------|------|--------------|
| 1 | Define `BuildLayout` (or `PackageBuild`) struct + constructor(s) + accessors in `package.rs` (pub) | Compiles |
| 2 | Implement deterministic path methods + `sanitized_name()` | Unit tests pass for paths |
| 3 | Add 4-6 unit tests in `package_test.rs` proving sibling layout + rejection of nested target | `cargo test -p faber --test package_test` or `cargo test --package faber package::` |
| 4 | (Optional) expose a thin `build_plan_for_package(input)` helper that reuses `discover_package` + `read_manifest` | Tests cover dir, manifest file, and entry-file inputs |
| 5 | Ensure no behavioral change to existing `compile_package` / `cmd_build` | Existing tests + manual `faber check` still pass |

## Epic / Workstream for Phase
Issue B (partial): Path Model and Package Metadata (from plan epic A)

- Focused: only the pure data model + tests. Writer (B) and Cargo adapter (C) are later phases.

## Checkpoint and Gate Plan for Phase 1
**Checkpoint Target** (from master plan):
- "Tests prove `target/faber`, `target/debug`, and `target/release` are siblings."
- "Tests explicitly reject or avoid `target/faber/target`."

**Gate Requirements**:
- `cargo test -p faber` (or scoped) passes with new layout tests
- `cargo check -p faber` clean
- No changes to public CLI or emitted artifacts yet
- Ledger updated with phase-1 delivery spec reference + test evidence
- **Poker-face** completion >= 85% against this spec before commit

**Verification Commands**:
```bash
cargo test -p faber --test package_test
cargo check -p faber
cargo clippy -p faber -- -D warnings
```

**Commit Message**: "Complete Phase 1: Build pipeline model (layout types + tests)"

## Open Questions Blocking Phase 1
- Exact struct name: `BuildLayout`, `PackageBuild`, `BuildPlan`, `CrateLayout`? (Recommend `BuildLayout` for neutrality)
- Should sanitization live in a small `fn sanitize_crate_name(name: &str) -> String` or on the struct?
- Does the model need to hold the full parsed `FaberManifest`, or just the derived name + paths? (Prefer minimal: name + paths; manifest can be re-read if needed later)
- Profile: represent as enum now or just two path methods? (Keep simple: two methods, profile flag added in phase 4)

## Companion Skills for This Phase
- None required beyond core (no docs change, no new runtime)
- After implementation: run `poker-face` (or manual equivalent) + update ledger

## Spec Revisions
(none yet - initial)

---

**Delivery Spec persisted before any implementation edits.**
