use super::*;
use crate::codegen::Target;
use crate::driver::{Config, Session};

fn analyze(source: &str) -> AnalyzedUnit {
    let session = Session::new(Config::default().with_target(Target::Faber));
    crate::driver::analyze_source(&session, "test.fab", source).expect("analysis succeeds")
}

fn dump_source(source: &str) -> String {
    dump_analyzed_unit(&analyze(source)).expect("MIR lowering succeeds")
}

#[test]
fn lowers_empty_function_shell_to_mir_dump() {
    let dump = dump_source("functio saluta() {}");

    assert_eq!(
        dump,
        "\
function f0 -> ty#5 {
  bb0:
    return
}
"
    );
}

#[test]
fn lowers_function_params_into_mir_params() {
    let dump = dump_source("functio saluta(textus nomen, numerus aetas) {}");

    assert_eq!(
        dump,
        "\
function f0 -> ty#5 {
  params:
    _0: ty#0
    _1: ty#1
  bb0:
    return
}
"
    );
}

#[test]
fn lowers_empty_entry_block_as_synthetic_function() {
    let dump = dump_source("incipit {}");

    assert_eq!(
        dump,
        "\
function f0 -> ty#5 {
  bb0:
    return
}
"
    );
}

#[test]
fn rejects_non_empty_entry_blocks_with_explicit_unsupported_error() {
    let unit = analyze(r#"incipit { nota "salve" }"#);
    let errors = lower_analyzed_unit(&unit).expect_err("non-empty entry is unsupported in phase 2");

    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("unsupported MIR lowering in phase 2: non-empty entry blocks"));
}

#[test]
fn rejects_unsupported_top_level_items_explicitly() {
    let unit = analyze("genus Persona { textus nomen }");
    let errors = lower_analyzed_unit(&unit).expect_err("structs are unsupported in phase 2");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "unsupported MIR lowering in phase 2: top-level struct");
}
