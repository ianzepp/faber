use super::*;
use clap::error::ErrorKind;
use clap::CommandFactory;
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

fn temp_dir_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("radix-main-test-dir-{}-{}-{}", label, std::process::id(), nanos));
    path
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
fn format_location_uses_line_and_column() {
    let source_file = crate::driver::SourceFile::inline("demo.fab", "alfa\nbeta\ngamma".to_owned());

    assert_eq!(format_location(&source_file, 0), "demo.fab:1:1");
    assert_eq!(format_location(&source_file, 5), "demo.fab:2:1");
    assert_eq!(format_location(&source_file, 8), "demo.fab:2:4");
}

#[test]
fn cmd_lex_parse_hir_check_emit_succeed_on_valid_file() {
    let file = write_temp_fab("pipeline", "incipit {}");
    let args = vec![file.clone()];

    cmd_lex(&args);
    cmd_parse(&args);
    cmd_hir(&args);
    cmd_mir(&args);
    cmd_check(CheckCommand { input: args.clone(), package: false, permissive: false });
    cmd_emit(EmitCommand {
        input: args,
        package: false,
        target: crate::codegen::Target::Rust,
        format: false,
        linter: false,
    });
}

#[test]
fn cmd_emit_supports_faber_target_flag() {
    let file = write_temp_fab("emit-faber", "incipit {}");
    cmd_emit(EmitCommand {
        input: vec![file],
        package: false,
        target: crate::codegen::Target::Faber,
        format: false,
        linter: false,
    });
}

#[test]
fn cmd_cli_ir_succeeds_on_valid_cli_file() {
    let file = write_temp_fab(
        "cli-ir",
        r#"
@ cli "tool"
@ optio verbose brevis "v" longum "verbose" typus bivalens
incipit argumenta args {}
"#,
    );

    cmd_cli_ir(&[file]);
}

#[test]
fn cmd_build_writes_file_output_to_disk() {
    let file = write_temp_fab("build", "incipit {}");
    let out_dir = temp_dir_path("build-out");
    let expected_name = PathBuf::from(&file)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("temp file stem")
        .to_owned();

    cmd_build(BuildCommand {
        input: file,
        out_dir: out_dir.clone(),
        package: false,
        release: false,
        target: crate::codegen::Target::Rust,
        format: false,
        linter: false,
    });

    let output = out_dir.join(expected_name).with_extension("rs");
    assert!(output.exists(), "expected output at {}", output.display());
}

#[test]
fn cmd_targets_reports_known_capabilities() {
    let rust = target_capabilities(crate::codegen::Target::Rust);
    let ts = target_capabilities(crate::codegen::Target::TypeScript);

    assert!(rust.package);
    assert!(rust.build);
    assert!(!ts.package);
    assert_eq!(target_name(crate::codegen::Target::Faber), "faber");
    assert_eq!(target_extension(crate::codegen::Target::Go), "go");
}

#[test]
fn cli_parses_check_permissive_flag() {
    let cli = RadixCli::try_parse_from(["radix", "check", "--permissive", "main.fab"]).expect("cli parse");
    match cli.command {
        RadixCommand::Check(args) => {
            assert!(args.permissive);
            assert!(!args.package);
            assert_eq!(args.input, vec!["main.fab"]);
        }
        other => panic!("expected check, got {:?}", other),
    }
}

#[test]
fn cli_parses_check_package_flag() {
    let cli = RadixCli::try_parse_from(["radix", "check", "--package", "pkg/main.fab"]).expect("cli parse");
    match cli.command {
        RadixCommand::Check(args) => {
            assert!(args.package);
            assert_eq!(args.input, vec!["pkg/main.fab"]);
        }
        other => panic!("expected check, got {:?}", other),
    }
}

#[test]
fn cli_accepts_target_aliases() {
    let cli = RadixCli::try_parse_from(["radix", "emit", "-t", "fab", "main.fab"]).expect("cli parse");
    match cli.command {
        RadixCommand::Emit(args) => {
            assert_eq!(args.target, CliTarget::Faber);
            assert!(!args.package);
            assert_eq!(args.input, vec!["main.fab"]);
        }
        other => panic!("expected emit, got {:?}", other),
    }
}

#[test]
fn cli_parses_emit_package_flag() {
    let cli = RadixCli::try_parse_from(["radix", "emit", "--package", "pkg/main.fab"]).expect("cli parse");
    match cli.command {
        RadixCommand::Emit(args) => {
            assert!(args.package);
            assert_eq!(args.input, vec!["pkg/main.fab"]);
        }
        other => panic!("expected emit, got {:?}", other),
    }
}

#[test]
fn cli_parses_mir_command() {
    let cli = RadixCli::try_parse_from(["radix", "mir", "main.fab"]).expect("cli parse");
    match cli.command {
        RadixCommand::Mir(args) => {
            assert_eq!(args.input, vec!["main.fab"]);
        }
        other => panic!("expected mir, got {:?}", other),
    }
}

#[test]
fn mir_output_for_source_returns_deterministic_text() {
    let output = mir_output_for_source("main.fab", "functio saluta() {}").expect("MIR output");

    assert_eq!(
        output,
        "\
function f0 -> ty#5 {
  bb0:
    return
}
"
    );
}

#[test]
fn mir_output_for_control_flow_source_returns_deterministic_text() {
    let output = mir_output_for_source(
        "main.fab",
        "functio signum(numerus n) → numerus { si n > 0 ergo redde n redde 0 }",
    )
    .expect("MIR output");

    assert_eq!(
        output,
        "\
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
  temps:
    %0: ty#3
    %1: ty#1
  bb0:
    %0 = _0 > const int 0: ty#3
    branch %0 bb1 bb2
  bb1:
    return _0
  bb2:
    %1 = const int 0: ty#1
    return %1
}
"
    );
}

#[test]
fn cli_help_surface_lists_current_commands_and_hides_legacy_alias() {
    let help = RadixCli::command().render_help().to_string();

    assert!(help.contains("Usage: radix <COMMAND>"));
    assert!(!help.contains("build"));
    assert!(help.contains("targets"));
    assert!(help.contains("check"));
    assert!(help.contains("emit"));
    assert!(help.contains("mir"));
    assert!(!help.contains("emit-package"));
}

#[test]
fn cli_rejects_removed_emit_package_alias() {
    let err =
        RadixCli::try_parse_from(["radix", "emit-package", "-t", "faber", "pkg/main.fab"]).expect_err("removed alias");

    assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
}
