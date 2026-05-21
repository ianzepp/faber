use super::{check_package, compile_package};
use radix::diagnostics::Diagnostic;
use radix::driver::Config;
use radix::Output;
use std::fs;
use std::path::{Path, PathBuf};
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
    fs::write(
        &entry,
        "importa ex \"lodash\" privata map\nincipit { scribe \"x\" }",
    )
    .expect("write entry");

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

    assert!(
        result.success(),
        "expected relative package compile success"
    );
}

#[test]
fn compile_package_mounts_wildcard_imported_cli_commands() {
    let dir = temp_dir("cli-mount");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        r#"
@ imperium "config/set"
@ alias "set"
@ operandus textus name
functio set_config() argumenta args {
  scribe args.name
}
"#,
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(
        result.success(),
        "expected mounted package compile success, got {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };

    assert!(output.code.contains("struct CliArgsJobsConfigSet"));
    assert!(output
        .code
        .contains("pub(crate) fn set_config(args: crate::CliArgsJobsConfigSet)"));
    assert!(output.code.contains("jobs::set_config(args);"));
    assert!(output.code.contains("Usage: tool jobs config set"));
    assert!(output
        .code
        .contains("command_parts[0] == \"jobs\" && command_parts[1] == \"set\""));
}

#[test]
fn check_package_validates_mounted_cli_commands_without_emitting() {
    let dir = temp_dir("check-cli-mount");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        r#"
@ imperium "run"
functio run() argumenta args {
  scribe "running"
}
"#,
    )
    .expect("write jobs");

    let diagnostics = check_package(&Config::default(), &dir);

    assert!(
        !diagnostics.iter().any(Diagnostic::is_error),
        "expected package check success, got {:?}",
        diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn compile_package_mounted_handlers_can_access_root_globals() {
    let dir = temp_dir("cli-mount-root-global");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ optio verbose longum "verbose" typus bivalens ubique
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        r#"
@ imperium "run"
functio run() argumenta args {
  scribe args.verbose
}
"#,
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(
        result.success(),
        "expected mounted handler to see root globals, got {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };

    assert!(output.code.contains("pub verbose: bool"));
    assert!(output.code.contains("println!(\"{}\", args.verbose);"));
}

#[test]
fn compile_package_rejects_mounted_local_binding_collision_with_root_global() {
    let dir = temp_dir("cli-mount-root-global-collision");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ optio verbose longum "verbose" typus bivalens ubique
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        r#"
@ imperium "run"
@ optio verbose longum "local-verbose"
functio run() {}
"#,
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("collides with a global CLI binding")));
}

#[test]
fn compile_package_rejects_named_import_mount_targets() {
    let dir = temp_dir("cli-mount-named");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata set_config ut jobs

@ cli "tool"
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        "@ imperium \"run\"\nfunctio set_config() {}",
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("must be a wildcard import alias")));
}

#[test]
fn compile_package_rejects_mounted_global_options() {
    let dir = temp_dir("cli-mount-global");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        "@ imperium \"run\"\n@ optio verbose longum \"verbose\" ubique\nfunctio run() {}",
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("'ubique' options")));
}

#[test]
fn compile_package_rejects_mounted_command_path_collisions() {
    let dir = temp_dir("cli-mount-collision");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ imperia "jobs" ex jobs
incipit argumenta args {}

@ imperium "jobs/run"
functio root_run() {}
"#,
    )
    .expect("write entry");
    fs::write(dir.join("jobs.fab"), "@ imperium \"run\"\nfunctio run() {}").expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("duplicate command path 'jobs/run'")));
}

#[test]
fn compile_package_rejects_mounted_alias_collisions() {
    let dir = temp_dir("cli-mount-alias-collision");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        r#"
@ imperium "one"
@ alias "same"
functio one() {}

@ imperium "two"
@ alias "same"
functio two() {}
"#,
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("duplicate command alias 'jobs/same'")));
}

#[test]
fn compile_package_does_not_expose_unmounted_imported_cli_modules() {
    let dir = temp_dir("cli-unmounted");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(dir.join("jobs.fab"), "@ imperium \"run\"\nfunctio run() {}").expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.success(), "expected package compile success");
    let Some(Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };

    assert!(!output.code.contains("jobs::run"));
    assert!(output.code.contains("Usage: tool"));
    assert!(!output.code.contains("<COMMAND>"));
}

#[test]
fn compile_package_rejects_import_cycles() {
    let dir = temp_dir("import-cycle");
    fs::write(
        dir.join("main.fab"),
        "importa ex \"./jobs\" privata * ut jobs\nincipit {}",
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        "importa ex \"./main\" privata * ut main\nfunctio run() {}",
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("import cycle detected")));
}

#[test]
fn compile_package_supports_manifest_example() {
    let dir = temp_dir("manifest");
    fs::write(dir.join("main.fab"), "incipit {}").expect("write package entry");
    fs::write(dir.join("faber.fab"), "ingressus \"main.fab\"\n").expect("write manifest");

    let result = compile_package(&Config::default(), &dir.join("faber.fab"));
    assert!(result.success(), "expected package compile success");
}

#[test]
fn compile_package_rejects_nested_module_mounts() {
    let dir = temp_dir("mount-cycle");
    fs::write(
        dir.join("main.fab"),
        r#"
importa ex "./jobs" privata * ut jobs

@ cli "tool"
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("jobs.fab"),
        r#"
@ imperia "again" ex jobs
@ imperium "run"
functio run() {}
"#,
    )
    .expect("write jobs");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("@ imperia module mounts may only be declared on the root CLI")));
}
