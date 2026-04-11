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
fn cmd_lex_parse_hir_check_emit_succeed_on_valid_file() {
    let file = write_temp_fab("pipeline", "incipit {}");
    let args = vec![file.clone()];

    cmd_lex(&args);
    cmd_parse(&args);
    cmd_hir(&args);
    cmd_check(CheckCommand { input: args.clone(), permissive: false });
    cmd_emit(EmitCommand { input: args, target: radix::codegen::Target::Rust });
}

#[test]
fn cmd_emit_supports_faber_target_flag() {
    let file = write_temp_fab("emit-faber", "incipit {}");
    cmd_emit(EmitCommand { input: vec![file], target: radix::codegen::Target::Faber });
}

#[test]
fn cmd_emit_package_supports_cli_example() {
    cmd_emit_package(EmitPackageCommand {
        path: "../../examples/exempla/cli/main.fab".to_owned(),
        target: radix::codegen::Target::Rust,
    });
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
        target: radix::codegen::Target::Rust,
    });

    let output = out_dir.join(expected_name).with_extension("rs");
    assert!(output.exists(), "expected output at {}", output.display());
}

#[test]
fn cmd_targets_reports_known_capabilities() {
    let rust = target_capabilities(radix::codegen::Target::Rust);
    let ts = target_capabilities(radix::codegen::Target::TypeScript);

    assert!(rust.package);
    assert!(rust.build);
    assert!(!ts.package);
    assert_eq!(target_name(radix::codegen::Target::Faber), "faber");
    assert_eq!(target_extension(radix::codegen::Target::Go), "go");
}

#[test]
fn cli_parses_legacy_emit_package_shape() {
    let cli = Cli::try_parse_from(["radix", "emit-package", "-t", "faber", "pkg/main.fab"]).expect("cli parse");
    match cli.command {
        Command::EmitPackage(args) => {
            assert_eq!(args.target, CliTarget::Faber);
            assert_eq!(args.path, "pkg/main.fab");
        }
        other => panic!("expected emit-package, got {:?}", other),
    }
}

#[test]
fn cli_parses_check_permissive_flag() {
    let cli = Cli::try_parse_from(["radix", "check", "--permissive", "main.fab"]).expect("cli parse");
    match cli.command {
        Command::Check(args) => {
            assert!(args.permissive);
            assert_eq!(args.input, vec!["main.fab"]);
        }
        other => panic!("expected check, got {:?}", other),
    }
}

#[test]
fn cli_accepts_target_aliases() {
    let cli = Cli::try_parse_from(["radix", "emit", "-t", "fab", "main.fab"]).expect("cli parse");
    match cli.command {
        Command::Emit(args) => {
            assert_eq!(args.target, CliTarget::Faber);
            assert_eq!(args.input, vec!["main.fab"]);
        }
        other => panic!("expected emit, got {:?}", other),
    }
}

#[test]
fn cli_parses_build_flags() {
    let cli =
        Cli::try_parse_from(["radix", "build", "-t", "ts", "-o", "opus", "--package", "main.fab"]).expect("cli parse");
    match cli.command {
        Command::Build(args) => {
            assert_eq!(args.target, CliTarget::TypeScript);
            assert_eq!(args.out_dir, PathBuf::from("opus"));
            assert!(args.package);
            assert_eq!(args.input, "main.fab");
        }
        other => panic!("expected build, got {:?}", other),
    }
}
