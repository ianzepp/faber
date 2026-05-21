# Standard Library Data Formats Factory Plan

**Status**: in-progress (Phases 0–2 complete; 3–9 deferred per owner directive)
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/stdlib-data-formats/`
**Depends On**: generated Rust crate layout, Cargo-backed `faber test`, stdlib interface files in `stdlib/norma`, Rust runtime crate `crates/norma`
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber needs first-class standard library support for common data formats:

- JSON for ubiquitous structured data interchange.
- TOML for project and tool configuration.

This work must not be a one-off patch around two modules. It should establish the reusable pattern for future standard library modules that have:

1. A Fabra-facing interface in `stdlib/norma`.
2. A Rust implementation in `crates/norma`.
3. Compiler knowledge for resolving stdlib imports.
4. Build-tool knowledge for linking the Rust runtime into generated packages.
5. Fabra tests that prove compiled programs can call the runtime backend.

The first implementation should prefer existing Rust crates for the actual format parsers and serializers, while isolating Faber from crate churn through a narrow runtime adapter boundary.

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
- `docs/grammatica/test.md`

Known blockers at plan creation:

- `pactum` methods do not currently parse annotations such as `@ externa`, but the stdlib interface files use them.
- Several stdlib files use ASCII `->` in places where the canonical grammar uses `→`; phase work must normalize syntax rather than relying on parser accidents.
- `quidlibet` maps to `Box<dyn std::any::Any>` in Rust codegen, while the current data-format runtime functions expect concrete crate values such as `serde_json::Value` and `toml::Value`.
- Generated package `Cargo.toml` does not inject a `norma` runtime dependency.
- Package import discovery currently focuses on local package files; stdlib imports need explicit resolution and linking semantics.

## Non-Negotiable Runtime Contract

Faber values must not leak backend crate-specific dynamic value types into the language ABI.

The standard library runtime should expose a single canonical data value representation, likely:

```rust
norma::datum::Valor
```

The exact name may change if the implementation finds a better Latin term, but the contract is:

- one runtime type represents structured dynamic data for JSON, TOML, database rows, and future data-oriented stdlib modules,
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

## Import and Linking Contract

Generated Rust packages must compile without manual edits when they import standard library modules.

Target behavior:

```fab
importa ex "norma/json" privata json
importa ex "norma/toml" privata toml
```

The compiler and build tool should:

- resolve `norma/*` imports against the configured stdlib root,
- typecheck against the `.fab` interface files,
- avoid copying stdlib implementation source into user packages unless that becomes an explicit design,
- emit Rust references to the `norma` runtime crate,
- inject `norma = { path = "<repo>/crates/norma" }` into generated `target/faber/Cargo.toml` when used,
- keep generated crate artifacts under the existing package-level `target/` contract.

This is the reference pattern for later runtime-backed stdlib modules.

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
| 3 | Rust codegen type and call bridge | Teach Radix how Faber stdlib data-format types map to Rust runtime calls. | A tiny generated Rust program can call a `norma` data-format function without manual edits. |
| 4 | Stdlib import resolution | Resolve `norma/*` imports through stdlib roots and typecheck against interface files. | A package importing `norma/json` sees typed functions and no unresolved identifiers. |
| 5 | Generated Cargo dependency injection | Inject runtime dependencies into generated package manifests based on used stdlib modules. | `target/faber/Cargo.toml` contains `norma` only when needed and package builds link successfully. |
| 6 | JSON reference implementation | Implement JSON end to end as the canonical example module. | Fabra package tests parse, inspect, serialize, and safely reject invalid JSON. |
| 7 | TOML implementation | Implement TOML on the same ABI and document TOML-specific constraints. | Fabra package tests parse config tables, serialize tables, and reject invalid/root-invalid TOML. |
| 8 | Docs and examples | Update grammar/stdlib docs and stale examples to match the shipped API. | README/docs/examples use canonical import paths and current function names. |
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

- Add or identify the Fabra-facing type name for standard data values.
- Map that type to `norma::datum::Valor` in Rust codegen.
- Ensure function signatures can pass and return `Valor`, `Option<Valor>`, `String`, numbers, booleans, lists, and maps.
- Avoid expanding `quidlibet` into the data-format ABI unless explicitly required.
- Add codegen tests for Rust type rendering and calls.

Checkpoint:

- Generated Rust uses `norma::datum::Valor` for the chosen Fabra data type.
- No generated `Box<dyn Any>` appears in JSON/TOML stdlib call signatures.

### Phase 4: Stdlib Import Resolution

Steps:

- Define stdlib import path rules for `norma/*`.
- Teach package loading or analysis to include interface files for typechecking without treating them as user package modules.
- Preserve module alias behavior:
  - `importa ex "norma/json" privata json`
  - `json.solve(...)`
- Ensure relative imports and local package imports keep their existing behavior.
- Add diagnostics for unknown stdlib modules.

Checkpoint:

- A package importing `norma/json` typechecks.
- A misspelled stdlib import gets a useful diagnostic.
- Existing local import tests remain green.

### Phase 5: Generated Cargo Dependency Injection

Steps:

- Detect when generated Rust uses the `norma` runtime crate.
- Emit deterministic dependency entries in generated `target/faber/Cargo.toml`.
- Keep the existing `[workspace]` generated-crate isolation.
- Avoid adding every possible stdlib dependency unconditionally unless the implementation explicitly chooses a simple first-pass policy and documents it.
- Add writer tests for generated manifest contents.

Checkpoint:

- Generated package manifests include `norma` when data-format modules are used.
- Packages without runtime-backed stdlib imports do not gain unnecessary dependencies, unless a deliberate all-runtime policy is documented.
- `cargo build --manifest-path target/faber/Cargo.toml --target-dir target` succeeds for a fixture.

### Phase 6: JSON Reference Implementation

Steps:

- Make JSON the reference end-to-end module.
- Implement parser/serializer/access helpers through `Valor`.
- Replace panics with language-visible failures.
- Add Fabra package fixture tests for:
  - parsing object text,
  - reading string/number/bool/null values,
  - nested path access,
  - serializing compact JSON,
  - serializing pretty JSON if retained,
  - invalid input through the safe parse function.

Checkpoint:

- `faber test examples/exempla/stdlib/packages/json` exits `0`.
- Generated Rust links `norma` and calls JSON runtime functions.
- Runtime Rust tests and compiler tests both pass.

### Phase 7: TOML Implementation

Steps:

- Implement TOML parse/serialize through `Valor`.
- Enforce TOML root table behavior explicitly.
- Decide and document datetime behavior.
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
- Document the runtime-backed stdlib implementation contract for future modules.
- Add notes to grammar docs if interface annotations or stdlib imports changed.

Checkpoint:

- No docs point to `norma/hal/json` if the canonical path is `norma/json`.
- Examples compile or are clearly marked as future/non-runnable.
- Future stdlib modules have a documented pattern to follow.

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

- Inspect generated manifests and generated Rust for expected stdlib linkage.
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

- JSON and TOML are accessible from compiled Faber packages through canonical stdlib imports.
- Generated Rust packages link the required runtime crate automatically.
- Runtime failures are surfaced through Faber semantics rather than Rust panics.
- Fabra package tests prove both modules work end to end.
- Docs and examples match the shipped API.
- The pattern is documented well enough to implement the next runtime-backed stdlib module without rediscovering the architecture.
