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
    let dump = dump_source("functio saluta() → vacuum { redde }");

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
    let dump = dump_source("functio ping() → vacuum { redde } functio usa() → vacuum { ping() redde }");

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
fn lowers_non_empty_entry_block_as_synthetic_function_body() {
    let dump = dump_source(r#"incipit { fixum numerus n ← 1 + 2 }"#);

    assert_eq!(
        dump,
        "\
function f0 -> ty#5 {
  locals:
    let _0: ty#1
  temps:
    %0: ty#1
  bb0:
    %0 = const int 1 + const int 2: ty#1
    _0 = %0: ty#1
    return
}
"
    );
}

#[test]
fn non_empty_entry_preserves_unsupported_inner_diagnostics() {
    let unit = analyze(r#"incipit { tacet }"#);
    let errors = lower_analyzed_unit(&unit).expect_err("unsupported entry content should still fail closed");

    assert_eq!(errors.len(), 1);
    assert!(errors[0]
        .message
        .contains("unsupported MIR lowering: tacet before statement-level MIR lowering"));
}

#[test]
fn ignores_top_level_type_metadata_items() {
    let program = lower_analyzed_unit(&analyze(
        "genus Persona { textus nomen } discretio Eventus { Bonum { textus nuntius } } functio salve() {}",
    ))
    .expect("type metadata should not block MIR lowering");

    assert_eq!(program.functions.len(), 1);
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
fn lowers_fac_cape_iace_to_local_handler_edge() {
    let dump = dump_source(r#"functio handled() → numerus { fac { iace "bad" } cape err { redde 0 } redde 1 }"#);

    assert!(dump.starts_with("function f0 -> ty#1 {\n"));
    assert!(dump.contains("locals:\n    let _0: ty#0\n"));
    assert!(dump.contains("%0 = const string sym#"));
    assert!(dump.contains("_0 = %0: ty#0\n    goto bb1"));
    assert!(dump.contains("bb1:\n    %1 = const int 0: ty#1\n    return %1"));
    assert!(!dump.contains("return_error"));
}

#[test]
fn lowers_fac_cape_failable_call_to_try_call_edge() {
    let dump = dump_source(
        r#"
functio fail() → numerus ⇥ textus { iace "bad" }
functio safe() → numerus { fac { redde fail() } cape err { redde 0 } }
"#,
    );

    assert!(dump.contains("function f0 -> ty#1 ⇥ ty#0"));
    assert!(dump.contains("function f1 -> ty#1"));
    assert!(dump.contains("%0 = try_call def#0() ok bb3 error _0 -> bb1"));
    assert!(dump.contains("bb1:\n    %1 = const int 0: ty#1\n    return %1"));
}

#[test]
fn lowers_dum_cape_iace_to_handler_and_loop_exit() {
    let dump = dump_source(
        r#"functio loop(bivalens ready) → numerus { varia numerus x ← 0 dum ready { iace "bad" } cape err { x ← 1 } redde x }"#,
    );

    assert!(dump.contains("branch _0 bb4 bb5"));
    assert!(dump.contains("_2 = %0: ty#0\n    goto bb1"));
    assert!(dump.contains("bb1:\n    _1 = const int 1: ty#1\n    goto bb2"));
    assert!(dump.contains("bb2:\n    return _1"));
}

#[test]
fn lowers_si_arm_cape_iace_to_arm_handler() {
    let dump = dump_source(
        r#"functio choose(bivalens ready) → numerus { si ready { iace "bad" } cape err { redde 0 } secus { redde 2 } redde 1 }"#,
    );

    assert!(dump.contains("branch _0 bb1 bb2"));
    assert!(dump.contains("_1 = %0: ty#0\n    goto bb4"));
    assert!(dump.contains("bb2:\n    %2 = const int 2: ty#1\n    return %2"));
    assert!(dump.contains("bb4:\n    %1 = const int 0: ty#1\n    return %1"));
    assert!(!dump.contains("return_error"));
}

#[test]
fn lowers_struct_construction_and_field_read() {
    let dump = dump_source(
        r#"
genus Persona { textus nomen numerus aetas }
functio nomen() → textus {
    fixum Persona p ← Persona { nomen = "Ada", aetas = 36 }
    redde p.nomen
}
"#,
    );

    assert!(dump.contains("construct struct def#"));
    assert!(dump.contains("sym#"));
    assert!(dump.contains("const string sym#"));
    assert!(dump.contains("const int 36"));
    assert!(dump.contains("return _0.sym#"));
}

#[test]
fn lowers_struct_construction_field_defaults() {
    let dump = dump_source(
        r#"
genus Persona { textus nomen numerus aetas = 0 }
functio aetas() → numerus {
    fixum Persona p ← Persona { nomen = "Ada" }
    redde p.aetas
}
"#,
    );

    assert!(dump.contains("construct struct def#"));
    assert!(dump.contains("const string sym#"));
    assert!(dump.contains("const int 0"));
    assert!(dump.contains("return _0.sym#"));
}

#[test]
fn lowers_array_spread_and_index_read() {
    let dump = dump_source(
        r#"
functio primus() → numerus {
    fixum lista<numerus> xs ← [1, 2]
    fixum lista<numerus> ys ← [0, sparge xs]
    redde ys[0]
}
"#,
    );

    assert!(dump.contains("construct array"));
    assert!(dump.contains("[const int 0, ..._0]"));
    assert!(dump.contains("return _1[const int 0]"));
}

#[test]
fn lowers_map_and_set_construction() {
    let map_dump = dump_source(
        r#"
functio lectio() → numerus {
    fixum tabula<textus, numerus> xs ← { a = 1, b = 2 }
    redde xs["a"]
}
"#,
    );
    assert!(map_dump.contains("construct map"));
    assert!(map_dump.contains("=> const int 1"));
    assert!(map_dump.contains("return _0[const string sym#"));

    let set_dump = dump_source("functio setum() → copia<numerus> { redde [1, 2] ∷ copia<numerus> }");
    assert!(set_dump.contains("construct set"));
    assert!(set_dump.contains("[const int 1, const int 2]"));
}

#[test]
fn lowers_field_and_index_assignment_places() {
    let field_dump = dump_source(
        r#"
genus Persona { textus nomen numerus aetas }
functio muta() → numerus {
    varia Persona p ← Persona { nomen = "Ada", aetas = 36 }
    p.aetas ← 37
    redde p.aetas
}
"#,
    );
    assert!(field_dump.contains("_0.sym#"));
    assert!(field_dump.contains("= const int 37"));

    let index_dump = dump_source(
        r#"
functio muta() → numerus {
    varia lista<numerus> xs ← [1, 2]
    xs[0] ← 5
    redde xs[0]
}
"#,
    );
    assert!(index_dump.contains("_0[const int 0] = const int 5"));
    assert!(index_dump.contains("return _0[const int 0]"));
}

#[test]
fn lowers_optional_chain_non_null_and_coalesce() {
    let chain_dump = dump_source(
        r#"
genus Persona { textus nomen }
functio maybe(Persona ∪ nihil p) → textus ∪ nihil {
    redde p?.nomen
}
"#,
    );
    assert!(chain_dump.contains("option chain(_0, .sym#"));

    let non_null_dump = dump_source(
        r#"
genus Persona { textus nomen }
functio certum(Persona ∪ nihil p) → textus {
    redde p!.nomen
}
"#,
    );
    assert!(non_null_dump.contains("option unwrap_assert(_0)"));
    assert!(non_null_dump.contains("return %0.sym#"));

    let coalesce_dump = dump_source(r#"functio maybe(textus ∪ nihil name) → textus { redde name vel "ignotus" }"#);
    assert!(coalesce_dump.contains("option coalesce(_0, const string sym#"));
}

#[test]
fn lowers_enum_variant_construction() {
    let dump = dump_source(
        r#"
discretio Eventus { Bonum { textus nuntius } }
functio crea() → Eventus {
    redde finge Bonum { nuntius = "ok" } ∷ Eventus
}
"#,
    );

    assert!(dump.contains("construct variant def#"));
    assert!(dump.contains("{sym#"));
    assert!(dump.contains("const string sym#"));
}

#[test]
fn rejects_unsupported_map_spread_shape() {
    let unit = analyze(
        r#"
functio malum() → tabula<textus, numerus> {
    fixum tabula<textus, numerus> base ← { a = 1 }
    redde { sparge base, b = 2 }
}
"#,
    );
    let errors = lower_analyzed_unit(&unit).expect_err("map spread remains unsupported in MIR");

    assert!(errors.iter().any(|err| err
        .message
        .contains("map spread before aggregate MIR lowering")));
}

#[test]
fn lowers_diagnostic_verbs_to_runtime_intrinsics() {
    let dump = dump_source(
        r#"
functio log(textus name) → vacuum {
    nota "salve"
    vide name
    mone "cave"
}
"#,
    );

    assert!(dump.contains("runtime diagnostic nota(const string sym#"));
    assert!(dump.contains("runtime diagnostic vide(_0)"));
    assert!(dump.contains("runtime diagnostic mone(const string sym#"));
}

#[test]
fn lowers_format_conversion_collection_and_provider_runtime_intrinsics() {
    let format_dump = dump_source(r#"functio greet(textus name) → textus { redde "Salve, §!"(name) }"#);
    assert!(format_dump.contains("runtime format_string template sym#"));
    assert!(!format_dump.contains("format!"));

    let conversion_dump = dump_source(r#"functio parse(textus raw) → numerus { redde raw ⇒ numerus<i32, Hex> vel 0 }"#);
    assert!(conversion_dump.contains("runtime convert runtime -> ty#"));
    assert!(conversion_dump.contains("hints [sym#"));
    assert!(conversion_dump.contains("fallback const int 0(_0)"));

    let collection_dump = dump_source(r#"functio count(lista<numerus> xs) → numerus { redde xs.longitudo() }"#);
    assert!(collection_dump.contains("runtime collection length(_0)"));
    assert!(!collection_dump.contains(".len"));

    let provider_dump = dump_source(
        r#"
importa ex "norma:hal/consolum" privata consolum
functio read() → ignotum {
    redde consolum.lege()
}
"#,
    );
    assert!(provider_dump.contains("runtime provider sym#"));
    assert!(provider_dump.contains("::sym#"));
}

#[test]
fn rejects_unsupported_runtime_method_shapes() {
    let unit = analyze(r#"functio malum(lista<numerus> xs) → numerus { redde xs.ordina() }"#);
    let errors = lower_analyzed_unit(&unit).expect_err("unsupported method remains fail-closed");

    assert!(errors.iter().any(|err| err
        .message
        .contains("method call before runtime/provider MIR lowering")));
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
        .contains("assignment target that is not an addressable place"));
}

#[test]
fn function_builder_hir_visitor_lowers_statement_blocks() {
    use crate::hir::{HirBlock, HirExpr, HirExprKind, HirId, HirLiteral, HirStmt, HirStmtKind};
    use crate::semantic::{Primitive, TypeTable};

    let types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let span = crate::lexer::Span::default();
    let mut builder = FunctionBuilder::new(&types);
    let entry = builder.fresh_block(span);
    builder.switch_to(entry);

    let block = HirBlock {
        stmts: vec![HirStmt {
            id: HirId(0),
            kind: HirStmtKind::Redde(Some(HirExpr {
                id: HirId(1),
                kind: HirExprKind::Literal(HirLiteral::Int(1)),
                ty: Some(numerus),
                span,
            })),
            span,
        }],
        expr: None,
        span,
    };

    <FunctionBuilder as HirVisitor>::visit_block(&mut builder, &block);

    assert!(builder.errors.is_empty());
    assert_eq!(builder.temps.len(), 1);
    assert_eq!(builder.blocks[0].statements.len(), 1);
    assert!(matches!(
        builder.blocks[0]
            .terminator
            .as_ref()
            .map(|terminator| &terminator.kind),
        Some(MirTerminatorKind::Return(Some(MirOperand::Temp(MirTempId(0)))))
    ));
}

#[test]
fn function_builder_expr_visitor_emits_value_mir() {
    use crate::hir::{HirBinOp, HirExpr, HirExprKind, HirId, HirLiteral};
    use crate::mir::MirPlaceBase;
    use crate::semantic::{Primitive, TypeTable};

    let types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let span = crate::lexer::Span::default();
    let mut builder = FunctionBuilder::new(&types);
    let entry = builder.fresh_block(span);
    builder.switch_to(entry);

    let lhs = HirExpr { id: HirId(0), kind: HirExprKind::Literal(HirLiteral::Int(1)), ty: Some(numerus), span };
    let rhs = HirExpr { id: HirId(1), kind: HirExprKind::Literal(HirLiteral::Int(2)), ty: Some(numerus), span };
    let expr = HirExpr {
        id: HirId(2),
        kind: HirExprKind::Binary(HirBinOp::Add, Box::new(lhs), Box::new(rhs)),
        ty: Some(numerus),
        span,
    };

    let operand = <FunctionBuilder as HirExprLoweringVisitor>::lower_expr_value(&mut builder, &expr);

    assert_eq!(operand, Some(MirOperand::Temp(MirTempId(0))));
    assert!(builder.errors.is_empty());
    assert_eq!(builder.temps.len(), 1);
    assert!(matches!(
        builder.blocks[0].statements.first().map(|stmt| &stmt.kind),
        Some(MirStmtKind::Assign { place: MirPlace { base: MirPlaceBase::Temp(MirTempId(0)), .. }, value })
            if matches!(value.kind, MirValueKind::Binary { op: MirBinOp::Add, .. })
    ));
}
