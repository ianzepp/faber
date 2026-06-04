use super::emit_llvm_text_probe;
use crate::driver::{Config, Session};
use crate::hir::DefId;
use crate::lexer::{Interner, Span};
use crate::mir::*;
use crate::semantic::{Primitive, TypeTable};
use crate::{driver, Output, Target};

fn span() -> Span {
    Span::new(0, 0)
}

fn ty(types: &TypeTable, primitive: Primitive) -> MirType {
    MirType::semantic(types.primitive(primitive))
}

fn call_probe_program(callee: MirCallee, types: &TypeTable) -> MirProgram {
    let vacuum = ty(types, Primitive::Vacuum);
    MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![MirStmt {
                    kind: MirStmtKind::Call { destination: None, callee, args: Vec::new() },
                    span: span(),
                }],
                terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                span: span(),
            }],
            return_ty: vacuum,
            error_ty: None,
            span: span(),
        }],
    }
}

fn function_id_call_program(types: &TypeTable) -> MirProgram {
    let number = ty(types, Primitive::Numerus);
    MirProgram {
        functions: vec![
            MirFunction {
                id: MirFunctionId(0),
                source: None,
                name: None,
                params: Vec::new(),
                locals: Vec::new(),
                temps: vec![MirTemp { id: MirTempId(0), ty: number, span: span() }],
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: vec![MirStmt {
                        kind: MirStmtKind::Call {
                            destination: Some(MirPlace::temp(MirTempId(0))),
                            callee: MirCallee::Function(MirFunctionId(1)),
                            args: Vec::new(),
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Temp(MirTempId(0)))),
                        span: span(),
                    },
                    span: span(),
                }],
                return_ty: number,
                error_ty: None,
                span: span(),
            },
            MirFunction {
                id: MirFunctionId(1),
                source: None,
                name: None,
                params: Vec::new(),
                locals: Vec::new(),
                temps: Vec::new(),
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Constant(MirConstant::Int(7)))),
                        span: span(),
                    },
                    span: span(),
                }],
                return_ty: number,
                error_ty: None,
                span: span(),
            },
        ],
    }
}

fn runtime_stmt_program(
    locals: Vec<MirLocal>,
    temps: Vec<MirTemp>,
    statements: Vec<MirStmt>,
    terminator: MirTerminatorKind,
    types: &TypeTable,
) -> MirProgram {
    MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals,
            temps,
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements,
                terminator: MirTerminator { kind: terminator, span: span() },
                span: span(),
            }],
            return_ty: ty(types, Primitive::Vacuum),
            error_ty: None,
            span: span(),
        }],
    }
}

fn aggregate_stmt_program(temps: Vec<MirTemp>, statements: Vec<MirStmt>, types: &TypeTable) -> MirProgram {
    runtime_stmt_program(Vec::new(), temps, statements, MirTerminatorKind::Return(None), types)
}

#[test]
fn llvm_text_target_emits_text_from_validated_mir() {
    let source = r#"
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("define i64 @adde(i64 %l0, i64 %l1)"));
    assert!(output.code.contains("%l0.addr = alloca i64"));
    assert!(output.code.contains("%load0 = load i64, ptr %l0.addr"));
    assert!(output.code.contains("%load1 = load i64, ptr %l1.addr"));
    assert!(output.code.contains("add i64 %load0, %load1"));
    assert!(output.code.contains("ret i64 %load"));
}

#[test]
fn llvm_text_target_emits_incipit_entry_symbol() {
    let source = r#"
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}

incipit {
    fixum _ n ← adde(1, 2)
    nota n
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("define void @incipit()"));
    assert!(output
        .code
        .contains("%call0 = call i64 @adde(i64 1, i64 2)"));
    assert!(output
        .code
        .contains("call void @__faber_runtime_diagnostic_nota_1_i64"));
}

#[test]
fn llvm_text_target_keeps_user_incipit_name_from_colliding_with_entry_symbol() {
    let types = TypeTable::new();
    let vacuum = ty(&types, Primitive::Vacuum);
    let mut interner = Interner::new();
    let incipit = interner.intern("incipit");
    let program = MirProgram {
        functions: vec![
            MirFunction {
                id: MirFunctionId(0),
                source: Some(DefId(10)),
                name: Some(incipit),
                params: Vec::new(),
                locals: Vec::new(),
                temps: Vec::new(),
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                    span: span(),
                }],
                return_ty: vacuum,
                error_ty: None,
                span: span(),
            },
            MirFunction {
                id: MirFunctionId(1),
                source: None,
                name: None,
                params: Vec::new(),
                locals: Vec::new(),
                temps: Vec::new(),
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                    span: span(),
                }],
                return_ty: vacuum,
                error_ty: None,
                span: span(),
            },
        ],
    };
    let output = emit_llvm_text_probe(&program, &types, &interner).expect("entry collision emits");

    assert!(output.contains("define void @incipit_f0()"));
    assert!(output.contains("define void @incipit()"));
}

#[test]
fn llvm_text_target_emits_fractus_arithmetic_and_comparisons() {
    let source = r#"
functio media(fractus a, fractus b) → fractus {
    redde (a + b) / 2.0
}

functio minor(fractus a, fractus b) → bivalens {
    redde a < b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output
        .code
        .contains("define double @media(double %l0, double %l1)"));
    assert!(output.code.contains("fadd double %load0, %load1"));
    assert!(output.code.contains("fdiv double %load2, 2.0"));
    assert!(output
        .code
        .contains("define i1 @minor(double %l0, double %l1)"));
    assert!(output.code.contains("fcmp olt double %load0, %load1"));
}

#[test]
fn llvm_text_target_emits_boolean_unary_binary_and_equality() {
    let source = r#"
functio logicum(bivalens a, bivalens b) → bivalens {
    redde non a et (a aut b)
}

functio idem(bivalens a, bivalens b) → bivalens {
    redde a ≡ b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("xor i1 %load0, true"));
    assert!(output.code.contains("or i1 %load1, %load2"));
    assert!(output.code.contains("and i1 %load3, %load4"));
    assert!(output.code.contains("icmp eq i1 %load0, %load1"));
}

#[test]
fn llvm_text_target_emits_integer_comparisons() {
    let source = r#"
functio minor(numerus a, numerus b) → bivalens {
    redde a < b
}

functio idem(numerus a, numerus b) → bivalens {
    redde a ≡ b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("icmp slt i64 %load0, %load1"));
    assert!(output.code.contains("icmp eq i64 %load0, %load1"));
}

#[test]
fn llvm_text_target_emits_text_handle_returns() {
    let source = r#"
functio label() → textus {
    redde "salve"
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );

    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("define ptr @label()"));
    assert!(output.code.contains("ret ptr"));
}

#[test]
fn llvm_text_target_emits_branch_return_cfg() {
    let source = r#"
functio ramus(bivalens flag) → numerus {
    si flag {
        redde 1
    }
    redde 0
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );

    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("bb0:"));
    assert!(output.code.contains("br i1 %load0, label %bb1, label %bb2"));
    assert!(output.code.contains("bb1:"));
    assert!(output.code.contains("ret i64 %load1"));
    assert!(output.code.contains("bb2:"));
    assert!(output.code.contains("ret i64 %load2"));
}

#[test]
fn llvm_text_target_emits_branch_join_cfg() {
    let source = r#"
functio positum(numerus n) → numerus {
    fixum numerus x ← n > 0 sic 1 secus 0
    redde x
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("bb0:"));
    assert!(output.code.contains("br i1 %load1, label %bb1, label %bb2"));
    assert!(output.code.contains("bb1:"));
    assert!(output.code.contains("br label %bb3"));
    assert!(output.code.contains("bb2:"));
    assert!(output.code.contains("br label %bb3"));
    assert!(output.code.contains("bb3:"));
    assert!(output.code.contains("ret i64 %load2"));
}

#[test]
fn llvm_text_target_emits_simple_loop_cfg() {
    let source = r#"
functio countdown(numerus n) → numerus {
    varia numerus x ← n
    dum x > 0 {
        x ← x - 1
    }
    redde x
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("bb1:"));
    assert!(output.code.contains("br i1 %load2, label %bb2, label %bb3"));
    assert!(output.code.contains("bb2:"));
    assert!(output.code.contains("sub i64 %load3, 1"));
    assert!(output.code.contains("br label %bb1"));
    assert!(output.code.contains("bb3:"));
    assert!(output.code.contains("ret i64 %load5"));
}

#[test]
fn llvm_text_target_emits_direct_scalar_function_calls() {
    let source = r#"
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}

functio duplex(numerus n) → numerus {
    redde adde(n, n)
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("define i64 @adde(i64 %l0, i64 %l1)"));
    assert!(output.code.contains("define i64 @duplex(i64 %l0)"));
    assert!(output.code.contains("%call"));
    assert!(output
        .code
        .contains("call i64 @adde(i64 %load0, i64 %load1)"));
    assert!(output.code.contains("store i64 %call"));
}

#[test]
fn llvm_text_target_emits_direct_scalar_call_chains() {
    let source = r#"
functio incrementum(numerus n) → numerus {
    redde n + 1
}

functio triplex(numerus n) → numerus {
    fixum numerus a ← incrementum(n)
    redde incrementum(a)
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("define i64 @incrementum(i64 %l0)"));
    assert!(output.code.contains("define i64 @triplex(i64 %l0)"));
    assert!(output.code.contains("call i64 @incrementum(i64 %load0)"));
    assert!(output.code.contains("call i64 @incrementum(i64 %load3)"));
}

#[test]
fn llvm_text_target_emits_vacuum_direct_calls() {
    let source = r#"
functio ping() → vacuum {
    redde
}

functio usa() → vacuum {
    ping()
    redde
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("define void @ping()"));
    assert!(output.code.contains("define void @usa()"));
    assert!(output.code.contains("call void @ping()"));
    assert!(!output.code.contains("= call void @ping()"));
}

#[test]
fn llvm_text_target_emits_function_id_callee() {
    let types = TypeTable::new();
    let program = function_id_call_program(&types);
    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("function id callee emits");

    assert!(output.contains("define i64 @f0()"));
    assert!(output.contains("define i64 @f1()"));
    assert!(output.contains("call i64 @f1()"));
}

#[test]
fn llvm_text_target_emits_diagnostic_assert_and_panic_runtime_declarations() {
    let types = TypeTable::new();
    let program = runtime_stmt_program(
        Vec::new(),
        Vec::new(),
        vec![
            MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: None,
                    call: MirRuntimeCall {
                        intrinsic: MirIntrinsic::Diagnostic(MirDiagnosticKind::Nota),
                        args: vec![MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(7)))],
                        return_ty: ty(&types, Primitive::Vacuum),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: None,
                    call: MirRuntimeCall {
                        intrinsic: MirIntrinsic::Assert,
                        args: vec![
                            MirOperand::Constant(MirConstant::Bool(true)),
                            MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(8))),
                        ],
                        return_ty: ty(&types, Primitive::Vacuum),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: None,
                    call: MirRuntimeCall {
                        intrinsic: MirIntrinsic::Panic,
                        args: vec![MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(9)))],
                        return_ty: ty(&types, Primitive::Numquam),
                    },
                },
                span: span(),
            },
        ],
        MirTerminatorKind::Return(None),
        &types,
    );
    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("runtime calls emit");

    assert!(output.contains("declare void @__faber_runtime_assert_2_i1_ptr(i1, ptr)"));
    assert!(output.contains("declare void @__faber_runtime_diagnostic_nota_1_ptr(ptr)"));
    assert!(output.contains("declare void @__faber_runtime_panic_1_ptr(ptr)"));
    assert!(output.contains("call void @__faber_runtime_diagnostic_nota_1_ptr(ptr inttoptr (i64 7 to ptr))"));
    assert!(output.contains("call void @__faber_runtime_assert_2_i1_ptr(i1 1, ptr inttoptr (i64 8 to ptr))"));
    assert!(output.contains("call void @__faber_runtime_panic_1_ptr(ptr inttoptr (i64 9 to ptr))"));
}

#[test]
fn llvm_text_target_emits_value_returning_runtime_calls() {
    let mut types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let text = ty(&types, Primitive::Textus);
    let list = MirType::semantic(types.array(types.primitive(Primitive::Numerus)));
    let program = runtime_stmt_program(
        Vec::new(),
        vec![
            MirTemp { id: MirTempId(0), ty: number, span: span() },
            MirTemp { id: MirTempId(1), ty: text, span: span() },
            MirTemp { id: MirTempId(2), ty: number, span: span() },
            MirTemp { id: MirTempId(3), ty: list, span: span() },
        ],
        vec![
            MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: Some(MirPlace::temp(MirTempId(0))),
                    call: MirRuntimeCall {
                        intrinsic: MirIntrinsic::Convert(MirConversion {
                            flavor: MirConversionFlavor::Runtime,
                            target_ty: number,
                            params: Vec::new(),
                            fallback: None,
                        }),
                        args: vec![MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(10)))],
                        return_ty: number,
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: Some(MirPlace::temp(MirTempId(1))),
                    call: MirRuntimeCall {
                        intrinsic: MirIntrinsic::FormatString { template: crate::lexer::Symbol(11) },
                        args: vec![MirOperand::Temp(MirTempId(0))],
                        return_ty: text,
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: Some(MirPlace::temp(MirTempId(2))),
                    call: MirRuntimeCall {
                        intrinsic: MirIntrinsic::Collection(MirCollectionOp::Length),
                        args: vec![MirOperand::Temp(MirTempId(3))],
                        return_ty: number,
                    },
                },
                span: span(),
            },
        ],
        MirTerminatorKind::Return(None),
        &types,
    );
    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("runtime values emit");

    assert!(output.contains("declare i64 @__faber_runtime_convert_runtime_1_ptr_to_i64(ptr)"));
    assert!(output.contains("declare ptr @__faber_runtime_format_1_i64_to_ptr(i64)"));
    assert!(output.contains("declare i64 @__faber_runtime_length_1_ptr_to_i64(ptr)"));
    assert!(output.contains("%rtcall0 = call i64 @__faber_runtime_convert_runtime_1_ptr_to_i64"));
    assert!(output.contains("store i64 %rtcall0, ptr %t0.addr"));
    assert!(output.contains("call ptr @__faber_runtime_format_1_i64_to_ptr"));
    assert!(output.contains("store ptr %rtcall"));
    assert!(output.contains("call i64 @__faber_runtime_length_1_ptr_to_i64"));
    assert!(output.contains("store i64 %rtcall"));
}

#[test]
fn llvm_text_target_emits_nil_as_null_handle() {
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let program = runtime_stmt_program(
        Vec::new(),
        vec![MirTemp { id: MirTempId(0), ty: number, span: span() }],
        vec![MirStmt {
            kind: MirStmtKind::RuntimeCall {
                destination: Some(MirPlace::temp(MirTempId(0))),
                call: MirRuntimeCall {
                    intrinsic: MirIntrinsic::Collection(MirCollectionOp::Length),
                    args: vec![MirOperand::Constant(MirConstant::Nil)],
                    return_ty: number,
                },
            },
            span: span(),
        }],
        MirTerminatorKind::Return(None),
        &types,
    );
    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("nil handle emits");

    assert!(output.contains("call i64 @__faber_runtime_length_1_ptr_to_i64(ptr null)"));
}

#[test]
fn llvm_text_target_emits_scalar_option_helpers() {
    let mut types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let boolean = ty(&types, Primitive::Bivalens);
    let option_number = MirType::semantic(types.option(types.primitive(Primitive::Numerus)));
    let program = runtime_stmt_program(
        Vec::new(),
        vec![
            MirTemp { id: MirTempId(0), ty: option_number, span: span() },
            MirTemp { id: MirTempId(1), ty: option_number, span: span() },
            MirTemp { id: MirTempId(2), ty: boolean, span: span() },
            MirTemp { id: MirTempId(3), ty: boolean, span: span() },
            MirTemp { id: MirTempId(4), ty: number, span: span() },
            MirTemp { id: MirTempId(5), ty: number, span: span() },
        ],
        vec![
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(0)),
                    value: MirValue {
                        id: MirValueId(0),
                        kind: MirValueKind::Option(MirOptionOp::None),
                        ty: option_number,
                        span: span(),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(1)),
                    value: MirValue {
                        id: MirValueId(1),
                        kind: MirValueKind::Option(MirOptionOp::Some(MirOperand::Constant(MirConstant::Int(7)))),
                        ty: option_number,
                        span: span(),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(2)),
                    value: MirValue {
                        id: MirValueId(2),
                        kind: MirValueKind::Option(MirOptionOp::IsNil(MirOperand::Temp(MirTempId(0)))),
                        ty: boolean,
                        span: span(),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(3)),
                    value: MirValue {
                        id: MirValueId(3),
                        kind: MirValueKind::Option(MirOptionOp::IsNonNil(MirOperand::Temp(MirTempId(1)))),
                        ty: boolean,
                        span: span(),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(4)),
                    value: MirValue {
                        id: MirValueId(4),
                        kind: MirValueKind::Option(MirOptionOp::Unwrap {
                            value: MirOperand::Temp(MirTempId(1)),
                            mode: MirOptionUnwrapMode::Assert,
                        }),
                        ty: number,
                        span: span(),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(5)),
                    value: MirValue {
                        id: MirValueId(5),
                        kind: MirValueKind::Option(MirOptionOp::Coalesce {
                            value: MirOperand::Temp(MirTempId(0)),
                            fallback: MirOperand::Constant(MirConstant::Int(9)),
                        }),
                        ty: number,
                        span: span(),
                    },
                },
                span: span(),
            },
        ],
        MirTerminatorKind::Return(None),
        &types,
    );
    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("scalar option helpers emit");

    assert!(output.contains("declare ptr @__faber_option_none_i64()"));
    assert!(output.contains("declare ptr @__faber_option_some_i64(i64)"));
    assert!(output.contains("declare i1 @__faber_option_is_nil(ptr)"));
    assert!(output.contains("declare i1 @__faber_option_is_non_nil(ptr)"));
    assert!(output.contains("declare i64 @__faber_option_unwrap_i64(ptr)"));
    assert!(output.contains("declare i64 @__faber_option_coalesce_i64(ptr, i64)"));
    assert!(output.contains("call ptr @__faber_option_none_i64()"));
    assert!(output.contains("call ptr @__faber_option_some_i64(i64 7)"));
    assert!(output.contains("call i1 @__faber_option_is_nil(ptr %load"));
    assert!(output.contains("call i1 @__faber_option_is_non_nil(ptr %load"));
    assert!(output.contains("call i64 @__faber_option_unwrap_i64(ptr %load"));
    assert!(output.contains("call i64 @__faber_option_coalesce_i64(ptr %load"));
}

#[test]
fn llvm_text_target_rejects_option_chain() {
    let mut types = TypeTable::new();
    let option_text = MirType::semantic(types.option(types.primitive(Primitive::Textus)));
    let program = runtime_stmt_program(
        Vec::new(),
        vec![
            MirTemp { id: MirTempId(0), ty: option_text, span: span() },
            MirTemp { id: MirTempId(1), ty: option_text, span: span() },
        ],
        vec![MirStmt {
            kind: MirStmtKind::Assign {
                place: MirPlace::temp(MirTempId(1)),
                value: MirValue {
                    id: MirValueId(0),
                    kind: MirValueKind::Option(MirOptionOp::Chain {
                        base: MirOperand::Temp(MirTempId(0)),
                        link: MirOptionChainLink::Index(MirOperand::Constant(MirConstant::Int(0))),
                    }),
                    ty: option_text,
                    span: span(),
                },
            },
            span: span(),
        }],
        MirTerminatorKind::Return(None),
        &types,
    );
    let error = emit_llvm_text_probe(&program, &types, &Interner::new()).expect_err("option chain remains deferred");

    assert!(error
        .message
        .contains("MIR-to-LLVM unsupported: option chain value"));
}

#[test]
fn llvm_text_target_emits_aggregate_handle_construction() {
    let mut types = TypeTable::new();
    let list = MirType::semantic(types.array(types.primitive(Primitive::Numerus)));
    let program = aggregate_stmt_program(
        vec![MirTemp { id: MirTempId(0), ty: list, span: span() }],
        vec![MirStmt {
            kind: MirStmtKind::Construct {
                destination: MirPlace::temp(MirTempId(0)),
                aggregate: MirAggregate {
                    kind: MirAggregateKind::Array,
                    ty: list,
                    fields: MirAggregateFields::Ordered(vec![
                        MirAggregateItem::Operand(MirOperand::Constant(MirConstant::Int(1))),
                        MirAggregateItem::Operand(MirOperand::Constant(MirConstant::Int(2))),
                        MirAggregateItem::Operand(MirOperand::Constant(MirConstant::Int(3))),
                    ]),
                },
            },
            span: span(),
        }],
        &types,
    );

    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("aggregate construct emits");

    assert!(output.contains("declare ptr @__faber_aggregate_array_3_i64_i64_i64(i64, i64, i64)"));
    assert!(output.contains("%agg0 = call ptr @__faber_aggregate_array_3_i64_i64_i64(i64 1, i64 2, i64 3)"));
    assert!(output.contains("store ptr %agg0, ptr %t0.addr"));
    assert!(output.contains("%t0.addr = alloca ptr"));
}

#[test]
fn llvm_text_target_emits_index_projection_reads_and_writes() {
    let mut types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let list = MirType::semantic(types.array(types.primitive(Primitive::Numerus)));
    let program = aggregate_stmt_program(
        vec![
            MirTemp { id: MirTempId(0), ty: list, span: span() },
            MirTemp { id: MirTempId(1), ty: number, span: span() },
        ],
        vec![
            MirStmt {
                kind: MirStmtKind::Construct {
                    destination: MirPlace::temp(MirTempId(0)),
                    aggregate: MirAggregate {
                        kind: MirAggregateKind::Array,
                        ty: list,
                        fields: MirAggregateFields::Ordered(vec![
                            MirAggregateItem::Operand(MirOperand::Constant(MirConstant::Int(1))),
                            MirAggregateItem::Operand(MirOperand::Constant(MirConstant::Int(2))),
                        ]),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(1)),
                    value: MirValue {
                        id: MirValueId(0),
                        kind: MirValueKind::Operand(MirOperand::Place(MirPlace {
                            base: MirPlaceBase::Temp(MirTempId(0)),
                            projections: vec![MirProjection::Index(MirOperand::Constant(MirConstant::Int(0)))],
                        })),
                        ty: number,
                        span: span(),
                    },
                },
                span: span(),
            },
            MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace {
                        base: MirPlaceBase::Temp(MirTempId(0)),
                        projections: vec![MirProjection::Index(MirOperand::Constant(MirConstant::Int(1)))],
                    },
                    value: MirValue {
                        id: MirValueId(1),
                        kind: MirValueKind::Operand(MirOperand::Constant(MirConstant::Int(9))),
                        ty: number,
                        span: span(),
                    },
                },
                span: span(),
            },
        ],
        &types,
    );

    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("projection helpers emit");

    assert!(output.contains("declare i64 @__faber_aggregate_index_i64_to_i64(ptr, i64)"));
    assert!(output.contains("declare void @__faber_aggregate_set_index_i64_i64(ptr, i64, i64)"));
    assert!(output.contains("call i64 @__faber_aggregate_index_i64_to_i64(ptr %load"));
    assert!(output.contains("store i64 %proj"));
    assert!(output.contains("call void @__faber_aggregate_set_index_i64_i64(ptr %load"));
}

#[test]
fn llvm_text_target_rejects_aggregate_spread() {
    let mut types = TypeTable::new();
    let list = MirType::semantic(types.array(types.primitive(Primitive::Numerus)));
    let program = aggregate_stmt_program(
        vec![MirTemp { id: MirTempId(0), ty: list, span: span() }],
        vec![MirStmt {
            kind: MirStmtKind::Construct {
                destination: MirPlace::temp(MirTempId(0)),
                aggregate: MirAggregate {
                    kind: MirAggregateKind::Array,
                    ty: list,
                    fields: MirAggregateFields::Ordered(vec![MirAggregateItem::Spread(MirOperand::Temp(MirTempId(0)))]),
                },
            },
            span: span(),
        }],
        &types,
    );

    let error = emit_llvm_text_probe(&program, &types, &Interner::new()).expect_err("spread remains deferred");

    assert!(error
        .message
        .contains("MIR-to-LLVM unsupported: aggregate spread"));
}

#[test]
fn llvm_text_target_rejects_provider_runtime_calls() {
    let types = TypeTable::new();
    let program = runtime_stmt_program(
        Vec::new(),
        Vec::new(),
        vec![MirStmt {
            kind: MirStmtKind::RuntimeCall {
                destination: None,
                call: MirRuntimeCall {
                    intrinsic: MirIntrinsic::Provider(MirProvider {
                        module: vec![crate::lexer::Symbol(1)],
                        name: crate::lexer::Symbol(2),
                    }),
                    args: Vec::new(),
                    return_ty: ty(&types, Primitive::Vacuum),
                },
            },
            span: span(),
        }],
        MirTerminatorKind::Return(None),
        &types,
    );
    let error = emit_llvm_text_probe(&program, &types, &Interner::new()).expect_err("provider remains deferred");

    assert!(error
        .message
        .contains("MIR-to-LLVM unsupported: provider runtime call"));
}

#[test]
fn llvm_text_target_emits_literal_scalar_switch_cfg() {
    let source = r#"
functio status(numerus code) → numerus {
    elige code {
        casu 200 {
            redde 1
        }
        ceterum {
            redde 0
        }
    }
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("switch i64 %load0, label %bb"));
    assert!(output.code.contains("i64 200, label %bb"));
    assert!(output.code.contains("store i64 1, ptr %t0.addr"));
    assert!(output.code.contains("store i64 0, ptr %t1.addr"));
    assert!(output.code.contains("unreachable"));
}

#[test]
fn llvm_text_target_emits_boolean_switch_cfg() {
    let types = TypeTable::new();
    let boolean = ty(&types, Primitive::Bivalens);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: vec![MirParam { local: MirLocalId(0), name: None, ty: boolean, span: span() }],
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: vec![
                MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Switch {
                            value: MirOperand::Place(MirPlace::local(MirLocalId(0))),
                            cases: vec![MirSwitchCase { value: MirConstant::Bool(true), target: MirBlockId(1) }],
                            default: MirBlockId(2),
                        },
                        span: span(),
                    },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(1),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Constant(MirConstant::Bool(true)))),
                        span: span(),
                    },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(2),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Constant(MirConstant::Bool(false)))),
                        span: span(),
                    },
                    span: span(),
                },
            ],
            return_ty: boolean,
            error_ty: None,
            span: span(),
        }],
    };
    let output = emit_llvm_text_probe(&program, &types, &Interner::new()).expect("boolean switch emits");

    assert!(output.contains("switch i1 %load0, label %bb2 ["));
    assert!(output.contains("i1 1, label %bb1"));
}

#[test]
fn llvm_text_target_rejects_text_switch_cfg() {
    let types = TypeTable::new();
    let text = ty(&types, Primitive::Textus);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: vec![MirParam { local: MirLocalId(0), name: None, ty: text, span: span() }],
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: Vec::new(),
                terminator: MirTerminator {
                    kind: MirTerminatorKind::Switch {
                        value: MirOperand::Place(MirPlace::local(MirLocalId(0))),
                        cases: vec![MirSwitchCase {
                            value: MirConstant::String(crate::lexer::Symbol(1)),
                            target: MirBlockId(1),
                        }],
                        default: MirBlockId(1),
                    },
                    span: span(),
                },
                span: span(),
            }],
            return_ty: ty(&types, Primitive::Vacuum),
            error_ty: None,
            span: span(),
        }],
    };
    let error = emit_llvm_text_probe(&program, &types, &Interner::new()).expect_err("text switch remains deferred");

    assert!(error
        .message
        .contains("MIR-to-LLVM unsupported: switch value type"));
}

#[test]
fn llvm_text_target_rejects_failable_terminators() {
    let types = TypeTable::new();
    let text = ty(&types, Primitive::Textus);
    let return_error = runtime_stmt_program(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        MirTerminatorKind::ReturnError(MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(1)))),
        &types,
    );
    let error = emit_llvm_text_probe(&return_error, &types, &Interner::new()).expect_err("return error is unsupported");

    assert!(error
        .message
        .contains("MIR-to-LLVM unsupported: return_error"));

    let try_call = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: true, span: span() }],
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: Vec::new(),
                terminator: MirTerminator {
                    kind: MirTerminatorKind::TryCall {
                        destination: None,
                        callee: MirCallee::Function(MirFunctionId(1)),
                        args: Vec::new(),
                        ok_block: MirBlockId(1),
                        error_place: MirPlace::local(MirLocalId(0)),
                        error_block: MirBlockId(2),
                    },
                    span: span(),
                },
                span: span(),
            }],
            return_ty: ty(&types, Primitive::Vacuum),
            error_ty: Some(text),
            span: span(),
        }],
    };
    let error = emit_llvm_text_probe(&try_call, &types, &Interner::new()).expect_err("try call is unsupported");

    assert!(error.message.contains("MIR-to-LLVM unsupported: try_call"));
}

#[test]
fn llvm_text_target_rejects_value_callee() {
    let types = TypeTable::new();
    let program = call_probe_program(MirCallee::Value(MirOperand::Constant(MirConstant::Unit)), &types);
    let error = emit_llvm_text_probe(&program, &types, &Interner::new()).expect_err("value callee is unsupported");

    assert!(error
        .message
        .contains("MIR-to-LLVM unsupported: value callee"));
}

#[test]
fn llvm_text_target_rejects_external_definition_call() {
    let types = TypeTable::new();
    let program = call_probe_program(MirCallee::Definition(DefId(99)), &types);
    let error =
        emit_llvm_text_probe(&program, &types, &Interner::new()).expect_err("external definition is unsupported");

    assert!(error
        .message
        .contains("MIR-to-LLVM unsupported: external definition call"));
}
