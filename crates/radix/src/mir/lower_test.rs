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
fn failable_function_dump_renders_alternate_exit_type() {
    let dump = dump_source(r#"functio fail() → numerus ⇥ textus { iace "bad" }"#);

    assert!(dump.starts_with("function f0 -> ty#1 ⇥ ty#0 {\n"));
    assert!(dump.contains("  temps:\n    %0: ty#0\n"));
    assert!(dump.contains("    %0 = const string sym#"));
    assert!(dump.contains("    return_error %0\n"));
}

#[test]
fn lowers_iace_to_return_error() {
    let dump = dump_source(
        r#"functio divide(numerus a, numerus b) → numerus ⇥ textus {
  si b ≡ 0 ergo iace "division by zero"
  redde a / b
}"#,
    );

    assert!(dump.contains("function f0 -> ty#1 ⇥ ty#0 {\n"));
    assert!(dump.contains("branch %0 bb1 bb2"));
    assert!(dump.contains("return_error %1"));
    assert!(dump.contains("return %2"));
}

#[test]
fn source_level_failable_calls_are_rejected_before_mir() {
    let session = Session::new(Config::default().with_target(Target::Faber));
    let source = r#"
functio fail() → numerus ⇥ textus { iace "bad" }
functio caller() → numerus ⇥ textus { redde fail() }
"#;

    let errors = match crate::driver::analyze_source(&session, "test.fab", source) {
        Ok(_) => panic!("ordinary failable calls should not reach MIR lowering"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|diagnostic| diagnostic
        .message
        .contains("failable call requires handling")));
}

#[test]
fn lowers_mori_to_panic_runtime_call_and_unreachable() {
    let dump = dump_source(r#"functio impossible() → vacuum { mori "impossible state" }"#);

    assert!(dump.contains("function f0 -> ty#5 {\n"));
    assert!(dump.contains("runtime panic(const string sym#"));
    assert!(dump.contains(") -> ty#6\n"));
    assert!(dump.contains("    unreachable\n"));
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
    let errors = lower_analyzed_unit(&unit).expect_err("non-empty entry is unsupported in phase 5B");

    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("unsupported MIR lowering in phase 5B: non-empty entry blocks"));
}

#[test]
fn rejects_unsupported_top_level_items_explicitly() {
    let unit = analyze("genus Persona { textus nomen }");
    let errors = lower_analyzed_unit(&unit).expect_err("structs are unsupported in phase 5B");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "unsupported MIR lowering in phase 5B: top-level struct");
}

#[test]
fn lowers_si_to_branch_and_join_blocks() {
    let dump = dump_source("functio signum(numerus n) → numerus { si n > 0 ergo redde n redde 0 }");

    assert_eq!(
        dump,
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
fn lowers_expression_valued_si_into_shared_destination() {
    let dump = dump_source("functio positum(numerus n) → numerus { fixum numerus x ← n > 0 sic 1 secus 0 redde x }");

    assert_eq!(
        dump,
        "\
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
    let _1: ty#1
  temps:
    %0: ty#3
  bb0:
    %0 = _0 > const int 0: ty#3
    branch %0 bb1 bb2
  bb1:
    _1 = const int 1: ty#1
    goto bb3
  bb2:
    _1 = const int 0: ty#1
    goto bb3
  bb3:
    return _1
}
"
    );
}

#[test]
fn lowers_dum_to_condition_body_and_after_blocks() {
    let dump = dump_source(
        "functio totalis(numerus n) → numerus { varia numerus i ← 0 varia numerus total ← 0 dum i < n { total ← total + i i ← i + 1 } redde total }",
    );

    assert_eq!(
        dump,
        "\
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
    var _1: ty#1
    var _2: ty#1
  temps:
    %0: ty#3
    %1: ty#1
    %2: ty#1
  bb0:
    _1 = const int 0: ty#1
    _2 = const int 0: ty#1
    goto bb1
  bb1:
    %0 = _1 < _0: ty#3
    branch %0 bb2 bb3
  bb2:
    %1 = _2 + _1: ty#1
    _2 = %1: ty#1
    %2 = _1 + const int 1: ty#1
    _1 = %2: ty#1
    goto bb1
  bb3:
    return _2
}
"
    );
}

#[test]
fn lowers_rumpe_and_perge_to_loop_targets() {
    let dump = dump_source(
        "functio primus(numerus n) → numerus { varia numerus i ← 0 dum i < n { i ← i + 1 si i < 3 ergo perge rumpe } redde i }",
    );

    assert_eq!(
        dump,
        "\
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
    var _1: ty#1
  temps:
    %0: ty#3
    %1: ty#1
    %2: ty#3
  bb0:
    _1 = const int 0: ty#1
    goto bb1
  bb1:
    %0 = _1 < _0: ty#3
    branch %0 bb2 bb3
  bb2:
    %1 = _1 + const int 1: ty#1
    _1 = %1: ty#1
    %2 = _1 < const int 3: ty#3
    branch %2 bb4 bb5
  bb3:
    return _1
  bb4:
    goto bb1
  bb5:
    goto bb3
}
"
    );
}

#[test]
fn closed_si_arms_do_not_emit_spurious_join_edges() {
    let dump = dump_source(
        "functio opta(numerus n) → numerus { varia numerus x ← n si x > 0 { redde x } secus { x ← x + 1 } redde x }",
    );

    assert_eq!(
        dump,
        "\
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
    var _1: ty#1
  temps:
    %0: ty#3
    %1: ty#1
  bb0:
    _1 = _0: ty#1
    %0 = _1 > const int 0: ty#3
    branch %0 bb1 bb2
  bb1:
    return _1
  bb2:
    %1 = _1 + const int 1: ty#1
    _1 = %1: ty#1
    goto bb3
  bb3:
    return _1
}
"
    );
}

#[test]
fn rejects_deferred_control_flow_constructs_with_explicit_diagnostics() {
    let iter_unit = analyze(
        "functio iterare(lista<numerus> nums) → numerus { varia numerus total ← 0 itera ex nums fixum n { total ← total + n } redde total }",
    );
    let iter_errors = lower_analyzed_unit(&iter_unit).expect_err("itera is deferred");
    assert_eq!(iter_errors.len(), 1);
    assert!(iter_errors[0]
        .message
        .contains("itera before iterator MIR lowering"));

    let discerne_unit =
        analyze("functio differ(numerus n) → numerus { discerne n { casu 0 { redde 0 } casu _ { redde n } } }");
    let discerne_errors = lower_analyzed_unit(&discerne_unit).expect_err("discerne is deferred");
    assert_eq!(discerne_errors.len(), 1);
    assert!(discerne_errors[0]
        .message
        .contains("discerne before switch MIR lowering"));

    let tempta_unit = analyze("functio temptare() → numerus { tempta { redde 1 } cape err { redde 0 } }");
    let tempta_errors = lower_analyzed_unit(&tempta_unit).expect_err("tempta is deferred");
    assert_eq!(tempta_errors.len(), 1);
    assert!(tempta_errors[0]
        .message
        .contains("tempta legacy local-handler surface deferred to Phase 5C"));
}

#[test]
fn tempta_with_inner_iace_remains_fail_closed_as_a_whole() {
    let unit = analyze(
        r#"functio handled() → numerus ⇥ textus {
  tempta {
    iace "later"
  } cape err {
    redde 0
  }
  redde 1
}"#,
    );
    let errors = lower_analyzed_unit(&unit).expect_err("tempta remains deferred even when it contains iace");

    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("tempta legacy local-handler surface deferred to Phase 5C"));
}

#[test]
fn rejects_fabricated_iace_without_alternate_exit_type() {
    use crate::hir::{HirExpr, HirExprKind, HirId, HirLiteral, HirStmt, HirStmtKind};

    let mut unit = analyze("functio fail() → numerus { redde 0 }");
    let textus = unit.types.primitive(crate::semantic::Primitive::Textus);
    let span = crate::lexer::Span::default();
    let crate::hir::HirItemKind::Function(function) = &mut unit.hir.items[0].kind else {
        panic!("expected function");
    };
    function.err_ty = None;
    function.body = Some(crate::hir::HirBlock {
        stmts: vec![HirStmt {
            id: HirId(10),
            kind: HirStmtKind::Expr(HirExpr {
                id: HirId(11),
                kind: HirExprKind::Throw(Box::new(HirExpr {
                    id: HirId(12),
                    kind: HirExprKind::Literal(HirLiteral::String(crate::lexer::Symbol(99))),
                    ty: Some(textus),
                    span,
                })),
                ty: Some(unit.types.primitive(crate::semantic::Primitive::Vacuum)),
                span,
            }),
            span,
        }],
        expr: None,
        span,
    });

    let errors = lower_analyzed_unit(&unit).expect_err("fabricated iace without ⇥ should fail");
    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("iace without a declared alternate-exit type"));
}

#[test]
fn rejects_diagnostic_verbs_with_construct_specific_diagnostics() {
    let unit = analyze(r#"functio malum() { nota "salve" }"#);
    let errors = lower_analyzed_unit(&unit).expect_err("nota is not phase 5B MIR");

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
    let entry = builder.fresh_block(span);
    builder.switch_to(entry);

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
    assert!(builder.blocks[0].statements.is_empty());
    assert_eq!(builder.errors.len(), 1);
    assert!(builder.errors[0]
        .message
        .contains("assignment target that is not a local place"));
}
