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
fn llvm_text_target_rejects_unsupported_mir_shapes() {
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

    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("MIR-to-LLVM unsupported")));
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
fn llvm_text_target_still_rejects_switch_cfg() {
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

    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("MIR-to-LLVM unsupported: switch")));
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
