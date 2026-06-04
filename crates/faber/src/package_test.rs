use super::{
    analysis_source_for_file, attach_library_provenance, check_package, compile_package,
    discover_build_layout, discover_package, emit_generated_crate, invoke_cargo_build,
    library_resolver_from_config, load_package, read_manifest, sanitize_crate_name, BuildLayout,
};
use crate::library::{LibraryProviderKind, LibraryResolver, ResolvedLibraryModule};
use radix::diagnostics::Diagnostic;
use radix::driver::Config;
use radix::driver::{analyze_source_with_cli_program, Session};
use radix::hir::{HirItemKind, LibraryItemKind, LibraryProvider};
use radix::Output;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
        "importa ex \"lodash\" privata map\nincipit { nota \"x\" }",
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("only supports local intra-package imports")));
}

#[test]
fn compile_package_resolves_builtin_norma_library_imports_without_local_modules() {
    let dir = temp_dir("norma-json-import");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma:json" privata json

incipit {
  fixum _ parsed ← json.solve("{}")
}
"#,
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(
        result.success(),
        "expected norma:json package compile success, got {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };

    assert!(output.code.contains("norma::json::solve"));
    assert!(!output.code.contains("crate::norma::json"));
}

#[test]
fn compile_package_resolves_builtin_norma_toml_library_imports() {
    let dir = temp_dir("norma-toml-import");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma:toml" privata toml

incipit {
  fixum _ parsed ← toml.solve("name = \"faber\"")
}
"#,
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(
        result.success(),
        "expected norma:toml package compile success, got {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };

    assert!(output.code.contains("norma::toml::solve"));
    assert!(!output.code.contains("crate::norma::toml"));
}

#[test]
fn compile_package_resolves_builtin_norma_hal_consolum_imports() {
    let dir = temp_dir("norma-hal-consolum-import");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma:hal/consolum" privata consolum

incipit {
  consolum.dic("salve")
}
"#,
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(
        result.success(),
        "expected norma:hal/consolum package compile success, got {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };

    assert!(output.code.contains("norma::hal::consolum::dic"));
    assert!(!output.code.contains("crate::norma::hal::consolum"));
}

#[test]
fn package_fixture_runs_norma_http_hal_against_local_server() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind local HTTP fixture");
    listener
        .set_nonblocking(true)
        .expect("set fixture listener nonblocking");
    let port = listener.local_addr().expect("local addr").port();

    let dir = temp_dir("norma-http-fixture");
    fs::create_dir_all(dir.join("src")).expect("src");
    fs::write(
        dir.join("faber.toml"),
        r#"
[package]
name = "norma-http-fixture"
version = "0.1.0"
"#,
    )
    .expect("manifest");
    fs::write(
        dir.join("src/main.fab"),
        format!(
            r#"
importa ex "norma:hal/http" privata http

incipiet {{
    fixum _ responsum ← cede http.petet("http://127.0.0.1:{port}/test")
    nota responsum.status()
    fixum _ caput ← responsum.caput("x-faber-test")
    si caput est nihil {{ nota "header:missing" }} secus {{ nota "header:present" }}
    nota responsum.corpus()
    fixum _ data ← responsum.corpus_json()
    si data.is_nihil() {{ nota "json:missing" }} secus {{ nota "json:present" }}
}}
"#
        ),
    )
    .expect("entry");

    let layout = discover_build_layout(&dir).expect("layout");
    let compile_result = compile_package(&Config::default(), &dir);
    assert!(
        compile_result.success(),
        "expected HTTP package compile success, got {:?}",
        compile_result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(Output::Rust(output)) = compile_result.output else {
        panic!("expected rust output");
    };
    assert!(output.code.contains("norma::hal::http::petet"));
    assert!(output.code.contains(".corpus_json()"));

    emit_generated_crate(
        &layout,
        &output.code,
        Some(&read_manifest(&layout.manifest_path).expect("manifest")),
    )
    .expect("emit generated crate");
    let binary = invoke_cargo_build(&layout, false).expect("cargo build");
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("accept one request: {err}"),
            }
        };
        stream
            .set_nonblocking(false)
            .expect("set fixture stream blocking");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).expect("read request");

        let body = r#"{"ok":true,"message":"salve"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\nx-faber-test: local\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });
    let run = Command::new(binary).output().expect("run generated binary");
    server.join().expect("server thread");

    assert!(run.status.success(), "fixture binary failed: {:?}", run);
    let stdout = String::from_utf8(run.stdout).expect("stdout utf8");
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        vec![
            "200",
            "header:present",
            r#"{"ok":true,"message":"salve"}"#,
            "json:present"
        ]
    );
}

#[test]
fn library_resolver_discovers_builtin_norma_hal_modules_without_allowlist() {
    let resolved = LibraryResolver::default()
        .resolve("norma:hal/solum")
        .expect("resolve should not fail")
        .expect("norma:hal/solum should resolve");

    assert_eq!(resolved.package, "norma");
    assert_eq!(resolved.module_path, vec!["hal", "solum"]);
    assert!(resolved
        .interface_path
        .ends_with("stdlib/norma/hal/solum.fab"));
    assert_eq!(resolved.provider, LibraryProviderKind::Builtin);
}

#[test]
fn check_package_typechecks_builtin_library_imports_against_interfaces() {
    let dir = temp_dir("norma-json-interface");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma:json" privata json

incipit {
  json.nonexistent("{}")
}
"#,
    )
    .expect("write entry");

    let diagnostics = check_package(&Config::default(), &entry);
    assert!(diagnostics
        .iter()
        .any(|diag| diag.message.contains("unknown method")));
}

#[test]
fn compile_package_reports_unknown_builtin_library_modules() {
    let dir = temp_dir("norma-nope");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma:nope" privata nope
incipit {}
"#,
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("unknown built-in library module `norma:nope`")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("hal/consolum") && diag.message.contains("hal/solum")));
}

#[test]
fn compile_package_rejects_old_norma_slash_library_imports() {
    let dir = temp_dir("norma-old-slash");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma/hal/http" privata http
incipit {}
"#,
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("built-in Norma imports use provider syntax; write \"norma:hal/http\"")));
}

#[test]
fn compile_package_keeps_relative_norma_paths_as_local_imports() {
    let dir = temp_dir("relative-norma-path");
    fs::create_dir_all(dir.join("norma")).expect("norma dir");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "./norma/json" privata local
incipit {}
"#,
    )
    .expect("write entry");
    fs::write(
        dir.join("norma/json.fab"),
        r#"
functio salve() → textus {
    redde "salve"
}
"#,
    )
    .expect("write local module");

    let result = compile_package(&Config::default(), &entry);
    assert!(
        result.success(),
        "expected relative local import success, got {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn compile_package_reports_unknown_provider_without_local_fallback() {
    let dir = temp_dir("unknown-provider");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "sqlite:client" privata client
incipit {}
"#,
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("unknown library provider `sqlite` in import `sqlite:client`")));
    assert!(!result.diagnostics.iter().any(|diag| diag
        .message
        .contains("only supports local intra-package imports")));
}

#[test]
fn aliased_norma_import_preserves_provider_identity_in_analysis() {
    let dir = temp_dir("aliased-norma-provider");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma:hal/http" privata http ut rete

incipit {
  fixum _ responsum ← cede rete.petet("http://127.0.0.1:9")
}
"#,
    )
    .expect("write entry");

    let config = Config::default();
    let spec = discover_package(&entry).expect("package");
    let resolver = library_resolver_from_config(&config);
    let files = load_package(&spec, &resolver).expect("load package");
    let file = files
        .iter()
        .find(|file| file.path == entry)
        .expect("entry file");
    let source = analysis_source_for_file(file).expect("analysis source");
    let session = Session::new(config);
    let mut analysis =
        analyze_source_with_cli_program(&session, &entry.display().to_string(), &source, None)
            .expect("analysis");
    attach_library_provenance(&mut analysis, &file.library_imports).expect("provenance");

    let rete = analysis
        .hir
        .items
        .iter()
        .find_map(|item| {
            let HirItemKind::Interface(interface) = &item.kind else {
                return None;
            };
            (analysis.interner.resolve(interface.name) == "rete").then_some(item.def_id)
        })
        .expect("rete interface");
    let binding = analysis
        .libraries
        .bindings
        .get(&rete)
        .expect("rete binding provenance");
    assert_eq!(
        binding.identity.provider,
        LibraryProvider::Builtin("norma".to_owned())
    );
    assert_eq!(binding.identity.module_path, vec!["hal", "http"]);
    assert_eq!(
        binding.rust_runtime_module.as_deref(),
        Some("norma::hal::http")
    );

    let replicatio = analysis
        .hir
        .items
        .iter()
        .find_map(|item| {
            let HirItemKind::Interface(interface) = &item.kind else {
                return None;
            };
            (analysis.interner.resolve(interface.name) == "Replicatio").then_some(item.def_id)
        })
        .expect("Replicatio interface");
    let item = analysis
        .libraries
        .items
        .get(&replicatio)
        .expect("Replicatio item provenance");
    assert_eq!(
        item.identity.provider,
        LibraryProvider::Builtin("norma".to_owned())
    );
    assert_eq!(item.identity.module_path, vec!["hal", "http"]);
    assert_eq!(item.exported_name, "Replicatio");
    assert_eq!(item.kind, LibraryItemKind::Interface);
    assert_eq!(
        item.rust_runtime_type.as_deref(),
        Some("norma::hal::http::Replicatio")
    );
    assert!(item.elide_rust_decl);
}

#[test]
fn aliased_norma_http_import_lowers_by_provider_identity() {
    let dir = temp_dir("aliased-norma-http-lowering");
    let entry = dir.join("main.fab");
    fs::write(
        &entry,
        r#"
importa ex "norma:hal/http" privata http ut rete

incipiet {
  fixum _ responsum ← cede rete.petet("http://127.0.0.1:9")
}
"#,
    )
    .expect("write entry");

    let result = compile_package(&Config::default(), &entry);
    assert!(
        result.success(),
        "expected aliased HTTP package compile success, got {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };

    assert!(output
        .code
        .contains("norma::hal::http::petet(\"http://127.0.0.1:9\".to_string()).await"));
    assert!(output
        .code
        .contains("let responsum: norma::hal::http::Replicatio ="));
    assert!(!output.code.contains("rete.petet"));
    assert!(!output.code.contains("pub trait Replicatio"));
}

#[test]
fn resolved_library_module_shape_can_describe_future_sqlite_without_rust_metadata() {
    let module = ResolvedLibraryModule::new(
        "sqlite",
        vec!["transactio".to_owned()],
        "/tmp/faber-libs/sqlite/transactio.fab",
        LibraryProviderKind::PackageDependency,
    );

    assert_eq!(module.package, "sqlite");
    assert_eq!(module.module_path, vec!["transactio"]);
    assert_eq!(module.module_name(), Some("transactio"));
    assert_eq!(module.provider, LibraryProviderKind::PackageDependency);
    assert!(module.interface_path.ends_with("sqlite/transactio.fab"));
}

#[test]
fn compile_package_resolves_relative_input_from_current_working_directory() {
    let dir = temp_dir("relative-input");
    let project_dir = dir.join("project");
    fs::create_dir_all(&project_dir).expect("create project dir");
    fs::write(project_dir.join("main.fab"), "incipit { nota \"salve\" }").expect("write entry");

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
  nota args.name
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
  nota "running"
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
  nota args.verbose
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
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("create src");
    fs::write(src.join("main.fab"), "incipit {}").expect("write package entry");
    fs::write(
        dir.join("faber.toml"),
        r#"
[package]
name = "manifest-example"
version = "0.1.0"

[paths]
source = "src"
entry = "main.fab"
"#,
    )
    .expect("write manifest");

    let result = compile_package(&Config::default(), &dir.join("faber.toml"));
    assert!(result.success(), "expected package compile success");
}

#[test]
fn compile_package_discovers_faber_toml_from_directory() {
    let dir = temp_dir("manifest-dir");
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("create src");
    fs::write(src.join("main.fab"), "incipit { nota \"ok\" }").expect("write package entry");
    fs::write(
        dir.join("faber.toml"),
        r#"
[package]
name = "manifest-dir"

[paths]
source = "src"
entry = "main.fab"
"#,
    )
    .expect("write manifest");

    let result = compile_package(&Config::default(), &dir);
    assert!(
        result.success(),
        "expected package directory compile success"
    );
}

#[test]
fn read_manifest_applies_default_paths_and_build_values() {
    let dir = temp_dir("manifest-defaults");
    let manifest = dir.join("faber.toml");
    fs::write(
        &manifest,
        r#"
[package]
name = "defaults"
"#,
    )
    .expect("write manifest");

    let manifest = read_manifest(&manifest).expect("read manifest");
    assert_eq!(manifest.package.name, "defaults");
    assert_eq!(manifest.package.version, "0.1.0");
    assert_eq!(manifest.package.edition, "2026");
    assert_eq!(manifest.paths.source, "src");
    assert_eq!(manifest.paths.entry, "main.fab");
    assert_eq!(manifest.build.target, "rust");
    assert_eq!(manifest.build.kind, "bin");
}

#[test]
fn compile_package_rejects_unsupported_manifest_target() {
    let dir = temp_dir("manifest-target");
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("create src");
    fs::write(src.join("main.fab"), "incipit {}").expect("write package entry");
    fs::write(
        dir.join("faber.toml"),
        r#"
[package]
name = "bad-target"

[build]
target = "go"
"#,
    )
    .expect("write manifest");

    let result = compile_package(&Config::default(), &dir);
    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("build.target 'go' is not supported")));
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

// ---------------------------------------------------------------------------
// Phase 1: BuildLayout path model tests (pure, no Cargo, sibling contract)
// ---------------------------------------------------------------------------

#[test]
fn build_layout_from_root_produces_sibling_debug_release_and_faber_dirs() {
    let layout = BuildLayout::from_package_root("/tmp/hello-world", "hello-world");

    assert_eq!(
        layout.package_root,
        Path::new("/tmp/hello-world").to_path_buf()
    );
    assert_eq!(
        layout.generated_crate_root,
        Path::new("/tmp/hello-world/target/faber").to_path_buf()
    );
    assert_eq!(
        layout.cargo_target_dir,
        Path::new("/tmp/hello-world/target").to_path_buf()
    );
    assert_eq!(
        layout.debug_binary,
        Path::new("/tmp/hello-world/target/debug/hello-world").to_path_buf()
    );
    assert_eq!(
        layout.release_binary,
        Path::new("/tmp/hello-world/target/release/hello-world").to_path_buf()
    );

    // Critical sibling contract: debug/release are peers of faber/, never under it
    let faber_target = layout.generated_crate_root.join("target");
    assert!(
        !layout.debug_binary.starts_with(&faber_target),
        "debug binary must not live under target/faber/target (would create nested target)"
    );
    assert!(
        !layout.release_binary.starts_with(&faber_target),
        "release binary must not live under target/faber/target"
    );
    assert_eq!(layout.binary_name(), "hello-world");
}

#[test]
fn sanitize_crate_name_handles_mixed_case_punctuation_and_digits() {
    assert_eq!(sanitize_crate_name("My Cool App!"), "my-cool-app");
    assert_eq!(sanitize_crate_name("Faber_Tool-2026"), "faber_tool-2026");
    assert_eq!(sanitize_crate_name("123pkg"), "p-123pkg");
    assert_eq!(sanitize_crate_name(""), "package");
    assert_eq!(sanitize_crate_name("___"), "package");
    assert_eq!(sanitize_crate_name("a/b\\c"), "a-b-c");
}

#[test]
fn discover_build_layout_supports_manifest_file_input() {
    let dir = temp_dir("layout-manifest-file");
    let manifest = dir.join("faber.toml");
    fs::write(
        &manifest,
        r#"
[package]
name = "Manifest-Pkg"
version = "0.2.0"
"#,
    )
    .expect("write manifest");

    let layout = discover_build_layout(&manifest).expect("discover from manifest file");
    assert_eq!(layout.binary_name(), "manifest-pkg");
    assert_eq!(layout.package_root, dir);
    assert!(layout.manifest_path.ends_with("faber.toml"));
    // still sibling even with odd casing in name
    assert!(layout
        .debug_binary
        .to_string_lossy()
        .ends_with("manifest-pkg"));
}

#[test]
fn discover_build_layout_supports_directory_with_manifest() {
    let dir = temp_dir("layout-dir-manifest");
    fs::create_dir_all(dir.join("src")).expect("src");
    fs::write(dir.join("src/main.fab"), "incipit {}").expect("entry");
    fs::write(
        dir.join("faber.toml"),
        r#"
[package]
name = "dir-pkg"
"#,
    )
    .expect("manifest");

    let layout = discover_build_layout(&dir).expect("discover from dir");
    assert_eq!(layout.binary_name(), "dir-pkg");
    assert_eq!(
        layout.generated_rust_entry,
        dir.join("target/faber/src/main.rs")
    );
}

#[test]
fn discover_build_layout_supports_entry_file_input_and_falls_back_to_dir_name() {
    let dir = temp_dir("layout-entry-no-manifest");
    let entry = dir.join("main.fab");
    fs::write(&entry, "incipit { nota \"x\" }").expect("entry");

    let layout = discover_build_layout(&entry).expect("discover from entry file");
    // falls back to directory name since no manifest
    let expected_name = dir.file_name().unwrap().to_string_lossy().to_string();
    assert_eq!(layout.binary_name(), sanitize_crate_name(&expected_name));
    assert!(layout.cargo_target_dir.ends_with("target"));
}

#[test]
fn build_layout_never_produces_faber_target_nested_path() {
    let layout = BuildLayout::from_package_root("/tmp/xyz", "xyz");
    let nested = layout.generated_crate_root.join("target");
    assert!(
        !layout.debug_binary.starts_with(&nested),
        "no target/faber/target path allowed"
    );
}

// ---------------------------------------------------------------------------
// Phase 2: Generated crate writer tests (no Cargo invocation)
// ---------------------------------------------------------------------------

#[test]
fn emit_generated_crate_writes_cargo_toml_and_main_rs_under_target_faber() {
    let pkg = temp_dir("emit-writer");
    fs::create_dir_all(pkg.join("src")).expect("src");
    fs::write(
        pkg.join("src/main.fab"),
        r#"incipit { nota "writer test" }"#,
    )
    .expect("entry");
    fs::write(
        pkg.join("faber.toml"),
        r#"
[package]
name = "emit-test"
version = "0.3.0"
"#,
    )
    .expect("manifest");

    let layout = discover_build_layout(&pkg).expect("layout");
    let compile_result = compile_package(&Config::default(), &pkg);
    assert!(compile_result.success(), "compile should succeed");
    let code = match &compile_result.output {
        Some(radix::Output::Rust(r)) => r.code.clone(),
        _ => panic!("expected rust output"),
    };

    let written = emit_generated_crate(
        &layout,
        &code,
        Some(&read_manifest(&layout.manifest_path).unwrap()),
    )
    .expect("emit");

    assert_eq!(written, layout.generated_crate_root);
    assert!(layout.generated_cargo_manifest.exists());
    assert!(layout.generated_rust_entry.exists());

    let cargo_toml = fs::read_to_string(&layout.generated_cargo_manifest).expect("read cargo");
    assert!(cargo_toml.contains("name = \"emit-test\""));
    assert!(cargo_toml.contains("edition = \"2021\""));
    assert!(cargo_toml.contains("0.3.0"));
    assert!(cargo_toml.contains("[dependencies]"));
    assert!(cargo_toml.contains("norma = { path = "));
    assert!(cargo_toml.contains(r#"tokio = { version = "1", features = ["rt", "net", "time"] }"#));

    let main_rs = fs::read_to_string(&layout.generated_rust_entry).expect("read main");
    assert!(main_rs.contains("Generated by faber build"));
    assert!(main_rs.contains("writer test")); // from the source string

    // No nested target created by the writer
    assert!(!layout.generated_crate_root.join("target").exists());
}

#[test]
fn emit_generated_crate_works_without_manifest_using_fallback_name() {
    let pkg = temp_dir("emit-no-manifest");
    let entry = pkg.join("main.fab");
    fs::write(&entry, "incipit {}").expect("entry");

    let layout = discover_build_layout(&entry).expect("layout");
    // Directly test the emit path with dummy code (no real compile needed for writer coverage)
    let dummy = "fn main(){}";
    let _ = emit_generated_crate(&layout, dummy, None).expect("emit fallback");

    let cargo = fs::read_to_string(&layout.generated_cargo_manifest).expect("cargo");
    assert!(cargo.contains(&format!("name = \"{}\"", layout.binary_name())));
}
