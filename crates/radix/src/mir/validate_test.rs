use super::*;
use crate::codegen::Target;
use crate::driver::{Config, Session};
use crate::lexer::{Span, Symbol};
use crate::mir::lower_analyzed_unit;
use crate::semantic::{InferVar, Primitive, Type, TypeTable};

fn span() -> Span {
    Span::new(0, 0)
}

fn ty(types: &TypeTable, primitive: Primitive) -> MirType {
    MirType::semantic(types.primitive(primitive))
}

fn valid_number_program(types: &TypeTable) -> MirProgram {
    let number = ty(types, Primitive::Numerus);
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
                statements: Vec::new(),
                terminator: MirTerminator {
                    kind: MirTerminatorKind::Return(Some(MirOperand::Constant(MirConstant::Int(1)))),
                    span: span(),
                },
                span: span(),
            }],
            return_ty: number,
            error_ty: None,
            span: span(),
        }],
    }
}

fn expect_validation_error(program: &MirProgram, types: &TypeTable, needle: &str) {
    let context = MirValidationContext::new(types);
    let errors = validate_program(program, &context).expect_err("MIR should be invalid");
    assert!(
        errors.iter().any(|error| error.message.contains(needle)),
        "expected validation error containing {needle:?}, got {errors:#?}"
    );
}

fn expect_validation_error_with_context(program: &MirProgram, context: &MirValidationContext<'_>, needle: &str) {
    let errors = validate_program(program, context).expect_err("MIR should be invalid");
    assert!(
        errors.iter().any(|error| error.message.contains(needle)),
        "expected validation error containing {needle:?}, got {errors:#?}"
    );
}

fn analyze(source: &str) -> crate::driver::AnalyzedUnit {
    let session = Session::new(Config::default().with_target(Target::Faber));
    crate::driver::analyze_source(&session, "test.fab", source).expect("analysis succeeds")
}

#[test]
fn rejects_invalid_block_target() {
    let types = TypeTable::new();
    let mut program = valid_number_program(&types);
    program.functions[0].blocks[0].terminator.kind = MirTerminatorKind::Goto(MirBlockId(99));

    expect_validation_error(&program, &types, "block bb99 does not exist");
}

#[test]
fn rejects_invalid_local_reference() {
    let types = TypeTable::new();
    let mut program = valid_number_program(&types);
    program.functions[0].blocks[0].terminator.kind =
        MirTerminatorKind::Return(Some(MirOperand::Place(MirPlace::local(MirLocalId(99)))));

    expect_validation_error(&program, &types, "local _99 does not exist");
}

#[test]
fn rejects_invalid_temp_reference() {
    let types = TypeTable::new();
    let mut program = valid_number_program(&types);
    program.functions[0].blocks[0].terminator.kind = MirTerminatorKind::Return(Some(MirOperand::Temp(MirTempId(99))));

    expect_validation_error(&program, &types, "temp %99 does not exist");
}

#[test]
fn rejects_value_operand_without_earlier_definition() {
    let types = TypeTable::new();
    let mut program = valid_number_program(&types);
    program.functions[0].blocks[0].terminator.kind = MirTerminatorKind::Return(Some(MirOperand::Value(MirValueId(99))));

    expect_validation_error(&program, &types, "value v99 is not defined earlier in MIR");
}

#[test]
fn rejects_value_operand_from_prior_block() {
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: Vec::new(),
            temps: vec![MirTemp { id: MirTempId(0), ty: number, span: span() }],
            blocks: vec![
                MirBlock {
                    id: MirBlockId(0),
                    statements: vec![MirStmt {
                        kind: MirStmtKind::Assign {
                            place: MirPlace::temp(MirTempId(0)),
                            value: MirValue {
                                id: MirValueId(0),
                                kind: MirValueKind::Operand(MirOperand::Constant(MirConstant::Int(1))),
                                ty: number,
                                span: span(),
                            },
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator { kind: MirTerminatorKind::Goto(MirBlockId(1)), span: span() },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(1),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Return(Some(MirOperand::Value(MirValueId(0)))),
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

    expect_validation_error(&program, &types, "value v0 is not defined earlier in MIR");
}

#[test]
fn rejects_return_type_mismatch() {
    let types = TypeTable::new();
    let mut program = valid_number_program(&types);
    program.functions[0].return_ty = ty(&types, Primitive::Textus);

    expect_validation_error(&program, &types, "return type mismatch");
}

#[test]
fn rejects_unresolved_infer_types() {
    let mut types = TypeTable::new();
    let infer = MirType::semantic(types.intern(Type::Infer(InferVar(0))));
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
            return_ty: infer,
            error_ty: None,
            span: span(),
        }],
    };

    expect_validation_error(&program, &types, "unresolved inference variable");
}

#[test]
fn rejects_return_error_without_alternate_exit_type() {
    let types = TypeTable::new();
    let mut program = valid_number_program(&types);
    program.functions[0].blocks[0].terminator.kind =
        MirTerminatorKind::ReturnError(MirOperand::Constant(MirConstant::String(Symbol(1))));

    expect_validation_error(&program, &types, "return_error in function without alternate-exit type");
}

#[test]
fn rejects_non_bivalens_branch_condition() {
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
            blocks: vec![
                MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Branch {
                            condition: MirOperand::Constant(MirConstant::Int(1)),
                            then_block: MirBlockId(1),
                            else_block: MirBlockId(1),
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
    };

    expect_validation_error(&program, &types, "branch condition is not bivalens");
}

#[test]
fn rejects_malformed_try_call_edges() {
    let types = TypeTable::new();
    let text = ty(&types, Primitive::Textus);
    let vacuum = ty(&types, Primitive::Vacuum);
    let program = MirProgram {
        functions: vec![
            MirFunction {
                id: MirFunctionId(0),
                source: None,
                name: None,
                params: Vec::new(),
                locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: false, span: span() }],
                temps: Vec::new(),
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::TryCall {
                            destination: None,
                            callee: MirCallee::Function(MirFunctionId(1)),
                            args: Vec::new(),
                            ok_block: MirBlockId(0),
                            error_place: MirPlace::local(MirLocalId(0)),
                            error_block: MirBlockId(99),
                        },
                        span: span(),
                    },
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
                error_ty: Some(text),
                span: span(),
            },
        ],
    };

    expect_validation_error(&program, &types, "block bb99 does not exist");
}

#[test]
fn rejects_try_call_without_known_failable_signature() {
    let types = TypeTable::new();
    let vacuum = ty(&types, Primitive::Vacuum);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: vacuum, mutable: false, span: span() }],
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: Vec::new(),
                terminator: MirTerminator {
                    kind: MirTerminatorKind::TryCall {
                        destination: None,
                        callee: MirCallee::Value(MirOperand::Constant(MirConstant::Unit)),
                        args: Vec::new(),
                        ok_block: MirBlockId(0),
                        error_place: MirPlace::local(MirLocalId(0)),
                        error_block: MirBlockId(0),
                    },
                    span: span(),
                },
                span: span(),
            }],
            return_ty: vacuum,
            error_ty: None,
            span: span(),
        }],
    };

    expect_validation_error(&program, &types, "try_call callee does not have a known failable signature");
}

#[test]
fn rejects_call_argument_count_and_type_mismatch() {
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let vacuum = ty(&types, Primitive::Vacuum);
    let program = MirProgram {
        functions: vec![
            MirFunction {
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
                            callee: MirCallee::Function(MirFunctionId(1)),
                            args: vec![MirOperand::Constant(MirConstant::String(Symbol(1)))],
                        },
                        span: span(),
                    }],
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
                params: vec![
                    MirParam { local: MirLocalId(0), name: None, ty: number, span: span() },
                    MirParam { local: MirLocalId(1), name: None, ty: number, span: span() },
                ],
                locals: vec![
                    MirLocal { id: MirLocalId(0), name: None, ty: number, mutable: false, span: span() },
                    MirLocal { id: MirLocalId(1), name: None, ty: number, mutable: false, span: span() },
                ],
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

    expect_validation_error(&program, &types, "call argument count mismatch");
    expect_validation_error(&program, &types, "call argument type mismatch");
}

#[test]
fn rejects_ordinary_call_to_failable_function() {
    let types = TypeTable::new();
    let text = ty(&types, Primitive::Textus);
    let vacuum = ty(&types, Primitive::Vacuum);
    let program = MirProgram {
        functions: vec![
            MirFunction {
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
                            callee: MirCallee::Function(MirFunctionId(1)),
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
    };

    expect_validation_error(&program, &types, "ordinary call callee is failable");
}

#[test]
fn rejects_runtime_call_destination_type_mismatch() {
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let text = ty(&types, Primitive::Textus);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: false, span: span() }],
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![MirStmt {
                    kind: MirStmtKind::RuntimeCall {
                        destination: Some(MirPlace::local(MirLocalId(0))),
                        call: MirRuntimeCall {
                            intrinsic: MirIntrinsic::Collection(MirCollectionOp::Length),
                            args: vec![MirOperand::Constant(MirConstant::String(Symbol(1)))],
                            return_ty: number,
                        },
                    },
                    span: span(),
                }],
                terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                span: span(),
            }],
            return_ty: ty(&types, Primitive::Vacuum),
            error_ty: None,
            span: span(),
        }],
    };

    expect_validation_error(&program, &types, "destination type mismatch");
}

#[test]
fn rejects_aggregate_payload_shape_mismatch() {
    let types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let program = MirProgram {
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
                            kind: MirAggregateKind::Map,
                            ty: number,
                            fields: MirAggregateFields::Ordered(vec![MirAggregateItem::Operand(MirOperand::Constant(
                                MirConstant::Int(1),
                            ))]),
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

    expect_validation_error(&program, &types, "aggregate payload shape does not match aggregate kind");
}

#[test]
fn rejects_typed_aggregate_mismatches() {
    let mut types = TypeTable::new();
    let number = ty(&types, Primitive::Numerus);
    let struct_def = crate::hir::DefId(42);
    let struct_ty = MirType::semantic(types.intern(Type::Struct(struct_def)));
    let field = Symbol(7);
    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: Vec::new(),
            temps: vec![MirTemp { id: MirTempId(0), ty: struct_ty, span: span() }],
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![MirStmt {
                    kind: MirStmtKind::Construct {
                        destination: MirPlace::temp(MirTempId(0)),
                        aggregate: MirAggregate {
                            kind: MirAggregateKind::Struct(struct_def),
                            ty: struct_ty,
                            fields: MirAggregateFields::Named(vec![
                                MirNamedOperand {
                                    name: field,
                                    value: MirOperand::Constant(MirConstant::String(Symbol(1))),
                                },
                                MirNamedOperand { name: field, value: MirOperand::Constant(MirConstant::Int(1)) },
                            ]),
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
            return_ty: struct_ty,
            error_ty: None,
            span: span(),
        }],
    };
    let mut context = MirValidationContext::new(&types);
    let mut fields = rustc_hash::FxHashMap::default();
    fields.insert(field, number);
    context.struct_fields.insert(struct_def, fields);

    expect_validation_error_with_context(&program, &context, "named aggregate field type mismatch");
    expect_validation_error_with_context(&program, &context, "named aggregate field is duplicated");
}

#[test]
fn rejects_runtime_intrinsic_argument_mismatch() {
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
                statements: vec![MirStmt {
                    kind: MirStmtKind::RuntimeCall {
                        destination: None,
                        call: MirRuntimeCall {
                            intrinsic: MirIntrinsic::Convert(MirConversion {
                                flavor: MirConversionFlavor::Runtime,
                                target_ty: vacuum,
                                params: Vec::new(),
                                fallback: None,
                            }),
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
    };

    expect_validation_error(&program, &types, "conversion runtime call expects 1 MIR arguments");
}

#[test]
fn representative_lowered_phase_3_to_7_mir_validates() {
    for source in [
        r#"functio arithmetic(numerus a, numerus b) → numerus { redde a + b }"#,
        r#"functio choose(bivalens ready) → numerus { si ready { redde 1 } secus { redde 2 } }"#,
        r#"functio maybe(textus ∪ nihil name) → textus { redde name vel "ignotus" }"#,
        r#"genus Persona { textus nomen numerus aetas = 0 } functio age() → numerus { fixum Persona p ← Persona { nomen = "Ada" } redde p.aetas }"#,
        r#"genus VacuumStruct {} functio empty_struct() → VacuumStruct { redde {} ∷ VacuumStruct }"#,
        r#"discretio Status { Active } functio active() → Status { redde finge Active ∷ Status }"#,
        r#"functio count(lista<numerus> xs) → numerus { redde xs.longitudo() }"#,
        r#"functio parse(textus raw) → numerus { redde raw ⇒ numerus<i32, Hex> vel 0 }"#,
        r#"functio log(textus name) → vacuum { nota "salve" vide name mone "cave" }"#,
    ] {
        lower_analyzed_unit(&analyze(source)).expect("lowered MIR validates");
    }
}
