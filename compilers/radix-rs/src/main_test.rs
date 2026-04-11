use super::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_fab_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("radix-main-test-{}-{}-{}.fab", label, std::process::id(), nanos));
    path
}

fn write_temp_fab(label: &str, content: &str) -> String {
    let path = temp_fab_path(label);
    std::fs::write(&path, content).expect("write temp fab");
    path.to_string_lossy().to_string()
}

#[test]
fn escape_json_escapes_special_chars() {
    let escaped = escape_json("a\\b\"c\nd\re\tf");
    assert_eq!(escaped, "a\\\\b\\\"c\\nd\\re\\tf");
}

#[test]
fn read_source_reads_file_argument() {
    let file = write_temp_fab("read", "incipit {}");
    let args = vec![file.clone()];

    let (name, source) = read_source(&args);
    assert_eq!(name, file);
    assert_eq!(source, "incipit {}");
}

#[test]
fn cmd_lex_parse_hir_check_emit_succeed_on_valid_file() {
    let file = write_temp_fab("pipeline", "incipit {}");
    let args = vec![file.clone()];

    cmd_lex(&args);
    cmd_parse(&args);
    cmd_hir(&args);
    cmd_check(&args);
    cmd_emit(&args);
}

#[test]
fn cmd_emit_supports_faber_target_flag() {
    let file = write_temp_fab("emit-faber", "incipit {}");
    let args = vec!["-t".to_owned(), "faber".to_owned(), file];
    cmd_emit(&args);
}

#[test]
fn cmd_emit_package_supports_cli_example() {
    let args = vec!["../../examples/exempla/cli/main.fab".to_owned()];
    cmd_emit_package(&args);
}
