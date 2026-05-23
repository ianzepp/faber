# Requirit Package Manager Factory Plan

**Status**: design captured, not scheduled
**Created**: 2026-05-23
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/requirit-package-manager/`
**Mode**: package manager design / future implementation epic
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

`faber` is already the user-facing project and package tool, but Faber does not yet have a package manager. The current package manifest, `faber.toml`, describes project identity, source layout, and build output. Adding dependencies directly to `faber.toml` would work, but it would make the file feel more like Cargo and blur different kinds of changes in diffs.

The design direction is to split package identity, dependency intent, and resolved dependency truth into separate files:

```text
faber.toml      describes this opus
requirit.toml   declares what this opus requires
requirit.lock   records what those requirements resolved to
```

This matches the mechanical commandment: each manifest file has one job.

## Current Reality

The current implementation has useful groundwork, but no general package manager:

- `crates/faber/src/package.rs` reads `faber.toml` only.
- Package compilation currently supports Rust binary output.
- Local package loading supports intra-package imports.
- `crates/faber/src/library.rs` resolves built-in `norma/...` modules from `stdlib/norma`.
- `LibraryProviderKind` already has a `PackageDependency` variant, but it is not wired to `requirit.toml` or a resolver graph.
- Existing package tests prove `norma/json`, `norma/toml`, and `norma/hal/...` imports resolve as built-in library modules.
- `explain/requirit.md` currently documents the `requirit` test modifier, not a manifest file.

So there is a real use for this design eventually: turning implicit or built-in-only library resolution into declared package requirements. There is not yet a broad enough dependency ecosystem to require a registry-backed implementation immediately.

## Design Law

### One File, One Job

`faber.toml` describes the current opus:

```toml
[opus]
nomen = "salve"
versio = "0.1.0"
editio = "2026"

[fontes]
radix = "src"
initium = "main.fab"

[exitus]
forma = "bin"
meta = "rust"
```

`requirit.toml` describes dependency intent:

```toml
[norma.json]
version = "0.35"

[norma.toml]
version = "0.35"

[local.instrumenta]
path = "../instrumenta"

[ian.math]
git = "https://example.com/ian/math.git"
rev = "abc123"
```

`requirit.lock` describes resolved dependency truth:

```toml
[[opus]]
nomen = "norma.json"
versio = "0.35.2"
source = "registry+https://example.invalid/faber"
checksum = "sha256-..."

[[opus]]
nomen = "ian.math"
rev = "abc123"
source = "git+https://example.com/ian/math.git"
checksum = "sha256-..."
```

### Requirements Are Opuses

Each top-level table in `requirit.toml` names another opus or module-like package identity. The file itself provides the dependency namespace, so this is unambiguous:

```toml
[norma.json]
version = "0.35"
```

This means "this opus requires `norma.json`", not "configure a local manifest namespace named `norma`".

### Declared Intent Is Separate From Resolved Truth

Humans edit `requirit.toml`. Faber owns `requirit.lock`.

Changing dependency intent should not modify project identity. Updating a resolved patch version should not require changing the human-authored dependency declaration unless the requested range changes.

### Imports Should Be Explainable From Requirements

Source imports such as:

```fab
importa ex "norma/json" privata json
```

should eventually be explainable from `requirit.toml` plus built-in package policy. `faber tidy` should be able to add missing requirements and remove unused requirements once package graph inspection is implemented.

## Usefulness Assessment

### Useful Now

- Captures a clean package-manager direction before `faber.toml` grows Cargo-shaped dependency tables.
- Gives future package work a file-boundary law.
- Gives current built-in `norma/...` import behavior a migration path toward declared requirements.
- Lines up with the existing `LibraryProviderKind::PackageDependency` placeholder.

### Not Yet Needed

- There is no public Faber registry.
- Most external library imports are not resolved today.
- Path and git dependencies are not loaded as full packages.
- The compiler does not yet compile dependency packages into a linked multi-package graph.
- `requirit.lock` cannot record meaningful registry checksums until package fetching exists.

### Implementation Judgment

This is implementable incrementally, but a full package manager would be premature until there are external packages or path dependencies worth consuming.

The best first implementation should not be a registry. It should be local and mechanical:

1. parse `requirit.toml`,
2. support `path` dependencies,
3. resolve imports through declared path dependencies,
4. generate `requirit.lock` for local/path/git facts,
5. add `faber tidy`, `faber tree`, and `faber verify`.

Registry and publish flows should come later.

## Proposed File Contract

### `faber.toml`

Describes this opus. It should not contain ordinary dependency declarations.

Current implementation still uses:

```toml
[package]
name = "salve"
version = "0.1.0"
edition = "2026"

[paths]
source = "src"
entry = "main.fab"

[build]
target = "rust"
kind = "bin"
```

A later manifest vocabulary cleanup may move this toward:

```toml
[opus]
nomen = "salve"
versio = "0.1.0"
editio = "2026"

[fontes]
radix = "src"
initium = "main.fab"

[exitus]
forma = "bin"
meta = "rust"
```

That vocabulary migration is related but not required for the first `requirit.toml` implementation.

### `requirit.toml`

Declared dependency intent. Each top-level table is a dependency:

```toml
[norma.json]
version = "0.35"

[local.instrumenta]
path = "../instrumenta"

[ian.math]
git = "https://example.com/ian/math.git"
rev = "abc123"
```

Version-only shorthand may be considered later, but table form is the canonical shape because it scales to `path`, `git`, `rev`, `features`, `target`, and future capability metadata.

### `requirit.lock`

Resolved dependency facts. This file is generated by Faber and should be committed for applications.

Open policy question: should reusable libraries commit `requirit.lock`, or should locks be application-only? The first implementation can generate it consistently and document commit policy later.

## CLI Shape

Initial package-manager commands:

```bash
faber add norma.json
faber remove norma.json
faber tidy
faber tree
faber verify
faber update
```

Later commands:

```bash
faber fetch
faber vendor
faber publish
faber search
```

Build commands should consume requirements automatically:

```bash
faber check
faber build
faber test
```

## Stage Graph

| Phase | Name | Goal | Checkpoint |
|-------|------|------|------------|
| 0 | Design confirmation | Confirm the three-file contract and dependency table shape. | Plan approved. |
| 1 | Inventory | Inspect package loading, library resolver, import semantics, generated Cargo dependency injection, and package tests. | Ledger identifies exact edit sites. |
| 2 | Requirit parser | Read and validate `requirit.toml` without changing resolution behavior. | Invalid dependency specs produce diagnostics; no package graph yet. |
| 3 | Path dependency resolution | Resolve declared local path dependencies and expose their interface modules to imports. | `importa ex "local/instrumenta"` can resolve through `requirit.toml`. |
| 4 | Lockfile v0 | Generate `requirit.lock` for path and git dependency facts. | Lockfile is deterministic and tool-owned. |
| 5 | Build integration | Make `faber check`, `build`, and `test` consume the resolved requirement graph. | Package commands work without manual dependency setup. |
| 6 | Tidy and tree | Add `faber tidy` and `faber tree`. | Tidy reconciles imports; tree explains dependency graph. |
| 7 | Git dependencies | Add git fetching and cache management. | Git dependencies resolve from lockfile when present. |
| 8 | Docs and explain | Document `requirit.toml`, `requirit.lock`, package commands, and import/dependency relation. | `faber explain manifest` and related entries are current. |
| 9 | Registry design | Decide registry protocol, publish flow, auth, namespace, and checksums. | Separate registry factory plan exists. |

## Open Questions

- Should dependency identities use dotted names (`norma.json`) as canonical, slash imports (`norma/json`) in source, or one spelling everywhere?
- Should `norma` remain implicit as the standard library, or should projects eventually declare `norma` modules in `requirit.toml`?
- Should `requirit.lock` be generated for every package, or only when non-builtin dependencies exist?
- Should libraries commit `requirit.lock`, or only applications?
- Should feature/capability selection live in `requirit.toml`, source annotations, or both?
- Should package dependency resolution happen before parsing imports, or should imports drive `faber tidy` updates after parsing?
- How should target-specific runtime dependencies be represented without leaking Cargo, npm, or Go module concepts into Faber manifests?

## First Useful Slice

The first implementation worth doing is:

```toml
# requirit.toml
[local.instrumenta]
path = "../instrumenta"
```

Acceptance criteria:

- `faber check` reads `requirit.toml` when present.
- Unknown fields are rejected.
- Missing paths produce clear diagnostics.
- Local path dependencies can expose `.fab` interface modules to `importa ex "..."`
- `faber tree` can show the local dependency.
- No registry, publish, auth, or network behavior is introduced.

Anything smaller than this is mostly file parsing with no user value. Anything larger risks building a registry before Faber has enough package pressure to justify one.

---

*Design maxim: `faber.toml` is identity, `requirit.toml` is need, `requirit.lock` is resolved truth.*
