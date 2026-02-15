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

    assert!(result.success());
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

#[test]
fn ego_field_access_no_longer_reports_non_struct_member_error() {
    let session = session(Target::Rust);
    let source = r#"genus Counter {
  numerus count: 0
  functio inc() -> numerus {
    ego.count = ego.count + 1
    redde ego.count
  }
}

incipit {
  fixum c = novum Counter
  scribe c.inc()
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
  fixum numbers = [1, 2, 3]
  fixum doubled = numbers.map(clausura numerus x: x * 2)
  scribe doubled
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
  functio draw() -> vacuum
}

genus Circle implet Drawable {
  functio draw() {
    scribe "ok"
  }
}

functio render(Drawable d) -> vacuum {
  d.draw()
}

incipit {
  fixum c = novum Circle
  render(c)
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("method call on non-struct value")));
}

#[test]
fn itera_de_array_index_no_longer_leaves_infer_types() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum xs = [10, 20, 30]
  itera de xs fixum idx {
    scribe xs[idx]
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
  itera pro 0..5 fixum i {
    scribe i
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
  scribe value
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
fn cura_arena_anonymous_scope_no_longer_reports_infer_variable_error() {
    let session = session(Target::Rust);
    let source = r#"incipit ergo cura arena {
  scribe "ok"
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("cannot infer variable type")));
}

#[test]
fn compile_supports_extended_binary_operators() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum a = 1
  fixum b = 2
  fixum si numerus maybe = nihil

  scribe a === b
  scribe a !== b
  scribe maybe est nihil
  scribe a intra 0..3
  scribe a inter [1, 2, 3]
  scribe maybe vel 0
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
  fixum flag = verum
  fixum si textus maybe = nihil
  fixum n = -3

  scribe non flag
  scribe nulla maybe
  scribe nonnulla maybe
  scribe nihil maybe
  scribe nonnihil maybe
  scribe negativum n
  scribe positivum n
  scribe verum flag
  scribe falsum flag
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
  adfirma 1 + 1 == 2
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
    let source = r#"proba omitte "blocked" "case one" {
  adfirma falsum
}

proba futurum "todo" "case two" {
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
    assert!(output.code.contains("#[ignore]"));
}

#[test]
fn compile_lowers_probandum_nested_cases_without_lowering_errors() {
    let session = session(Target::Rust);
    let source = r#"probandum "suite" {
  praepara omnia {
    fixum x = 1
    scribe x
  }

  proba "case one" {
    adfirma 1 == 1
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
  functio crescere(numerus factor) -> vacuum {
    ego.diameter = ego.diameter * factor
  }
}

incipit {
  fixum c = novum Circulus
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
  scribe numbers[0]
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
  fixum point = novum Point { x: 4, y: 6 }
  ex point fixum x, y
  fixum numerus sum = x + y
  scribe sum
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
fn optional_chain_no_longer_reports_lowering_stub() {
    let session = session(Target::Rust);
    let source = r#"genus User {
  textus name: "Anon"
  lista<numerus> nums: [1, 2, 3]
}

functio id(textus x) -> textus {
  redde x
}

incipit {
  fixum si User maybeUser = nihil
  fixum a = maybeUser?.name
  fixum b = maybeUser?.nums?[0]
  fixum si (textus) -> textus maybeFn = id
  fixum c = maybeFn?("x")
  scribe a, b, c
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

functio id(textus x) -> textus {
  redde x
}

incipit {
  fixum si User maybeUser = nihil
  fixum a = maybeUser?.name
  fixum si (textus) -> textus maybeFn = id
  fixum c = maybeFn?("x")
  scribe a, c
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust output");
    };
    assert!(output
        .code
        .contains(".as_ref().map(|__faber_opt| __faber_opt."));
    assert!(output
        .code
        .contains(".and_then(|__faber_opt| Some(__faber_opt("));
}

#[test]
fn ab_pipeline_no_longer_reports_lowering_stub() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum items = [
    { valor: 10, visibilis: verum },
    { valor: 20, visibilis: falsum },
    { valor: 30, visibilis: verum }
  ]
  fixum nums = [1, 2, 3, 4, 5]
  fixum visible = ab items visibilis, prima 2
  fixum sumFirst = ab nums, prima 3, summa
  scribe visible, sumFirst
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
  fixum items = [
    { valor: 10, visibilis: verum },
    { valor: 20, visibilis: falsum },
    { valor: 30, visibilis: verum }
  ]
  fixum nums = [1, 2, 3, 4, 5]
  fixum top = ab items visibilis, prima 2
  fixum total = ab nums, prima 3, summa
  scribe top, total
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
    let source = r#"functio getResponse() -> objectum {
  redde { body: "ok" }
}

incipit {
  fixum response = getResponse()
  scribe response
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown type")));
}

#[test]
fn curator_param_type_no_longer_reports_unknown_type() {
    let session = session(Target::Rust);
    let source = r#"functio createUser(textus name, curator alloc) -> textus {
  redde name
}

incipit {
  scribe "ok"
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown type")));
}

#[test]
fn quidlibet_container_annotation_no_longer_reports_unknown_type() {
    let session = session(Target::Rust);
    let source = r#"incipit {
  fixum lista<quidlibet> docs = [] innatum lista<quidlibet>
  scribe docs
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
  fixum n = "ff" numeratum<i32, Hex>
  scribe n
}"#;
    let result = compile(&session, "test.fab", source);

    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("unknown type")));
}
