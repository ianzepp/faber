use super::{compile, Config, Session};
use crate::codegen::Target;

fn session(target: Target) -> Session {
    Session::new(Config::default().with_target(target))
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
    let result = compile(&session, "test.fab", "incipit {\n  scribe nope\n}");

    assert!(result.output.is_none());
    assert!(!result.success());
    assert!(result.diagnostics.iter().any(|d| d.is_error()));
}

#[test]
fn rust_target_reports_cura_arena_noop_warning() {
    let session = session(Target::Rust);
    let source = "incipit {\n  cura arena fixum mem {\n  }\n}";
    let result = compile(&session, "test.fab", source);

    assert!(result.diagnostics.iter().any(|d| {
        !d.is_error()
            && d.message
                .contains("cura arena has no effect for Rust targets")
    }));
}

#[test]
fn compile_accepts_textus_concatenation_and_compound_add() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  varia textus s = "salve"
  s += "!"
  scribe "dicit: " + s
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn compile_accepts_finge_variant_construction() {
    let session = session(Target::Rust);
    let source = r#"discretio Event {
  Click { numerus x, numerus y },
  Quit
}

incipit {
  fixum Event e1 = finge Click { x: 1, y: 2 } qua Event
  fixum Event e2 = finge Quit qua Event
  scribe e1
  scribe e2
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn compile_accepts_array_and_ex_destructuring_bindings() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum xs = [1, 2]
  fixum [a, b] = xs
  scribe a
  scribe b

  fixum person = { name: "Marcus", age: 1 }
  ex person fixum name
  scribe name
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(matches!(result.output, Some(crate::Output::Rust(_))));
}

#[test]
fn compile_accepts_param_alias_binding() {
    let session = session(Target::Rust);
    let source = r#"functio greet(textus name, si bivalens formal ut f) -> vacuum {
  si f {
    scribe name
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
fn import_alias_usage_no_longer_reports_unknown_identifier() {
    let session = session(Target::Rust);
    let source = r#"importa ex "../../norma/hal/consolum" privata consolum

incipit {
  consolum.fundeLineam("x")
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown identifier")));
}

#[test]
fn ab_property_filter_no_longer_reports_unknown_identifier() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum users = [{ activus: verum }]
  fixum active = ab users activus
  scribe active
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown identifier")));
}

#[test]
fn compile_lowers_scriptum_without_stub_diagnostic() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum name = "Marcus"
  fixum msg = scriptum("salve, §!", name)
  scribe msg
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result.diagnostics.iter().all(|d| !d
        .message
        .contains("scriptum interpolation lowering is placeholder-only")));
}

#[test]
fn rust_output_uses_format_macro_for_scriptum() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum value = 7
  scribe scriptum("valor: §", value)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output.code.contains("format!(\"valor: {}\""));
}
