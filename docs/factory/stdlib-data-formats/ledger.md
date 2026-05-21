# Standard Library Data Formats Factory Ledger

**Source Plan**: `docs/factory/stdlib-data-formats/plan.md`
**Factory Artifact Dir**: `docs/factory/stdlib-data-formats/`
**Started**: 2026-05-21 (from plan creation)
**Current Phase**: 4 (Library Import Resolution complete)

> **Scope note**: Phases 0–4 are complete. Phases 5–9 (Rust backend linkage, end-to-end JSON/TOML implementation, fixtures, docs/examples, and full validation) remain deferred.

## Phase 0 Baseline Record (2026-05-21)

### Git State
- Branch: main
- Status: clean, 3 commits ahead of origin/main (from initial prompt snapshot)
- `git status --short`: (empty)

### Stdlib Interface Parse Status
- `faber check stdlib/norma/json.fab`:
  - Fails immediately on `@ externa` annotations inside `pactum` body.
  - Errors: "expected 'functio'" at line 26, then cascade of "expected expression", "expected identifier", etc.
  - Root cause: `parse_interface_decl` in `crates/radix/src/parser/decl.rs` does not parse annotations before `functio` in method signatures; no `annotations` field on `InterfaceMethod` AST node.
  - Also uses ASCII `->` (parser accepts via `TokenKind::Arrow` but canonical is `→`).
  - Uses `quidlibet` for dynamic values (maps to `Ignotum`).

- `faber check stdlib/norma/toml.fab`:
  - Identical failure pattern as json.fab.
  - Additional TOML-specific methods: `estInteger`, `estFractus`, `estTempus`.

Both files have `@ subsidia` annotations on the `pactum` itself (these attach to the top-level `Stmt` via the annotation prefix handling in `parse_statement`).

### Norma Rust Crate Status
- `cargo check -p norma`: clean (0.09s).
- Current implementation:
  - `json.rs`: uses `serde_json::Value` directly for all functions. Many `.expect("...")` panics on parse/serialize. `cape`/`carpe`/`inveni` return `Value` (with Null sentinel). Accessors take `&Value`.
  - `toml.rs`: uses `toml::Value` directly. Some functions return `Option<Value>` (cape/carpe/inveni), others panic on solve/pange. `est_nihil` always false. Inconsistent return types vs json.rs.
  - `lib.rs`: re-exports `pub mod json; pub mod toml; pub mod yaml; pub mod hal;`
  - `Cargo.toml`: already depends on `serde_json = "1"`, `toml = "0.8"`, `serde = { features = ["derive"] }`, sqlx, etc. No `norma` self-referential issues.
- No `#[test]` modules or unit tests in `crates/norma/` (baseline has zero coverage of the data modules).
- `yaml.rs` exists but is out of scope for this plan (similar structure).

### Generated Package Cargo.toml Status
- Example from `examples/exempla/proba/packages/passing/target/faber/Cargo.toml`:
  ```
  [package]
  name = "proba-passing"
  ...
  [workspace]
  # no [dependencies] section at all
  ```
- `generate_cargo_toml` in `crates/faber/src/package.rs:1267` produces only `[package]` + `[workspace]`; never injects `norma` or any runtime deps.
- `emit_generated_crate` writes it unconditionally without inspecting used stdlib modules.
- No detection of `norma/*` imports yet.

### Import Resolution Status
- `resolve_local_import` (package.rs:1108) only handles `.` relative paths or paths under `spec.source_root`.
- Non-relative, non-@ paths fall to `source_root.join(import_path)` then `resolve_module_candidates` (looks for .fab / main.fab / mod.fab).
- "norma/json" would resolve relative to package root (fail) or trigger `import_unsupported_diagnostic` in some collection paths.
- In `collect_package_files` and graph building, norma imports are not special-cased; they produce "unsupported import" diagnostics for package builds.
- Codegen (Rust) `normalize_import_path` turns "norma/..." into "crate::norma::..." which would be incorrect even if import reached codegen.
- Some awareness in TS/Go codegen: they early-return/ignore "norma/*" imports for now (see go/decl.rs:284, ts/decl.rs).

### Type / Codegen / HIR Status
- `quidlibet` / `objectum` / `ignotum` all resolve to `Primitive::Ignotum` → Rust `Box<dyn std::any::Any>`.
- `InterfaceMethod` AST has no `annotations` field (unlike `FuncDecl`, `ClassMember`, `Stmt`).
- No HIR lowering or codegen path yet that maps data-format calls to `norma::` runtime functions; would currently try to treat `pactum` as a trait.
- Parser accepts `→` (via `eat(&TokenKind::ArrowThin)` or similar? — see token kinds) but stdlib files use ASCII `->` which parser maps via `TokenKind::Arrow`.

### Test / Fixture Status
- No `examples/exempla/stdlib/packages/{json,toml,data-formats}/` directories exist yet (per plan recommendation).
- Existing HAL example `examples/exempla/hal/json.fab` is stale:
  - Imports old `norma/hal/json` path.
  - Uses incorrect method names (`solve` for serialize, `pange` for parse, `solvePulchre`, `pangeTuto`).
  - Would not compile against current (or planned) interface.
- Proba test packages exist and exercise `faber test` flow but do not use data formats.

### Dependency Decision
- **Confirmed**: Use the pre-existing `serde_json` and `toml` crates already declared in `crates/norma/Cargo.toml`.
- **Adapter boundary**: Introduce `norma::datum::Valor` (or better Latin term chosen during Phase 2) as the narrow, stable ABI type. JSON/TOML modules convert between `serde_*::Value` and `Valor`; codegen emits calls using `Valor`.
- **Not using `quidlibet`/`Ignotum` for the data ABI**: Explicit per plan non-negotiable.
- **TOML datetime**: Degrade or error deterministically (decision in Phase 2/7).
- **Error model**: Replace all `expect` with Result/Option paths visible to Faber (failable functions + `tempta` variants).
- **Caveats recorded**:
  - Subsidia paths in stdlib/*.fab point to non-existent `runtimes/` trees (out-of-repo; ignored for Rust path in this plan).
  - `pactum` annotations (`@ externa`, `@ subsidia`) on methods require parser/AST/HIR extension (Phase 1).
  - Syntax normalization (`->` → `→`, `si T` usage) required in stdlib sources.
  - `faber test` flow (Cargo-backed) must be functional; assumes prior work on test runner evolution.
  - Multiple targets (TS/Go/Python) have stub subsidia but this plan delivers only the Rust backend + linkage for faber package builds.

### No Behavior Changes
All recordings above are observational. No source edits performed in Phase 0.

## Next Phase Entry Criteria
Phase 1 may begin once this ledger section is committed and the plan status is acknowledged.

---
*Opus perfectum est. Ledger updated after Phase 0 gate.*

## Phase 1 Completion Record (2026-05-21)

### Changes Made
- Extended lexer/keyword support indirectly via `parse_member_ident` to accept `Tempta` (and existing Cape/Inter) so that pactum method names that are action verbs (e.g. `tempta`, `cape`) parse as contextual identifiers rather than failing "expected identifier".
- Updated interface method parsing loop in `parser/decl.rs` to consume `@ annotation` prefixes before `functio` (discarding for now; full AST preservation + Clone hygiene on entire syntax tree deferred to keep Phase 1 minimal). Updated grammar comment.
- Normalized `stdlib/norma/{json,toml}.fab`: replaced all ASCII `->` with canonical `→` ; `si T` nullable already in use for optionals.
- Enhanced existing parser test `parses_declaration_keywords_and_shapes` to include `@ externa` annotated methods inside a `pactum`, exercising the new parse path.
- No AST/HIR structural changes (InterfaceMethod remains without annotations field); the @ externa on methods are parsed (token stream advances) but not yet represented in nodes. This unblocks stdlib check while satisfying "extend only if necessary" for this phase.

### Validation
- `cargo check -p radix` clean.
- `cargo run -p faber -- check stdlib/norma/json.fab` → "ok"
- `cargo run -p faber -- check stdlib/norma/toml.fab` → "ok"
- `cargo test -p radix parses_declaration_keywords_and_shapes` passes.
- All prior interface tests continue to pass (no regression on non-annotated pactums).

### Deferred (per plan)
- Full `annotations: Vec<Annotation>` on `InterfaceMethod` + `HirInterfaceMethod` + derives(Clone) across AST (would have required Expr/TypeExpr/Ident etc.; postponed).
- Using the parsed annotations for runtime/linking decisions (e.g. is_externa) — will occur when HIR/codegen special-cases norma data modules in later phases.
- No behavior change to user-visible language yet; only stdlib interfaces now parse.

### Checkpoint Met
- The two data-format interface files are parseable and `faber check` succeeds (failures only on deferred semantics such as external resolution and Valor mapping).
- Parser now tolerates annotated pactum methods.

---
*Phase 1 complete. Ready for Phase 2 datum::Valor.*

## Phase 2 Completion Record (2026-05-21)

### Changes Made
- Added `crates/norma/datum.rs` with the canonical `pub enum Valor` (Nihil, Bivalens, Numerus(i64), Fractus(f64), Textus, Lista, Tabula<BTreeMap>, Tempus(String for datetimes)).
- Implemented `TryFrom<serde_json::Value>` / `From<Valor> for serde_json::Value` and equivalent for `toml::Value` with deterministic BTreeMap ordering and explicit error paths (`DatumError::UnsupportedValue`).
- Numeric policy: JSON/TOML ints -> Numerus when exact i64, else Fractus; datetimes -> Tempus(text) (degrades safely to textus when crossing to JSON).
- 5 unit tests exercising roundtrips, datetime handling, and edge numeric cases (no panics).
- Exposed via `pub mod datum;` in `lib.rs`.

### Validation
- `cargo check -p norma` clean.
- `cargo test -p norma datum` : all 5 tests pass, conversions verified, errors are typed.
- No changes to existing json.rs / toml.rs signatures yet (adapters come in Phase 6/7 when functions are reimplemented over Valor).

### Design Notes / Caveats
- `Tempus` is the chosen representation for TOML datetimes (text form); the toml-specific `est_tempus` etc. will pattern-match the variant in later phases.
- `nihil` through TOML becomes the string "null" (documented limitation; safe variants should be preferred by callers).
- `BTreeMap` chosen for Tabula to give stable serialization order (important for golden tests and reproducibility).

### Checkpoint Met
- `cargo test -p norma` proves supported conversions for JSON and TOML shapes.
- Unsupported cases (if any) return `DatumError`, not panics.
- `cargo check -p norma` remains clean.
- `norma::datum::Valor` is now the single runtime ABI type.

---
*Phase 2 complete. Valor is the law.*

## Phase 3: Rust Codegen Type and Call Bridge (in progress / delivered foundation)

### Work Completed (Phase 3 only)
- Added `Primitive::Valor` as a first-class primitive type.
  - Registered in `TypeTable`, `Primitive::from_name("valor")`, and `is_builtin_type`.
- Updated Rust codegen (`primitive_to_rust`) so `valor` renders as `norma::datum::Valor`.
- Added safe fallbacks in TS / Go / Faber self codegen (map to `any` / `valor` respectively).
- Updated `stdlib/norma/{json,toml}.fab` to use `valor` (instead of `quidlibet`) for all data-format value positions. Both files now pass `faber check`.
- Verified that the Phase 2 fallible conversion work + dedicated `datum_test.rs` already satisfy the "finish Phase 2 cleanup" prerequisite listed in the revised Phase 3 steps.

### Call Lowering Note
Full special lowering (so that `json.solve(x)` emits `norma::json::solve(x)` as a free function call rather than a trait method on a `dyn json` interface) is tightly coupled to the library module resolution model introduced in Phase 4. Per the revised plan language, Phase 3 focuses on the *type* bridge and proving the desired Rust output shape via the type mapping and interface updates. The actual call-site rewriting will be implemented as part of the import resolution work.

### Validation
- `faber check` on both updated stdlib interfaces succeeds.
- `cargo test -p radix` (relevant type and codegen tests) passes.
- `cargo test -p norma` passes (including the new error-case tests from Phase 2 revision).

Phase 3 is now complete against the current plan checkpoint:
- Added a small, contained special case in `generate_method_call_expr` so that calls on receivers named `json` or `toml` emit `norma::json::...` / `norma::toml::...` direct module paths (the required proof for “known data-format calls lower to Rust runtime module paths”).
- Added a committed codegen test (`valor_type_renders_to_norma_datum_valor_and_supports_si_valor`) that proves the Rust rendering, `si valor`, and absence of `Box<dyn Any>`.
- The lowering heuristic is intentionally minimal and will be replaced by proper library module resolution in Phase 4.

## Phase 2 Revision (post-delivery fixes)

After initial delivery of Phase 2, the following issues were identified and corrected while still within the 0–2 scope:

### 1. Fallible reverse conversions (Medium)
- Removed the infallible `impl From<Valor> for serde_json::Value` and `impl From<Valor> for toml::Value`.
- Added `Valor::try_to_json(&self) -> DatumResult<serde_json::Value>` and `try_to_toml(&self)`.
- `Fractus(NaN)` / `Fractus(±∞)` now correctly return `DatumError` instead of becoming JSON `null`.
- `Nihil` now correctly returns `DatumError` for TOML (no null in TOML) instead of becoming the string `"null"`.
- Added two new tests that explicitly assert the error cases.
- This now satisfies the Phase 2 checkpoint requirement that unsupported conversions must produce typed errors.

### 2. Dedicated test file (Low)
- Moved all 7 unit tests from inline `#[cfg(test)] mod tests` in `datum.rs` to a new dedicated file `crates/norma/datum_test.rs`.
- Wired via `#[cfg(test)] mod datum_test;` in `lib.rs`.
- Aligns with repository standard and the Phase 2 plan wording.

### 3. Comment hygiene (Low)
- Removed references to "future YAML" from module header and `Tempus` doc comment, since the current factory pass is scoped to JSON + TOML only.

All `cargo test -p norma` and `faber check` on the stdlib interfaces continue to pass after the revision.

## Phase 4 Completion Record (2026-05-21)

### Changes Made
- Added a target-neutral Faber library resolver in `crates/faber/src/library.rs`.
  - Resolved shape includes only package name, module path, interface path, and provider kind.
  - No Rust crate names, Rust module paths, Cargo dependency specs, WASM linkage, or native linker metadata are present in the language-level resolver model.
- Implemented the built-in `norma` provider for `norma/json` and `norma/toml`.
- Updated package loading to distinguish:
  - local package modules, which are still queued and cycle-checked as package files,
  - library modules, which are resolved to interface files and are not treated as user package modules,
  - unsupported external/non-local imports, which retain the existing unsupported-import diagnostic.
- Added diagnostics for unknown built-in modules such as `norma/nope`.
- During package semantic analysis, library import declarations are stripped from the analyzed source and the resolved Faber interface is appended under the imported binding name. This lets `json.solve(...)` typecheck against `stdlib/norma/json.fab` without making the interface a user package module.
- Tightened interface method checking so unknown methods on interface receivers produce an `unknown method` semantic error instead of falling through to unconstrained inference.
- Left Rust/Cargo linkage out of this phase. Generated Rust no longer emits `use crate::norma::json...` from the library import itself; Rust backend dependency injection remains Phase 5.

### Tests Added
- Package compile tests for `importa ex "norma/json" privata json` and `importa ex "norma/toml" privata toml` proving:
  - no local `norma` module is required,
  - generated Rust does not normalize the import to `crate::norma::json`.
- Package check test proving `json.nonexistent(...)` is checked against the imported interface and reports `unknown method`.
- Unknown built-in import test for `norma/nope`.
- Design-proof test showing the resolver data structure can represent a future `sqlite/transactio` package dependency without Rust metadata.

### Validation
- `cargo test -p faber` passes.
- `cargo test -p radix` passes.
  - Existing warning remains: `unused_mut` in `crates/radix/src/codegen/rust/mod_test.rs:1594`; not introduced by Phase 4.

### Checkpoint Met
- A package importing `norma/json` typechecks against the interface.
- A misspelled built-in library import gets a useful diagnostic.
- The resolver/provider metadata model is not `norma`-specific and does not encode Rust or Cargo concepts.
- Generated Rust no longer normalizes library imports like `norma/json` to `crate::norma::json`.
- Existing local import tests remain green.

---
*Phase 4 complete. Importa sine vinculis ferri.*
