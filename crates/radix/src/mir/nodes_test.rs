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
                                intrinsic: MirIntrinsic::Print,
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
    runtime print(const string sym#9) -> ty#5
    goto bb2
  bb2:
    return
}
"
    );
}
