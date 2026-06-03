//! Best-effort formatters and linters for emitted target code.

/// Run the appropriate formatter for generated target code, if available.
///
/// Returns the formatted code on success, or an error message if the formatter
/// could not be executed or failed. The command layer treats those errors as
/// warnings so formatter availability never changes compiler correctness.
pub fn format_generated_code(target: crate::codegen::Target, code: &str) -> Result<String, String> {
    match target {
        crate::codegen::Target::Rust => run_formatter("rustfmt", &["--edition", "2021"], code),
        crate::codegen::Target::Go => run_formatter("gofmt", &[], code),
        crate::codegen::Target::TypeScript => {
            // Try prettier first (most common), fall back to deno fmt if available
            if let Ok(formatted) = run_formatter("prettier", &["--parser", "typescript"], code) {
                return Ok(formatted);
            }
            run_formatter("deno", &["fmt", "--ext", "ts", "-"], code)
        }
        crate::codegen::Target::Faber => {
            // The Faber emitter is already the pretty-printer.
            // In the future we can hook a dedicated `faber fmt` here if one is added.
            Ok(code.to_string())
        }
        crate::codegen::Target::WasmText | crate::codegen::Target::LlvmText => Ok(code.to_string()),
    }
}

/// Run a linter with auto-fix on generated target code where possible.
///
/// This is intentionally best-effort. If the linter is not installed or fails,
/// we return an error and the caller can decide to keep the original code.
pub fn lint_generated_code(target: crate::codegen::Target, code: &str) -> Result<String, String> {
    match target {
        crate::codegen::Target::Rust => lint_rust_code(code),
        crate::codegen::Target::Go => {
            // Go has limited auto-fix linters. For now we can run `golangci-lint --fix` if present,
            // but a simple first version is to just return the code (or run gofmt again).
            // Future: implement proper golangci-lint support.
            Ok(code.to_string())
        }
        crate::codegen::Target::TypeScript => {
            // Try biome or eslint --fix
            if let Ok(fixed) = run_formatter("biome", &["check", "--apply", "--stdin-file-path", "main.ts"], code) {
                return Ok(fixed);
            }
            run_formatter(
                "eslint",
                &[
                    "--fix-dry-run",
                    "--stdin",
                    "--stdin-filename",
                    "main.ts",
                    "--format",
                    "json",
                ],
                code,
            )
            .map(|_| code.to_string()) // eslint --fix-dry-run doesn't rewrite; real --fix needs files
        }
        crate::codegen::Target::Faber => Ok(code.to_string()),
        crate::codegen::Target::WasmText | crate::codegen::Target::LlvmText => Ok(code.to_string()),
    }
}

/// Rust-specific linter using a temporary Cargo project + clippy --fix.
/// This is the most powerful auto-fix we can currently offer.
fn lint_rust_code(code: &str) -> Result<String, String> {
    use std::fs;
    use std::process::Command;

    // Create a unique temp directory (similar to make_temp_root)
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("radix-lint-{}", nanos));
    let src_dir = temp_dir.join("src");

    fs::create_dir_all(&src_dir).map_err(|e| format!("failed to create temp src dir: {e}"))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, code).map_err(|e| format!("failed to write temp main.rs: {e}"))?;

    let cargo_toml = temp_dir.join("Cargo.toml");
    fs::write(
        &cargo_toml,
        "[package]\nname = \"lint-target\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .map_err(|e| format!("failed to write Cargo.toml: {e}"))?;

    // Run cargo clippy --fix (best effort)
    let output = Command::new("cargo")
        .args([
            "clippy",
            "--fix",
            "--allow-dirty",
            "--allow-staged",
            "--allow-no-vcs",
            "--quiet",
            "--",
            "-D",
            "warnings",
        ])
        .current_dir(&temp_dir)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let fixed = fs::read_to_string(&main_rs).map_err(|e| format!("failed to read fixed code: {e}"))?;
            let _ = fs::remove_dir_all(&temp_dir);
            Ok(fixed)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let _ = fs::remove_dir_all(&temp_dir);
            Err(format!("cargo clippy --fix exited with status {}: {stderr}", output.status))
        }
        Err(e) => {
            let _ = fs::remove_dir_all(&temp_dir);
            Err(format!("failed to run cargo clippy: {e} (is clippy installed?)"))
        }
    }
}

/// Helper to invoke an external formatter via stdin/stdout.
fn run_formatter(cmd: &str, args: &[&str], input: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not spawn {cmd}: {e} (is it installed?)"))?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "failed to open stdin".to_string())?;
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("failed to write to {cmd} stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to wait for {cmd}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{cmd} failed: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
