# Standard Library Data Formats Factory Ledger

**Source Plan**: `docs/factory/stdlib-data-formats/plan.md`
**Factory Artifact Dir**: `docs/factory/stdlib-data-formats/`
**Started**: 2026-05-21 (from plan creation)
**Current Phase**: 0 (Baseline)

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
