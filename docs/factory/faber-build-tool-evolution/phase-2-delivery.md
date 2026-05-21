# Phase 2 Delivery Spec: Generated Rust Crate Emission

**Parent**: faber-build-tool-evolution/plan.md
**Phase**: 2

## Interpreted Problem
Now that `BuildLayout` gives us exact paths, we must actually materialize `<pkg>/target/faber/{Cargo.toml, src/main.rs}` from a successful package compilation, instead of the legacy single `main.rs` (or `target/main.rs`).

The writer must be pure enough to test in isolation (temp dir + compile_package + emit + inspect files), and must produce a Cargo.toml that lets the generated code compile with `cargo build --manifest-path ... --target-dir target`.

## Normalized Spec (Phase 2)
- Add `emit_generated_crate(layout: &BuildLayout, rust_code: &str, meta: Option<&FaberManifest>) -> Result<PathBuf, ...>`
- Inside: `create_dir_all` for `generated_crate_root/src`, write `Cargo.toml` and `generated_rust_entry`.
- `Cargo.toml` content:
  ```toml
  [package]
  name = "<sanitized binary_name>"
  version = "<from manifest or 0.1.0>"
  edition = "2021"
  ```
- Stale handling: overwrite the two target files deliberately (no full clean of target/ to preserve Cargo incremental if any).
- In `cmd_build` package branch: after compile success, discover layout, read meta if present, call emitter, print the generated_rust_entry (or crate root) for now.
- Non-package builds unchanged.
- New unit test(s): create temp package (manifest+entry), compile_package, emit_generated_crate, assert the two files exist, contain expected strings, no "target/faber/target" created.

## Repo Baseline
- `cmd_build` lives at bottom of package.rs (~1084)
- `compile_package` already returns the full assembled crate string for packages.
- `read_manifest` available.
- We will call `discover_build_layout` (added phase 1) only for package=true cases.

## Work for Phase
1. Implement `emit_generated_crate` + `generate_cargo_toml` helper.
2. Wire minimal change inside the package success arm of `cmd_build`.
3. Add 1-2 writer-only tests that never invoke Cargo.
4. Verify `faber build <pkg>` now creates the tree under its own target/faber/ (and still no nested target).

## Checkpoint
- Package build produces inspectable `target/faber/Cargo.toml` + `src/main.rs`
- Generated files not committed
- Writer tests pass without Cargo
- Existing single-file builds and `faber check` unaffected

## Gate
- `cargo test -p faber`
- `cargo clippy -p faber -- -D warnings`
- Manual smoke: init temp pkg, faber build it, ls target/faber/ shows the two files, target/debug/ does not exist yet (Cargo not called)

**Poker-face target**: >= 90% (writer + integration point done, full Cargo run in phase 3)
