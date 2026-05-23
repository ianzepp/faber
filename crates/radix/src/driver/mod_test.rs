use super::{compile, Config, Session};
use crate::codegen::Target;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn session(target: Target) -> Session {
    Session::new(Config::default().with_target(target))
}

fn faber_roundtrip(source: &str) -> String {
    let session = session(Target::Faber);
    let first = compile(&session, "roundtrip.fab", source);
    assert!(
        first.success(),
        "first faber compile failed: {:?}",
        first
            .diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
    );
    let Some(crate::Output::Faber(first_output)) = first.output else {
        panic!("expected faber output");
    };

    let second = compile(&session, "roundtrip.fab", &first_output.code);
    assert!(
        second.success(),
        "second faber compile failed: {:?}\n{}",
        second
            .diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>(),
        first_output.code
    );
    let Some(crate::Output::Faber(second_output)) = second.output else {
        panic!("expected faber output");
    };

    assert_eq!(
        first_output.code, second_output.code,
        "faber emit should stabilize after one roundtrip"
    );
    second_output.code
}

fn compile_rust_source_with_rustc(code: &str, label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("radix-{label}-{}-{stamp}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("main.rs");
    let binary = dir.join("main");
    std::fs::write(&source, code).expect("write generated rust");

    let output = Command::new("rustc")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("execute rustc");
    assert!(
        output.status.success(),
        "rustc failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    binary
}

#[test]
fn compile_rust_success_emits_output() {
    let session = session(Target::Rust);
    let result = compile(&session, "test.fab", "incipit {}");

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn compile_faber_success_emits_output() {
    let session = session(Target::Faber);
    let result = compile(&session, "test.fab", "incipit {}");

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Faber(_))));
}

#[test]
fn compile_typescript_success_emits_output() {
    let session = session(Target::TypeScript);
    let result = compile(&session, "test.fab", "incipit {}");

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::TypeScript(_))));
}

#[test]
fn typescript_output_preserves_diagnostic_severity() {
    let session = session(Target::TypeScript);
    let result = compile(
        &session,
        "test.fab",
        r#"incipit {
  nota "n"
  vide "d"
  mone "w"
}"#,
    );

    assert!(result.success());
    let Some(crate::Output::TypeScript(output)) = result.output else {
        panic!("expected TypeScript output");
    };
    assert!(output.code.contains("console.log(\"n\")"));
    assert!(output.code.contains("console.debug(\"d\")"));
    assert!(output.code.contains("console.warn(\"w\")"));
}

#[test]
fn rust_single_command_cli_generates_compilable_parser_and_runtime_behavior() {
    let session = session(Target::Rust);
    let source = r#"@ cli "echo"
@ versio "1.2.3"
@ descriptio "Echo text"
@ optio loud brevis "l" longum "loud" typus bivalens descriptio "Loud output"
@ optio name longum "name" typus textus vel "Roma"
@ optio count longum "count" typus numerus
@ operandus numerus code
@ operandus ceteri textus words descriptio "Words"
incipit argumenta args exitus args.code {
  nota args.name
  nota args.loud
  nota args.count
  nota args.words
}"#;
    let result = compile(&session, "cli.fab", source);

    assert!(
        result.success(),
        "diagnostics: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };
    assert!(output.code.contains("struct CliArgs"));
    assert!(output.code.contains("parse_cli_args_or_exit"));
    assert!(!output
        .code
        .contains("runnable CLI code generation is not implemented"));

    let binary = compile_rust_source_with_rustc(&output.code, "single-cli");
    let help = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("run help");
    assert!(help.status.success());
    let help_stdout = String::from_utf8_lossy(&help.stdout);
    assert!(help_stdout.contains("Usage: echo [OPTIONS] <code> [words...]"));
    assert!(help_stdout.contains("--version"));

    let version = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("run version");
    assert!(version.status.success());
    assert_eq!(String::from_utf8_lossy(&version.stdout).trim(), "1.2.3");

    let run = Command::new(&binary)
        .args(["--name", "Alba", "--count", "3", "-l", "7", "salve", "munde"])
        .output()
        .expect("run cli");
    assert_eq!(run.status.code(), Some(7));
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "Alba\ntrue\nSome(3)\n[\"salve\", \"munde\"]\n"
    );

    let defaults = Command::new(&binary)
        .arg("0")
        .output()
        .expect("run cli defaults");
    assert!(defaults.status.success());
    assert_eq!(String::from_utf8_lossy(&defaults.stdout), "Roma\nfalse\nNone\n[]\n");
}

#[test]
fn rust_single_command_cli_reports_parse_errors() {
    let session = session(Target::Rust);
    let source = r#"@ cli "need"
@ optio count longum "count" typus numerus
@ operandus textus file
incipit argumenta args {
  nota args.file
}"#;
    let result = compile(&session, "cli.fab", source);
    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };
    let binary = compile_rust_source_with_rustc(&output.code, "parse-errors");

    let unknown = Command::new(&binary)
        .arg("--bogus")
        .output()
        .expect("run unknown");
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown option '--bogus'"));

    let version = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("run version without metadata");
    assert_eq!(version.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&version.stderr).contains("unknown option '--version'"));

    let missing_value = Command::new(&binary)
        .arg("--count")
        .output()
        .expect("run missing value");
    assert_eq!(missing_value.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing_value.stderr).contains("missing value for --count"));

    let missing_operand = Command::new(&binary)
        .args(["--count", "4"])
        .output()
        .expect("run missing operand");
    assert_eq!(missing_operand.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing_operand.stderr).contains("missing operand 'file'"));
}

#[test]
fn rust_single_command_cli_supports_fixed_exitus() {
    let session = session(Target::Rust);
    let source = r#"@ cli "fixed"
incipit argumenta args exitus 5 {
  nota "done"
}"#;
    let result = compile(&session, "cli.fab", source);
    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };
    let binary = compile_rust_source_with_rustc(&output.code, "fixed-exitus");

    let run = Command::new(&binary).output().expect("run fixed exitus");
    assert_eq!(run.status.code(), Some(5));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "done\n");
}

#[test]
fn cli_codegen_gates_non_rust_targets() {
    let source = r#"@ cli "tool"
incipit argumenta args {}"#;
    let ts = compile(&session(Target::TypeScript), "cli.fab", source);
    assert!(ts.output.is_none());
    assert!(ts
        .diagnostics
        .iter()
        .any(|d| d.message.contains("only implemented for Rust")));
}

#[test]
fn rust_subcommand_cli_dispatches_aliases_nested_commands_and_command_args() {
    let session = session(Target::Rust);
    let source = r#"@ cli "tool"
@ versio "2.0.0"
@ optio verbose brevis "v" longum "verbose" typus bivalens ubique
incipit argumenta args {}

@ imperium "jobs/list"
@ alias "ls"
@ descriptio "List jobs"
@ optio limit longum "limit" typus numerus vel 20
functio list() argumenta args → vacuum {
  nota args.verbose
  nota args.limit
}

@ imperium "jobs/show"
@ operandus numerus id
functio show() argumenta args → vacuum {
  nota args.verbose
  nota args.id
}"#;
    let result = compile(&session, "cli.fab", source);
    assert!(
        result.success(),
        "diagnostics: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };
    let binary = compile_rust_source_with_rustc(&output.code, "subcommand-cli");

    let nested = Command::new(&binary)
        .args(["--verbose", "jobs", "list", "--limit", "3"])
        .output()
        .expect("run nested command");
    assert!(nested.status.success());
    assert_eq!(String::from_utf8_lossy(&nested.stdout), "true\n3\n");

    let alias = Command::new(&binary)
        .args(["ls"])
        .output()
        .expect("run alias");
    assert!(alias.status.success());
    assert_eq!(String::from_utf8_lossy(&alias.stdout), "false\n20\n");

    let operand = Command::new(&binary)
        .args(["jobs", "show", "42"])
        .output()
        .expect("run operand command");
    assert!(operand.status.success());
    assert_eq!(String::from_utf8_lossy(&operand.stdout), "false\n42\n");

    let help = Command::new(&binary)
        .args(["jobs", "list", "--help"])
        .output()
        .expect("run command help");
    assert!(help.status.success());
    let help_stdout = String::from_utf8_lossy(&help.stdout);
    assert!(help_stdout.contains("Usage: tool jobs list [OPTIONS]"));
    assert!(help_stdout.contains("--verbose"));
    assert!(help_stdout.contains("--limit"));

    let missing = Command::new(&binary).output().expect("run missing command");
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stdout).contains("Usage: tool [OPTIONS] <COMMAND>"));

    let unknown = Command::new(&binary)
        .arg("bogus")
        .output()
        .expect("run unknown command");
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown command 'bogus'"));
}

#[test]
fn rust_subcommand_cli_prefers_longest_matching_command_path() {
    let session = session(Target::Rust);
    let source = r#"@ cli "tool"
incipit argumenta args {}

@ imperium "jobs"
functio jobs() → vacuum {
  nota "root"
}

@ imperium "jobs/list"
functio list() → vacuum {
  nota "list"
}"#;
    let result = compile(&session, "cli.fab", source);
    assert!(
        result.success(),
        "diagnostics: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected rust output");
    };
    let binary = compile_rust_source_with_rustc(&output.code, "subcommand-prefix");

    let nested = Command::new(&binary)
        .args(["jobs", "list"])
        .output()
        .expect("run nested command");
    assert!(nested.status.success());
    assert_eq!(String::from_utf8_lossy(&nested.stdout), "list\n");

    let root = Command::new(&binary)
        .arg("jobs")
        .output()
        .expect("run root command");
    assert!(root.status.success());
    assert_eq!(String::from_utf8_lossy(&root.stdout), "root\n");
}

#[test]
fn compile_reports_lex_errors() {
    let session = session(Target::Rust);
    let result = compile(&session, "test.fab", "😀");

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|d| d.is_error()));
}

#[test]
fn compile_reports_parse_errors() {
    let session = session(Target::Rust);
    let result = compile(&session, "test.fab", "functio x(");

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|d| d.is_error()));
}

#[test]
fn compile_reports_semantic_errors() {
    let session = session(Target::Rust);
    let result = compile(&session, "test.fab", "incipit {\n  nota nope\n}");

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|d| d.is_error()));
}

#[test]
fn incipit_argumenta_compiles_for_faber_target() {
    let session = session(Target::Faber);
    let source = r#"incipit argumenta args {
  nota args
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Faber(_))));
    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("invalid expression produced during lowering")));
}

#[test]
fn structured_cape_iace_fac_custodi_compile_for_faber_target() {
    let session = session(Target::Faber);
    let source = r#"functio probe(numerus n) → vacuum ⇥ textus {
  custodi {
    si n < 0 {
      iace "neg"
    }
  }

  fac {
    fac {
      iace "boom"
    } cape err {
      nota err
    }
  } cape capta {
    nota capta
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Faber(_))));
    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("invalid expression produced during lowering")));
}

#[test]
fn ergo_iace_mori_and_tacet_compile_for_faber_target() {
    let session = session(Target::Faber);
    let source = r#"functio probe(bivalens ok) → vacuum ⇥ textus {
  si ok ergo iace "boom"
  secus ergo mori "bad"

  si ok ergo tacet
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Faber(_))));
    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("invalid expression produced during lowering")));
}

#[test]
fn rust_target_rejects_exception_constructs() {
    let session = session(Target::Rust);
    let source = r#"functio probe(bivalens ok) → vacuum ⇥ textus {
  si ok ergo iace "boom"

  fac {
    iace "bad"
  } cape err {
    nota err
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.is_error() && d.message.contains("iace is not supported for Rust targets")));
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.is_error() && d.message.contains("cape is not supported for Rust targets")));
}

#[test]
fn unsupported_expression_kind_reports_lowering_error() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ x ← lege
  nota x
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|d| d.is_error()
        && d.message
            .contains("unsupported expression kind in lowering")));
}

#[test]
fn sed_regex_literals_no_longer_report_unsupported_lowering_error() {
    let session = session(Target::Faber);
    let source = r#"incipit {
  fixum _ pattern ← sed "\d+" g
  nota pattern
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let code = match result.output {
        Some(crate::Output::Faber(out)) => out.code,
        _ => panic!("expected faber output"),
    };
    assert!(code.contains("sed \"\\d+\" g"));
    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("unsupported expression kind in lowering")));
}

#[test]
fn rust_target_reports_cura_arena_noop_warning() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  cura "arena" fixum _ mem {
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.diagnostics.iter().any(|d| {
        !d.is_error()
            && d.message
                .contains("cura \"arena\" has no effect for Rust targets")
    }));
}

#[test]
fn typescript_target_skips_rust_specific_cura_arena_warning() {
    let session = session(Target::TypeScript);
    let source = r#"incipit {
  cura "arena" fixum _ mem {
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.diagnostics.iter().all(|d| {
        !d.message
            .contains("cura \"arena\" has no effect for Rust targets")
    }));
}

#[test]
fn warning_only_semantic_diagnostics_still_emit_output() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ unused ← 1
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
    assert!(result.diagnostics.iter().any(|d| !d.is_error()));
    assert!(result.diagnostics.iter().all(|d| !d.is_error()));
}

#[test]
fn compile_accepts_textus_concatenation_and_compound_add() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  varia textus s ← "salve"
  s ⊕ "!"
  nota "dicit: " + s
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn backtick_template_string_is_not_faber_syntax() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ name ← "Mundus"
  fixum _ message ← `Hello ${name}`
  nota message
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unexpected character: '`'")));
}

#[test]
fn block_string_literal_uses_quote_glyphs() {
    let session = session(Target::Rust);
    let source = "incipit {\n  fixum _ quote ← ❝he said \"salve\"❞\n  nota quote\n}";
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("he said \\\"salve\\\""));
}

#[test]
fn single_quote_string_is_not_faber_syntax() {
    let session = session(Target::Rust);
    let source = "incipit {\n  fixum _ value ← 'hello'\n  nota value\n}";
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unexpected character")));
}

#[test]
fn triple_quote_string_is_not_faber_syntax() {
    let session = session(Target::Rust);
    let source = "incipit {\n  fixum _ value ← \"\"\"hello\"\"\"\n  nota value\n}";
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|d| d
        .message
        .contains("triple-quote string literals are not supported")));
}

#[test]
fn slash_line_comment_is_not_faber_syntax() {
    let session = session(Target::Rust);
    let source = "incipit {\n  // old comment\n  nota \"ok\"\n}";
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected expression")));
}

#[test]
fn slash_block_comment_is_not_faber_syntax() {
    let session = session(Target::Rust);
    let source = "incipit {\n  /* old comment */\n  nota \"ok\"\n}";
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("expected expression")));
}

#[test]
fn compile_accepts_finge_variant_construction() {
    let session = session(Target::Rust);
    let source = r#"discretio Event {
  Click { numerus x, numerus y },
  Quit
}

incipit {
  fixum Event e1 ← finge Click { x: 1, y: 2 } ⇢ Event
  fixum Event e2 ← finge Quit ⇢ Event
  nota e1
  nota e2
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn ad_stmt_arguments_allow_unresolved_endpoint_identifiers() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  ad "notificatio:mitte" ("User logged in", userId)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown identifier")));
}

#[test]
fn compile_accepts_array_and_ex_destructuring_bindings() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ xs ← [1, 2]
  fixum [a, b] ← xs
  nota a
  nota b

  fixum _ person ← { name: "Marcus", age: 1 }
  ex person fixum name
  nota name
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn enum_member_access_no_longer_reports_unknown_identifier() {
    let session = session(Target::Rust);
    let source = r#"ordo Color { rubrum, viridis }

incipit {
  fixum _ color ← Color.rubrum
  elige color {
    casu Color.rubrum { nota "r" }
    casu Color.viridis { nota "v" }
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown identifier")));
}

#[test]
fn compile_accepts_param_alias_binding() {
    let session = session(Target::Rust);
    let source = r#"functio greet(textus name, bivalens formal sponte ut f) → vacuum {
  si f {
    nota name
  }
}

incipit {
  greet("A", verum)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn unary_verum_and_falsum_accept_ignotum_operands() {
    let session = session(Target::Rust);
    let source = r#"functio check(ignotum x) → vacuum {
  si verum x { nota "t" }
  si falsum x { nota "f" }
}

incipit {
  check(verum)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("boolean operand required")));
}

#[test]
fn externa_top_level_var_without_initializer_no_longer_errors() {
    let session = session(Target::Rust);
    let source = r#"@ externa
fixum ignotum Bun

@ externa
functio require(textus path) → ignotum

incipit {
  nota Bun
  nota require("x")
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("top-level variable declaration requires initializer")));
}

#[test]
fn spread_array_argument_can_satisfy_multi_parameter_function() {
    let session = session(Target::Rust);
    let source = r#"functio add(numerus a, numerus b) → numerus {
  redde a + b
}

incipit {
  fixum numerus[] values = [3, 7]
  nota add(sparge values)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("wrong number of arguments")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("argument type mismatch")));
}

#[test]
fn import_alias_usage_no_longer_reports_unknown_identifier() {
    let session = session(Target::Rust);
    let source = r#"importa ex "../../norma/hal/consolum" privata consolum

    incipit {
  consolum.fundeLineam("x")
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
    assert!(result.diagnostics.iter().all(|d| !d.is_error()));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown identifier")));
}

#[test]
fn duplicate_import_module_bindings_no_longer_report_duplicate_definition() {
    let session = session(Target::Rust);
    let source = r#"importa ex "helpers" privata item
importa ex "helpers" privata item

incipit {
  nota "ok"
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("duplicate definition")));
}

#[test]
fn ignotum_receiver_method_calls_no_longer_leave_infer_type() {
    let session = session(Target::Rust);
    let source = r#"@ externa
fixum ignotum process

incipit {
  fixum _ args = process.argv ⇢ lista<textus>
  nota args.longitudo()
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
}

#[test]
fn go_target_rejects_ad_with_explicit_policy_diagnostic() {
    let session = session(Target::Go);
    let source = r#"incipit {
  ad "fasciculus:lege" ("hello.txt") → textus pro response {
    nota response
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.is_error() && d.message.contains("ad is not supported for Go targets")));
}

#[test]
fn go_target_rejects_member_access_on_externa_ignotum_binding() {
    let session = session(Target::Go);
    let source = r#"@ externa
fixum ignotum process

incipit {
  nota process.argv
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|d| d.is_error()
        && d.message
            .contains("member access on @ externa ignotum is not supported for Go targets")));
}

#[test]
fn go_target_allows_externa_with_explicit_cast_contract() {
    let session = session(Target::Go);
    let source = r#"@ externa
functio argv() → ignotum

incipit {
  fixum _ args ← argv() ⇢ lista<textus>
  nota args.longitudo()
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(
        result.success(),
        "{:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
    );
    assert!(matches!(result.output, Some(crate::Output::Go(_))));
    assert!(result.diagnostics.iter().all(|d| !d.is_error()));
}

#[test]
fn ab_property_filter_no_longer_reports_unknown_identifier() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ users ← [{ activus: verum }]
  fixum _ active ← ab users activus
  nota active
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown identifier")));
}

#[test]
fn ab_pipeline_from_object_member_no_longer_leaves_infer_types() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ users ← [
    { nomen: "Marcus", activus: verum, aetas: 25 },
    { nomen: "Julia", activus: falsum, aetas: 30 }
  ]
  fixum _ data ← { users: users }
  fixum _ active ← ab data.users activus
  nota active
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer variable type")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
}

#[test]
fn ex_object_varia_bindings_accept_same_type_reassignment() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ data ← { count: 100, active: verum }
  ex data varia count, active
  count ← 200
  active ← falsum
  nota count, active
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("expression type mismatch")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("assignment type mismatch")));
}

#[test]
fn compile_lowers_scriptum_without_stub_diagnostic() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ name ← "Marcus"
  fixum _ msg ← scriptum("salve, §!", name)
  nota msg
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("scriptum interpolation lowering is placeholder-only")));
}

#[test]
fn rust_compile_accepts_explicit_infer_type_marker() {
    let session = session(Target::Rust);
    let source = r#"functio answer() → _ {
  redde 42
}

incipit {
  fixum _ name ← "Marcus"
  varia _ count ← answer()
  count ← count + 1
  nota "§: §"(name, count)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(
        result.success(),
        "expected compile success, got diagnostics: {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.as_str())
            .collect::<Vec<_>>()
    );
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("fn answer() -> i64"));
    assert!(output.code.contains("let name: String"));
    assert!(output.code.contains("let mut count: i64"));
}

#[test]
fn rust_output_uses_format_macro_for_scriptum() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ value ← 7
  nota scriptum("valor: §", value)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("format!(\"valor: {}\""));
}

#[test]
fn rust_output_uses_zero_based_numbered_scriptum_placeholders() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ first ← "prima"
  fixum _ second ← "secunda"
  nota scriptum("ordo: §1, §0", first, second)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("format!(\"ordo: {1}, {0}\""));
}

#[test]
fn rust_output_uses_string_literal_format_application() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ value ← 7
  nota "valor: §"(value)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("format!(\"valor: {}\""));
}

#[test]
fn rust_output_uses_numbered_string_literal_format_application() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ first ← "prima"
  fixum _ second ← "secunda"
  nota "ordo: §1, §0"(first, second)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("format!(\"ordo: {1}, {0}\""));
}

#[test]
fn rust_output_supports_unicode_textus_index_and_range_slice() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ ch ← "Salve, §!"[7]
  fixum _ prefix ← "hello world"[0‥5]
  fixum _ whole ← "hello world"[0 usque 10]
  fixum _ stepped ← "abcdef"[0‥6 per 2]
  nota ch, prefix, whole, stepped
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".chars().nth("));
    assert!(output.code.contains(".step_by("));
    assert!(output.code.contains("collect::<String>()"));
}

#[test]
fn typescript_output_supports_unicode_textus_index_and_range_slice() {
    let session = session(Target::TypeScript);
    let source = r#"incipit {
  fixum _ ch ← "Salve, §!"[7]
  fixum _ prefix ← "hello world"[0‥5]
  fixum _ whole ← "hello world"[0 usque 10]
  fixum _ stepped ← "abcdef"[0‥6 per 2]
  nota ch, prefix, whole, stepped
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::TypeScript(output)) = result.output else {
        panic!("expected TypeScript output");
    };
    assert!(output.code.contains("Array.from("));
    assert!(output.code.contains(".filter((_, __faber_i)"));
    assert!(output.code.contains(".join(\"\")"));
}

#[test]
fn go_output_supports_unicode_textus_index_and_range_slice() {
    let session = session(Target::Go);
    let source = r#"incipit {
  fixum _ ch ← "Salve, §!"[7]
  fixum _ prefix ← "hello world"[0‥5]
  fixum _ whole ← "hello world"[0 usque 10]
  fixum _ stepped ← "abcdef"[0‥6 per 2]
  nota ch, prefix, whole, stepped
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Go(output)) = result.output else {
        panic!("expected Go output");
    };
    assert!(output.code.contains("[]rune("));
    assert!(output.code.contains("__faber_step"));
    assert!(output.code.contains("return string(__faber_out)"));
}

#[test]
fn rust_output_converts_textus_literals_to_owned_strings() {
    let session = session(Target::Rust);
    let source = r#"functio pick(bivalens flag) → textus {
  si flag {
    redde "yes"
  }
  redde "no"
}

incipit {
  varia textus name ← "Marcus"
  fixum textus[] words ← ["a", "b"]
  fixum textus ∪ nihil maybe ← nihil
  fixum _ fallback ← maybe vel "x"
  nota pick(verum), name, words, fallback
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output
        .code
        .contains("let mut name: String = \"Marcus\".to_string();"));
    assert!(output.code.contains("return \"yes\".to_string();"));
    assert!(output
        .code
        .contains("vec![\"a\".to_string(), \"b\".to_string()]"));
    assert!(output.code.contains("unwrap_or(\"x\".to_string())"));
}

#[test]
fn rust_output_wraps_optional_local_initializers_in_some() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum textus ∪ nihil actualName ← "Marcus"
  fixum textus ∪ nihil maybe ← nihil
  nota actualName, maybe
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output
        .code
        .contains("let actualName: Option<String> = Some(\"Marcus\".to_string());"));
    assert!(output.code.contains("let maybe: Option<String> = None;"));
}

#[test]
fn rust_output_uses_or_for_option_coalesce() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum textus ∪ nihil a ← nihil
  fixum textus ∪ nihil b ← nihil
  fixum _ first ← a vel b
  nota first
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".or("));
}

#[test]
fn rust_output_uses_debug_format_for_collection_scribe() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum textus[] names ← ["a", "b"]
  nota names
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("println!(\"{:?}\", names);"));
}

#[test]
fn rust_output_avoids_redundant_parentheses_in_common_contexts() {
    let session = session(Target::Rust);
    let source = r#"functio sum(numerus a, numerus b) → numerus {
  redde a + b
}

incipit {
  varia numerus x ← 1
  x ← x + 10
  dum x > 0 {
    si x > 5 {
      nota sum(x * 2, x + 1)
      rumpe
    }
    x ← x - 1
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(!output.code.contains("if ("));
    assert!(!output.code.contains("while ("));
    assert!(!output.code.contains("return (a + b);"));
    assert!(!output.code.contains("sum((x * 2), (x + 1))"));
}

#[test]
fn rust_output_normalizes_relative_import_paths() {
    let session = session(Target::Rust);
    let source = r#"importa ex "../../norma/hal/consolum" privata consolum
importa ex "./commands/greet" privata greet
importa ex "@hono/hono" privata Context

incipit {
  nota "ok"
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output
        .code
        .contains("use crate::norma::hal::consolum::consolum;"));
    assert!(output.code.contains("use crate::commands::greet::greet;"));
    assert!(!output.code.contains("..::"));
    assert!(!output.code.contains(".::"));
    assert!(!output.code.contains("@hono"));
}

#[test]
fn ego_field_access_no_longer_reports_non_struct_member_error() {
    let session = session(Target::Rust);
    let source = r#"genus Counter {
  numerus count: 0
  functio inc() → numerus {
    ego.count = ego.count + 1
    redde ego.count
  }
}

incipit {
  fixum _ c = {} ⇢ Counter
  nota c.inc()
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("field access on non-struct value")));
}

#[test]
fn array_method_call_no_longer_reports_non_struct_member_error() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ numbers = [1, 2, 3]
  fixum _ doubled = numbers.map(clausura numerus x: x * 2)
  nota doubled
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("method call on non-struct value")));
}

#[test]
fn interface_method_call_no_longer_reports_non_struct_member_error() {
    let session = session(Target::Rust);
    let source = r#"pactum Drawable {
  functio draw() → vacuum
}

genus Circle implet Drawable {
  functio draw() {
    nota "ok"
  }
}

functio render(Drawable d) → vacuum {
  d.draw()
}

incipit {
  fixum _ c = {} ⇢ Circle
  render(c)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("method call on non-struct value")));
}

#[test]
fn object_member_and_index_chains_no_longer_report_index_errors() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ config = { name: "test", value: 42 }
  nota config["name"]

  fixum _ data = { items: ["first", "second"] }
  nota data.items[0]
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("array index must be numerus")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("indexing requires array or map")));
}

#[test]
fn array_method_closure_argument_no_longer_reports_argument_type_mismatch() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ numbers = [1, 2, 3]
  nota numbers.map(clausura numerus x: x * 2)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("argument type mismatch")));
}

#[test]
fn compact_inferred_closure_argument_uses_expected_method_signature() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ numbers ← [1, 2, 3]
  nota numbers.map(_ x ∴ x * 2)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success(), "{:?}", result.diagnostics);
}

#[test]
fn compact_fac_closure_body_typechecks_with_redde() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ predicate ← numerus n → bivalens ∴ fac {
    redde n > 1
  }
  nota predicate(2)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success(), "{:?}", result.diagnostics);
}

#[test]
fn compact_fac_closure_cape_body_typechecks_with_redde() {
    let session = session(Target::Faber);
    let source = r#"functio parseFlag(textus value) → bivalens ⇥ textus {
  si value ≡ "bad" ergo iace "bad"
  redde verum
}

incipit {
  fixum _ predicate ← textus value → bivalens ∴ fac {
    redde parseFlag(value)
  } cape err {
    redde falsum
  }
  nota predicate("bad")
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success(), "{:?}", result.diagnostics);
}

#[test]
fn legacy_block_closure_body_typechecks_with_redde() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ predicate ← clausura numerus n → bivalens {
    redde n > 1
  }
  nota predicate(2)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success(), "{:?}", result.diagnostics);
}

#[test]
fn faber_output_prefers_compact_closure_syntax() {
    let session = session(Target::Faber);
    let source = r#"incipit {
  fixum _ double ← clausura numerus x: x * 2
}"#;
    let result = compile(&session, "test.fab", source);

    let Some(crate::Output::Faber(output)) = result.output else {
        panic!("expected Faber output");
    };
    assert!(output.code.contains("numerus x ∴ x * 2"));
    assert!(!output.code.contains("clausura numerus x"));
}

#[test]
fn module_method_call_in_condition_no_longer_leaves_infer_type() {
    let session = session(Target::Rust);
    let source = r#"importa ex "../../norma/hal/consolum" privata consolum

incipit {
  si consolum.estTerminale() {
    nota "Interactive mode"
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("condition must be bivalens")));
}

#[test]
fn module_method_call_statements_no_longer_leave_infer_type() {
    let session = session(Target::Rust);
    let source = r#"importa ex "../../norma/hal/consolum" privata consolum

incipit {
  consolum.fundeLineam("x")
  consolum.errorLineam("y")
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
}

#[test]
fn itera_de_array_index_no_longer_leaves_infer_types() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ xs ← [10, 20, 30]
  itera de xs fixum idx {
    nota xs[idx]
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
}

#[test]
fn itera_pro_range_no_longer_leaves_infer_types() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  itera pro 0‥5 fixum i {
    nota i
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
}

#[test]
fn itera_pro_inclusive_range_glyph_compiles() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  itera pro 0…5 fixum i {
    nota i
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
}

#[test]
fn deferred_local_assignment_can_drive_inference() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  varia value
  value = 42
  nota value
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer variable type")));
    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("variable declaration needs a type or initializer")));
}

#[test]
fn cura_allocator_scope_no_longer_reports_infer_variable_error() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  cura "arena" fixum _ alloc {
    nota "ok"
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer variable type")));
}

#[test]
fn curata_alias_binding_is_visible_in_function_body() {
    let session = session(Target::Rust);
    let source = r#"functio useAllocator() curata alloc ut a {
  nota a
}

incipit {
  useAllocator()
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown identifier")));
}

#[test]
fn compile_supports_extended_binary_operators() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ a ← 1
  fixum _ b ← 2
  fixum numerus ∪ nihil maybe ← nihil

  nota a ≡ b
  nota a ≠ b
  nota maybe est nihil
  nota a intra 0‥3
  nota a inter [1, 2, 3]
  nota maybe vel 0
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unsupported binary operator")));
}

#[test]
fn compile_supports_extended_unary_operators() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ flag ← verum
  fixum textus ∪ nihil maybe ← nihil
  fixum _ n ← -3

  nota non flag
  nota nulla maybe
  nota nonnulla maybe
  nota nihil maybe
  nota nonnihil maybe
  nota negativum n
  nota positivum n
  nota verum flag
  nota falsum flag
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unsupported unary operator")));
}

#[test]
fn compile_lowers_top_level_proba_to_rust_test_function() {
    let session = session(Target::Rust);
    let source = r#"proba "one plus one equals two" {
  adfirma 1 + 1 ≡ 2
}

incipit {}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unhandled statement kind in lowering")));
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("#[test]"));
    assert!(output.code.contains("fn proba_"));
}

#[test]
fn compile_lowers_proba_omit_and_futurum_as_ignored_tests() {
    let session = session(Target::Rust);
    let source = r#"proba "case one" omitte "blocked" {
  adfirma falsum
}

proba "case two" futurum "todo" {
  adfirma verum
}

incipit {}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unhandled statement kind in lowering")));
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output
        .code
        .contains("#[ignore = \"faber: omitte - blocked\"]"));
    assert!(output
        .code
        .contains("#[ignore = \"faber: futurum - todo\"]"));
}

#[test]
fn compile_lowers_probandum_nested_cases_without_lowering_errors() {
    let session = session(Target::Rust);
    let source = r#"probandum "suite" {
  praepara omnia {
    fixum _ x = 1
    nota x
  }

  proba "case one" {
    adfirma 1 ≡ 1
  }

  probandum "nested" {
    proba "case two" {
      adfirma verum
    }
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unhandled statement kind in lowering")));
}

#[test]
fn ego_field_assignment_no_longer_reports_assignment_type_mismatch() {
    let session = session(Target::Rust);
    let source = r#"genus Circulus {
  varia numerus diameter: 1
  functio crescere(numerus factor) → vacuum {
    ego.diameter = ego.diameter * factor
  }
}

incipit {
  fixum _ c = {} ⇢ Circulus
  c.crescere(2)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("assignment type mismatch")));
}

#[test]
fn typed_array_index_assignment_no_longer_reports_assignment_type_mismatch() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  varia numerus[] numbers = [3, 7]
  numbers[0] = 9
  nota numbers[0]
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("assignment type mismatch")));
}

#[test]
fn ex_destructured_object_fields_can_be_used_in_arithmetic() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ point = { x: 4, y: 6 } ⇢ Point
  ex point fixum x, y
  fixum numerus sum = x + y
  nota sum
}

genus Point {
  numerus x: 0
  numerus y: 0
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("expression type mismatch")));
}

#[test]
fn object_destructured_aliases_can_be_used_in_arithmetic() {
    let session = session(Target::Rust);
    let source = r#"functio totalis(Point point) → numerus {
  fixum { x ut left, y } ← point
  redde left + y
}

genus Point {
  numerus x = 0
  numerus y = 0
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(
        result.success(),
        "expected object destructuring aliases to compile, got {:?}",
        result.diagnostics
    );
}

#[test]
fn array_destructured_vars_can_be_used_in_arithmetic() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ coords = [100, 200]
  varia [x, y] = coords
  x = x + 50
  y = y + 50
  nota x, y
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("numeric operands required")));
}

#[test]
fn method_return_values_can_participate_in_numeric_comparisons() {
    let session = session(Target::Rust);
    let source = r#"functio accessArray(lista<numerus> items, numerus index) → numerus {
  si index < 0 aut index ≥ items.longitudo() {
    mori "Index out of bounds"
  }
  redde items[index]
}

incipit {
  fixum _ nums = [1, 2, 3]
  nota accessArray(nums, 1)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("numeric operands required")));
}

#[test]
fn repeated_owned_calls_no_longer_report_use_after_move() {
    let session = session(Target::Rust);
    let source = r#"functio sumArray(numerus[] nums) → numerus {
  varia numerus total = 0
  itera ex nums fixum n {
    total = total + n
  }
  redde total
}

functio maxValue(numerus[] nums) → numerus {
  varia numerus max = nums[0]
  itera ex nums fixum n {
    si n > max {
      max = n
    }
  }
  redde max
}

incipit {
  fixum _ numbers = [1, 2, 3, 4, 5]
  nota sumArray(numbers)
  nota maxValue(numbers)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("use after move")));
}

#[test]
fn assignment_and_tempta_flow_no_longer_report_use_after_move() {
    let session = session(Target::Rust);
    let source = r#"functio process(textus name) → vacuum {
  varia _ resource = "pending"
  tempta {
    resource = name
    si name ≡ "" {
      iace "Empty name"
    }
    nota resource
  }
  cape err {
    nota err
  }
  demum {
    nota resource
  }
}

incipit {
  process("")
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("use after move")));
}

#[test]
fn empty_array_and_spread_literals_no_longer_report_annotation_or_type_errors() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ empty = []
  fixum _ first = [1, 2, 3]
  fixum _ extended = [0, sparge first, 99]
  nota empty, extended
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("empty array needs type annotation")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("incompatible types")));
}

#[test]
fn cursor_iteration_accumulator_from_empty_array_no_longer_reports_inference_error() {
    let session = session(Target::Rust);
    let source = r#"@ cursor
functio rangeSync(numerus n) → numerus {
  itera pro 0‥n fixum i {
    cede i
  }
}

incipit {
  varia _ syncResults = []
  itera ex rangeSync(5) fixum num {
    syncResults.appende(num * 2)
  }
  nota syncResults
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer expression type")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("empty array needs type annotation")));
}

#[test]
fn optional_chain_no_longer_reports_lowering_stub() {
    let session = session(Target::Rust);
    let source = r#"genus User {
  textus name: "Anon"
  lista<numerus> nums: [1, 2, 3]
}

functio id(textus x) → textus {
  redde x
}

incipit {
  fixum User ∪ nihil maybeUser ← nihil
  fixum _ a ← maybeUser?.name
  fixum _ b ← maybeUser?.nums?[0]
  fixum (textus) → textus ∪ nihil maybeFn ← id
  fixum _ c ← maybeFn?("x")
  nota a, b, c
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("STUB: optional-chain lowering requires dedicated null-safe HIR node")));
}

#[test]
fn rust_output_emits_option_map_and_and_then_for_optional_chain() {
    let session = session(Target::Rust);
    let source = r#"genus User {
  textus name: "Anon"
}

functio id(textus x) → textus {
  redde x
}

incipit {
  fixum User ∪ nihil maybeUser ← nihil
  fixum _ a ← maybeUser?.name
  fixum (textus) → textus ∪ nihil maybeFn ← id
  fixum _ c ← maybeFn?("x")
  nota a, c
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output
        .code
        .contains(".as_ref().map(|__faber_opt| __faber_opt."));
    assert!(output.code.contains("__faber_opt.name.clone()"));
    assert!(output
        .code
        .contains(".and_then(|__faber_opt| Some(__faber_opt("));
}

#[test]
fn rust_output_translates_stdlib_lista_methods() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  varia _ xs ← [1, 2]
  xs.appende(3)
  fixum _ n ← xs.longitudo()
  nota n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".push(3)"));
    assert!(output.code.contains(".len() as i64"));
    compile_rust_source_with_rustc(&output.code, "stdlib-lista-methods");
}

#[test]
fn struct_construction_requires_fields_and_checks_defaults() {
    let session = session(Target::Rust);
    let missing = r#"genus User {
  textus name
  textus email sponte
}

incipit {
  fixum _ u ← { email: "a@example.com" } ⇢ User
}"#;
    let missing_result = compile(&session, "missing.fab", missing);
    assert!(!missing_result.success());
    assert!(missing_result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("missing required struct field")));

    let bad_default = r#"genus User {
  numerus score : "bad"
}

incipit {
  fixum _ u ← { } ⇢ User
}"#;
    let default_result = compile(&session, "default.fab", bad_default);
    assert!(!default_result.success());
    assert!(default_result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("field default type mismatch")));
}

#[test]
fn ab_pipeline_no_longer_reports_lowering_stub() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ items ← [
    { valor: 10, visibilis: verum },
    { valor: 20, visibilis: falsum },
    { valor: 30, visibilis: verum }
  ]
  fixum _ nums ← [1, 2, 3, 4, 5]
  fixum _ visible ← ab items visibilis, prima 2
  fixum _ sumFirst ← ab nums, prima 3, summa
  nota visible, sumFirst
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("STUB: collection pipeline lowering requires dedicated HIR node")));
}

#[test]
fn rust_output_emits_iterator_pipeline_for_ab_expr() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ items ← [
    { valor: 10, visibilis: verum },
    { valor: 20, visibilis: falsum },
    { valor: 30, visibilis: verum }
  ]
  fixum _ nums ← [1, 2, 3, 4, 5]
  fixum _ top ← ab items visibilis, prima 2
  fixum _ total ← ab nums, prima 3, summa
  nota top, total
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".iter()"));
    assert!(output.code.contains(".filter("));
    assert!(output.code.contains(".take("));
    assert!(output.code.contains(".sum::<i64>()"));
}

#[test]
fn objectum_return_type_no_longer_reports_unknown_type() {
    let session = session(Target::Rust);
    let source = r#"functio getResponse() → objectum {
  redde { body: "ok" }
}

incipit {
  fixum _ response ← getResponse()
  nota response
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown type")));
}

#[test]
fn curator_param_type_is_not_the_allocator_contract() {
    let session = session(Target::Rust);
    let source = r#"functio createUser(textus name, curator alloc) → textus {
  redde name
}

incipit {
  nota "ok"
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("unknown type")));
}

#[test]
fn quidlibet_container_annotation_no_longer_reports_unknown_type() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum lista<quidlibet> docs ← [] ⇢ lista<quidlibet>
  nota docs
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown type")));
}

#[test]
fn conversio_type_params_no_longer_report_unknown_type() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ n ← "ff" ⇒ numerus<i32, Hex>
  nota n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown type")));
}

#[test]
fn verte_vel_no_longer_reports_invalid_cast() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ data ← 42
  fixum _ asText ← data ⇢ textus
  fixum _ parsed ← "invalid" ⇒ numerus vel 0
  fixum _ cache ← { alice: 95, bob: 87 } ⇢ tabula<textus, numerus>
  fixum _ items ← [] ⇢ lista<textus>
  nota asText, parsed, cache, items
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("invalid cast")));
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("empty array needs type annotation")));
}

#[test]
fn rust_output_emits_verte_construction_and_coalesce_unwrap() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum textus ∪ nihil name ← nihil
  fixum _ display ← name vel "Anonymous"
  fixum _ cache ← { alice: 95 } ⇢ tabula<textus, numerus>
  fixum _ items ← [] ⇢ lista<textus>
  nota display, cache, items
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".unwrap_or("));
    assert!(output
        .code
        .contains("std::collections::HashMap::<String, i64>::new()"));
    assert!(output.code.contains(".insert(\"alice\".to_string(), 95)"));
    assert!(output.code.contains("Vec::<String>::new()"));
}

#[test]
fn rust_output_wraps_sponte_fields_in_verte_struct_literals() {
    let session = session(Target::Rust);
    let source = r#"genus User {
  textus name
  textus email sponte
  numerus score sponte
}

incipit {
  fixum _ a ← { name: "Ada" } ⇢ User
  fixum _ b ← { name: "Lin", email: "lin@example.com" } ⇢ User
  nota a.name
  nota b.name
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("pub email: Option<String>,"));
    assert!(output.code.contains("pub score: Option<i64>,"));
    assert!(output.code.contains("email: None,"));
    assert!(output.code.contains("score: None,"));
    assert!(output
        .code
        .contains("email: Some(\"lin@example.com\".to_string()),"));
}

#[test]
fn rust_output_applies_nullable_field_defaults_in_verte_struct_literals() {
    let session = session(Target::Rust);
    let source = r#"genus User {
  textus name
  textus nickname : "Anonymous"
  textus ∪ nihil email : "Anonymous"
}

incipit {
  fixum _ a ← { name: "Ada" } ⇢ User
  fixum _ b ← { name: "Lin", email: nihil } ⇢ User
  fixum _ c ← { name: "Ken", email: "ken@example.com" } ⇢ User
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("nickname: \"Anonymous\".to_string(),"));
    assert!(output.code.contains("pub email: Option<String>,"));
    assert!(output
        .code
        .contains("email: Some(\"Anonymous\".to_string()),"));
    assert!(output.code.contains("email: None,"));
    assert!(output
        .code
        .contains("email: Some(\"ken@example.com\".to_string()),"));
    assert!(!output.code.contains(".to_string().to_string()"));
    compile_rust_source_with_rustc(&output.code, "nullable-field-default");
}

#[test]
fn ignotum_callee_no_longer_reports_not_callable() {
    let session = session(Target::Rust);
    let source = r#"functio invoke(ignotum callee) → ignotum {
  redde callee(1)
}

incipit {
  nota invoke
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("callee is not callable")));
}

#[test]
fn elige_accepts_enum_variant_case_values() {
    let session = session(Target::Rust);
    let source = r#"ordo Color { rubrum, viridis, caeruleum }

incipit {
  fixum _ color ← Color.rubrum
  elige color {
    casu Color.rubrum { nota "R" }
    casu Color.viridis { nota "G" }
    casu Color.caeruleum { nota "B" }
  }
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("elige case value must be a literal")));
}

#[test]
fn ordo_exemplum_no_longer_reports_elige_literal_error() {
    let session = session(Target::Rust);
    let source = std::fs::read_to_string("../../examples/exempla/ordo/ordo.fab").expect("read ordo exemplum");
    let result = compile(&session, "ordo.fab", &source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("elige case value must be a literal")));
}

#[test]
fn optional_params_no_longer_require_all_arguments() {
    let session = session(Target::Rust);
    let source = r#"functio greet(textus nomen, textus titulus sponte) → textus {
  si titulus est nihil {
    redde nomen
  }
  redde titulus
}

functio paginate(numerus pagina sponte vel 1, numerus per_pagina sponte vel 10) → numerus {
  redde pagina + per_pagina
}

functio analyze(textus source, de numerus depth sponte) → numerus {
  si depth est nihil {
    redde source.longitudo()
  }
  redde depth
}

incipit {
  nota greet("Marcus")
  nota greet("Marcus", "Dominus")
  nota paginate()
  nota paginate(2)
  nota paginate(2, 25)
  nota analyze("code")
  nota analyze("code", 5)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("wrong number of arguments")));
}

#[test]
fn nullable_union_type_forms_are_canonicalized() {
    let session = session(Target::Rust);
    let source = r#"typus MaybeValue = textus ∪ numerus ∪ nihil
typus EitherValue = textus ∪ numerus

functio nullable_alias() → MaybeValue {
  redde 1
}

functio plain_alias() → EitherValue {
  redde "ok"
}

functio duplicate_nullable() → textus ∪ textus ∪ nihil {
  redde "ok"
}

functio nullable_direct() → textus ∪ nihil {
  redde nihil
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(
        result.success(),
        "expected canonical union forms to compile: {:?}",
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn plain_union_does_not_accept_nihil_without_nullable_member() {
    let session = session(Target::Rust);
    let source = r#"functio badplain() → textus ∪ numerus {
  redde nihil
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("return type does not match function signature")));
}

#[test]
fn degenerate_nihil_only_union_is_rejected() {
    let session = session(Target::Rust);
    let source = r#"functio bad() → nihil ∪ nihil {
  redde nihil
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("union type cannot consist only of 'nihil'")));
}

#[test]
fn conversio_glyph_form_compiles_to_parse() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ n ← "22" ⇒ numerus
  nota n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".parse::<i64>().unwrap()"));
}

#[test]
fn conversio_emits_parse_not_as_cast() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ n ← "42" ⇒ numerus
  nota n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".parse::<i64>().unwrap()"));
    assert!(!output.code.contains(" as i64"));
}

#[test]
fn conversio_with_fallback_emits_unwrap_or() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ n ← "bad" ⇒ numerus vel 0
  nota n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".parse::<i64>().unwrap_or(0)"));
}

#[test]
fn conversio_to_textus_emits_to_string() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ n ← 42
  fixum _ s ← n ⇒ textus
  nota s
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(".to_string()"));
}

#[test]
fn verte_qua_still_emits_as_cast() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum _ n ← 42
  fixum _ f ← n ⇢ fractus
  nota f
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains(" as f64"));
}

#[test]
fn conversio_glyph_form_roundtrips_through_faber_codegen() {
    let session = session(Target::Faber);
    let source = r#"incipit {
  fixum _ n ← "22" ⇒ numerus
  nota n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Faber(output)) = result.output else {
        panic!("expected faber output");
    };
    assert!(output.code.contains("⇒ numerus"));
}

#[test]
fn conversio_with_fallback_roundtrips_through_faber_codegen() {
    let session = session(Target::Faber);
    let source = r#"incipit {
  fixum _ n ← "bad" ⇒ numerus vel 0
  nota n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Faber(output)) = result.output else {
        panic!("expected faber output");
    };
    assert!(output.code.contains("⇒ numerus vel 0"));
}

#[test]
fn fac_do_while_roundtrips_through_faber_codegen() {
    let source = r#"incipit {
  varia _ counter ← 0
  fac {
    counter ← counter + 1
    nota counter
  } dum counter < 3
}"#;

    let output = faber_roundtrip(source);
    assert!(output.contains("fac {"));
    assert!(output.contains("} dum counter < 3"));
}

#[test]
fn discerne_alias_and_multi_subject_roundtrip_through_faber_codegen() {
    let source = r#"discretio Event {
  Click { numerus x, numerus y },
  Quit
}

functio compare(Event left, Event right) → vacuum {
  discerne left, right {
    casu Click ut l, Quit ut r {
      nota l.x
      nota r
    }
    ceterum {
      nota "other"
    }
  }
}"#;

    let output = faber_roundtrip(source);
    assert!(output.contains("discerne left, right"));
    assert!(output.contains("casu Click ut l, Quit ut r"));
}

#[test]
fn ad_roundtrips_through_faber_codegen() {
    let source = r#"incipit {
  ad "fasciculus:lege" ("hello.txt") → textus pro content {
    nota content
  } cape err {
    nota err
  }
}"#;

    let output = faber_roundtrip(source);
    assert!(output.contains("ad \"fasciculus:lege\" (\"hello.txt\") → textus pro content {"));
    assert!(output.contains("cape err {"));
}

#[test]
fn rejects_old_verte_aliases_as_postfix_operators() {
    // Post clean-break: qua/innatum/novum no longer produce Verte tokens.
    // Using them in postfix position now yields clear parse diagnostics (not silent aliasing).
    let session = session(Target::Rust);
    for alias in &["qua", "innatum", "novum"] {
        // Use parenthesized expr to force postfix cast site and unambiguous parse error.
        let source = format!("incipit {{ nota (42 {} textus) }}", alias);
        let result = compile(&session, "test.fab", &source);

        assert!(!result.success(), "source using old postfix alias '{}' must fail", alias);
        assert!(
            result.diagnostics.iter().any(|d| d.is_error()),
            "expected at least one error diagnostic for old alias '{}'",
            alias
        );
        // The precise message is parser-level "expected ')'" (or similar) at the alias token,
        // because the ident no longer matches the Verte check in parse_postfix.
        // We assert a clear error position around the alias rather than brittle string match.
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.is_error() && d.span.is_some_and(|s| s.start > 10 && s.start < 30)),
            "error should be reported near the alias token for '{}'",
            alias
        );
    }
}
