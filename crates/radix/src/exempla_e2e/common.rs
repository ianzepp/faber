use super::types::E2eResult;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn command_available(command: &str, args: &[&str]) -> bool {
    Command::new(command).args(args).output().is_ok()
}

pub(super) fn make_temp_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("radix-rs-e2e-{nanos}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

pub(super) fn collect_exempla_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_exempla_files_recursive(dir, &mut files);
    files
}

pub(super) fn collect_exempla_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_exempla_files_recursive(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("fab") {
            out.push(path);
        }
    }
}
pub(super) fn read_expected_stdout(fab_path: &Path) -> Option<String> {
    let expected_path = fab_path.with_extension("expected");
    let content = fs::read_to_string(expected_path).ok()?;
    Some(normalize_newline(&content))
}

pub(super) fn is_expected_failure(path: &Path, expected_failures: &[&str]) -> bool {
    expected_failures
        .iter()
        .any(|expected| path.ends_with(expected))
}

pub(super) fn expected_runtime_failure<'a>(path: &Path, expected_failures: &'a [(&str, &str)]) -> Option<&'a str> {
    expected_failures
        .iter()
        .find_map(|(expected_path, expected_message)| path.ends_with(expected_path).then_some(*expected_message))
}

pub(super) fn format_result_paths(results: &[&E2eResult]) -> String {
    results
        .iter()
        .map(|result| result.path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn normalize_newline(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end_matches('\n').to_owned()
}

pub(super) fn format_diagnostics(result: &crate::CompileResult) -> String {
    if result.diagnostics.is_empty() {
        "no diagnostics".to_owned()
    } else {
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

pub(super) fn format_diagnostic_messages(diagnostics: &[crate::Diagnostic]) -> String {
    if diagnostics.is_empty() {
        "no diagnostics".to_owned()
    } else {
        diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
