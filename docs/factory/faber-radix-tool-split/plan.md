# Faber/Radix Tool Split Factory Plan

**Status**: completed
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-radix-tool-split/`
**Intended Release**: v0.34.0 transition release
**Commit Policy**: Commit after each completed phase and validation gate pass

## Decision Record

Faber should become the user-facing project and package tool. Radix should remain the compiler engine and compiler-developer CLI.

The split is conceptual first and binary-level second:

- **Faber**: language product name and Cargo-like project tool.
- **Radix**: compiler implementation, library API, and direct compiler inspection binary.
- **Norma**: standard library definitions and Rust runtime support.

Do not pull back or delete the existing v0.33.0 Faber-named release unless the assets are broken or unsafe. Treat v0.33.0 as the first renewed Faber CLI release and clarify the transition in release notes and docs.

## Boundary

Radix must not become blind single-file transpilation. The compiler owns semantic correctness over a resolved compilation graph.

Radix should own:

- lexing, parsing, AST, HIR, semantic analysis, diagnostics, and codegen
- analysis of a source file plus explicitly supplied local module graph
- standard library declaration and `@ verte` metadata loading
- extern declarations needed for target-aware checking
- cross-module type/name resolution once modules are provided
- compiler inspection commands such as `lex`, `parse`, `hir`, `emit`, and raw `check`

Faber should own:

- manifest and workspace discovery
- local package graph construction
- dependency resolution policy, lockfiles, and registry/git/path dependencies
- build profiles, artifact directories, run/test/doc/publish workflows
- user command ergonomics
- invoking the Radix library with an explicit resolved graph

## v0.34 Command Policy

v0.34 should add the new shape without avoidable breakage from v0.33.

Primary user commands:

```bash
faber check
faber build
faber targets
faber init
faber run
faber test
```

Compiler-developer commands:

```bash
radix lex
radix parse
radix hir
radix emit
radix check
radix targets
radix cli-ir
```

Compatibility commands for one or more transition releases:

```bash
faber lex
faber parse
faber hir
faber emit
faber cli-ir
```

These compatibility commands may either run as aliases or print a clear migration message. Prefer aliases in v0.34 because v0.33 is the only Faber-named binary release.

## Target Source Shape

Do not keep the Cargo workspace under a top-level directory named `radix` once `faber` and `radix` are sibling crates. That shape makes Radix look like it owns Faber. Promote the Cargo workspace to the repository root as part of this split.

```text
faber/
├── Cargo.toml      # Cargo workspace manifest
├── crates/
│   ├── faber/      # user-facing project/package CLI, bin "faber"
│   ├── radix/      # compiler library plus bin "radix"
│   └── norma/      # Rust runtime support crate
├── stdlib/
│   └── norma/      # Faber stdlib definitions and @ verte metadata
├── examples/
├── docs/
└── scripta/
```

Migration target:

```text
radix/Cargo.toml   -> Cargo.toml
radix/crates/*     -> crates/*
radix/stdlib       -> stdlib
```

Do not rename the repository root. The repo is already the Faber project. The current `radix/` directory is a historical workspace container and should disappear after the workspace promotion.

The resulting crate meanings are:

```text
crates/faber      # package/project tool
crates/radix      # compiler library and compiler-dev binary
crates/norma      # Rust runtime support
stdlib/norma      # Faber stdlib source declarations
```

The existing compiler crate remains the compiler library after it moves from `radix/crates/radix` to `crates/radix`. Its current `src/main.rs` command surface should be split so direct compiler commands move to a `radix` binary and package/product commands move to a new `faber` crate.

## Workspace Move Policy

The workspace promotion is large enough to deserve its own phase before adding the new Faber crate. Keep it mostly mechanical:

```text
radix/Cargo.toml   -> Cargo.toml
radix/crates       -> crates
radix/stdlib       -> stdlib
```

Then update repo-local command paths from `--manifest-path radix/Cargo.toml` to the root workspace manifest where appropriate.

Avoid a top-level `workspace/`, `packages/`, or `rust/` directory. Those names add another generic container while the repo root can already be the workspace root.

If the implementing session decides the workspace promotion is too risky to combine with the binary split, it may make Phase 1 stop after adding a root workspace manifest and compatibility paths, but the final v0.34 target should still remove the misleading top-level `radix/` container.

## Rust-Only Tooling Policy

This repository should not require Bun or Node for normal development after the split. Existing Bun/TypeScript files are migration residue unless a phase proves otherwise.

Default target:

- Remove `package.json`, `bun.lock`, `tsconfig.json`, ESLint config, and TypeScript helper scripts from the active workflow.
- Remove or archive obsolete `runtimes/norma-ts` material if it is no longer part of the active compiler/runtime contract.
- Replace `bun run ...` validation with direct Cargo and small shell commands.
- Keep Markdown/Faber formatting lightweight. If Prettier is still desired for docs, it should be explicitly justified as a docs-only optional tool, not a project build dependency.

## Phase Index

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Preflight and current-state capture | Record dirty tree, command surface, release state, and baseline validation status. | Factory ledger created; no behavior changed. |
| 1 | Workspace promotion | Move the Cargo workspace from `radix/` to the repo root. | Root `Cargo.toml`, `crates/radix`, `crates/norma`, and `stdlib/norma` are canonical. |
| 2 | Node/Bun retirement | Remove active Bun/Node/TypeScript tooling from this Rust project, or document any proven exception. | Cargo commands are the primary validation surface; no `bun run` gate remains. |
| 3 | Radix binary extraction | Rename/add the compiler-dev binary as `radix` while preserving compiler library APIs. | `radix lex/parse/hir/emit/check/targets/cli-ir` work. |
| 4 | Faber crate scaffold | Add `crates/faber` as the user-facing binary depending on `radix`. | `faber check/build/targets` route through the existing compiler/package behavior. |
| 5 | Package ownership move | Move package discovery, manifest, and project orchestration out of Radix driver into Faber-owned modules or a shared package library. | Radix no longer owns package policy; Faber package commands still work. |
| 6 | Compatibility aliases | Keep v0.33 command compatibility on `faber` for compiler inspection commands. | Old `faber emit/lex/parse/hir/cli-ir` surface works or emits explicit migration diagnostics. |
| 7 | Release and docs transition | Update README, grammar CLI docs, release notes, workflows, and artifact packaging for two binaries. | v0.34 docs explain Faber/Radix/Norma roles and v0.33 continuity. |
| 8 | Validation and polish | Run full Rust checks, examples, release packaging smoke tests, and command help review. | Cargo fmt/test/clippy/build and binary smoke tests pass. |

## Phase Details

### Phase 0: Preflight and Current-State Capture

Steps:

- Inspect `git status --short` and preserve unrelated in-flight work.
- Capture current `faber targets` output.
- Capture current release workflow assets and v0.33 tag state.
- Create or update a factory ledger in this directory.
- Run baseline validation if the worktree is suitable. If existing edits prevent an honest baseline, record that instead of cleaning them up.

Checkpoint:

- Ledger records baseline truth and known dirty files.
- No source behavior has changed.

### Phase 1: Workspace Promotion

Steps:

- Move `radix/Cargo.toml` to root `Cargo.toml`.
- Move `radix/crates/radix` to `crates/radix`.
- Move `radix/crates/norma` to `crates/norma`.
- Move `radix/stdlib` to `stdlib`.
- Update docs, tests, and workflows that hard-code `radix/Cargo.toml`, `radix/crates/*`, or `radix/stdlib/*`.
- Keep command names and behavior unchanged in this phase except for file paths.
- Do not introduce `crates/faber` yet.

Checkpoint:

- `cargo test --manifest-path Cargo.toml` works from repo root.
- Existing `faber` binary behavior still works from `crates/radix`.
- No top-level `radix/` workspace container remains, unless the implementing session explicitly records a staged exception in the ledger.

### Phase 2: Node/Bun Retirement

Steps:

- Audit the remaining non-Rust active surfaces:
  - `package.json`
  - `bun.lock`
  - `index.ts`
  - `tsconfig.json`
  - `eslint.config.mjs`
  - `scripta/*.ts`
  - `runtimes/norma-ts`
- Delete obsolete TypeScript scripts and runtime files from the active repo unless the implementing session proves they remain part of the current compiler contract.
- Replace root script documentation with direct Cargo commands.
- Update AGENTS/README command guidance away from `bun run`.
- Remove Bun/Node from CI and release scripts.
- Replace Prettier/ESLint checks with either no docs formatter gate or an explicitly optional docs-only check.

Checkpoint:

- No required build, test, lint, release, or docs command depends on Bun or Node.
- `package.json`, `bun.lock`, `tsconfig.json`, and ESLint config are removed unless a documented exception is recorded.
- `rg -n "bun run|bunx|node|typescript|eslint|prettier" README.md AGENTS.md docs .github scripta` has only intentional historical references or no results.

### Phase 3: Radix Binary Extraction

Steps:

- Change `crates/radix/Cargo.toml` so the compiler-dev binary is named `radix`.
- Keep the crate library name `radix`.
- Preserve direct compiler commands:
  - `lex`
  - `parse`
  - `hir`
  - `emit`
  - `check`
  - `targets`
  - `cli-ir`
- Update tests that assume the old binary name.
- Avoid moving package policy in this phase unless it is required for compilation.

Checkpoint:

- `cargo run --manifest-path Cargo.toml -p radix --bin radix -- targets` works.
- Direct compiler inspection commands still pass their current tests.

### Phase 4: Faber Crate Scaffold

Steps:

- Add `crates/faber`.
- Add it to the workspace members.
- Make the new binary name `faber`.
- Depend on the `radix` crate by path.
- Port user-facing commands from the existing CLI:
  - `check`
  - `build`
  - `targets`
- Add placeholders or minimal implementations only for commands that are deliberately in scope for v0.34.

Checkpoint:

- `cargo run --manifest-path Cargo.toml -p faber -- check examples/exempla/salve-munde.fab` works.
- `cargo run --manifest-path Cargo.toml -p faber -- build examples/exempla/salve-munde.fab` writes the expected target output.
- `cargo run --manifest-path Cargo.toml -p faber -- targets` reports the same target capabilities as Radix.

### Phase 5: Package Ownership Move

Steps:

- Move package discovery, manifest interpretation, and local graph orchestration out of `crates/radix/src/driver/project.rs`.
- Keep Radix responsible for checking and compiling a resolved graph, not for discovering package policy.
- If a shared crate is useful, add one only when it reduces coupling. Do not add a shared crate just for appearance.
- Preserve current behavior:
  - directory input maps to `main.fab`
  - `faber.toml` supports `[paths] source` and `entry`
  - manifest dependencies are deferred until dependency resolution exists
  - package compilation remains Rust-only unless a separate phase expands it
  - package-local `@ imperia` CLI mounts continue to work

Checkpoint:

- Faber package commands preserve current passing package tests.
- Radix driver APIs no longer encode Cargo-like project policy.
- Diagnostics retain file/span attribution across package-local modules.

### Phase 6: Compatibility Aliases

Steps:

- Keep v0.33-era `faber lex`, `faber parse`, `faber hir`, `faber emit`, and `faber cli-ir`.
- Prefer running aliases in v0.34 rather than hard failures.
- If an alias prints a warning, keep it concise and avoid polluting stdout for commands whose stdout is machine-readable. Put warnings on stderr only.
- Add tests for compatibility behavior.

Checkpoint:

- Existing v0.33 docs/examples either still work or have an explicit migration path.
- Machine-readable commands still produce parseable stdout.

### Phase 7: Release and Docs Transition

Steps:

- Update root README and `crates/radix/README.md`.
- Add or update `crates/faber/README.md` if useful.
- Update `docs/grammatica/cli.md` and `docs/grammatica/targets.md`.
- Update release workflow to package both `faber` and `radix` binaries, or explicitly document why only `faber` ships.
- Update v0.34 release notes draft to explain:
  - v0.33 remains valid
  - Faber is the project tool
  - Radix is the compiler/dev tool
  - old Faber compiler-inspection commands remain temporary compatibility aliases

Checkpoint:

- Docs no longer imply that Faber and Radix are the same layer.
- Release workflow artifact names are unambiguous.

### Phase 8: Validation and Polish

Steps:

- Run Rust formatting, tests, lint, and release build:

```bash
cargo fmt --manifest-path Cargo.toml --all -- --check
cargo test --manifest-path Cargo.toml
cargo clippy --manifest-path Cargo.toml --all-targets --all-features -- -D warnings
cargo build --release --manifest-path Cargo.toml -p faber
cargo build --release --manifest-path Cargo.toml -p radix
```

- Run binary smoke tests:

```bash
cargo run --manifest-path Cargo.toml -p faber -- targets
cargo run --manifest-path Cargo.toml -p radix --bin radix -- targets
cargo run --manifest-path Cargo.toml -p faber -- check examples/exempla/salve-munde.fab
cargo run --manifest-path Cargo.toml -p radix --bin radix -- emit -t rust examples/exempla/salve-munde.fab
cargo run --manifest-path Cargo.toml -p faber -- emit -t rust examples/exempla/salve-munde.fab
```

- Review `--help` output for both binaries.
- Run a release packaging dry-run where feasible.

Checkpoint:

- Full validation gate passes.
- Both binaries have coherent help text.
- The factory ledger records completed phases and commits.

## Non-Goals

- Do not implement registry dependency resolution in this split.
- Do not add lockfiles in this split unless package ownership move requires a placeholder.
- Do not expand package compilation beyond Rust in this split.
- Do not remove v0.33 compatibility aliases in v0.34.
- Do not make Radix blind to module graphs or stdlib metadata.
- Do not rename Norma.
- Do not keep Bun/Node merely as a wrapper around Cargo.

## Open Questions for the Implementing Session

- Should v0.34 ship both `faber` and `radix` binaries in the same release archive, or ship `faber` only and keep `radix` as a source-built developer binary?
- Should `faber emit` remain a long-term expert command, or should it be scheduled for removal after one compatibility release?
- Should package graph data live in `crates/faber`, `crates/radix`, or a new crate after the first extraction pass reveals the coupling?
- Is any TypeScript runtime material still active enough to keep in-tree, or should it move to the archive/sibling history repo?

## Completion Definition

The split is complete for v0.34 when:

- `faber` is built from a Faber-owned crate or package-tool entrypoint.
- `radix` is available as the direct compiler/dev binary.
- Radix compiler APIs remain usable without package-tool policy.
- Faber package commands still support current Rust package compilation behavior.
- v0.33 command compatibility is intentionally preserved or explicitly documented.
- release docs and workflow match the binary names being shipped.
- normal development no longer requires Bun or Node.
- full validation passes.
