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
  locals:
    let _0: ty#0
    let _1: ty#1
  bb0:
    return
}
"
    );
}

#[test]
fn lowers_function_params_as_mir_locals() {
    let program =
        lower_analyzed_unit(&analyze("functio saluta(textus nomen, numerus aetas) {}")).expect("MIR lowering succeeds");
    let function = &program.functions[0];

    assert_eq!(function.params.len(), 2);
    assert_eq!(function.locals.len(), 2);
    assert_eq!(function.params[0].local, function.locals[0].id);
    assert_eq!(function.params[1].local, function.locals[1].id);
    assert!(!function.locals[0].mutable);
    assert!(!function.locals[1].mutable);
}

#[test]
fn lowers_explicit_no_value_redde_as_trivial_return() {
    let dump = dump_source("functio saluta() { redde }");

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
fn lowers_primitive_constants_and_explicit_redde() {
    let dump = dump_source(
        r#"functio constantes() → bivalens {
  varia textus t ← "salve"
  varia fractus f ← 1.5
  varia bivalens b ← verum
  redde b
}"#,
    );

    assert_eq!(
        dump,
        "\
function f0 -> ty#3 {
  locals:
    var _0: ty#0
    var _1: ty#2
    var _2: ty#3
  bb0:
    _0 = const string sym#4: ty#0
    _1 = const float 1.5: ty#2
    _2 = const bool true: ty#3
    return _2
}
"
    );
}

#[test]
fn materializes_constant_redde_operands_with_types() {
    let int_dump = dump_source("functio unum() → numerus { redde 1 }");
    assert_eq!(
        int_dump,
        "\
function f0 -> ty#1 {
  temps:
    %0: ty#1
  bb0:
    %0 = const int 1: ty#1
    return %0
}
"
    );

    let nil_dump = dump_source("functio nullum() → nihil { redde nihil }");
    assert_eq!(
        nil_dump,
        "\
function f0 -> ty#4 {
  temps:
    %0: ty#4
  bb0:
    %0 = const nil: ty#4
    return %0
}
"
    );
}

#[test]
fn lowers_params_and_local_reads_into_places() {
    let dump = dump_source("functio idem(numerus n) → numerus { redde n }");

    assert_eq!(
        dump,
        "\
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
  bb0:
    return _0
}
"
    );
}

#[test]
fn lowers_locals_assignment_and_binary_ops() {
    let dump = dump_source("functio computa() → numerus { varia numerus x ← 1 x ← x + 2 redde x }");

    assert_eq!(
        dump,
        "\
function f0 -> ty#1 {
  locals:
    var _0: ty#1
  temps:
    %0: ty#1
  bb0:
    _0 = const int 1: ty#1
    %0 = _0 + const int 2: ty#1
    _0 = %0: ty#1
    return _0
}
"
    );
}

#[test]
fn lowers_unary_ops_to_typed_temps() {
    let dump = dump_source("functio logicum(bivalens a) → bivalens { redde non a }");

    assert_eq!(
        dump,
        "\
function f0 -> ty#3 {
  params:
    _0: ty#3
  locals:
    let _0: ty#3
  temps:
    %0: ty#3
  bb0:
    %0 = not _0: ty#3
    return %0
}
"
    );
}

#[test]
fn lowers_direct_calls_to_definition_callees() {
    let dump =
        dump_source("functio duplex(numerus n) → numerus { redde n * 2 } functio usa() → numerus { redde duplex(4) }");

    assert_eq!(
        dump,
        "\
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
  temps:
    %0: ty#1
  bb0:
    %0 = _0 * const int 2: ty#1
    return %0
}

function f1 -> ty#1 {
  temps:
    %0: ty#1
  bb0:
    %0 = call def#0(const int 4)
    return %0
}
"
    );
}

#[test]
fn lowers_vacuum_direct_calls_without_destinations() {
    let dump = dump_source("functio ping() { redde } functio usa() { ping() redde }");

    assert_eq!(
        dump,
        "\
function f0 -> ty#5 {
  bb0:
    return
}

function f1 -> ty#5 {
  bb0:
    call def#0()
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
    let errors = lower_analyzed_unit(&unit).expect_err("non-empty entry is unsupported in phase 3");

    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("unsupported MIR lowering in phase 3: non-empty entry blocks"));
}

#[test]
fn rejects_unsupported_top_level_items_explicitly() {
    let unit = analyze("genus Persona { textus nomen }");
    let errors = lower_analyzed_unit(&unit).expect_err("structs are unsupported in phase 3");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "unsupported MIR lowering in phase 3: top-level struct");
}

#[test]
fn rejects_si_and_dum_with_construct_specific_diagnostics() {
    let si_unit = analyze("functio malum(numerus a) → numerus { si a > 0 { redde a } redde 0 }");
    let si_errors = lower_analyzed_unit(&si_unit).expect_err("si is phase 4 control flow");
    assert_eq!(si_errors.len(), 1);
    assert!(si_errors[0]
        .message
        .contains("si before control-flow MIR lowering"));

    let dum_unit = analyze("functio malum() { dum verum { } }");
    let dum_errors = lower_analyzed_unit(&dum_unit).expect_err("dum is phase 4 control flow");
    assert_eq!(dum_errors.len(), 1);
    assert!(dum_errors[0]
        .message
        .contains("dum before control-flow MIR lowering"));
}

#[test]
fn rejects_diagnostic_verbs_with_construct_specific_diagnostics() {
    let unit = analyze(r#"functio malum() { nota "salve" }"#);
    let errors = lower_analyzed_unit(&unit).expect_err("nota is not phase 3 MIR");

    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("nota before print/runtime intrinsic MIR lowering"));
}

#[test]
fn rejects_assignment_targets_that_are_not_places() {
    use crate::hir::{DefId, HirExpr, HirExprKind, HirId, HirLiteral};
    use crate::lexer::Symbol;
    use crate::semantic::{Primitive, TypeTable};

    let types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let span = crate::lexer::Span::default();
    let mut builder = FunctionBuilder::new(&types);
    builder.add_param(DefId(0), Symbol(0), numerus, span);

    let path = HirExpr { id: HirId(0), kind: HirExprKind::Path(DefId(0)), ty: Some(numerus), span };
    let one = HirExpr { id: HirId(1), kind: HirExprKind::Literal(HirLiteral::Int(1)), ty: Some(numerus), span };
    let lhs = HirExpr {
        id: HirId(2),
        kind: HirExprKind::Binary(crate::hir::HirBinOp::Add, Box::new(path), Box::new(one)),
        ty: Some(numerus),
        span,
    };
    let rhs = HirExpr { id: HirId(3), kind: HirExprKind::Literal(HirLiteral::Int(2)), ty: Some(numerus), span };
    let assignment =
        HirExpr { id: HirId(4), kind: HirExprKind::Assign(Box::new(lhs), Box::new(rhs)), ty: Some(numerus), span };

    assert!(builder.lower_assignment_expr(&assignment).is_none());
    assert!(builder.statements.is_empty());
    assert_eq!(builder.errors.len(), 1);
    assert!(builder.errors[0]
        .message
        .contains("assignment target that is not a local place"));
}
