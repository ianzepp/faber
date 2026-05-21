# Phase 3 Delivery Spec: Cargo Backend Invocation

**Phase**: 3

## Problem
We have the generated crate on disk. Now invoke Cargo so that `faber build` actually produces the executable in the correct peer directory `target/debug/<name>` (or release).

Must:
- Run `cargo build --manifest-path <pkg>/target/faber/Cargo.toml --target-dir <pkg>/target`
- Forward stdout/stderr or at least preserve useful output
- Exit nonzero and report on Cargo failure
- On success, print the final binary path (or clear success)
- Never cause Cargo to create target/faber/target (by using --target-dir correctly)

## Spec
- Add a `invoke_cargo_build(layout: &BuildLayout, release: bool) -> Result<PathBuf, ...>` that returns the binary path on success.
- Use `std::process::Command::new("cargo")`
  .arg("build")
  .arg("--manifest-path").arg(&layout.generated_cargo_manifest)
  .arg("--target-dir").arg(&layout.cargo_target_dir)
  .args( if release { vec!["--release"] } else { vec![] } )
- Inherit stdout/stderr or capture and replay on error.
- On success return the computed debug or release binary path.
- In cmd_build, after emit_generated_crate, call invoke, print the binary path on success.

## Tests / Verification
- After phase, `faber build /tmp/pkg` must leave an executable at target/debug/<sanitized-name>
- `file` or `ls -l` shows it's a binary (or run it)
- No target/faber/target/ directory appears
- Cargo errors (e.g. bad code) surface to user

## Gate
Smoke in plan: build produces the binary artifact.

This phase delivers the "build" in "faber build".
