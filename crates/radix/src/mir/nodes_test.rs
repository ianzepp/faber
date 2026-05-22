use super::*;
use crate::lexer::Span;
use crate::mir::dump_program;
use crate::semantic::{Primitive, TypeTable};

fn span() -> Span {
    Span::new(0, 0)
}

#[test]
fn mir_type_wraps_semantic_type_and_reserves_layout_slot() {
    let types = TypeTable::new();
    let text = types.primitive(Primitive::Textus);
    let plain = MirType::semantic(text);
    let laid_out = MirType::with_layout(text, MirLayoutId(7));

    assert_eq!(plain.semantic_id(), text);
    assert_eq!(plain.layout_id(), None);
    assert_eq!(laid_out.semantic_id(), text);
    assert_eq!(laid_out.layout_id(), Some(MirLayoutId(7)));
}

#[test]
fn dump_program_renders_basic_blocks_deterministically() {
    let types = TypeTable::new();
    let number = MirType::semantic(types.primitive(Primitive::Numerus));

    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![MirLocal { id: MirLocalId(0), name: None, ty: number, mutable: false, span: span() }],
            temps: vec![MirTemp { id: MirTempId(0), ty: number, span: span() }],
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![MirStmt {
                    kind: MirStmtKind::Assign {
                        place: MirPlace::local(MirLocalId(0)),
                        value: MirValue {
                            id: MirValueId(0),
                            kind: MirValueKind::Binary {
                                op: MirBinOp::Add,
                                lhs: MirOperand::Constant(MirConstant::Int(1)),
                                rhs: MirOperand::Constant(MirConstant::Int(2)),
                            },
                            ty: number,
                            span: span(),
                        },
                    },
                    span: span(),
                }],
                terminator: MirTerminator {
                    kind: MirTerminatorKind::Return(Some(MirOperand::Place(MirPlace::local(MirLocalId(0))))),
                    span: span(),
                },
                span: span(),
            }],
            return_ty: number,
            error_ty: None,
            span: span(),
        }],
    };

    assert_eq!(
        dump_program(&program),
        "\
function f0 -> ty#1 {
  locals:
    let _0: ty#1
  temps:
    %0: ty#1
  bb0:
    _0 = const int 1 + const int 2: ty#1
    return _0
}
"
    );
}

#[test]
fn dump_program_renders_runtime_calls_and_branches() {
    let types = TypeTable::new();
    let boolean = MirType::semantic(types.primitive(Primitive::Bivalens));
    let vacuum = MirType::semantic(types.primitive(Primitive::Vacuum));

    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(3),
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
                        kind: MirTerminatorKind::Branch {
                            condition: MirOperand::Place(MirPlace::local(MirLocalId(0))),
                            then_block: MirBlockId(1),
                            else_block: MirBlockId(2),
                        },
                        span: span(),
                    },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(1),
                    statements: vec![MirStmt {
                        kind: MirStmtKind::RuntimeCall {
                            destination: None,
                            call: MirRuntimeCall {
                                intrinsic: MirIntrinsic::Diagnostic(MirDiagnosticKind::Scribe),
                                args: vec![MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(9)))],
                                return_ty: vacuum,
                            },
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator { kind: MirTerminatorKind::Goto(MirBlockId(2)), span: span() },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(2),
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

    assert_eq!(
        dump_program(&program),
        "\
function f3 -> ty#5 {
  params:
    _0: ty#3
  bb0:
    branch _0 bb1 bb2
  bb1:
    runtime diagnostic scribe(const string sym#9) -> ty#5
    goto bb2
  bb2:
    return
}
"
    );
}

#[test]
fn dump_program_renders_aggregate_payloads_and_operand_index_projection() {
    let types = TypeTable::new();
    let text = MirType::semantic(types.primitive(Primitive::Textus));
    let number = MirType::semantic(types.primitive(Primitive::Numerus));

    let indexed_place = MirPlace {
        base: MirPlaceBase::Local(MirLocalId(1)),
        projections: vec![MirProjection::Index(MirOperand::Constant(MirConstant::String(
            crate::lexer::Symbol(14),
        )))],
    };

    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![
                MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: false, span: span() },
                MirLocal { id: MirLocalId(1), name: None, ty: number, mutable: false, span: span() },
            ],
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![
                    MirStmt {
                        kind: MirStmtKind::Construct {
                            destination: MirPlace::local(MirLocalId(0)),
                            aggregate: MirAggregate {
                                kind: MirAggregateKind::Struct(DefId(7)),
                                ty: text,
                                fields: MirAggregateFields::Named(vec![
                                    MirNamedOperand {
                                        name: crate::lexer::Symbol(11),
                                        value: MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(12))),
                                    },
                                    MirNamedOperand {
                                        name: crate::lexer::Symbol(13),
                                        value: MirOperand::Constant(MirConstant::Int(36)),
                                    },
                                ]),
                            },
                        },
                        span: span(),
                    },
                    MirStmt {
                        kind: MirStmtKind::Construct {
                            destination: MirPlace::local(MirLocalId(1)),
                            aggregate: MirAggregate {
                                kind: MirAggregateKind::Map,
                                ty: number,
                                fields: MirAggregateFields::Keyed(vec![MirKeyValueOperand {
                                    key: MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(14))),
                                    value: MirOperand::Constant(MirConstant::Int(1)),
                                }]),
                            },
                        },
                        span: span(),
                    },
                ],
                terminator: MirTerminator {
                    kind: MirTerminatorKind::Return(Some(MirOperand::Place(indexed_place))),
                    span: span(),
                },
                span: span(),
            }],
            return_ty: number,
            error_ty: None,
            span: span(),
        }],
    };

    assert_eq!(
        dump_program(&program),
        "\
function f0 -> ty#1 {
  locals:
    let _0: ty#0
    let _1: ty#1
  bb0:
    _0 = construct struct def#7: ty#0 {sym#11: const string sym#12, sym#13: const int 36}
    _1 = construct map: ty#1 {const string sym#14 => const int 1}
    return _1[const string sym#14]
}
"
    );
}

#[test]
fn dump_program_renders_option_operations() {
    let types = TypeTable::new();
    let text = MirType::semantic(types.primitive(Primitive::Textus));

    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![
                MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: false, span: span() },
                MirLocal { id: MirLocalId(1), name: None, ty: text, mutable: false, span: span() },
                MirLocal { id: MirLocalId(2), name: None, ty: text, mutable: false, span: span() },
            ],
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![
                    MirStmt {
                        kind: MirStmtKind::Assign {
                            place: MirPlace::local(MirLocalId(0)),
                            value: MirValue {
                                id: MirValueId(0),
                                kind: MirValueKind::Option(MirOptionOp::Some(MirOperand::Constant(
                                    MirConstant::String(crate::lexer::Symbol(9)),
                                ))),
                                ty: text,
                                span: span(),
                            },
                        },
                        span: span(),
                    },
                    MirStmt {
                        kind: MirStmtKind::Assign {
                            place: MirPlace::local(MirLocalId(1)),
                            value: MirValue {
                                id: MirValueId(1),
                                kind: MirValueKind::Option(MirOptionOp::Chain {
                                    base: MirOperand::Place(MirPlace::local(MirLocalId(0))),
                                    link: MirOptionChainLink::Field(crate::lexer::Symbol(10)),
                                }),
                                ty: text,
                                span: span(),
                            },
                        },
                        span: span(),
                    },
                    MirStmt {
                        kind: MirStmtKind::Assign {
                            place: MirPlace::local(MirLocalId(2)),
                            value: MirValue {
                                id: MirValueId(2),
                                kind: MirValueKind::Option(MirOptionOp::Coalesce {
                                    value: MirOperand::Place(MirPlace::local(MirLocalId(1))),
                                    fallback: MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(11))),
                                }),
                                ty: text,
                                span: span(),
                            },
                        },
                        span: span(),
                    },
                ],
                terminator: MirTerminator {
                    kind: MirTerminatorKind::Return(Some(MirOperand::Place(MirPlace::local(MirLocalId(2))))),
                    span: span(),
                },
                span: span(),
            }],
            return_ty: text,
            error_ty: None,
            span: span(),
        }],
    };

    assert_eq!(
        dump_program(&program),
        "\
function f0 -> ty#0 {
  locals:
    let _0: ty#0
    let _1: ty#0
    let _2: ty#0
  bb0:
    _0 = option some(const string sym#9): ty#0
    _1 = option chain(_0, .sym#10): ty#0
    _2 = option coalesce(_1, const string sym#11): ty#0
    return _2
}
"
    );
}

#[test]
fn dump_program_renders_structured_runtime_intrinsics() {
    let types = TypeTable::new();
    let text = MirType::semantic(types.primitive(Primitive::Textus));
    let number = MirType::semantic(types.primitive(Primitive::Numerus));
    let vacuum = MirType::semantic(types.primitive(Primitive::Vacuum));

    let program = MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(0),
            source: None,
            name: None,
            params: Vec::new(),
            locals: vec![
                MirLocal { id: MirLocalId(0), name: None, ty: text, mutable: false, span: span() },
                MirLocal { id: MirLocalId(1), name: None, ty: number, mutable: false, span: span() },
            ],
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: MirBlockId(0),
                statements: vec![
                    MirStmt {
                        kind: MirStmtKind::RuntimeCall {
                            destination: None,
                            call: MirRuntimeCall {
                                intrinsic: MirIntrinsic::Diagnostic(MirDiagnosticKind::Mone),
                                args: vec![MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(20)))],
                                return_ty: vacuum,
                            },
                        },
                        span: span(),
                    },
                    MirStmt {
                        kind: MirStmtKind::RuntimeCall {
                            destination: Some(MirPlace::local(MirLocalId(0))),
                            call: MirRuntimeCall {
                                intrinsic: MirIntrinsic::FormatString { template: crate::lexer::Symbol(21) },
                                args: vec![MirOperand::Constant(MirConstant::String(crate::lexer::Symbol(22)))],
                                return_ty: text,
                            },
                        },
                        span: span(),
                    },
                    MirStmt {
                        kind: MirStmtKind::RuntimeCall {
                            destination: Some(MirPlace::local(MirLocalId(1))),
                            call: MirRuntimeCall {
                                intrinsic: MirIntrinsic::Convert(MirConversion {
                                    flavor: MirConversionFlavor::Runtime,
                                    target_ty: number,
                                    params: Vec::new(),
                                    fallback: Some(MirOperand::Constant(MirConstant::Int(0))),
                                }),
                                args: vec![MirOperand::Place(MirPlace::local(MirLocalId(0)))],
                                return_ty: number,
                            },
                        },
                        span: span(),
                    },
                    MirStmt {
                        kind: MirStmtKind::RuntimeCall {
                            destination: Some(MirPlace::local(MirLocalId(1))),
                            call: MirRuntimeCall {
                                intrinsic: MirIntrinsic::Collection(MirCollectionOp::Length),
                                args: vec![MirOperand::Place(MirPlace::local(MirLocalId(0)))],
                                return_ty: number,
                            },
                        },
                        span: span(),
                    },
                    MirStmt {
                        kind: MirStmtKind::RuntimeCall {
                            destination: Some(MirPlace::local(MirLocalId(0))),
                            call: MirRuntimeCall {
                                intrinsic: MirIntrinsic::Provider(MirProvider {
                                    module: vec![crate::lexer::Symbol(30), crate::lexer::Symbol(31)],
                                    name: crate::lexer::Symbol(32),
                                }),
                                args: Vec::new(),
                                return_ty: text,
                            },
                        },
                        span: span(),
                    },
                ],
                terminator: MirTerminator {
                    kind: MirTerminatorKind::Return(Some(MirOperand::Place(MirPlace::local(MirLocalId(0))))),
                    span: span(),
                },
                span: span(),
            }],
            return_ty: text,
            error_ty: None,
            span: span(),
        }],
    };

    assert_eq!(
        dump_program(&program),
        "\
function f0 -> ty#0 {
  locals:
    let _0: ty#0
    let _1: ty#1
  bb0:
    runtime diagnostic mone(const string sym#20) -> ty#5
    _0 = runtime format_string template sym#21(const string sym#22) -> ty#0
    _1 = runtime convert runtime -> ty#1 fallback const int 0(_0) -> ty#1
    _1 = runtime collection length(_0) -> ty#1
    _0 = runtime provider sym#30/sym#31::sym#32() -> ty#0
    return _0
}
"
    );
}
