use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("radix-project-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn compile_package_reports_unresolved_external_imports() {
    let dir = temp_dir("external-import");
    let entry = dir.join("main.fab");
    fs::write(&entry, "importa ex \"lodash\" privata map\nincipit { scribe \"x\" }").expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("only supports local intra-package imports")));
}

#[test]
fn compile_package_resolves_relative_input_from_current_working_directory() {
    let dir = temp_dir("relative-input");
    let project_dir = dir.join("project");
    fs::create_dir_all(&project_dir).expect("create project dir");
    fs::write(project_dir.join("main.fab"), "incipit { scribe \"salve\" }").expect("write entry");

    let original_cwd = std::env::current_dir().expect("current dir");
    std::env::set_current_dir(&dir).expect("set current dir");

    let result = compile_package(&Config::default(), Path::new("./project/main.fab"));

    std::env::set_current_dir(original_cwd).expect("restore current dir");

    assert!(result.success(), "expected relative package compile success");
}
