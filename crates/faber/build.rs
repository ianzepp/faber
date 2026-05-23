//! Build-time embedding for the `faber explain` reference corpus.
//!
//! The runtime command should not depend on the source tree being present, so
//! Markdown entries from `explain/` are converted into a deterministic Rust
//! slice during the Cargo build.
//!
//! This build script deliberately performs only filesystem collection and Rust
//! literal generation. Corpus parsing, schema validation, and cross-reference
//! checks stay in `explain.rs`, where tests can exercise the same code path the
//! CLI uses and diagnostics can name the original Markdown file.
//!
//! BUILD CONTRACT
//! ==============
//! - Rebuild when the corpus directory or any Markdown entry changes.
//! - Emit entries in filename order for deterministic binary output.
//! - Allow a missing corpus directory so partial source distributions still
//!   compile, but fail hard on unreadable files inside an existing corpus.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let explain_dir = manifest_dir.join("../../explain");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", explain_dir.display());

    let entries = match read_entries(&explain_dir) {
        Ok(entries) => entries,
        Err(err) => panic!(
            "failed to read explain corpus from {}: {err}",
            explain_dir.display()
        ),
    };

    // The generated file is data only: parsing and validation stay in
    // `explain.rs`, where diagnostics can refer back to the source filename.
    let mut generated = String::from("&[\n");
    for (filename, source) in entries {
        generated.push_str("    RawEntry {\n");
        generated.push_str(&format!("        filename: {filename:?},\n"));
        generated.push_str(&format!("        source: {source:?},\n"));
        generated.push_str("    },\n");
    }
    generated.push_str("]\n");

    fs::write(out_dir.join("explain_entries.rs"), generated).expect("write explain_entries.rs");
}

/// Read Markdown explain entries in deterministic filename order.
///
/// Missing corpora are accepted so partial source distributions can still build
/// the crate, but malformed or unreadable files remain hard build failures.
fn read_entries(explain_dir: &Path) -> io::Result<Vec<(String, String)>> {
    if !explain_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(explain_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("explain entry filename is valid UTF-8")
            .to_owned();
        let source = fs::read_to_string(&path)?;
        entries.push((filename, source));
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(entries)
}
