# Standard Library Data Formats Factory Plan

**Status**: in-progress (Phases 0–4 complete; 5–9 deferred)
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/stdlib-data-formats/`
**Depends On**: generated Rust crate layout, Cargo-backed `faber test`, stdlib interface files in `stdlib/norma`, Rust runtime crate `crates/norma`
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber needs first-class standard library support for common data formats:

- JSON for ubiquitous structured data interchange.
- TOML for project and tool configuration.

This work must not be a one-off patch around two modules. It should establish the reusable pattern for future Faber libraries that have:

1. A Fabra-facing interface in `stdlib/norma`.
2. A target-specific backend implementation, with Rust in `crates/norma` as the first backend.
3. Compiler knowledge for resolving Faber library imports.
4. Target-backend knowledge for linking or embedding the implementation required by that target.
5. Fabra tests that prove compiled programs can call the runtime backend.

The first Rust implementation should prefer existing Rust crates for the actual format parsers and serializers, while isolating Faber from crate churn through a narrow target-backend adapter boundary.

## Current Baseline

Relevant files already exist:

- `stdlib/norma/json.fab`
- `stdlib/norma/toml.fab`
- `crates/norma/json.rs`
- `crates/norma/toml.rs`
- `crates/norma/Cargo.toml`
- `crates/faber/src/package.rs`
- `crates/faber/src/main.rs`
- `crates/radix/src/parser/decl.rs`
- `crates/radix/src/codegen/rust/`
- `explain/proba.md` and `explain/probandum.md`

Known blockers at plan creation:

- `pactum` methods do not currently parse annotations such as `@ externa`, but the stdlib interface files use them.
- Several stdlib files use ASCII `->` in places where the canonical grammar uses `→`; phase work must normalize syntax rather than relying on parser accidents.
- `quidlibet` maps to `Box<dyn std::any::Any>` in Rust codegen, while the current data-format runtime functions expect concrete crate values such as `serde_json::Value` and `toml::Value`.
- Generated Rust package `Cargo.toml` does not inject a `norma` runtime dependency.
- Package import discovery currently focuses on local package files; library imports need explicit target-neutral resolution, with target-specific linking handled later.

## Non-Negotiable Runtime Contract

Faber values must not leak backend crate-specific dynamic value types into the language ABI.

The standard library runtime should expose a single canonical data value representation, likely:

```rust
norma::datum::Valor
```

The exact name may change if the implementation finds a better Latin term, but the contract is:

- one runtime type represents structured dynamic data for JSON, TOML, database rows, and future data-oriented Faber library modules,
- JSON/TOML implementations convert between that runtime type and their parser crate's native value type,
- Faber codegen maps the Fabra stdlib data type to this runtime type,
- `quidlibet` remains a general escape hatch and should not become the implicit format-value ABI.

Required value space:

- `nihil`
- `bivalens`
- `numerus`
- `fractus`
- `textus`
- `lista<Valor>`
- `tabula<textus, Valor>`
- a representation for TOML datetime, either as a tagged runtime value or a documented text conversion

The implementation must choose explicit behavior for numeric boundaries, map key ordering, TOML datetime values, and unsupported values. Unsupported cases should produce deterministic errors, not panics.

## Error Contract

Runtime parser and serializer failures must be language-visible and testable.

- Failable functions should integrate with Faber's existing error path instead of panicking in Rust.
- Safe variants should return `si Valor` or another explicit optional/result shape.
- Runtime functions in `crates/norma` should not use `expect` for user input or serialization failures.
- The compiler should preserve enough information for generated Rust to compile without ad hoc unwraps.

Initial naming can preserve the existing Latin verbs:

- `pange` for composing/serializing a value to text.
- `solve` for parsing text into a value.
- `tempta` for parse-if-valid behavior.
- `cape`, `carpe`, and `inveni` for access helpers if they remain part of the public API.

## Library Import and Backend Contract

Faber library import resolution must be target-neutral. A library import resolves to a Faber interface module and provider identity, not to a Rust crate, Cargo dependency, WASM component, object file, or native symbol. `norma` is the first built-in provider, but the architecture must leave room for future external Faber libraries such as SQLite wrappers and for non-Rust compilation targets.

Target behavior:

```fab
importa ex "norma:json" privata json
importa ex "norma:toml" privata toml
```

Future non-builtin behavior should use the same conceptual path:

```fab
importa ex "sqlite" privata sqlite
importa ex "sqlite/transactio" privata transactio
```

The compiler should treat both examples as library imports:

- resolve import specifiers through a library resolver,
- typecheck against Faber `.fab` interface files,
- avoid copying library implementation source into user packages unless that becomes an explicit design,
- preserve the provider and module identity needed by target backends,
- keep target-specific implementation details out of library syntax and language-level import resolution.

Target backends then consume the resolved Faber library identity and decide how to link or embed the implementation for that target:

- the Rust backend may lower `norma:json` to `norma::json` and inject a Cargo dependency,
- a WASM backend may lower it to a component import, runtime module, or builtin section,
- a native backend may lower it to an object/library dependency, intrinsic, or native symbol table entry.

For the current data-format work, the only implemented provider should be the built-in `norma` provider:

```text
specifier: norma:json
provider: builtin
interface: <repo>/stdlib/norma/json.fab
```

A future SQLite package should fit the same language-level resolved shape:

```text
specifier: sqlite
provider: package dependency
interface: <package-cache-or-path>/sqlite.fab
```

Faber should typecheck against Faber interfaces, not Rust APIs. Cargo dependency specs, Rust module paths, WASM imports, and native linker inputs belong to target-backend linkage metadata, not to Faber import semantics.

## Test Fixture Contract

The implementation must add repeatable Fabra package tests before any phase is called complete.

Recommended fixture root:

```text
examples/exempla/stdlib/packages/
├── json/
├── toml/
└── data-formats/
```

Each fixture is a normal Faber package:

```text
<fixture>/
├── faber.toml
└── src/
    └── main.fab
```

Tests should run through:

```bash
cargo run -p faber -- test examples/exempla/stdlib/packages/json
cargo run -p faber -- test examples/exempla/stdlib/packages/toml
cargo run -p faber -- test examples/exempla/stdlib/packages/data-formats
```

The tests should prove compiled Faber code calls the Rust runtime backend, not only compiler unit tests or direct Rust runtime unit tests.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline and dependency decision | Capture current parser, runtime, and build status. | Ledger records current failures and crate choices; no behavior changed. |
| 1 | Stdlib interface parsing | Make stdlib `.fab` interface files valid, parseable, and checkable. | `faber check stdlib/norma/json.fab` and `toml.fab` succeed or fail only on intentionally deferred semantics. |
| 2 | Canonical runtime data value | Add the shared `norma::datum::Valor` ABI and conversion helpers. | Rust tests prove conversion to/from JSON/TOML backend values for supported shapes. |
| 3 | Rust codegen type and call bridge | Teach Radix how Faber data-format values and library-backed module calls map to Rust runtime code. | Generated Rust uses `norma::datum::Valor` and runtime module calls for known data-format symbols, with no `Box<dyn Any>` in the ABI. |
| 4 | Library import resolution | Introduce a target-neutral Faber library resolver and implement `norma` as the first built-in provider. | `norma:json` resolves through Faber provider metadata; the model can also describe a future external SQLite package without assuming Rust. |
| 5 | Rust backend linkage | Teach the Rust backend how resolved Faber libraries map to Rust modules and Cargo dependencies. | `target/faber/Cargo.toml` contains `norma` when built-in data-format modules are used, without putting Cargo knowledge in language-level import resolution. |
| 6 | JSON reference implementation | Implement JSON end to end as the canonical example module. | Fabra package tests parse, inspect, serialize, and safely reject invalid JSON. |
| 7 | TOML implementation | Implement TOML on the same ABI and document TOML-specific constraints. | Fabra package tests parse config tables, serialize tables, and reject invalid/root-invalid TOML. |
| 8 | Docs and examples | Update grammar/library docs and stale examples to match the shipped API and provider model. | README/docs/examples use canonical import paths and document the library interface/backend pattern. |
| 9 | Full validation and release readiness | Run compiler, runtime, package, and fixture gates. | `scripta/ci` plus all stdlib package tests pass; plan status can move to complete. |

## Phase Details

### Phase 0: Baseline and Dependency Decision

Steps:

- Inspect `git status --short`.
- Record current `faber check` behavior for `stdlib/norma/json.fab` and `toml.fab`.
- Record current `cargo check -p norma` behavior.
- Record current generated `Cargo.toml` behavior for a package with no stdlib runtime dependency.
- Create `docs/factory/stdlib-data-formats/ledger.md` if implementation spans multiple turns.

Checkpoint:

- Current truth is recorded.
- Dependency decision and caveats are explicit.
- No source behavior changed in this phase.

### Phase 1: Stdlib Interface Parsing

Steps:

- Confirm the intended grammar for annotations on `pactum` methods.
- Extend parser and AST only if the language should support method annotations inside interfaces.
- Preserve annotations through HIR if they carry runtime/linking meaning.
- Normalize stdlib interface syntax to canonical Faber:
  - type-first parameters,
  - `→` return arrows,
  - `si T` nullable syntax,
  - no invented `T?` syntax unless the grammar already supports it and docs say it is canonical.
- Ensure parse errors are precise for malformed stdlib interfaces.

Checkpoint:

- The two data-format interface files are parseable.
- Existing interface parsing tests still pass.
- New parser tests cover annotated `pactum` methods.

### Phase 2: Canonical Runtime Data Value

Steps:

- Add a focused runtime module, likely `crates/norma/datum.rs`.
- Define `Valor` and helper constructors/accessors.
- Add conversion modules or traits for:
  - `serde_json::Value`,
  - `toml::Value` / `toml::Table`.
- Decide how TOML datetime values degrade or error.
- Add Rust unit tests in dedicated test files or focused crate tests, following repo standards.

Checkpoint:

- `cargo test -p norma` proves supported conversions.
- Unsupported conversions return typed errors, not panics.
- Existing `cargo check -p norma` remains clean.

### Phase 3: Rust Codegen Type and Call Bridge

Steps:

- Finish any Phase 2 ABI cleanup before depending on it:
  - `Valor` to backend-value conversions should be fallible where unsupported values exist,
  - tests should live in the crate's normal test layout rather than making the runtime file the long-term pattern.
- Add or identify the Fabra-facing type name for standard data values. Preferred public spelling for this plan is `valor`.
- Update JSON/TOML interface signatures to use `valor` rather than `quidlibet` for parse/serialize/access APIs.
- Map `valor` to `norma::datum::Valor` in Rust codegen.
- Ensure function signatures can pass and return `norma::datum::Valor`, `Option<norma::datum::Valor>`, `String`, numbers, booleans, lists, and maps.
- Avoid expanding `quidlibet` into the data-format ABI unless explicitly required for a separate dynamic-host interop path.
- Define the codegen shape for library-backed module calls before import resolution is fully general:
  - `json.solve(...)` should lower to a Rust runtime module call such as `norma::json::solve(...)`,
  - `toml.solve(...)` should lower similarly,
  - these calls must not lower as Rust trait dispatch from `pactum json` / `pactum toml`.
- Keep this phase focused on type/call meaning. It may use targeted compiler tests or hand-assembled HIR/module metadata rather than a full package import path.
- Add codegen tests for:
  - `valor` type rendering,
  - function signatures using `valor` and `si valor`,
  - generated Rust call paths for known data-format runtime calls,
  - absence of `Box<dyn Any>` in JSON/TOML stdlib signatures.

Checkpoint:

- Generated Rust uses `norma::datum::Valor` for the `valor` data type.
- JSON/TOML `.fab` interfaces no longer expose `quidlibet` as the primary data-format value type.
- Known data-format calls lower to Rust runtime module paths, not Rust traits or unresolved local modules.
- No generated `Box<dyn Any>` appears in JSON/TOML stdlib call signatures.

### Phase 4: Library Import Resolution

Steps:

- Introduce a general library import resolution layer. Implement only the built-in `norma` provider in this phase, but keep the model suitable for future external Faber packages.
- Define a resolved library module shape that separates:
  - Faber package name, e.g. `norma` or future `sqlite`,
  - module path, e.g. `json`, `toml`, `transactio`,
  - interface file path for Faber typechecking,
  - provider kind, initially `builtin`.
- Do not include Rust crate names, Rust module paths, Cargo dependency specs, WASM linkage, native object paths, or other target-specific implementation details in the language-level resolved import shape.
- Represent import resolution as a distinction between local package modules and library modules, for example:
  - local relative imports keep the existing package source behavior,
  - `norma:json` and `norma:toml` become library module resolutions,
  - future dependency imports such as `sqlite` or `sqlite/transactio` should fit the same data structure.
- Centralize provider-specific string checks. `norma/` may be recognized inside the built-in provider, but `norma/` checks should not be scattered across package loading, resolver, codegen, and target backend linkage.
- Teach package loading or analysis to include library interface files for typechecking without treating them as user package modules.
- Preserve module alias behavior:
  - `importa ex "norma:json" privata json`
  - `json.solve(...)`
- Record enough target-neutral provider metadata for later phases:
  - generated Rust can ask its own backend registry how `norma:json` should lower,
  - future WASM or native backends can provide their own linkage decisions,
  - unsupported targets can produce targeted diagnostics without changing import syntax.
- Add diagnostics for unknown built-in library modules, such as `norma/nope`.
- Add a small design-proof test or fixture for the future package model. It does not need to implement SQLite, but should prove the resolved module shape can represent:
  - package `sqlite`,
  - module path such as `transactio`,
  - provider kind,
  - interface path.
- Ensure relative imports and local package imports keep their existing behavior.

Checkpoint:

- A package importing `norma:json` or `norma:toml` typechecks against the interface.
- A misspelled built-in library import gets a useful diagnostic.
- The resolver/provider metadata model is not `norma`-specific, does not encode Rust or Cargo concepts, and can describe a future external SQLite wrapper package.
- Generated Rust no longer normalizes library imports like `norma:json` to `crate::norma::json`.
- Existing local import tests remain green.

### Phase 5: Rust Backend Linkage

Steps:

- Consume target-neutral resolved library module identities from Phase 4.
- Add a Rust-backend linkage registry or equivalent mapping that is explicitly outside language-level import resolution.
- For built-in `norma`, map the resolved Faber library modules to Rust backend paths:
  - `norma:json` -> `norma::json`,
  - `norma:toml` -> `norma::toml`.
- For built-in `norma`, emit a deterministic path dependency in generated `target/faber/Cargo.toml`.
- Keep the Rust backend dependency representation broad enough for future external libraries:
  - crates.io package/version,
  - local path,
  - git/rev,
  - feature flags.
- Keep the existing `[workspace]` generated-crate isolation.
- Avoid adding every possible library dependency unconditionally unless the implementation explicitly chooses a simple first-pass policy and documents it.
- Do not implement Cargo dependency solving in Faber. The Rust backend copies normalized Rust dependency specs; Cargo resolves transitive Rust dependencies.
- Add writer tests for generated manifest contents.

Checkpoint:

- Generated package manifests include `norma` when built-in data-format modules are used.
- Packages without runtime-backed library imports do not gain unnecessary dependencies, unless a deliberate all-runtime policy is documented.
- A fake external Rust backend dependency spec can be represented without teaching Faber language-level import resolution how to fetch/build that crate itself.
- `cargo build --manifest-path target/faber/Cargo.toml --target-dir target` succeeds for a fixture.

### Phase 6: JSON Reference Implementation

Steps:

- Make JSON the reference end-to-end module.
- Implement parser/serializer/access helpers through `Valor`.
- Replace panics with language-visible failures.
- Route calls through the Phase 3/4 library-backed call path, not special JSON-only codegen.
- Use `norma:json` as the first proof that a Faber interface can be resolved target-neutrally and then backed by the Rust adapter for the Rust target.
- Add Fabra package fixture tests for:
  - parsing object text,
  - reading string/number/bool/null values,
  - nested path access,
  - serializing compact JSON,
  - serializing pretty JSON if retained,
  - invalid input through the safe parse function.

Checkpoint:

- `faber test examples/exempla/stdlib/packages/json` exits `0`.
- Generated Rust links `norma`, calls JSON runtime functions through the library backend metadata, and does not require manual Cargo edits.
- Runtime Rust tests and compiler tests both pass.

### Phase 7: TOML Implementation

Steps:

- Implement TOML parse/serialize through `Valor`.
- Enforce TOML root table behavior explicitly.
- Decide and document datetime behavior.
- Use the same target-neutral library resolution and Rust-backend linkage path proven by JSON, not TOML-specific resolver hooks.
- Add Fabra package fixture tests for:
  - project-style config parsing,
  - nested table access,
  - arrays,
  - serialization of a table,
  - invalid TOML input,
  - non-table root handling if the backend can represent it.

Checkpoint:

- `faber test examples/exempla/stdlib/packages/toml` exits `0`.
- TOML-specific constraints are documented and tested.

### Phase 8: Docs and Examples

Steps:

- Update stdlib documentation for data formats.
- Update stale examples under `examples/exempla/hal/` or move them to the canonical stdlib example location.
- Document import paths and function names.
- Document the target-neutral Faber library interface contract for future modules.
- Include a short future-package example, such as SQLite:
  - Faber interface package,
  - target-neutral provider metadata,
  - example Rust adapter crate as one backend,
  - example Cargo dependency spec copied by the Rust backend only.
- State explicitly that Faber typechecks Faber interfaces; Cargo owns Rust dependency resolution only when the Rust backend is selected.
- Add notes to grammar docs if interface annotations or library imports changed.

Checkpoint:

- No docs point to `norma/hal/json` if the canonical path is `norma:json`.
- Examples compile or are clearly marked as future/non-runnable.
- Future Faber library modules have a documented pattern to follow.

### Phase 9: Full Validation and Release Readiness

Steps:

- Run the normal repo gates:

```bash
./scripta/ci
```

- Run focused package fixtures:

```bash
cargo run -p faber -- test examples/exempla/stdlib/packages/json
cargo run -p faber -- test examples/exempla/stdlib/packages/toml
cargo run -p faber -- test examples/exempla/stdlib/packages/data-formats
```

- Inspect generated manifests and generated Rust for expected Rust-backend stdlib linkage.
- Ensure no generated `target/` contents are committed.
- Update the ledger and mark this plan complete only after all phase checkpoints are actually met.

Checkpoint:

- Full validation passes.
- The implementation is a reusable stdlib pattern, not a format-specific exception.

## Companion Checks

Use these review passes before completing major phases:

- **Clean-break check**: remove any temporary compatibility names from stale examples unless they are intentionally supported.
- **Consequences check**: verify new import/linking rules do not break local package imports.
- **Poker-face check**: compare actual behavior against this plan before marking a phase complete.
- **Security check**: review parser behavior for untrusted JSON and TOML input risks.

## Completion Definition

This plan is complete only when:

- JSON and TOML are accessible from compiled Faber packages through canonical library imports.
- Faber library resolution remains target-neutral and does not encode Rust or Cargo concepts.
- Generated Rust packages link the required runtime crate automatically when the Rust backend is selected.
- Runtime failures are surfaced through Faber semantics rather than Rust panics.
- Fabra package tests prove both modules work end to end.
- Docs and examples match the shipped API.
- The pattern is documented well enough to implement the next runtime-backed Faber library, including an external package such as SQLite and backend-specific implementations, without rediscovering the architecture.
