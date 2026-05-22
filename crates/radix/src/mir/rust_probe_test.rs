use super::*;
use crate::codegen::{self, Target};
use crate::driver::{Config, Session};
use crate::lexer::{Interner, Span};
use crate::semantic::{Primitive, TypeTable};
use crate::Output;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_ID: AtomicUsize = AtomicUsize::new(0);

fn span() -> Span {
    Span::new(0, 0)
}

fn ty(types: &TypeTable, primitive: Primitive) -> MirType {
    MirType::semantic(types.primitive(primitive))
}

fn validate(program: &MirProgram, types: &TypeTable) {
    validate_program(program, &MirValidationContext::new(types)).expect("test MIR is validated before probe emission");
}

fn compile_generated(code: &str) {
    rustc(code, None);
}

fn run_generated(code: &str, main: &str) {
    rustc(code, Some(main));
}

fn rustc(code: &str, main: Option<&str>) {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("faber-mir-rust-probe-{id}"));
    std::fs::create_dir_all(&dir).expect("create temp compile dir");
    let source = dir.join("probe.rs");
    let output = dir.join("probe-bin");
    let mut full_source = String::from(code);
    if let Some(main) = main {
        full_source.push('\n');
        full_source.push_str(main);
        full_source.push('\n');
    }
    std::fs::write(&source, full_source).expect("write generated Rust source");

    let mut command = Command::new("rustc");
    command
        .arg("--edition=2021")
        .arg(&source)
        .arg("-o")
        .arg(&output);
    if main.is_none() {
        command.arg("--crate-type").arg("lib");
    }
    let compile = command.output().expect("run rustc");
    assert!(
        compile.status.success(),
        "generated Rust failed to compile\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    if main.is_some() {
        let run = Command::new(&output).output().expect("run generated Rust");
        assert!(
            run.status.success(),
            "generated Rust failed at runtime\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

fn emit(program: &MirProgram, types: &TypeTable, interner: &Interner) -> String {
    validate(program, types);
    emit_rust_probe(program, types, interner).expect("probe emits Rust")
}

#[test]
fn emits_primitive_arithmetic_function_that_compiles_and_runs() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: vec![
                MirParam { local: MirLocalId(0), name: None, ty: number, span: span() },
                MirParam { local: MirLocalId(1), name: None, ty: number, span: span() },
            ],
            locals: vec![
                MirLocal { id: MirLocalId(0), name: None, ty: number, mutable: false, span: span() },
                MirLocal { id: MirLocalId(1), name: None, ty: number, mutable: false, span: span() },
            ],
            temps: vec![MirTemp { id: MirTempId(0), ty: number, span: span() }],
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![MirStmt {
                    kind: MirStmtKind::Assign {
                        place: MirPlace::temp(MirTempId(0)),
                        value: MirValue {
                            id: MirValueId(0),
                            kind: MirValueKind::Binary {
                                op: MirBinOp::Add,
                                lhs: MirOperand::Place(MirPlace::local(MirLocalId(0))),
                                rhs: MirOperand::Place(MirPlace::local(MirLocalId(1))),
                            },
                            ty: number,
                            span: span(),
                        },
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
        }],
    };

    let code = emit(&program, &types, &interner);
    run_generated(&code, "fn main() { assert_eq!(__faber_fn_0(2, 3), 5); }");
}

#[test]
fn emits_supported_primitive_constants() {
    let mut interner = Interner::new();
    let salve = interner.intern("salve");
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let float = ty(&types, Primitive::Fractus);
    let boolean = ty(&types, Primitive::Bivalens);
    let text = ty(&types, Primitive::Textus);
    let vacuum = ty(&types, Primitive::Vacuum);
    let nil = ty(&types, Primitive::Nihil);
    let program = MirProgram {
        functions: vec![
            constant_function(MirFunctionId(0), MirConstant::Float(1.5), float),
            constant_function(MirFunctionId(1), MirConstant::Bool(true), boolean),
            constant_function(MirFunctionId(2), MirConstant::String(salve), text),
            constant_function(MirFunctionId(3), MirConstant::Unit, vacuum),
            constant_function(MirFunctionId(4), MirConstant::Nil, nil),
            constant_function(MirFunctionId(5), MirConstant::Int(7), number),
        ],
    };

    let code = emit(&program, &types, &interner);
    run_generated(
        &code,
        r#"fn main() {
            assert_eq!(__faber_fn_0(), 1.5);
            assert_eq!(__faber_fn_1(), true);
            assert_eq!(__faber_fn_2(), String::from("salve"));
            assert_eq!(__faber_fn_3(), ());
            assert_eq!(__faber_fn_4(), ());
            assert_eq!(__faber_fn_5(), 7);
        }"#,
    );
}

#[test]
fn emits_direct_function_call_between_mir_functions() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let callee = MirFunction {
        id: MirFunctionId(0),
        source: None,
        name: None,
        params: vec![MirParam { local: MirLocalId(0), name: None, ty: number, span: span() }],
        locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: number, mutable: false, span: span() }],
        temps: vec![MirTemp { id: MirTempId(0), ty: number, span: span() }],
        blocks: vec![MirBlock {
            id: MirBlockId(0),
            statements: vec![MirStmt {
                kind: MirStmtKind::Assign {
                    place: MirPlace::temp(MirTempId(0)),
                    value: MirValue {
                        id: MirValueId(0),
                        kind: MirValueKind::Binary {
                            op: MirBinOp::Mul,
                            lhs: MirOperand::Place(MirPlace::local(MirLocalId(0))),
                            rhs: MirOperand::Constant(MirConstant::Int(2)),
                        },
                        ty: number,
                        span: span(),
                    },
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
    };
    let caller = MirFunction {
        id: MirFunctionId(1),
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
                    callee: MirCallee::Function(MirFunctionId(0)),
                    args: vec![MirOperand::Constant(MirConstant::Int(4))],
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
    };
    let program = MirProgram { functions: vec![callee, caller] };

    let code = emit(&program, &types, &interner);
    run_generated(&code, "fn main() { assert_eq!(__faber_fn_1(), 8); }");
}

#[test]
fn emits_branch_and_loop_control_flow_that_runs() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let boolean = ty(&types, Primitive::Bivalens);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![
                MirLocal { id: MirLocalId(0), name: None, ty: number, mutable: true, span: span() },
                MirLocal { id: MirLocalId(1), name: None, ty: number, mutable: true, span: span() },
            ],
            temps: vec![
                MirTemp { id: MirTempId(0), ty: boolean, span: span() },
                MirTemp { id: MirTempId(1), ty: number, span: span() },
                MirTemp { id: MirTempId(2), ty: number, span: span() },
            ],
            blocks: vec![
                MirBlock {
                    id: MirBlockId(0),
                    statements: vec![
                        assign_local(MirLocalId(0), MirValueId(0), MirConstant::Int(0), number),
                        assign_local(MirLocalId(1), MirValueId(1), MirConstant::Int(0), number),
                    ],
                    terminator: MirTerminator { kind: MirTerminatorKind::Goto(MirBlockId(1)), span: span() },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(1),
                    statements: vec![MirStmt {
                        kind: MirStmtKind::Assign {
                            place: MirPlace::temp(MirTempId(0)),
                            value: MirValue {
                                id: MirValueId(2),
                                kind: MirValueKind::Binary {
                                    op: MirBinOp::Lt,
                                    lhs: MirOperand::Place(MirPlace::local(MirLocalId(1))),
                                    rhs: MirOperand::Constant(MirConstant::Int(3)),
                                },
                                ty: boolean,
                                span: span(),
                            },
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Branch {
                            condition: MirOperand::Temp(MirTempId(0)),
                            then_block: MirBlockId(2),
                            else_block: MirBlockId(3),
                        },
                        span: span(),
                    },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(2),
                    statements: vec![
                        assign_temp_binary(MirTempId(1), MirValueId(3), MirLocalId(0), MirBinOp::Add, 2, number),
                        assign_from_temp(MirLocalId(0), MirValueId(4), MirTempId(1), number),
                        assign_temp_binary(MirTempId(2), MirValueId(5), MirLocalId(1), MirBinOp::Add, 1, number),
                        assign_from_temp(MirLocalId(1), MirValueId(6), MirTempId(2), number),
                    ],
                    terminator: MirTerminator { kind: MirTerminatorKind::Goto(MirBlockId(1)), span: span() },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(3),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Place(MirPlace::local(MirLocalId(0))))),
                        span: span(),
                    },
                    span: span(),
                },
            ],
            return_ty: number,
            error_ty: None,
            span: span(),
        }],
    };

    let code = emit(&program, &types, &interner);
    run_generated(&code, "fn main() { assert_eq!(__faber_fn_0(), 6); }");
}

#[test]
fn lowered_mir_probe_matches_existing_rust_backend_for_selected_fixture() {
    let source = "functio duplex(numerus n) → numerus { redde n * 2 } functio usa() → numerus { redde duplex(4) }";
    let session = Session::new(Config::default().with_target(Target::Faber));
    let unit = crate::driver::analyze_source(&session, "test.fab", source).expect("analysis succeeds");
    let mir = lower_analyzed_unit(&unit).expect("lowered MIR validates");

    let probe_code = emit_rust_probe(&mir, &unit.types, &unit.interner).expect("probe emits lowered MIR");
    run_generated(&probe_code, "fn main() { assert_eq!(__faber_fn_1(), 8); }");

    let Output::Rust(rust_output) =
        codegen::generate(Target::Rust, &unit.hir, &unit.types, &unit.interner).expect("HIR Rust backend emits")
    else {
        panic!("expected Rust output");
    };
    run_generated(&rust_output.code, "fn main() { assert_eq!(usa(), 8); }");
}

#[test]
fn emits_library_style_functions_without_main() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let vacuum = ty(&types, Primitive::Vacuum);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
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
        }],
    };

    let code = emit(&program, &types, &interner);
    compile_generated(&code);
}

#[test]
fn fail_closed_for_unsupported_mir_shapes() {
    let mut interner = Interner::new();
    let bad = interner.intern("bad");
    let types = TypeTable::new();
    let text = ty(&types, Primitive::Textus);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: Vec::new(),
                terminator: MirTerminator {
                    kind: MirTerminatorKind::ReturnError(MirOperand::Constant(MirConstant::String(bad))),
                    span: span(),
                },
                span: span(),
            }],
            return_ty: ty(&types, Primitive::Vacuum),
            error_ty: Some(text),
            span: span(),
        }],
    };

    validate(&program, &types);
    let error = emit_rust_probe(&program, &types, &interner).expect_err("return_error is outside probe scope");
    assert!(error
        .message
        .contains("MIR-to-Rust unsupported: return_error"));
}

#[test]
fn fail_closed_for_unsupported_statement_and_terminator_shapes() {
    for (program, needle) in [
        unsupported_runtime_call_program(),
        unsupported_construct_program(),
        unsupported_try_call_program(),
        unsupported_switch_program(),
        unsupported_projection_program(),
        unsupported_indirect_call_program(),
    ] {
        let types = TypeTable::new();
        validate(&program, &types);
        let error = emit_rust_probe(&program, &types, &Interner::new()).expect_err("shape is outside probe scope");
        assert!(
            error.message.contains(needle),
            "expected unsupported diagnostic containing {needle:?}, got {:?}",
            error.message
        );
    }

    let (program, types, needle) = unsupported_option_program();
    validate(&program, &types);
    let error = emit_rust_probe(&program, &types, &Interner::new()).expect_err("option is outside probe scope");
    assert!(error.message.contains(needle));
}

fn assign_local(local: MirLocalId, id: MirValueId, constant: MirConstant, ty: MirType) -> MirStmt {
    MirStmt {
        kind: MirStmtKind::Assign {
            place: MirPlace::local(local),
            value: MirValue { id, kind: MirValueKind::Operand(MirOperand::Constant(constant)), ty, span: span() },
        },
        span: span(),
    }
}

fn assign_temp_binary(
    temp: MirTempId,
    id: MirValueId,
    lhs: MirLocalId,
    op: MirBinOp,
    rhs: i64,
    ty: MirType,
) -> MirStmt {
    MirStmt {
        kind: MirStmtKind::Assign {
            place: MirPlace::temp(temp),
            value: MirValue {
                id,
                kind: MirValueKind::Binary {
                    op,
                    lhs: MirOperand::Place(MirPlace::local(lhs)),
                    rhs: MirOperand::Constant(MirConstant::Int(rhs)),
                },
                ty,
                span: span(),
            },
        },
        span: span(),
    }
}

fn assign_from_temp(local: MirLocalId, id: MirValueId, temp: MirTempId, ty: MirType) -> MirStmt {
    MirStmt {
        kind: MirStmtKind::Assign {
            place: MirPlace::local(local),
            value: MirValue { id, kind: MirValueKind::Operand(MirOperand::Temp(temp)), ty, span: span() },
        },
        span: span(),
    }
}

fn constant_function(id: MirFunctionId, constant: MirConstant, return_ty: MirType) -> MirFunction {
    MirFunction {
        id,
        source: None,
        name: None,
        params: Vec::new(),
        locals: Vec::new(),
        temps: Vec::new(),
        blocks: vec![MirBlock {
            id: MirBlockId(0),
            statements: Vec::new(),
            terminator: MirTerminator {
                kind: MirTerminatorKind::Return(Some(MirOperand::Constant(constant))),
                span: span(),
            },
            span: span(),
        }],
        return_ty,
        error_ty: None,
        span: span(),
    }
}

fn unsupported_runtime_call_program() -> (MirProgram, &'static str) {
    let types = TypeTable::new();
    let vacuum = ty(&types, Primitive::Vacuum);
    (
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
                        kind: MirStmtKind::RuntimeCall {
                            destination: None,
                            call: MirRuntimeCall {
                                intrinsic: MirIntrinsic::Diagnostic(MirDiagnosticKind::Nota),
                                args: Vec::new(),
                                return_ty: vacuum,
                            },
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                    span: span(),
                }],
                return_ty: vacuum,
                error_ty: None,
                span: span(),
            }],
        },
        "MIR-to-Rust unsupported: runtime_call",
    )
}

fn unsupported_construct_program() -> (MirProgram, &'static str) {
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    (
        MirProgram {
            functions: vec![MirFunction {
                id: MirFunctionId(0),
                source: None,
                name: None,
                params: Vec::new(),
                locals: Vec::new(),
                temps: vec![MirTemp { id: MirTempId(0), ty: number, span: span() }],
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: vec![MirStmt {
                        kind: MirStmtKind::Construct {
                            destination: MirPlace::temp(MirTempId(0)),
                            aggregate: MirAggregate {
                                kind: MirAggregateKind::Tuple,
                                ty: number,
                                fields: MirAggregateFields::Ordered(Vec::new()),
                            },
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
            }],
        },
        "MIR-to-Rust unsupported: construct",
    )
}

fn unsupported_try_call_program() -> (MirProgram, &'static str) {
    let types = TypeTable::new();
    let text = ty(&types, Primitive::Textus);
    let vacuum = ty(&types, Primitive::Vacuum);
    (
        MirProgram {
            functions: vec![
                MirFunction {
                    id: MirFunctionId(0),
                    source: None,
                    name: None,
                    params: Vec::new(),
                    locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: true, span: span() }],
                    temps: Vec::new(),
                    blocks: vec![
                        MirBlock {
                            id: MirBlockId(0),
                            statements: Vec::new(),
                            terminator: MirTerminator {
                                kind: MirTerminatorKind::TryCall {
                                    destination: None,
                                    callee: MirCallee::Function(MirFunctionId(1)),
                                    args: Vec::new(),
                                    ok_block: MirBlockId(1),
                                    error_place: MirPlace::local(MirLocalId(0)),
                                    error_block: MirBlockId(1),
                                },
                                span: span(),
                            },
                            span: span(),
                        },
                        MirBlock {
                            id: MirBlockId(1),
                            statements: Vec::new(),
                            terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                            span: span(),
                        },
                    ],
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
                    error_ty: Some(text),
                    span: span(),
                },
            ],
        },
        "MIR-to-Rust unsupported: try_call",
    )
}

fn unsupported_switch_program() -> (MirProgram, &'static str) {
    let types = TypeTable::new();
    let vacuum = ty(&types, Primitive::Vacuum);
    (
        MirProgram {
            functions: vec![MirFunction {
                id: MirFunctionId(0),
                source: None,
                name: None,
                params: Vec::new(),
                locals: Vec::new(),
                temps: Vec::new(),
                blocks: vec![
                    MirBlock {
                        id: MirBlockId(0),
                        statements: Vec::new(),
                        terminator: MirTerminator {
                            kind: MirTerminatorKind::Switch {
                                value: MirOperand::Constant(MirConstant::Int(0)),
                                cases: Vec::new(),
                                default: MirBlockId(1),
                            },
                            span: span(),
                        },
                        span: span(),
                    },
                    MirBlock {
                        id: MirBlockId(1),
                        statements: Vec::new(),
                        terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                        span: span(),
                    },
                ],
                return_ty: vacuum,
                error_ty: None,
                span: span(),
            }],
        },
        "MIR-to-Rust unsupported: switch",
    )
}

fn unsupported_projection_program() -> (MirProgram, &'static str) {
    let types = TypeTable::new();
    let text = ty(&types, Primitive::Textus);
    (
        MirProgram {
            functions: vec![MirFunction {
                id: MirFunctionId(0),
                source: None,
                name: None,
                params: vec![MirParam { local: MirLocalId(0), name: None, ty: text, span: span() }],
                locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: false, span: span() }],
                temps: Vec::new(),
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Place(MirPlace {
                            base: MirPlaceBase::Local(MirLocalId(0)),
                            projections: vec![MirProjection::Index(MirOperand::Constant(MirConstant::Int(0)))],
                        }))),
                        span: span(),
                    },
                    span: span(),
                }],
                return_ty: text,
                error_ty: None,
                span: span(),
            }],
        },
        "MIR-to-Rust unsupported: place projection",
    )
}

fn unsupported_indirect_call_program() -> (MirProgram, &'static str) {
    let types = TypeTable::new();
    let vacuum = ty(&types, Primitive::Vacuum);
    (
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
                        kind: MirStmtKind::Call {
                            destination: None,
                            callee: MirCallee::Value(MirOperand::Constant(MirConstant::Unit)),
                            args: Vec::new(),
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                    span: span(),
                }],
                return_ty: vacuum,
                error_ty: None,
                span: span(),
            }],
        },
        "MIR-to-Rust unsupported: indirect call",
    )
}

fn unsupported_option_program() -> (MirProgram, TypeTable, &'static str) {
    let mut types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let option_number = MirType::semantic(types.option(number.semantic_id()));
    (
        MirProgram {
            functions: vec![MirFunction {
                id: MirFunctionId(0),
                source: None,
                name: None,
                params: Vec::new(),
                locals: Vec::new(),
                temps: vec![MirTemp { id: MirTempId(0), ty: option_number, span: span() }],
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: vec![MirStmt {
                        kind: MirStmtKind::Assign {
                            place: MirPlace::temp(MirTempId(0)),
                            value: MirValue {
                                id: MirValueId(0),
                                kind: MirValueKind::Option(MirOptionOp::Some(MirOperand::Constant(MirConstant::Int(
                                    1,
                                )))),
                                ty: option_number,
                                span: span(),
                            },
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Temp(MirTempId(0)))),
                        span: span(),
                    },
                    span: span(),
                }],
                return_ty: option_number,
                error_ty: None,
                span: span(),
            }],
        },
        types,
        "MIR-to-Rust unsupported: type Option",
    )
}
